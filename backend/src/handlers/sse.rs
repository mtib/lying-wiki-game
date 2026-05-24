use crate::AppState as AS;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[derive(Deserialize)]
pub struct SseQuery {
    pub token: String,
}

pub async fn sse_handler(
    State(state): State<Arc<AS>>,
    Path(code): Path<String>,
    Query(query): Query<SseQuery>,
) -> impl IntoResponse {
    let (room_arc, tx) = match state.rooms.get(&code)
        .map(|e| (e.room.clone(), e.tx.clone()))
    {
        Some(pair) => pair,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Subscribe before broadcasting so we don't miss the initial snapshot.
    let rx = tx.subscribe();

    // Mark player connected and send initial snapshot.
    {
        let mut room = room_arc.lock().await;
        match room.find_player_by_token_mut(&query.token) {
            Some(player) => player.connected = true,
            None => return Err(StatusCode::UNAUTHORIZED),
        }
        // Broadcast full state so all clients (including this one via the stream above) get it.
        let snapshot = room.to_snapshot();
        let msg = json!({"type": "room_state", "data": snapshot}).to_string();
        let _ = tx.send(msg);
    }

    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        let msg = msg.ok()?;
        Some(Ok::<Event, Infallible>(Event::default().data(msg)))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
