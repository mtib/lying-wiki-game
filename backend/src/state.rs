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
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            http: reqwest::Client::builder()
                .user_agent("lying-wiki-game/0.1 (https://github.com/mtib/lying-wiki-game)")
                .build()
                .expect("failed to build http client"),
        }
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
            RoomState::RoundReveal(_) => {
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
        let wrong_name: String = room
            .players
            .iter()
            .find(|p| p.name != "Alice" && p.name != owner_name)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| {
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
        // After guess, current_topic is consumed. Use the log to find owner.
        let owner_name_from_log = room.log.last().unwrap().owner_name.clone();
        let owner_token = room
            .players
            .iter()
            .find(|p| p.name == owner_name_from_log)
            .unwrap()
            .token
            .clone();
        // State is RoundReveal. Set current_topic for the submit to use.
        // Actually: current_topic was cleared in submit_guess? Let's check.
        // Looking at submit_guess: it clones topic but doesn't clear current_topic.
        // current_topic is cleared in submit_topic when in RoundReveal state.
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
