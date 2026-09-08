---
id: "02-app-data-endpoints"
title: "Implement API endpoints and game persistence"
status: "queued"
priority: 3
depends_on: ["02-rocket-auth-routes", "01-game-engine"]
skills_required:
  - ".agents/skills/rocket.md"
  - ".agents/skills/vendor/surrealdb/surrealql"
  - ".agents/skills/vendor/surrealdb/surrealql-functions"
verification:
  - "cargo check --bin backend"
  - "cargo test --test game_api_tests"
---

### Objective
Implement Rocket route handlers, request/response DTOs, and SurrealDB persistence logic to support game lifecycle and player moves.

### Routing & Mounting Conventions
All handlers will be mounted under `/api` in `backend/src/main.rs`. Use Rocket 0.5 parameter syntax (`<id>`) rather than `{id}`.

### Unauthenticated Endpoints

- `GET /games/active?limit=20&start_after=<timestamp>`:
  - Query parameters: `limit` (max 100, default 20), optional `start_after` for pagination.
  - Returns JSON array of active or pending games:
    `{ id, name, starting_time, current_turn, player_count }`.
  - Filter: `starting_time + (number_of_turns * (turn_duration + interim)) > time::now()`.

- `GET /games/past?limit=20&start_after=<timestamp>`:
  - Returns completed games:
    `{ id, name, starting_time, ending_time, number_of_turns, player_count, highest_score }`.

### Authenticated Endpoints (Guarded by `user: AuthenticatedUser`)

- `POST /games`:
  - Creates a new `BzGame` starting 2 minutes in the future.
  - Returns `201 Created` with `{ id, name, starting_time }`.

- `GET /games/info/<id>`:
  - Returns public metadata for a game. Mask/omit `pieces` if game is still in progress.

- `GET /games/me`:
  - Returns the active game currently joined by `user.id`, or `204 No Content` / `{}` if none.

- `POST /games/join/<id>`:
  - Enters `user.id` into the game. Fails if game is already completed or full.

- `GET /games/players/<id>`:
  - Returns list of player names/IDs currently registered in the game.

- `POST /games/move/<id>`:
  - Request body: `{ turn_number: usize, x: i32, y: i32 }`.
  - Identifies player via `AuthenticatedUser` session.
  - Validates turn timing, game membership, and board move legality via `BzGame`.
  - Persists updated board state and score.

### Persistence Implementation Details
- Inject database state using Rocket's `&State<Surreal<Client>>` or extend `LocalDbPool` beyond simple KV where range queries and sorting are required.
- Keys/Tables:
  - Games: `game:<game_id>`
  - Player game state: `game_player:<game_id>_<user_id>`
- Ensure date/time fields use ISO 8601 strings or SurrealDB `datetime` types for reliable ordering.