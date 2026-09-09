# Bazingo2 Backend Architecture and Development Guide

## Project Overview
Bazingo2 is a full-stack web application centered on a turn-based board game. The Rust backend exposes a Rocket API, manages authenticated game sessions, and persists game state in SurrealDB. The frontend is a Svelte SPA that communicates with the API and presents the game UI, but the backend remains the source of truth for player state, match state, and game progression.

The current codebase is not a generic starter app. It is a game service that manages:

- user session and OAuth authentication
- SurrealDB-backed key-value storage of game and player records
- a game engine that defines board state, legal moves, turn timing, and scoring
- HTTP endpoints that create games, join players, query active and completed games, and process turns
- static asset hosting and SPA fallback behavior for the frontend

This document summarizes the actual implementation currently present in the repository so future edits align with the project’s real architecture rather than the older RSKel skeleton assumptions.

## System Architecture
The backend is organized around a small number of core responsibilities:

1. Bootstrapping and application wiring
   - `backend/src/main.rs` creates the Rocket app, initializes SurrealDB via `DbPool::from_env()`, loads OAuth configuration, and mounts the routes.
   - Global state is managed with Rocket `State<T>` and includes the database pool, auth config, and the Google OAuth provider.

2. Persistence abstraction
   - `backend/src/db/mod.rs` declares the `KVStore` trait and error types.
   - `backend/src/db/pool.rs` provides a concrete `DbPool<T>` implementation using SurrealDB local or remote connections.
   - This abstraction is intentionally simple: store and retrieve JSON values by a key such as `game:<id>` or `game_player:<game_id>_<user_id>`.

3. Game engine
   - `backend/src/game/engine.rs` contains the pure game logic for board placement, piece validation, scoring, and turn rules.
   - It has no database dependency and acts as the authoritative rules engine.

4. Service layer
   - `backend/src/game/services.rs` applies the engine rules to persisted application data.
   - It creates match records, loads and saves player records, computes turn chronology, and resolves conflicts.

5. Route layer
   - `backend/src/routes/*.rs` exposes HTTP operations for auth, user session data, and game interactions.
   - The `games` module includes the API for listing games, creating matches, joining games, reading state, and making moves.

6. Auth and user security
   - `backend/src/auth/*` and `backend/src/guards/*` implement session-based user checks and OAuth login handling.
   - `AuthenticatedUser` is used to gate access to protected game routes.

## Persistence Model
The persistence layer is written around SurrealDB as a document and key-value store, with a custom `KVStore` abstraction above the lower-level database client. This makes the rest of the application independent from SurrealDB specifics while still allowing JSON serialization and keyed storage.

### Error model and trait interface
The database layer defines a small custom error enum:

- `SurrealDB(String)`
- `Serialization(String)`
- `InvalidKey(String)`

The `KVStore` trait exposes:

- `get(&self, key: &str) -> Result<Option<Value>>`
- `list_table(&self, table: &str) -> Result<Vec<Value>>`
- `set(&self, key: &str, value: Value) -> Result<()>`
- `increment(&self, key: &str, delta: i64) -> Result<i64>`

This design is consistent with the project’s use of key-based records, where records are stored as JSON values under keys such as:

- `counter:global_home_visits`
- `game:<uuid>`
- `game_player:<game_id>_<user_id>`

### `DbPool<T>` implementation
`DbPool` is generic over SurrealDB connection types:

- `DbPool<Client>` for remote SurrealDB connections
- `DbPool<LocalDb>` for local RocksDB-backed SurrealDB databases used in development or local testing

The local database configuration is created from environment variables:

- `SURREALDB_PATH` default: `./data/surrealdb`
- `SURREALDB_NS` default: `main`
- `SURREALDB_DB` default: `main`
- `SURREALDB_USER` default: `bazingo2_local`
- `SURREALDB_PASS` default: `local_only_duh`

The implementation also supports an in-memory SurrealDB database for tests via `DbPool::new_in_memory(...)`.

### Keying conventions and storage patterns
The code uses a `table:id` pattern, split via `split_key(key)`. For example:

- `game:1234` becomes table `game` and id `1234`
- `game_player:game_abc_user_123` becomes table `game_player` and id `game_abc_user_123`

This is important because the project stores not only counters but also complete JSON objects:

- `GameRecord` for the active match definition
- `PlayerGameRecord` for each user’s state in a game
- raw `Value` payloads for counters and other application metadata

The `set` implementation uses `UPSERT type::thing($tb, $id) CONTENT { value: $value }`, so a single record is updated by replacing or creating the document at the keyed SurrealDB record location. `list_table` reads all values from a given table via `SELECT VALUE value FROM type::table($table)`. This is not a classic relational model; it is a lightweight document-store abstraction designed around simple JSON records and stable keys.

### Increment semantics and concurrency
The `increment` method is deliberately simple and is the critical piece when the app needs atomic-like counter increments: it reads the current value, adds `delta`, and writes the new value back. This is not a full transactional compare-and-swap system, but it works for the current app patterns, especially for counters such as the global home visits total.

This matches the older design intent expressed in the project’s early prompt, but in the actual codebase, it is used for values maintained under `counter:*` keys and for simple state tracking. The service layer is the one that coordinates more complex game-state transitions and uses persisted JSON records to validate turn ordering and move legality.

## Game Engine
The pure rules engine lives in `backend/src/game/engine.rs`. It is intentionally free of database and HTTP concerns. It models the board, pieces, legal placements, scoring, and turn progression.

### Core data types
The engine defines the following types:

- `BzGameboard = Vec<Vec<bool>>`
- `BzGamepiece = Vec<Vec<bool>>`
- `BzGame { pieces: Vec<BzGamepiece>, turns: u32 }`
- `BzPlayer { gameboard: BzGameboard, last_turn_played: u32 }`
- `TurnScore { score: i32, gameboard: BzGameboard, scored_blocks: Vec<(usize, usize, usize, usize)> }`

The board is a 5x5 grid, and each game piece is a 3x3 binary matrix. A `true` value marks occupied cells. This is a classic placement puzzle style: pieces are placed onto a board, rows/columns are scored when contiguous rectangular groups are filled, and filled blocks are cleared from the board.

### Piece generation and validation
The engine defines `PIECE_PATTERNS`, a list of 32 octal-encoded piece layouts. These are converted using `gamepiece_from_octal` into 3x3 boolean matrices. `all_gamepieces()` builds the complete piece set, and `BzGame::new(turns)` shuffles and assigns the piece list for a game.

The engine validates both the board and the piece shape before accepting a move:

- `validate_board()` ensures a 5x5 board shape
- `validate_piece()` ensures a 3x3 piece shape
- `place_piece()` rejects invalid turns, illegal placements, and attempts to play the same turn twice

A crucial rule is that a move must be for the current turn number and must not exceed the configured game length. The engine also ensures pieces cannot overlap existing true cells on the board and prevents placements from going outside the board boundaries.

### Placement and legality rules
`place_piece(game, player, turn, x, y)` is the central move validator. It performs the following checks:

- turn must be non-zero
- turn cannot exceed `game.turns`
- turn must be strictly greater than `player.last_turn_played`
- the referenced piece exists for that turn
- the board and piece are valid shapes
- each occupied cell of the piece fits within the board bounds
- no occupied cell collides with existing board content

If all checks pass, the engine returns the next board state. This means the engine is a deterministic, pure state transition function: given a previous player state and a move, it tells you whether the move is legal and what the resulting board would be.

### Scoring and end-of-game logic
The scoring system is implemented by `compute_turn_score()` and `score_blocks()`. It awards points for completed rectangular regions on the board, then clears those regions from the board. The constants are meaningful:

- `FAILURE_PENALTY = -7`
- `FINAL_EMPTY_BOARD_BONUS = 15`

The function also subtracts a penalty for missed turns using:

- `missed_turns = turn.saturating_sub(player.last_turn_played + 1)`

This means if a player skips a turn, they are tracked as having missed one or more turns, and their score is reduced. The final turn bonus is granted when the board is empty at the end of the last turn. This creates a strong incentive to clear the board and finish efficiently while still respecting the turn sequence.

## Service Layer and Game State Management
The service layer in `backend/src/game/services.rs` is where the persistence model and game rules meet. It defines `GameRecord`, `PlayerGameRecord`, and service-level helper functions responsible for storing and retrieving data from SurrealDB.

### Records
`GameRecord` contains the match metadata:

- id
- name
- starting_time
- ending_time
- number_of_turns
- turn_duration
- interim
- current_turn
- pieces
- completed

`PlayerGameRecord` stores each user’s per-game state:

- game_id
- user_id
- name
- gameboard
- score
- last_turn_played

These structures are serialized to JSON with Serde and saved under the appropriate key in SurrealDB.

### Game creation and player joins
`create_game()` creates a new game with a unique UUID, assigns a name, and stores the initial `GameRecord`. It also sets a start time and end time based on `DEFAULT_NUMBER_OF_TURNS`, `DEFAULT_TURN_DURATION_SECONDS`, and `DEFAULT_INTERIM_SECONDS`.

`join_game()` checks whether the game is still active and whether the user is already in the match. It then creates a `PlayerGameRecord` if the caller is joining for the first time. `list_players()` enumerates all recorded players in a game by scanning the `game_player` table.

### Turn advancement and current turn logic
`current_turn(game, now)` computes which turn should currently be active based on the game start time and the turn cycle. The game’s `turn_duration` and `interim` periods define each slot. This logic prevents stale moves and allows the server to reject requests that are not for the current turn.

The move flow in `move_player()` is the heart of the application logic:

1. Load the game and player record from storage.
2. Validate the target turn number against the expected current turn.
3. Build the engine representation of the game board and player state.
4. Call `place_piece()` to verify the move.
5. Call `compute_turn_score()` to score and clear board blocks.
6. Update the player score and game state.
7. Save the updated player and game records.

This cleanly separates the validation work, the scoring logic, and the persistence layer while still keeping all state transitions in a single server-side operation.

## API Shape
The HTTP layer is defined under `backend/src/routes/`. The main game endpoints are mounted under `/api` and include:

- GET `/api/games/active` - list active games
- GET `/api/games/past` - list past games
- POST `/api/games` - create a new game
- GET `/api/games/info/<id>` - read metadata for a game
- GET `/api/games/me` - read the current authenticated user’s active game
- POST `/api/games/join/<id>` - join a game
- GET `/api/games/players/<id>` - list players in a game
- POST `/api/games/move/<id>` - make a move for the current turn

Protected routes are guarded by `AuthenticatedUser`, so a user must be authenticated and have a valid session before they can create or join a match. The API is intentionally small but structurally consistent: each operation either loads persisted records, validates a turn against the engine rules, or updates the board and score for a player.

## Auth and User Model
Authentication is handled in the backend’s auth modules and through a user guard. The app is designed with OAuth in mind and currently includes a Google OAuth provider implementation (`backend/src/auth/google.rs`). The service loads `AuthConfig` from environment variables and constructs a provider at startup.

User information is read from the session or OAuth callback and stored in the current request context for route access. Guarded routes then read the authenticated user and use values such as `user.id` and `user.name` as part of game membership and player state.

## Static Files and SPA Fallback
The server also mounts the frontend distribution under `/` and configures a SPA fallback catcher so browser routes still load correctly when a user refreshes the page or requests a client-side route directly.

The `file_server()` route ensures the built frontend assets are served, while the catch-all fallback allows routing to be managed by the frontend application rather than by Rocket. This is important because the project uses Svelte SPA routing, and users can navigate between route paths without requiring a full server-side route for every screen.

## Operational Doctrine for Future Changes
When modifying the backend, follow these rules to keep the architecture coherent:

1. Keep engine logic pure and database-free.
   - The logic in `backend/src/game/engine.rs` should never depend on HTTP, Rocket, or SurrealDB.
2. Keep persistence keyed and JSON-based.
   - The database abstraction is intentionally simple; use small record documents keyed by stable names rather than introducing a second storage model.
3. Keep service logic authoritative.
   - The service layer decides whether a move is valid, whether the game is active, and whether the player can make a turn.
4. Keep routes thin.
   - Route handlers should validate auth and translate service results into HTTP responses rather than embedding business logic directly.
5. Keep player and game state in sync.
   - When a move is processed, the server must update both the player record and the game record in a consistent sequence.
6. Preserve the key conventions.
   - Existing keys like `game:*`, `game_player:*`, and `counter:*` are part of the project’s operational fingerprint.

## Toolchain and Local Workflow
Use the Rust toolchain in the usual Cargo environment. Common commands are:

- `cargo run` for the backend server
- `cargo test` for unit tests
- `cargo check` for quick compilation validation

The backend is also configured to load environment variables from `.env.local` in debug builds via `dotenvy::from_filename(".env.local").ok();`.

This project is not a generic toy API: it is a stateful game service whose correctness depends on a carefully separated architecture between rules, persistence, service logic, and HTTP transport.

The most important conceptual model is this:

- `engine` answers: “Is the move legal, and what score does it produce?”
- `services` answers: “What record should be saved and what game/player state is affected?”
- `routes` answers: “What HTTP request should this become, and what response should the client receive?”

When in doubt, continue from that layering and avoid mixing concerns across those boundaries.

