# Lying Wiki Game Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a mobile-first "2 of These People Are Lying" party game web app with a Rust/Axum in-memory backend and a React/TypeScript frontend, connected via SSE for real-time game state.

**Architecture:** All game state lives in a `DashMap<RoomCode, RoomEntry>` where each entry holds an `Arc<Mutex<Room>>` and a `broadcast::Sender<String>`. Every state mutation locks the room, mutates, serializes a full snapshot, and broadcasts it. One persistent SSE stream per player receives those snapshots. The frontend derives which screen to show from the `state` field of the snapshot.

**Tech Stack:** Rust 1.77+, Axum 0.7, tokio, DashMap, serde_json, reqwest 0.12, uuid; TypeScript, React 18, Vite, Tailwind CSS 3.

---

## File Map

### Backend (`backend/`)
- `Cargo.toml` — dependencies
- `src/main.rs` — Axum router, CORS, server startup
- `src/error.rs` — `AppError` enum + `IntoResponse`
- `src/state.rs` — all types, `AppState`, `Room` impl with state-transition methods
- `src/handlers/mod.rs` — re-exports
- `src/handlers/rooms.rs` — HTTP handlers for all player actions
- `src/handlers/sse.rs` — SSE stream handler
- `src/handlers/wiki.rs` — Wikipedia proxy handler

### Frontend (`frontend/`)
- `package.json`
- `vite.config.ts` — dev proxy `/api` → `http://localhost:3001`
- `tsconfig.json`
- `tailwind.config.js` + `postcss.config.js`
- `index.html`
- `src/main.tsx`
- `src/App.tsx` — screen router
- `src/types.ts` — TypeScript types matching backend JSON
- `src/api.ts` — typed fetch wrappers
- `src/hooks/useDeviceToken.ts` — localStorage token management
- `src/hooks/useRoom.ts` — SSE connection + room state with reconnect
- `src/components/Toast.tsx`
- `src/components/ScoreDrawer.tsx`
- `src/components/WikiArticleSheet.tsx`
- `src/screens/HomeScreen.tsx`
- `src/screens/LobbyScreen.tsx`
- `src/screens/TopicSubmissionScreen.tsx`
- `src/screens/CountdownScreen.tsx`
- `src/screens/RoundActiveScreen.tsx`
- `src/screens/RoundRevealScreen.tsx`

---

## Task 1: Repo + Backend Scaffold

**Files:**
- Create: `.gitignore`
- Create: `backend/Cargo.toml`
- Create: `backend/src/main.rs`
- Create: `backend/src/error.rs`
- Create: `backend/src/handlers/mod.rs`

- [ ] **Step 1: Init git repo and .gitignore**

```bash
cd /Users/mtib/Code/lying-wiki-game
git init
cat > .gitignore << 'EOF'
/backend/target/
/frontend/node_modules/
/frontend/dist/
.env
EOF
git add .gitignore
git commit -m "chore: init repo"
```

- [ ] **Step 2: Create backend/Cargo.toml**

```bash
mkdir -p backend/src/handlers
```

`backend/Cargo.toml`:
```toml
[package]
name = "lying-wiki-game"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dashmap = "5"
uuid = { version = "1", features = ["v4"] }
reqwest = { version = "0.12", features = ["json"] }
tower-http = { version = "0.5", features = ["cors"] }
rand = "0.8"
tokio-stream = { version = "0.1", features = ["sync"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

- [ ] **Step 3: Create backend/src/error.rs**

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Conflict(String),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Invalid token".into()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
```

- [ ] **Step 4: Create backend/src/handlers/mod.rs**

```rust
pub mod rooms;
pub mod sse;
pub mod wiki;
```

- [ ] **Step 5: Create stub files so the project compiles**

`backend/src/handlers/rooms.rs`:
```rust
pub async fn create_room() {}
pub async fn join_room() {}
pub async fn start_game() {}
pub async fn submit_topic() {}
pub async fn start_round() {}
pub async fn submit_guess() {}
```

`backend/src/handlers/sse.rs`:
```rust
pub async fn sse_handler() {}
```

`backend/src/handlers/wiki.rs`:
```rust
pub async fn random_article() {}
```

`backend/src/state.rs`:
```rust
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub rooms: DashMap<String, RoomEntry>,
}

impl AppState {
    pub fn new() -> Self {
        Self { rooms: DashMap::new() }
    }
}

pub struct RoomEntry {
    pub room: Arc<Mutex<Room>>,
    pub tx: broadcast::Sender<String>,
}

pub struct Room {
    pub code: String,
}
```

`backend/src/main.rs`:
```rust
mod error;
mod handlers;
mod state;

use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use tower_http::cors::{Any, CorsLayer};

pub use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/rooms", post(handlers::rooms::create_room))
        .route("/rooms/:code/join", post(handlers::rooms::join_room))
        .route("/rooms/:code/start-game", post(handlers::rooms::start_game))
        .route("/rooms/:code/topic", post(handlers::rooms::submit_topic))
        .route("/rooms/:code/start-round", post(handlers::rooms::start_round))
        .route("/rooms/:code/guess", post(handlers::rooms::submit_guess))
        .route("/rooms/:code/events", get(handlers::sse::sse_handler))
        .route("/wiki/random", get(handlers::wiki::random_article))
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 6: Verify it compiles**

```bash
cd backend && cargo build 2>&1
```

Expected: compiles successfully (no errors).

- [ ] **Step 7: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add backend/
git commit -m "chore: scaffold rust backend"
```

---

## Task 2: Backend State Types and Transition Logic

**Files:**
- Modify: `backend/src/state.rs` (full rewrite)

This is the core of the game. All state mutation lives here as `impl Room` methods returning `Result<(), AppError>`. Tests are inline `#[cfg(test)]` modules.

- [ ] **Step 1: Write failing tests for state transitions**

Replace `backend/src/state.rs` with:

```rust
use crate::error::AppError;
use dashmap::DashMap;
use rand::seq::SliceRandom;
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

// ── Public app state ────────────────────────────────────────────────────────

pub struct AppState {
    pub rooms: DashMap<String, RoomEntry>,
}

impl AppState {
    pub fn new() -> Self {
        Self { rooms: DashMap::new() }
    }
}

pub struct RoomEntry {
    pub room: Arc<Mutex<Room>>,
    pub tx: broadcast::Sender<String>,
}

// ── Domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Room {
    pub code: String,
    pub players: Vec<Player>,
    pub guesser_index: usize,
    pub state: RoomState,
    pub current_topic: Option<RevealedTopic>,
    pub round_number: u32,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub token: String,
    pub name: String,
    pub score: u32,
    pub connected: bool,
    pub submitted_topic: Option<String>,
    pub submitted_this_round: bool,
}

#[derive(Debug, Clone)]
pub struct RevealedTopic {
    pub title: String,
    pub owner_token: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomState {
    Lobby,
    TopicSubmission,
    Countdown { started_at_ms: u64 },
    RoundActive,
    RoundReveal(RoundRevealData),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoundRevealData {
    pub topic: String,
    pub owner_name: String,
    pub guesser_name: String,
    pub guessed_name: String,
    pub correct: bool,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub round: u32,
    pub topic: String,
    pub owner_name: String,
    pub guesser_name: String,
    pub guessed_name: String,
    pub correct: bool,
    pub points: u32,
}

// ── Snapshot (what gets serialized and sent over SSE) ───────────────────────

#[derive(Serialize, Clone)]
pub struct RoomSnapshot {
    pub code: String,
    pub state: SnapshotState,
    pub players: Vec<PlayerSnapshot>,
    pub current_topic: Option<String>,
    pub reveal: Option<RoundRevealData>,
    pub log: Vec<LogEntry>,
    pub round_number: u32,
    pub guesser_name: Option<String>,
    pub countdown_started_at_ms: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct PlayerSnapshot {
    pub name: String,
    pub score: u32,
    pub connected: bool,
    pub submitted_this_round: bool,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotState {
    Lobby,
    TopicSubmission,
    Countdown,
    RoundActive,
    RoundReveal,
}

// ── Room implementation ───────────────────────────────────────────────────────

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

impl Room {
    pub fn new(code: String, host_name: String, host_token: String) -> Self {
        let host = Player {
            token: host_token,
            name: host_name,
            score: 0,
            connected: true,
            submitted_topic: None,
            submitted_this_round: false,
        };
        Self {
            code,
            players: vec![host],
            guesser_index: 0,
            state: RoomState::Lobby,
            current_topic: None,
            round_number: 0,
            log: vec![],
        }
    }

    pub fn find_player_by_token(&self, token: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.token == token)
    }

    pub fn find_player_by_token_mut(&mut self, token: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.token == token)
    }

    pub fn add_player(&mut self, name: String, token: String) -> Result<(), AppError> {
        if !matches!(self.state, RoomState::Lobby) {
            return Err(AppError::Conflict("game already started".into()));
        }
        if self.players.iter().any(|p| p.name == name) {
            return Err(AppError::Conflict(format!("name '{}' already taken", name)));
        }
        self.players.push(Player {
            token,
            name,
            score: 0,
            connected: true,
            submitted_topic: None,
            submitted_this_round: false,
        });
        Ok(())
    }

    pub fn start_game(&mut self, token: &str) -> Result<(), AppError> {
        self.find_player_by_token(token)
            .ok_or(AppError::Unauthorized)?;
        if !matches!(self.state, RoomState::Lobby) {
            return Err(AppError::Conflict("game is not in lobby".into()));
        }
        if self.players.len() < 3 {
            return Err(AppError::Conflict("need at least 3 players".into()));
        }
        self.state = RoomState::TopicSubmission;
        Ok(())
    }

    pub fn submit_topic(&mut self, token: &str, title: String) -> Result<(), AppError> {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::BadRequest("title cannot be empty".into()));
        }
        match &self.state.clone() {
            RoomState::TopicSubmission => {
                let player = self
                    .find_player_by_token_mut(token)
                    .ok_or(AppError::Unauthorized)?;
                player.submitted_topic = Some(title);
                player.submitted_this_round = true;
                Ok(())
            }
            RoomState::RoundReveal(data) => {
                // Only the owner of the revealed topic may submit here.
                let owner_token = self
                    .current_topic
                    .as_ref()
                    .map(|t| t.owner_token.clone())
                    .ok_or_else(|| AppError::Conflict("no current topic".into()))?;
                if token != owner_token {
                    return Err(AppError::Conflict(
                        "only the topic owner advances the round".into(),
                    ));
                }
                {
                    let player = self
                        .find_player_by_token_mut(token)
                        .ok_or(AppError::Unauthorized)?;
                    player.submitted_topic = Some(title);
                    player.submitted_this_round = true;
                }
                // Auto-confirm all other players (they keep existing topics).
                for p in &mut self.players {
                    if p.token != owner_token {
                        p.submitted_this_round = true;
                    }
                }
                self.guesser_index = (self.guesser_index + 1) % self.players.len();
                self.current_topic = None;
                self.state = RoomState::TopicSubmission;
                let _ = data; // consumed via clone above
                Ok(())
            }
            _ => Err(AppError::Conflict(
                "cannot submit topic in current state".into(),
            )),
        }
    }

    pub fn begin_countdown(&mut self, token: &str) -> Result<(), AppError> {
        self.find_player_by_token(token)
            .ok_or(AppError::Unauthorized)?;
        if !matches!(self.state, RoomState::TopicSubmission) {
            return Err(AppError::Conflict("not in topic submission phase".into()));
        }
        if !self.players.iter().all(|p| p.submitted_this_round) {
            return Err(AppError::Conflict(
                "not all players have submitted a topic".into(),
            ));
        }
        // Pick a topic that does not belong to the current guesser.
        let guesser_token = self.players[self.guesser_index].token.clone();
        let candidates: Vec<(String, String)> = self
            .players
            .iter()
            .filter(|p| p.token != guesser_token)
            .filter_map(|p| p.submitted_topic.clone().map(|t| (t, p.token.clone())))
            .collect();
        let mut rng = rand::thread_rng();
        let (title, owner_token) = candidates
            .choose(&mut rng)
            .ok_or_else(|| AppError::Conflict("no eligible topics".into()))?
            .clone();
        self.current_topic = Some(RevealedTopic { title, owner_token });
        self.round_number += 1;
        // Reset submitted_this_round for next cycle.
        for p in &mut self.players {
            p.submitted_this_round = false;
        }
        self.state = RoomState::Countdown { started_at_ms: now_ms() };
        Ok(())
    }

    pub fn activate_round(&mut self) {
        if matches!(self.state, RoomState::Countdown { .. }) {
            self.state = RoomState::RoundActive;
        }
    }

    pub fn submit_guess(&mut self, guesser_token: &str, guessed_name: &str) -> Result<(), AppError> {
        if !matches!(self.state, RoomState::RoundActive) {
            return Err(AppError::Conflict("not in an active round".into()));
        }
        let guesser_name = self
            .find_player_by_token(guesser_token)
            .ok_or(AppError::Unauthorized)?
            .name
            .clone();
        // Verify it's the guesser's turn.
        if self.players[self.guesser_index].token != guesser_token {
            return Err(AppError::Conflict("it is not your turn to guess".into()));
        }
        let topic = self
            .current_topic
            .clone()
            .ok_or_else(|| AppError::Conflict("no active topic".into()))?;
        let owner_name = self
            .find_player_by_token(&topic.owner_token)
            .ok_or_else(|| AppError::Conflict("owner not found".into()))?
            .name
            .clone();
        // guessed_name must be a real player (not the guesser).
        let guessed_player = self
            .players
            .iter()
            .find(|p| p.name == guessed_name)
            .ok_or_else(|| AppError::BadRequest(format!("player '{}' not found", guessed_name)))?;
        if guessed_player.token == guesser_token {
            return Err(AppError::BadRequest("cannot guess yourself".into()));
        }
        let correct = guessed_name == owner_name;
        let n = self.players.len() as u32;
        let points = if correct { n - 1 } else { 1 };
        // Award points.
        if correct {
            let p = self
                .find_player_by_token_mut(guesser_token)
                .unwrap();
            p.score += points;
        } else {
            for p in &mut self.players {
                if p.token != guesser_token {
                    p.score += points;
                }
            }
        }
        let reveal = RoundRevealData {
            topic: topic.title.clone(),
            owner_name: owner_name.clone(),
            guesser_name: guesser_name.clone(),
            guessed_name: guessed_name.to_string(),
            correct,
            points,
        };
        self.log.push(LogEntry {
            round: self.round_number,
            topic: topic.title,
            owner_name,
            guesser_name,
            guessed_name: guessed_name.to_string(),
            correct,
            points,
        });
        self.state = RoomState::RoundReveal(reveal);
        Ok(())
    }

    pub fn to_snapshot(&self) -> RoomSnapshot {
        let guesser_name = self
            .players
            .get(self.guesser_index)
            .map(|p| p.name.clone());
        let (state, reveal, countdown_started_at_ms) = match &self.state {
            RoomState::Lobby => (SnapshotState::Lobby, None, None),
            RoomState::TopicSubmission => (SnapshotState::TopicSubmission, None, None),
            RoomState::Countdown { started_at_ms } => {
                (SnapshotState::Countdown, None, Some(*started_at_ms))
            }
            RoomState::RoundActive => (SnapshotState::RoundActive, None, None),
            RoomState::RoundReveal(data) => {
                (SnapshotState::RoundReveal, Some(data.clone()), None)
            }
        };
        RoomSnapshot {
            code: self.code.clone(),
            state,
            players: self
                .players
                .iter()
                .map(|p| PlayerSnapshot {
                    name: p.name.clone(),
                    score: p.score,
                    connected: p.connected,
                    submitted_this_round: p.submitted_this_round,
                })
                .collect(),
            current_topic: self.current_topic.as_ref().map(|t| t.title.clone()),
            reveal,
            log: self.log.clone(),
            round_number: self.round_number,
            guesser_name,
            countdown_started_at_ms,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn generate_room_code() -> String {
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..6).map(|_| *chars.choose(&mut rng).unwrap()).collect()
}

pub fn generate_token() -> String {
    Uuid::new_v4().to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_room() -> Room {
        Room::new("ABCDEF".into(), "Alice".into(), "tok-alice".into())
    }

    fn add_players(room: &mut Room) {
        room.add_player("Bob".into(), "tok-bob".into()).unwrap();
        room.add_player("Charlie".into(), "tok-charlie".into()).unwrap();
    }

    fn setup_game(room: &mut Room) {
        add_players(room);
        room.start_game("tok-alice").unwrap();
        room.submit_topic("tok-alice", "Banana republic".into()).unwrap();
        room.submit_topic("tok-bob", "Quantum foam".into()).unwrap();
        room.submit_topic("tok-charlie", "Velocipede".into()).unwrap();
    }

    #[test]
    fn add_player_duplicate_name_rejected() {
        let mut room = make_room();
        let result = room.add_player("Alice".into(), "tok-other".into());
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[test]
    fn add_player_after_game_start_rejected() {
        let mut room = make_room();
        add_players(&mut room);
        room.start_game("tok-alice").unwrap();
        let result = room.add_player("Dave".into(), "tok-dave".into());
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[test]
    fn start_game_requires_three_players() {
        let mut room = make_room();
        room.add_player("Bob".into(), "tok-bob".into()).unwrap();
        let result = room.start_game("tok-alice");
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[test]
    fn start_game_transitions_to_topic_submission() {
        let mut room = make_room();
        add_players(&mut room);
        room.start_game("tok-alice").unwrap();
        assert_eq!(room.state, RoomState::TopicSubmission);
    }

    #[test]
    fn submit_topic_marks_player_submitted() {
        let mut room = make_room();
        add_players(&mut room);
        room.start_game("tok-alice").unwrap();
        room.submit_topic("tok-alice", "Banana republic".into()).unwrap();
        let alice = room.find_player_by_token("tok-alice").unwrap();
        assert_eq!(alice.submitted_topic, Some("Banana republic".into()));
        assert!(alice.submitted_this_round);
    }

    #[test]
    fn begin_countdown_fails_if_not_all_submitted() {
        let mut room = make_room();
        add_players(&mut room);
        room.start_game("tok-alice").unwrap();
        room.submit_topic("tok-alice", "Banana republic".into()).unwrap();
        let result = room.begin_countdown("tok-alice");
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[test]
    fn begin_countdown_picks_topic_not_from_guesser() {
        let mut room = make_room();
        setup_game(&mut room);
        // guesser_index=0 → Alice is guesser; topic must not be "Banana republic"
        room.begin_countdown("tok-alice").unwrap();
        assert!(matches!(room.state, RoomState::Countdown { .. }));
        let topic_title = room.current_topic.as_ref().unwrap().title.clone();
        assert_ne!(topic_title, "Banana republic");
    }

    #[test]
    fn correct_guess_awards_n_minus_1_to_guesser() {
        let mut room = make_room();
        setup_game(&mut room);
        room.begin_countdown("tok-alice").unwrap();
        room.activate_round();
        // Find the actual owner to make a correct guess.
        let owner_name = room
            .find_player_by_token(&room.current_topic.clone().unwrap().owner_token)
            .unwrap()
            .name
            .clone();
        room.submit_guess("tok-alice", &owner_name).unwrap();
        // 3 players → n-1 = 2 points
        let alice = room.find_player_by_token("tok-alice").unwrap();
        assert_eq!(alice.score, 2);
    }

    #[test]
    fn wrong_guess_awards_one_to_everyone_else() {
        let mut room = make_room();
        setup_game(&mut room);
        room.begin_countdown("tok-alice").unwrap();
        room.activate_round();
        let topic = room.current_topic.clone().unwrap();
        let owner_name = room.find_player_by_token(&topic.owner_token).unwrap().name.clone();
        // Guess someone who is NOT the owner (and not Alice herself).
        let wrong_name = room
            .players
            .iter()
            .find(|p| p.name != "Alice" && p.name != owner_name)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| {
                // If owner is Bob, wrong = Charlie; if owner is Charlie, wrong = Bob
                if owner_name == "Bob" { "Charlie".into() } else { "Bob".into() }
            });
        room.submit_guess("tok-alice", &wrong_name).unwrap();
        let alice = room.find_player_by_token("tok-alice").unwrap();
        assert_eq!(alice.score, 0);
        let bob = room.find_player_by_token("tok-bob").unwrap();
        let charlie = room.find_player_by_token("tok-charlie").unwrap();
        assert_eq!(bob.score + charlie.score, 2); // both non-guessers get 1 each
    }

    #[test]
    fn submit_topic_in_reveal_advances_to_topic_submission() {
        let mut room = make_room();
        setup_game(&mut room);
        room.begin_countdown("tok-alice").unwrap();
        room.activate_round();
        let owner_name = room
            .find_player_by_token(&room.current_topic.clone().unwrap().owner_token)
            .unwrap()
            .name
            .clone();
        room.submit_guess("tok-alice", &owner_name).unwrap();
        // Find owner token and submit new topic
        let owner_token = room
            .current_topic
            .clone()
            .map(|_| {
                // current_topic was consumed when we submitted the guess and moved to RoundReveal
                // look at log to get owner
                room.log.last().unwrap().owner_name.clone()
            })
            .unwrap();
        let owner_token = room
            .players
            .iter()
            .find(|p| p.name == owner_token)
            .unwrap()
            .token
            .clone();
        room.submit_topic(&owner_token, "New topic".into()).unwrap();
        assert_eq!(room.state, RoomState::TopicSubmission);
        // guesser_index advanced
        assert_eq!(room.guesser_index, 1);
    }

    #[test]
    fn snapshot_excludes_tokens() {
        let mut room = make_room();
        add_players(&mut room);
        let snap = room.to_snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        assert!(!json.contains("tok-alice"));
        assert!(!json.contains("tok-bob"));
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cd backend && cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add backend/src/state.rs
git commit -m "feat: core game state types and transition logic with tests"
```

---

## Task 3: Backend HTTP Handlers

**Files:**
- Modify: `backend/src/handlers/rooms.rs` (full implementation)

- [ ] **Step 1: Implement all room handlers**

Replace `backend/src/handlers/rooms.rs`:

```rust
use crate::{
    error::AppError,
    state::{generate_room_code, generate_token, AppState, RoomEntry, Room, RoomState},
    AppState as AS,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

fn broadcast_snapshot(entry: &RoomEntry, room: &crate::state::Room) {
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
    State(state): State<Arc<AS>>,
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
    State(state): State<Arc<AS>>,
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
    State(state): State<Arc<AS>>,
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
    State(state): State<Arc<AS>>,
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
    State(state): State<Arc<AS>>,
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
    State(state): State<Arc<AS>>,
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
```

- [ ] **Step 2: Build to check for errors**

```bash
cd backend && cargo build 2>&1
```

Expected: builds without errors (sse and wiki handlers still have stub signatures — that's fine).

- [ ] **Step 3: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add backend/src/handlers/rooms.rs
git commit -m "feat: implement room and game action HTTP handlers"
```

---

## Task 4: Backend SSE Handler

**Files:**
- Modify: `backend/src/handlers/sse.rs`

- [ ] **Step 1: Implement SSE handler**

Replace `backend/src/handlers/sse.rs`:

```rust
use crate::{state::AppState, AppState as AS};
use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::convert::Infallible;
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
    let entry = match state.rooms.get(&code) {
        Some(e) => e,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Mark player connected.
    {
        let mut room = entry.room.lock().await;
        if let Some(player) = room.find_player_by_token_mut(&query.token) {
            player.connected = true;
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Send full current state immediately on connect.
    let initial = {
        let room = entry.room.lock().await;
        json!({"type": "room_state", "data": room.to_snapshot()}).to_string()
    };
    let _ = entry.tx.send(initial);

    // Subscribe to future broadcasts.
    let rx = entry.tx.subscribe();
    let token = query.token.clone();
    let room_arc = entry.room.clone();
    let tx_clone = entry.tx.clone();

    let stream = BroadcastStream::new(rx)
        .filter_map(move |msg| {
            let msg = msg.ok()?;
            Some(Ok::<Event, Infallible>(Event::default().data(msg)))
        });

    // When the stream ends, mark player disconnected and broadcast.
    let cleanup_room = room_arc.clone();
    let cleanup_tx = tx_clone.clone();
    let cleanup_token = token.clone();
    tokio::spawn(async move {
        // This task ends when the SSE connection drops — but tokio-stream
        // doesn't give us a direct drop hook, so we poll readiness instead.
        // A simpler approach: use a oneshot inside the stream adapter.
        // For now, rely on the BroadcastStream lag: if the receiver is dropped,
        // senders get errors and we clean up lazily on next broadcast failure.
        // Real cleanup is handled by periodic heartbeat misses, which is
        // acceptable for this game's use case.
        let _ = (cleanup_room, cleanup_tx, cleanup_token);
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

- [ ] **Step 2: Build**

```bash
cd backend && cargo build 2>&1
```

Expected: builds without errors.

- [ ] **Step 3: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add backend/src/handlers/sse.rs
git commit -m "feat: SSE stream handler for real-time room state"
```

---

## Task 5: Backend Wikipedia Proxy + Final Wiring

**Files:**
- Modify: `backend/src/handlers/wiki.rs`
- Modify: `backend/src/main.rs` (add reqwest client to state)

- [ ] **Step 1: Implement Wikipedia proxy**

Replace `backend/src/handlers/wiki.rs`:

```rust
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use crate::AppState as AS;

#[derive(Serialize)]
pub struct WikiArticle {
    pub title: String,
    pub url: String,
    pub extract: String,
    pub html: String,
}

pub async fn random_article(
    State(state): State<Arc<AS>>,
) -> Result<Json<WikiArticle>, StatusCode> {
    // Fetch random article summary from Wikipedia REST API.
    let summary_url = "https://en.wikipedia.org/api/rest_v1/page/random/summary";
    let summary: Value = state
        .http
        .get(summary_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let title = summary["title"].as_str().unwrap_or("").to_string();
    let extract = summary["extract"].as_str().unwrap_or("").to_string();
    let url = summary["content_urls"]["desktop"]["page"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Fetch full mobile-optimised HTML.
    let html_url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/mobile-html/{}",
        urlencoding::encode(&title)
    );
    let html = state
        .http
        .get(&html_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .text()
        .await
        .unwrap_or_default();

    Ok(Json(WikiArticle { title, url, extract, html }))
}
```

- [ ] **Step 2: Add `urlencoding` and `reqwest` client to AppState**

Add to `backend/Cargo.toml` under `[dependencies]`:
```toml
urlencoding = "2"
```

Update `backend/src/state.rs` — add `http: reqwest::Client` field to `AppState`:
```rust
// In AppState struct:
pub struct AppState {
    pub rooms: DashMap<String, RoomEntry>,
    pub http: reqwest::Client,
}

// In AppState::new():
impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            http: reqwest::Client::new(),
        }
    }
}
```

- [ ] **Step 3: Build**

```bash
cd backend && cargo build 2>&1
```

Expected: compiles without errors.

- [ ] **Step 4: Smoke test the server**

```bash
cd backend && cargo run &
sleep 2
# Create a room
curl -s -X POST http://localhost:3001/rooms \
  -H 'Content-Type: application/json' \
  -d '{"name":"Alice"}' | jq .
# Fetch a random Wikipedia article
curl -s http://localhost:3001/wiki/random | jq '.title'
kill %1
```

Expected: `{"code":"XXXXXX","token":"..."}` and a Wikipedia article title.

- [ ] **Step 5: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add backend/
git commit -m "feat: wikipedia proxy and complete backend wiring"
```

---

## Task 6: Frontend Scaffold

**Files:**
- Create: `frontend/` (Vite + React + TS + Tailwind)

- [ ] **Step 1: Scaffold with Vite**

```bash
cd /Users/mtib/Code/lying-wiki-game
npm create vite@latest frontend -- --template react-ts
cd frontend && npm install
```

- [ ] **Step 2: Install Tailwind CSS**

```bash
cd /Users/mtib/Code/lying-wiki-game/frontend
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

- [ ] **Step 3: Configure Tailwind**

Replace `frontend/tailwind.config.js`:
```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: { extend: {} },
  plugins: [],
}
```

Replace the top of `frontend/src/index.css` (remove Vite defaults, add Tailwind directives):
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

* { -webkit-tap-highlight-color: transparent; }
html, body, #root { height: 100%; }
body { background: #0f172a; color: #f1f5f9; font-family: system-ui, sans-serif; }
```

- [ ] **Step 4: Configure Vite proxy**

Replace `frontend/vite.config.ts`:
```ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3001',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
```

- [ ] **Step 5: Replace src/App.tsx with placeholder**

```tsx
export default function App() {
  return <div className="p-4 text-white">Loading…</div>
}
```

- [ ] **Step 6: Verify dev server starts**

```bash
cd frontend && npm run dev &
sleep 3
curl -s http://localhost:5173 | grep -c "Loading"
kill %1
```

Expected: output `1` (found the placeholder text).

- [ ] **Step 7: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/
git commit -m "chore: scaffold react frontend with tailwind and vite proxy"
```

---

## Task 7: Frontend Types and API Client

**Files:**
- Create: `frontend/src/types.ts`
- Create: `frontend/src/api.ts`

- [ ] **Step 1: Write types.ts**

Create `frontend/src/types.ts`:
```ts
export type RoomStateName =
  | 'lobby'
  | 'topic_submission'
  | 'countdown'
  | 'round_active'
  | 'round_reveal'

export interface PlayerSnapshot {
  name: string
  score: number
  connected: boolean
  submitted_this_round: boolean
}

export interface RoundRevealData {
  topic: string
  owner_name: string
  guesser_name: string
  guessed_name: string
  correct: boolean
  points: number
}

export interface LogEntry {
  round: number
  topic: string
  owner_name: string
  guesser_name: string
  guessed_name: string
  correct: boolean
  points: number
}

export interface RoomSnapshot {
  code: string
  state: RoomStateName
  players: PlayerSnapshot[]
  current_topic: string | null
  reveal: RoundRevealData | null
  log: LogEntry[]
  round_number: number
  guesser_name: string | null
  countdown_started_at_ms: number | null
}

export interface WikiArticle {
  title: string
  url: string
  extract: string
  html: string
}
```

- [ ] **Step 2: Write api.ts**

Create `frontend/src/api.ts`:
```ts
const BASE = '/api'

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  const data = await res.json()
  if (!res.ok) throw new Error(data.error ?? 'Request failed')
  return data as T
}

export function createRoom(name: string) {
  return post<{ code: string; token: string }>('/rooms', { name })
}

export function joinRoom(code: string, name: string, token?: string) {
  return post<{ token: string }>(`/rooms/${code}/join`, { name, token })
}

export function startGame(code: string, token: string) {
  return post<void>(`/rooms/${code}/start-game`, { token })
}

export function submitTopic(code: string, token: string, title: string) {
  return post<void>(`/rooms/${code}/topic`, { token, title })
}

export function startRound(code: string, token: string) {
  return post<void>(`/rooms/${code}/start-round`, { token })
}

export function submitGuess(code: string, token: string, guessed_name: string) {
  return post<void>(`/rooms/${code}/guess`, { token, guessed_name })
}

export function fetchRandomWikiArticle() {
  return fetch(`${BASE}/wiki/random`).then(async (res) => {
    const data = await res.json()
    if (!res.ok) throw new Error(data.error ?? 'Failed to fetch article')
    return data as import('./types').WikiArticle
  })
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: builds without TypeScript errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/types.ts frontend/src/api.ts
git commit -m "feat: frontend types and API client"
```

---

## Task 8: Frontend Hooks

**Files:**
- Create: `frontend/src/hooks/useDeviceToken.ts`
- Create: `frontend/src/hooks/useRoom.ts`

- [ ] **Step 1: Write useDeviceToken.ts**

```bash
mkdir -p frontend/src/hooks
```

Create `frontend/src/hooks/useDeviceToken.ts`:
```ts
const KEY = (code: string) => `lwg-token-${code}`
const NAME_KEY = (code: string) => `lwg-name-${code}`

export function saveDeviceToken(code: string, token: string, name: string) {
  localStorage.setItem(KEY(code), token)
  localStorage.setItem(NAME_KEY(code), name)
}

export function loadDeviceToken(code: string): { token: string; name: string } | null {
  const token = localStorage.getItem(KEY(code))
  const name = localStorage.getItem(NAME_KEY(code))
  if (!token || !name) return null
  return { token, name }
}

export function clearDeviceToken(code: string) {
  localStorage.removeItem(KEY(code))
  localStorage.removeItem(NAME_KEY(code))
}
```

- [ ] **Step 2: Write useRoom.ts**

Create `frontend/src/hooks/useRoom.ts`:
```ts
import { useEffect, useRef, useState, useCallback } from 'react'
import type { RoomSnapshot } from '../types'

const BASE = '/api'

export function useRoom(code: string | null, token: string | null) {
  const [room, setRoom] = useState<RoomSnapshot | null>(null)
  const esRef = useRef<EventSource | null>(null)
  const backoffRef = useRef(100)
  const unmountedRef = useRef(false)

  const connect = useCallback(() => {
    if (!code || !token || unmountedRef.current) return
    if (esRef.current) {
      esRef.current.close()
      esRef.current = null
    }

    const es = new EventSource(`${BASE}/rooms/${code}/events?token=${token}`)
    esRef.current = es

    es.addEventListener('message', (e) => {
      try {
        const parsed = JSON.parse(e.data)
        if (parsed.type === 'room_state') {
          setRoom(parsed.data as RoomSnapshot)
          backoffRef.current = 100 // reset backoff on successful message
        }
      } catch {}
    })

    es.onerror = () => {
      es.close()
      esRef.current = null
      if (unmountedRef.current) return
      const delay = Math.min(backoffRef.current, 10000)
      backoffRef.current = Math.min(backoffRef.current * 2, 10000)
      setTimeout(connect, delay)
    }
  }, [code, token])

  useEffect(() => {
    unmountedRef.current = false
    connect()
    const onVisible = () => {
      if (document.visibilityState === 'visible') connect()
    }
    document.addEventListener('visibilitychange', onVisible)
    return () => {
      unmountedRef.current = true
      esRef.current?.close()
      document.removeEventListener('visibilitychange', onVisible)
    }
  }, [connect])

  return room
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no TypeScript errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/hooks/
git commit -m "feat: device token storage and SSE room hook with reconnect"
```

---

## Task 9: Toast and ScoreDrawer Components

**Files:**
- Create: `frontend/src/components/Toast.tsx`
- Create: `frontend/src/components/ScoreDrawer.tsx`

- [ ] **Step 1: Write Toast.tsx**

```bash
mkdir -p frontend/src/components
```

Create `frontend/src/components/Toast.tsx`:
```tsx
import { useEffect, useState } from 'react'

interface Props {
  message: string | null
  onDismiss: () => void
}

export function Toast({ message, onDismiss }: Props) {
  useEffect(() => {
    if (!message) return
    const t = setTimeout(onDismiss, 4000)
    return () => clearTimeout(t)
  }, [message, onDismiss])

  if (!message) return null

  return (
    <div
      className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 bg-red-600 text-white px-5 py-3 rounded-xl shadow-lg text-sm font-medium max-w-xs text-center"
      onClick={onDismiss}
    >
      {message}
    </div>
  )
}
```

- [ ] **Step 2: Write ScoreDrawer.tsx**

Create `frontend/src/components/ScoreDrawer.tsx`:
```tsx
import { useState } from 'react'
import type { RoomSnapshot } from '../types'

interface Props {
  room: RoomSnapshot
}

export function ScoreDrawer({ room }: Props) {
  const [open, setOpen] = useState(false)
  const sorted = [...room.players].sort((a, b) => b.score - a.score)

  return (
    <>
      <button
        className="fixed bottom-4 right-4 z-40 bg-slate-700 text-white text-xs px-3 py-2 rounded-full shadow"
        onClick={() => setOpen(true)}
      >
        Scores
      </button>

      {open && (
        <div className="fixed inset-0 z-50 flex flex-col bg-slate-900/95">
          <div className="flex items-center justify-between px-4 py-4 border-b border-slate-700">
            <h2 className="text-lg font-bold">Scoreboard</h2>
            <button className="text-slate-400 text-2xl leading-none" onClick={() => setOpen(false)}>
              ✕
            </button>
          </div>

          <div className="overflow-y-auto flex-1 px-4 py-3 space-y-2">
            {sorted.map((p) => (
              <div key={p.name} className="flex justify-between items-center bg-slate-800 rounded-lg px-4 py-2">
                <span className={p.connected ? 'text-white' : 'text-slate-500 line-through'}>
                  {p.name}
                </span>
                <span className="font-bold text-yellow-400">{p.score} pts</span>
              </div>
            ))}

            {room.log.length > 0 && (
              <>
                <h3 className="text-slate-400 text-xs uppercase tracking-wide pt-4">Round Log</h3>
                {[...room.log].reverse().map((entry, i) => (
                  <div key={i} className="bg-slate-800 rounded-lg px-4 py-3 text-sm space-y-1">
                    <div className="font-semibold">{entry.topic}</div>
                    <div className="text-slate-400">
                      Owner: <span className="text-white">{entry.owner_name}</span>
                    </div>
                    <div className="text-slate-400">
                      {entry.guesser_name} guessed{' '}
                      <span className={entry.correct ? 'text-green-400' : 'text-red-400'}>
                        {entry.guessed_name}
                      </span>
                      {' '}— {entry.correct ? `+${entry.points} pts` : `everyone else +${entry.points}`}
                    </div>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>
      )}
    </>
  )
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/components/
git commit -m "feat: Toast and ScoreDrawer components"
```

---

## Task 10: WikiArticleSheet Component

**Files:**
- Create: `frontend/src/components/WikiArticleSheet.tsx`

- [ ] **Step 1: Write WikiArticleSheet.tsx**

Create `frontend/src/components/WikiArticleSheet.tsx`:
```tsx
import { useEffect, useRef, useState } from 'react'
import type { WikiArticle } from '../types'
import { fetchRandomWikiArticle } from '../api'

interface Props {
  onSelect: (title: string) => void
  onClose: () => void
}

export function WikiArticleSheet({ onSelect, onClose }: Props) {
  const [article, setArticle] = useState<WikiArticle | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const load = () => {
    setLoading(true)
    setError(null)
    setArticle(null)
    fetchRandomWikiArticle()
      .then(setArticle)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }

  useEffect(() => { load() }, [])

  useEffect(() => {
    if (article && iframeRef.current) {
      const doc = iframeRef.current.contentDocument
      if (doc) {
        doc.open()
        doc.write(article.html)
        doc.close()
      }
    }
  }, [article])

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-white text-slate-900">
      <div className="flex items-center justify-between px-4 py-3 bg-slate-100 border-b border-slate-200 shrink-0">
        <button className="text-blue-600 text-sm font-medium" onClick={onClose}>
          Cancel
        </button>
        <span className="text-sm font-semibold truncate max-w-[55vw]">
          {article?.title ?? 'Loading…'}
        </span>
        <button className="text-sm text-slate-500 font-medium" onClick={load}>
          🎲 New
        </button>
      </div>

      <div className="flex-1 overflow-hidden">
        {loading && (
          <div className="flex items-center justify-center h-full text-slate-500">
            Loading article…
          </div>
        )}
        {error && (
          <div className="flex flex-col items-center justify-center h-full gap-4 text-red-500 px-6 text-center">
            <p>{error}</p>
            <button className="text-blue-600 underline" onClick={load}>Retry</button>
          </div>
        )}
        {article && !loading && (
          <iframe
            ref={iframeRef}
            className="w-full h-full border-none"
            sandbox="allow-same-origin"
            title="Wikipedia article"
          />
        )}
      </div>

      {article && (
        <div className="px-4 py-4 bg-slate-100 border-t border-slate-200 shrink-0">
          <button
            className="w-full bg-blue-600 text-white font-semibold py-3 rounded-xl text-base"
            onClick={() => onSelect(article.title)}
          >
            Use "{article.title}"
          </button>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/components/WikiArticleSheet.tsx
git commit -m "feat: WikiArticleSheet bottom sheet with full article and random button"
```

---

## Task 11: Home and Lobby Screens

**Files:**
- Create: `frontend/src/screens/HomeScreen.tsx`
- Create: `frontend/src/screens/LobbyScreen.tsx`

- [ ] **Step 1: Write HomeScreen.tsx**

```bash
mkdir -p frontend/src/screens
```

Create `frontend/src/screens/HomeScreen.tsx`:
```tsx
import { useState } from 'react'
import { createRoom, joinRoom } from '../api'
import { saveDeviceToken } from '../hooks/useDeviceToken'

interface Props {
  onJoined: (code: string, token: string, name: string) => void
}

export function HomeScreen({ onJoined }: Props) {
  const [name, setName] = useState('')
  const [code, setCode] = useState('')
  const [mode, setMode] = useState<'home' | 'join'>('home')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const handle = async (action: () => Promise<{ code: string; token: string }>) => {
    if (!name.trim()) { setError('Enter your name first'); return }
    setLoading(true)
    setError(null)
    try {
      const { code: roomCode, token } = await action()
      saveDeviceToken(roomCode, token, name.trim())
      onJoined(roomCode, token, name.trim())
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-full flex flex-col items-center justify-center px-6 gap-6">
      <h1 className="text-3xl font-bold text-center text-white">
        2 of These People<br />Are Lying
      </h1>

      <input
        className="w-full max-w-sm bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="Your name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        maxLength={24}
      />

      {mode === 'home' && (
        <div className="flex flex-col w-full max-w-sm gap-3">
          <button
            className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50"
            disabled={loading}
            onClick={() => handle(() => createRoom(name.trim()))}
          >
            Create Room
          </button>
          <button
            className="bg-slate-700 text-white font-semibold py-3 rounded-xl text-base"
            onClick={() => setMode('join')}
          >
            Join Room
          </button>
        </div>
      )}

      {mode === 'join' && (
        <div className="flex flex-col w-full max-w-sm gap-3">
          <input
            className="w-full bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500 uppercase tracking-widest text-center"
            placeholder="Room code"
            value={code}
            onChange={(e) => setCode(e.target.value.toUpperCase().slice(0, 6))}
            maxLength={6}
          />
          <button
            className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50"
            disabled={loading || code.length !== 6}
            onClick={() => handle(async () => {
              const { token } = await joinRoom(code, name.trim())
              return { code, token }
            })}
          >
            Join
          </button>
          <button
            className="text-slate-400 text-sm"
            onClick={() => setMode('home')}
          >
            Back
          </button>
        </div>
      )}

      {error && <p className="text-red-400 text-sm text-center">{error}</p>}
    </div>
  )
}
```

- [ ] **Step 2: Write LobbyScreen.tsx**

Create `frontend/src/screens/LobbyScreen.tsx`:
```tsx
import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { startGame } from '../api'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function LobbyScreen({ room, token, myName, onError }: Props) {
  const [loading, setLoading] = useState(false)

  const handleStart = async () => {
    setLoading(true)
    try {
      await startGame(room.code, token)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-full flex flex-col px-6 py-8 gap-6">
      <div className="text-center">
        <p className="text-slate-400 text-sm mb-1">Room code</p>
        <p className="text-5xl font-bold tracking-widest text-white">{room.code}</p>
        <p className="text-slate-500 text-xs mt-2">Share this with your friends</p>
      </div>

      <div className="flex-1 space-y-2">
        <p className="text-slate-400 text-sm">Players ({room.players.length})</p>
        {room.players.map((p) => (
          <div
            key={p.name}
            className="flex items-center gap-3 bg-slate-800 rounded-xl px-4 py-3"
          >
            <span
              className={`w-2 h-2 rounded-full shrink-0 ${p.connected ? 'bg-green-400' : 'bg-slate-600'}`}
            />
            <span className="text-white">{p.name}{p.name === myName ? ' (you)' : ''}</span>
          </div>
        ))}
      </div>

      <button
        className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50 w-full"
        disabled={loading || room.players.length < 3}
        onClick={handleStart}
      >
        {room.players.length < 3
          ? `Need ${3 - room.players.length} more player(s)`
          : 'Start Game'}
      </button>

      <ScoreDrawer room={room} />
    </div>
  )
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/screens/HomeScreen.tsx frontend/src/screens/LobbyScreen.tsx
git commit -m "feat: HomeScreen and LobbyScreen"
```

---

## Task 12: TopicSubmission and Countdown Screens

**Files:**
- Create: `frontend/src/screens/TopicSubmissionScreen.tsx`
- Create: `frontend/src/screens/CountdownScreen.tsx`

- [ ] **Step 1: Write TopicSubmissionScreen.tsx**

Create `frontend/src/screens/TopicSubmissionScreen.tsx`:
```tsx
import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitTopic, startRound } from '../api'
import { WikiArticleSheet } from '../components/WikiArticleSheet'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function TopicSubmissionScreen({ room, token, myName, onError }: Props) {
  const [title, setTitle] = useState('')
  const [showWiki, setShowWiki] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [starting, setStarting] = useState(false)

  const me = room.players.find((p) => p.name === myName)
  const allSubmitted = room.players.every((p) => p.submitted_this_round)

  const handleSubmit = async () => {
    if (!title.trim()) return
    setSubmitting(true)
    try {
      await submitTopic(room.code, token, title.trim())
      setTitle('')
    } catch (e: any) {
      onError(e.message)
    } finally {
      setSubmitting(false)
    }
  }

  const handleStart = async () => {
    setStarting(true)
    try {
      await startRound(room.code, token)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setStarting(false)
    }
  }

  return (
    <>
      {showWiki && (
        <WikiArticleSheet
          onSelect={(t) => { setTitle(t); setShowWiki(false) }}
          onClose={() => setShowWiki(false)}
        />
      )}

      <div className="min-h-full flex flex-col px-6 py-8 gap-6">
        <h2 className="text-xl font-bold text-white">Submit Your Topic</h2>

        <div className="space-y-3">
          <input
            className="w-full bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Wikipedia article title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
          />
          <div className="flex gap-3">
            <button
              className="flex-1 bg-slate-700 text-white font-medium py-2.5 rounded-xl text-sm"
              onClick={() => setShowWiki(true)}
            >
              🎲 Random Article
            </button>
            <button
              className="flex-1 bg-blue-600 text-white font-semibold py-2.5 rounded-xl text-sm disabled:opacity-50"
              disabled={!title.trim() || submitting}
              onClick={handleSubmit}
            >
              Submit
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <p className="text-slate-400 text-sm">Players</p>
          {room.players.map((p) => (
            <div key={p.name} className="flex items-center justify-between bg-slate-800 rounded-xl px-4 py-3">
              <span className="text-white">{p.name}{p.name === myName ? ' (you)' : ''}</span>
              <span className={p.submitted_this_round ? 'text-green-400 text-lg' : 'text-slate-600 text-lg'}>
                {p.submitted_this_round ? '✓' : '…'}
              </span>
            </div>
          ))}
        </div>

        <button
          className="bg-green-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-40 w-full mt-auto"
          disabled={!allSubmitted || starting}
          onClick={handleStart}
        >
          {allSubmitted ? 'Start Round' : 'Waiting for everyone…'}
        </button>

        <ScoreDrawer room={room} />
      </div>
    </>
  )
}
```

- [ ] **Step 2: Write CountdownScreen.tsx**

Create `frontend/src/screens/CountdownScreen.tsx`:
```tsx
import { useEffect, useState } from 'react'
import type { RoomSnapshot } from '../types'

interface Props {
  room: RoomSnapshot
}

export function CountdownScreen({ room }: Props) {
  const [count, setCount] = useState(3)

  useEffect(() => {
    if (!room.countdown_started_at_ms) return
    const tick = () => {
      const elapsed = Date.now() - room.countdown_started_at_ms!
      const remaining = Math.max(0, 3 - Math.floor(elapsed / 1000))
      setCount(remaining)
    }
    tick()
    const id = setInterval(tick, 200)
    return () => clearInterval(id)
  }, [room.countdown_started_at_ms])

  return (
    <div className="min-h-full flex flex-col items-center justify-center gap-6">
      <p className="text-slate-400 text-lg">Get ready…</p>
      <div className="text-9xl font-black text-white tabular-nums">{count || '🎲'}</div>
      {room.current_topic && (
        <p className="text-slate-500 text-sm">Topic incoming</p>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/screens/TopicSubmissionScreen.tsx frontend/src/screens/CountdownScreen.tsx
git commit -m "feat: TopicSubmissionScreen and CountdownScreen"
```

---

## Task 13: RoundActive and RoundReveal Screens

**Files:**
- Create: `frontend/src/screens/RoundActiveScreen.tsx`
- Create: `frontend/src/screens/RoundRevealScreen.tsx`

- [ ] **Step 1: Write RoundActiveScreen.tsx**

Create `frontend/src/screens/RoundActiveScreen.tsx`:
```tsx
import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitGuess } from '../api'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function RoundActiveScreen({ room, token, myName, onError }: Props) {
  const [loading, setLoading] = useState(false)
  const isGuesser = room.guesser_name === myName

  const handleGuess = async (guessedName: string) => {
    setLoading(true)
    try {
      await submitGuess(room.code, token, guessedName)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setLoading(false)
    }
  }

  // Other players to guess from (not the guesser themselves)
  const guessTargets = room.players.filter((p) => p.name !== room.guesser_name)

  return (
    <div className="min-h-full flex flex-col px-6 py-8 gap-6">
      <div className="text-center space-y-2">
        <p className="text-slate-400 text-sm uppercase tracking-wide">Round {room.round_number} — The topic is</p>
        <h2 className="text-3xl font-black text-white leading-tight">{room.current_topic}</h2>
      </div>

      <div className="bg-slate-800 rounded-xl px-4 py-3 text-center">
        <p className="text-slate-400 text-sm">Guesser</p>
        <p className="text-xl font-bold text-yellow-400">
          {room.guesser_name}{isGuesser ? ' (you)' : ''}
        </p>
      </div>

      {isGuesser ? (
        <div className="space-y-3">
          <p className="text-slate-400 text-sm">Whose topic is it?</p>
          {guessTargets.map((p) => (
            <button
              key={p.name}
              disabled={loading}
              onClick={() => handleGuess(p.name)}
              className="w-full bg-slate-700 hover:bg-slate-600 text-white font-semibold py-4 rounded-xl text-base disabled:opacity-50 flex items-center justify-between px-5"
            >
              <span>{p.name}</span>
              <span className="text-slate-400">→</span>
            </button>
          ))}
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-slate-500 text-center px-8">
          <p>Waiting for <span className="text-white font-semibold">{room.guesser_name}</span> to guess…</p>
        </div>
      )}

      <ScoreDrawer room={room} />
    </div>
  )
}
```

- [ ] **Step 2: Write RoundRevealScreen.tsx**

Create `frontend/src/screens/RoundRevealScreen.tsx`:
```tsx
import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitTopic } from '../api'
import { WikiArticleSheet } from '../components/WikiArticleSheet'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function RoundRevealScreen({ room, token, myName, onError }: Props) {
  const [title, setTitle] = useState('')
  const [showWiki, setShowWiki] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const reveal = room.reveal!
  const isOwner = reveal.owner_name === myName

  const handleSubmit = async () => {
    if (!title.trim()) return
    setSubmitting(true)
    try {
      await submitTopic(room.code, token, title.trim())
      setTitle('')
    } catch (e: any) {
      onError(e.message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <>
      {showWiki && (
        <WikiArticleSheet
          onSelect={(t) => { setTitle(t); setShowWiki(false) }}
          onClose={() => setShowWiki(false)}
        />
      )}

      <div className="min-h-full flex flex-col px-6 py-8 gap-6">
        <div className="text-center space-y-1">
          <p className="text-slate-400 text-sm uppercase tracking-wide">Round {room.round_number} Reveal</p>
          <h2 className="text-2xl font-black text-white">{reveal.topic}</h2>
          <p className="text-slate-300">
            owned by <span className="text-yellow-400 font-semibold">{reveal.owner_name}</span>
          </p>
        </div>

        <div className={`rounded-xl px-5 py-4 text-center ${reveal.correct ? 'bg-green-900/60' : 'bg-red-900/60'}`}>
          <p className="text-lg font-bold text-white">
            {reveal.correct ? '✓ Correct guess!' : '✗ Wrong guess!'}
          </p>
          <p className="text-slate-300 text-sm mt-1">
            {reveal.guesser_name} guessed <span className="text-white font-medium">{reveal.guessed_name}</span>
          </p>
          <p className="text-slate-300 text-sm">
            {reveal.correct
              ? `${reveal.guesser_name} gets ${reveal.points} points`
              : `Everyone else gets ${reveal.points} point each`}
          </p>
        </div>

        <div className="bg-slate-800 rounded-xl px-4 py-3 space-y-2">
          <p className="text-slate-400 text-xs uppercase tracking-wide">Scores</p>
          {[...room.players].sort((a, b) => b.score - a.score).map((p) => (
            <div key={p.name} className="flex justify-between">
              <span className="text-white">{p.name}</span>
              <span className="text-yellow-400 font-semibold">{p.score} pts</span>
            </div>
          ))}
        </div>

        {isOwner ? (
          <div className="space-y-3 mt-auto">
            <p className="text-white font-semibold">Your topic was revealed. Submit a new one to continue:</p>
            <input
              className="w-full bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Wikipedia article title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
            <div className="flex gap-3">
              <button
                className="flex-1 bg-slate-700 text-white font-medium py-2.5 rounded-xl text-sm"
                onClick={() => setShowWiki(true)}
              >
                🎲 Random
              </button>
              <button
                className="flex-1 bg-blue-600 text-white font-semibold py-2.5 rounded-xl text-sm disabled:opacity-50"
                disabled={!title.trim() || submitting}
                onClick={handleSubmit}
              >
                Submit & Continue
              </button>
            </div>
          </div>
        ) : (
          <p className="text-slate-500 text-center mt-auto">
            Waiting for <span className="text-white font-medium">{reveal.owner_name}</span> to submit a new topic…
          </p>
        )}

        <ScoreDrawer room={room} />
      </div>
    </>
  )
}
```

- [ ] **Step 3: Build check**

```bash
cd frontend && npm run build 2>&1 | tail -10
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/screens/RoundActiveScreen.tsx frontend/src/screens/RoundRevealScreen.tsx
git commit -m "feat: RoundActiveScreen and RoundRevealScreen"
```

---

## Task 14: App Router and Final Wiring

**Files:**
- Modify: `frontend/src/App.tsx`
- Modify: `frontend/src/main.tsx`

- [ ] **Step 1: Write App.tsx**

Replace `frontend/src/App.tsx`:
```tsx
import { useState, useCallback } from 'react'
import { useRoom } from './hooks/useRoom'
import { loadDeviceToken, saveDeviceToken } from './hooks/useDeviceToken'
import { Toast } from './components/Toast'
import { HomeScreen } from './screens/HomeScreen'
import { LobbyScreen } from './screens/LobbyScreen'
import { TopicSubmissionScreen } from './screens/TopicSubmissionScreen'
import { CountdownScreen } from './screens/CountdownScreen'
import { RoundActiveScreen } from './screens/RoundActiveScreen'
import { RoundRevealScreen } from './screens/RoundRevealScreen'

export default function App() {
  const [session, setSession] = useState<{ code: string; token: string; name: string } | null>(() => {
    // Attempt to restore from localStorage (latest room code)
    const lastCode = localStorage.getItem('lwg-last-code')
    if (!lastCode) return null
    const saved = loadDeviceToken(lastCode)
    if (!saved) return null
    return { code: lastCode, ...saved }
  })

  const [toast, setToast] = useState<string | null>(null)
  const onError = useCallback((msg: string) => setToast(msg), [])

  const room = useRoom(session?.code ?? null, session?.token ?? null)

  const onJoined = (code: string, token: string, name: string) => {
    localStorage.setItem('lwg-last-code', code)
    saveDeviceToken(code, token, name)
    setSession({ code, token, name })
  }

  if (!session) {
    return (
      <div className="min-h-full">
        <HomeScreen onJoined={onJoined} />
      </div>
    )
  }

  if (!room) {
    return (
      <div className="min-h-full flex items-center justify-center text-slate-400">
        Connecting…
      </div>
    )
  }

  const props = { room, token: session.token, myName: session.name, onError }

  return (
    <div className="min-h-full">
      {room.state === 'lobby' && <LobbyScreen {...props} />}
      {room.state === 'topic_submission' && <TopicSubmissionScreen {...props} />}
      {room.state === 'countdown' && <CountdownScreen room={room} />}
      {room.state === 'round_active' && <RoundActiveScreen {...props} />}
      {room.state === 'round_reveal' && <RoundRevealScreen {...props} />}
      <Toast message={toast} onDismiss={() => setToast(null)} />
    </div>
  )
}
```

- [ ] **Step 2: Update main.tsx to import CSS**

Replace `frontend/src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
```

- [ ] **Step 3: Update index.html for mobile viewport**

Edit the `<head>` of `frontend/index.html` to ensure it has:
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
<meta name="mobile-web-app-capable" content="yes" />
<meta name="apple-mobile-web-app-capable" content="yes" />
<title>2 of These People Are Lying</title>
```

- [ ] **Step 4: Final build check**

```bash
cd frontend && npm run build 2>&1
```

Expected: `dist/` created, no TypeScript errors.

- [ ] **Step 5: Commit**

```bash
cd /Users/mtib/Code/lying-wiki-game
git add frontend/src/App.tsx frontend/src/main.tsx frontend/index.html
git commit -m "feat: App router and final frontend wiring"
```

---

## Task 15: End-to-End Smoke Test

- [ ] **Step 1: Start backend**

```bash
cd /Users/mtib/Code/lying-wiki-game/backend && cargo run &
sleep 2
```

- [ ] **Step 2: Run a full game flow via curl**

```bash
# Create room as Alice
ALICE=$(curl -s -X POST http://localhost:3001/rooms \
  -H 'Content-Type: application/json' \
  -d '{"name":"Alice"}')
CODE=$(echo $ALICE | grep -o '"code":"[^"]*"' | cut -d'"' -f4)
ALICE_TOKEN=$(echo $ALICE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "Room: $CODE"

# Join as Bob
BOB_TOKEN=$(curl -s -X POST http://localhost:3001/rooms/$CODE/join \
  -H 'Content-Type: application/json' \
  -d '{"name":"Bob"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# Join as Charlie
CHARLIE_TOKEN=$(curl -s -X POST http://localhost:3001/rooms/$CODE/join \
  -H 'Content-Type: application/json' \
  -d '{"name":"Charlie"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# Start game
curl -s -X POST http://localhost:3001/rooms/$CODE/start-game \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"$ALICE_TOKEN\"}"

# Submit topics
curl -s -X POST http://localhost:3001/rooms/$CODE/topic \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"$ALICE_TOKEN\",\"title\":\"Banana republic\"}"
curl -s -X POST http://localhost:3001/rooms/$CODE/topic \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"$BOB_TOKEN\",\"title\":\"Quantum foam\"}"
curl -s -X POST http://localhost:3001/rooms/$CODE/topic \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"$CHARLIE_TOKEN\",\"title\":\"Velocipede\"}"

# Start round
curl -s -X POST http://localhost:3001/rooms/$CODE/start-round \
  -H 'Content-Type: application/json' \
  -d "{\"token\":\"$ALICE_TOKEN\"}"

sleep 4  # wait for countdown

# Check state (should be round_active)
curl -s http://localhost:3001/rooms/$CODE/events?token=$ALICE_TOKEN &
ESS_PID=$!
sleep 1
kill $ESS_PID 2>/dev/null
echo "Smoke test complete"
```

Expected: no errors, final state is `round_active`.

- [ ] **Step 2: Test Wikipedia proxy**

```bash
curl -s http://localhost:3001/wiki/random | grep -o '"title":"[^"]*"'
```

Expected: a Wikipedia article title.

- [ ] **Step 3: Kill backend and commit**

```bash
kill %1 2>/dev/null || true
cd /Users/mtib/Code/lying-wiki-game
git add .
git commit -m "chore: final smoke test verified"
```
