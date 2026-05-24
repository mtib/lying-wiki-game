use crate::{
    error::AppError,
    state::{generate_room_code, generate_token, AppState, RoomEntry, Room},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

fn broadcast_snapshot(entry: &crate::state::RoomEntry, room: &crate::state::Room) {
    let snapshot = room.to_snapshot();
    let msg = json!({"type": "room_state", "data": snapshot}).to_string();
    let _ = entry.tx.send(msg);
}

// ── Create room ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub code: String,
    pub token: String,
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }
    let token = generate_token();
    let code = loop {
        let c = generate_room_code();
        if !state.rooms.contains_key(&c) {
            break c;
        }
    };
    let room = Room::new(code.clone(), name, token.clone());
    let (tx, _) = broadcast::channel(64);
    state.rooms.insert(
        code.clone(),
        RoomEntry {
            room: Arc::new(Mutex::new(room)),
            tx,
        },
    );
    Ok(Json(CreateRoomResponse { code, token }))
}

// ── Join room ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct JoinRoomRequest {
    pub name: String,
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct JoinRoomResponse {
    pub token: String,
}

pub async fn join_room(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    Json(body): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, AppError> {
    let entry = state
        .rooms
        .get(&code)
        .ok_or_else(|| AppError::NotFound("room not found".into()))?;
    let mut room = entry.room.lock().await;

    // Reconnect if token provided and matches an existing player.
    if let Some(ref t) = body.token {
        if let Some(player) = room.find_player_by_token_mut(t) {
            player.connected = true;
            broadcast_snapshot(&entry, &room);
            return Ok(Json(JoinRoomResponse { token: t.clone() }));
        }
    }

    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }
    let token = generate_token();
    room.add_player(name, token.clone())?;
    broadcast_snapshot(&entry, &room);
    Ok(Json(JoinRoomResponse { token }))
}

// ── Start game ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

pub async fn start_game(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    Json(body): Json<TokenRequest>,
) -> Result<StatusCode, AppError> {
    let entry = state
        .rooms
        .get(&code)
        .ok_or_else(|| AppError::NotFound("room not found".into()))?;
    let mut room = entry.room.lock().await;
    room.start_game(&body.token)?;
    broadcast_snapshot(&entry, &room);
    Ok(StatusCode::OK)
}

// ── Submit topic ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SubmitTopicRequest {
    pub token: String,
    pub title: String,
}

pub async fn submit_topic(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    Json(body): Json<SubmitTopicRequest>,
) -> Result<StatusCode, AppError> {
    let entry = state
        .rooms
        .get(&code)
        .ok_or_else(|| AppError::NotFound("room not found".into()))?;
    let mut room = entry.room.lock().await;
    room.submit_topic(&body.token, body.title)?;
    broadcast_snapshot(&entry, &room);
    Ok(StatusCode::OK)
}

// ── Start round ───────────────────────────────────────────────────────────────

pub async fn start_round(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    Json(body): Json<TokenRequest>,
) -> Result<StatusCode, AppError> {
    let entry = state
        .rooms
        .get(&code)
        .ok_or_else(|| AppError::NotFound("room not found".into()))?;
    let room_arc = entry.room.clone();
    let tx = entry.tx.clone();
    {
        let mut room = room_arc.lock().await;
        room.begin_countdown(&body.token)?;
        broadcast_snapshot(&entry, &room);
    }
    // Transition to RoundActive after 3 seconds.
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let mut room = room_arc.lock().await;
        room.activate_round();
        let snapshot = room.to_snapshot();
        let msg = json!({"type": "room_state", "data": snapshot}).to_string();
        let _ = tx.send(msg);
    });
    Ok(StatusCode::OK)
}

// ── Submit guess ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct GuessRequest {
    pub token: String,
    pub guessed_name: String,
}

pub async fn submit_guess(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    Json(body): Json<GuessRequest>,
) -> Result<StatusCode, AppError> {
    let entry = state
        .rooms
        .get(&code)
        .ok_or_else(|| AppError::NotFound("room not found".into()))?;
    let mut room = entry.room.lock().await;
    room.submit_guess(&body.token, &body.guessed_name)?;
    broadcast_snapshot(&entry, &room);
    Ok(StatusCode::OK)
}
