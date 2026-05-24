# 2 of These People Are Lying

A mobile-first web app for playing Tom Scott's party game. Players join a shared room, each secretly submits a Wikipedia article title, and one title is revealed each round. Everyone bluffs that the revealed topic is theirs — the designated guesser tries to spot the real owner.

## How to play

1. One person creates a room and shares the 6-character code (or QR code) with everyone.
2. All players join on their phones.
3. Each player submits a Wikipedia article title — use the 🎲 button to browse a random article.
4. When everyone has submitted, any player taps **Start Round**.
5. A topic is revealed. Players bluff. The guesser taps who they think owns it.
6. Points are awarded:
   - Correct guess → guesser gets **n−1** points
   - Wrong guess → everyone else gets **1** point
7. The topic owner submits a new article and play continues.

## Tech stack

| Layer | Technology |
|---|---|
| Backend | Rust · Axum · tokio · DashMap |
| Frontend | TypeScript · React 18 · Vite · Tailwind CSS 3 |
| Real-time | Server-Sent Events (SSE) |
| State | In-memory (no database) |

## Development

### Prerequisites

- Rust 1.77+ (`rustup update stable`)
- Node.js 22+ and npm

### Run locally

**Backend** (port 3001):
```bash
cd backend
cargo run
```

**Frontend** (port 5173, proxies `/api` → backend):
```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in multiple browser tabs to test multiplayer.

### Run tests

```bash
# Backend unit tests
cd backend && cargo test

# Frontend type check + build
cd frontend && npm run build
```

## Deployment

The app is stateless — server restart clears all rooms. It's designed as a single-host party tool.

### Build

```bash
# Backend binary
cd backend && cargo build --release
# Binary: backend/target/release/lying-wiki-game

# Frontend static assets
cd frontend && npm run build
# Assets: frontend/dist/
```

### Serve with a reverse proxy (e.g. Caddy)

Serve the frontend `dist/` as static files and proxy `/api/*` to the backend:

```caddy
yourdomain.com {
    handle /api/* {
        rewrite * {path}
        reverse_proxy localhost:3001 {
            header_up X-Real-IP {remote_host}
        }
    }
    handle {
        root * /path/to/frontend/dist
        try_files {path} /index.html
        file_server
    }
}
```

Set the backend to strip `/api` when it receives requests, or use a path prefix in the Caddy config that rewrites before forwarding (the current backend listens on `/rooms`, `/wiki` — no `/api` prefix).

### Docker (single container example)

```dockerfile
# backend/Dockerfile
FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lying-wiki-game /usr/local/bin/lying-wiki-game
COPY --from=builder /app/frontend-dist /usr/share/lying-wiki-game/static
EXPOSE 3001
CMD ["lying-wiki-game"]
```

### Environment

The backend binds to `0.0.0.0:3001` by default. CORS is currently open (`*`) — tighten this for production by setting allowed origins in `backend/src/main.rs`.

## Notes

- Game state lives in memory — a server restart ends all active games.
- Wikipedia HTML is fetched server-side (avoids mobile CORS issues) and rendered in a sandboxed iframe.
- SSE reconnects automatically with exponential backoff; players rejoin by token stored in `localStorage`.
