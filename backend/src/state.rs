use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub rooms: DashMap<String, RoomEntry>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            http: reqwest::Client::new(),
        }
    }
}

pub struct RoomEntry {
    pub room: Arc<Mutex<Room>>,
    pub tx: broadcast::Sender<String>,
}

pub struct Room {
    pub code: String,
}
