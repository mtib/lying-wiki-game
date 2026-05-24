# Lying Wiki Game — Design Spec

**Date:** 2026-05-24

## Overview

A mobile-first web app for playing Tom Scott's "2 of These People Are Lying" party game. Players join a shared room, each submits a Wikipedia article title, and one title is revealed per round. Players bluff that the revealed topic is theirs; the designated guesser tries to identify the real owner. Points are tracked in real time.

---

## Architecture

**Monorepo:**
```
lying-wiki-game/
  backend/   # Rust, Axum
  frontend/  # TypeScript, React (Vite)
```

**Backend:** Axum HTTP server. All game state lives in memory as `DashMap<RoomCode, Arc<Mutex<Room>>>`. No database. One binary serves the REST + SSE API. In development the frontend runs on its own Vite dev server; in production the Rust binary can serve the compiled frontend assets.

**Frontend:** React SPA (Vite). Communicates via:
- `POST` endpoints for all player actions
- One persistent SSE stream per player that delivers full room state snapshots on every change

**Reconnection:** On mount and on `visibilitychange`, the client checks the SSE connection and re-subscribes if closed. The server holds player seats indefinitely (for the lifetime of the server process). On reconnect the SSE endpoint immediately emits a full room state snapshot so the client snaps back to the correct screen.

**Wikipedia proxy:** The backend proxies requests to `https://en.wikipedia.org/api/rest_v1/` to avoid CORS issues on mobile clients.

---

## Player Identity

- Players enter a display name when creating or joining a room.
- On join, the server returns a **device token** (opaque random string).
- The token is persisted in `localStorage` keyed by room code.
- All subsequent requests include the token. The SSE stream is identified by token (`?token=...`).
- If a player closes the app and reopens it, they rejoin automatically using the stored token and receive a fresh SSE stream with current state.

---

## Game State Machine

```
Lobby → TopicSubmission → Countdown → RoundActive → RoundReveal → TopicSubmission → ...
```

### Lobby
- Players join with a 6-character alphanumeric room code.
- Room displays all connected players with live connected/disconnected indicators.
- Any player can press **Start Game** (requires ≥ 3 players).

### TopicSubmission
- Each player must confirm their topic for the round before play can begin.
- The previous round's topic **owner** must submit a new article title (their old topic was revealed; they cannot reuse it).
- All other players have their existing topic pre-confirmed automatically (`submitted_this_round = true`), but may change it before the round starts.
- The UI shows a checklist: players with a confirmed topic for this round show a green tick; the owner shows pending until they submit.
- Any player can press **Start Round** once all players are marked confirmed for this round.

### Countdown
- 3-second animated countdown shown to all players simultaneously.
- Transitions automatically to RoundActive.

### RoundActive
- **Topic selection:** The server picks uniformly at random from all submitted topics that do **not** belong to the current guesser.
- The selected topic title is displayed prominently to all players.
- The current guesser is highlighted by name.
- **If you are the guesser:** a list of all other players is shown; tap one to submit your guess.
- **If you are not the guesser:** waiting view with the topic displayed.
- Submitting a guess transitions to RoundReveal.

### RoundReveal
- Shows: topic title, true owner, guesser's pick, whether the guess was correct, points awarded.
- **Guesser correct:** guesser receives `n − 1` points (n = total players in room).
- **Guesser wrong:** every player except the guesser receives 1 point.
- The topic **owner** is prompted to submit a new topic to continue.
- Any other player sees a "waiting for [owner] to submit" message.
- Once the owner submits, transitions back to TopicSubmission.

### Guesser Order
- Guesser rotates in the order players joined the room (round-robin by join index).

---

## Data Model (In-Memory)

```rust
struct Room {
    code: String,
    players: Vec<Player>,
    guesser_index: usize,
    state: RoomState,
    current_topic: Option<RevealedTopic>,
    round_number: u32,
    log: Vec<LogEntry>,
}

struct Player {
    token: String,
    name: String,
    score: u32,
    connected: bool,
    submitted_topic: Option<String>,   // article title
    submitted_this_round: bool,
}

struct RevealedTopic {
    title: String,
    owner_token: String,
}

struct LogEntry {
    round: u32,
    topic: String,
    owner_name: String,
    guesser_name: String,
    guessed_name: String,
    correct: bool,
    points: u32,
}

enum RoomState {
    Lobby,
    TopicSubmission,
    Countdown { started_at: Instant },
    RoundActive,
    RoundReveal { correct: bool },
}
```

---

## API

### Room & Player Actions
| Method | Path | Body | Description |
|--------|------|------|-------------|
| `POST` | `/rooms` | `{ name }` | Create room. Returns `{ code, token }`. |
| `POST` | `/rooms/:code/join` | `{ name }` | Join room. Returns `{ token }`. |
| `POST` | `/rooms/:code/topic` | `{ token, title }` | Submit article title. |
| `POST` | `/rooms/:code/start-round` | `{ token }` | Start round (any player, all topics submitted). |
| `POST` | `/rooms/:code/guess` | `{ token, guessed_token }` | Guesser submits guess. |
| `POST` | `/rooms/:code/end-round` | `{ token }` | Advance from RoundReveal (topic owner submits new topic first). |

### SSE Stream
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/rooms/:code/events?token=...` | SSE stream. Emits `room_state` events. |

**SSE event format:**
```json
{ "type": "room_state", "data": { /* full Room snapshot, token fields redacted */ } }
```

The server emits a full snapshot on every state change and immediately on connection. No delta logic.

### Wikipedia Proxy
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/wiki/random` | Returns `{ title, url, extract, html }` for a random English Wikipedia article. |

---

## Frontend Screens

### Home
- Name input + "Create Room" / "Join Room" (enter 6-char code).
- Device token loaded from `localStorage` on mount; if found for a code, rejoin automatically.

### Lobby
- Room code displayed large for sharing.
- Player list with connected indicators.
- **Start Game** button (enabled when ≥ 3 players connected).

### Topic Submission
- Text field pre-filled with current topic if already submitted.
- **Random Article** button: fetches `/wiki/random`, opens a bottom sheet with the full article HTML rendered in a scroll view. "Use This Article" button fills the title field and closes the sheet.
- Submission checklist (all players, green tick when submitted).
- **Start Round** button (any player, visible when all submitted).

### Round Active
- Topic title displayed prominently.
- Guesser name highlighted ("It's [name]'s turn to guess").
- **Guesser view:** list of all other players as tappable cards.
- **Non-guesser view:** topic displayed, waiting indicator.
- Collapsed scoreboard drawer at bottom.

### Round Reveal
- Topic, true owner, guesser's pick, correct/wrong indicator, points awarded.
- If you are the topic owner: prompt to submit a new topic (inline text field or navigates to Topic Submission).
- If you are not the owner: "Waiting for [owner] to submit their next topic."

### Scoreboard Drawer
- Always accessible (swipe up or tap tab).
- Live scores table + full game log (round, topic, owner, guesser, result, points).

---

## SSE Reconnection Detail

- Client uses `EventSource` wrapped in a custom hook.
- On `error` event or `visibilitychange` to visible, client closes and reopens the `EventSource`.
- Exponential backoff (100 ms → 200 ms → 400 ms … cap 10 s) between reconnect attempts.
- Server removes stale SSE sender handles lazily (when a send fails).
- Player `connected` flag updated: set `true` when SSE stream opens, set `false` when all SSE handles for that token are closed/errored.

---

## Error Handling

- All POST endpoints return `{ error: string }` with an appropriate HTTP status on failure.
- Invalid/unknown token → 401.
- Action not valid for current room state → 409.
- Room not found → 404.
- Frontend shows a toast for error responses; non-blocking.

---

## Out of Scope

- Persistent storage / server-restart recovery
- Authentication beyond device tokens
- Game history across sessions
- Room expiry / cleanup (rooms live until the server restarts)
- Wikipedia article search
