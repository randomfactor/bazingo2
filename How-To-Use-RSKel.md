# How To Use RSKel

This repository is a starter application for building multi-user web apps, especially games. It combines:

- A Rust backend using Rocket
- A SurrealDB-backed persistence layer wrapper
- A Svelte SPA frontend for browser UI
- Session-based authentication with Google OAuth as the default provider

The structure is deliberately small and extensible so an AI agent or developer can add new features without fighting framework glue.

## 1. Project overview

### Backend

Key backend files:

- `backend/src/main.rs` — app bootstrap, mounted routes, state management, CORS
- `backend/src/config.rs` — environment-based config values such as Google credentials and session secret
- `backend/src/auth/provider.rs` — OAuth provider trait and user payload contract
- `backend/src/auth/google.rs` — Google OAuth implementation
- `backend/src/routes/auth.rs` — login, callback, logout flow
- `backend/src/routes/user.rs` — user/session profile endpoint
- `backend/src/guards/auth_guard.rs` — auth guard that rejects unauthenticated requests
- `backend/src/db/mod.rs` and `backend/src/db/pool.rs` — persistence abstraction and database setup

The core backend behavior is:

1. Boot Rocket
2. Connect to the persistence layer
3. Initialize an auth provider (Google by default)
4. Expose protected and public API routes under `/api` and `/auth`
5. Serve the SPA from `/` when running in production

### Frontend

Key frontend files:

- `frontend/src/App.svelte` — root app shell, session check, app router
- `frontend/src/routes.js` — SPA route map and protected route guards
- `frontend/src/lib/auth.svelte.js` — global auth state and session-fetch logic
- `frontend/src/components/Navbar.svelte` — navigation UI
- `frontend/src/views/*.svelte` — individual pages, for example Login, Home, Profile, About

The frontend is a Svelte SPA using hash routing (`#/...`) via `svelte-spa-router`.

### Persistence layer

The project uses a small key-value abstraction over SurrealDB:

- `KVStore::set(key, value)` stores JSON-compatible data
- `KVStore::get(key)` reads JSON-compatible data
- `KVStore::increment(key, delta)` increments counters atomically

This is a simple pattern for a starter and is a good fit for game prototypes where a record is a namespaced JSON object.

## 2. How authentication works in this project

The default flow is OAuth via Google:

- `GET /auth/google/login` starts Google authorization
- `GET /auth/google/callback` validates the response, creates a session, and sets the `session_id` cookie
- `GET /api/me` reads the session and returns the current user
- `POST /auth/logout` invalidates the session

The session cookie is checked by `AuthenticatedUser::from_request()` in `backend/src/guards/auth_guard.rs`.

If a caller does not have a valid `session_id`, the request fails with `401 Unauthorized`.

This is the core rule for protecting routes:

- Put `user: AuthenticatedUser` in the route handler arguments.
- Rocket will enforce the guard automatically.

Example:

```rust
#[get("/me")]
pub async fn me(user: AuthenticatedUser) -> (Status, Json<UserResponse>) {
    (
        Status::Ok,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        }),
    )
}
```

If there is no valid session, the request is rejected before the handler runs.

## 3. Users: how to get the logged-in user's name and ID

The user data is stored in the session record and also mirrored as a user record.

During Google callback, the code creates a session key like:

```rust
let session_key = format!("session:{session_id}");
```

and stores JSON such as:

```rust
json!({
    "id": user.id.clone(),
    "email": user.email.clone(),
    "name": user.name.clone(),
})
```

The authenticated user type is defined in `backend/src/guards/auth_guard.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub name: String,
}
```

The user payload can be retrieved by the frontend via `/api/me`:

```js
const response = await fetch('/api/me', {
  method: 'GET',
  credentials: 'include',
})

const data = await response.json()
console.log(data.id)
console.log(data.name)
```

This is exactly how `frontend/src/lib/auth.svelte.js` populates `authState.user`:

```js
if (response.ok) {
  const data = await response.json()
  authState.user = data
  authState.isAuthenticated = true
}
```

After that, in Svelte pages you can read:

```svelte
{authState.user?.name}
{authState.user?.id}
```

This is the canonical pattern for getting the logged-in user name and ID in this app.

## 4. Adding another authentication provider beyond Google

The project is already structured for provider extension.

### Pattern to follow

1. Add a new provider module under `backend/src/auth/`.
2. Implement the `OAuthProvider` trait from `backend/src/auth/provider.rs`.
3. Create provider-specific login and callback routes in `backend/src/routes/auth.rs` or a new file.
4. Add a frontend button or redirect in the login page.
5. Store the resulting user in the same session model.

### Provider contract

The required trait is:

```rust
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    async fn authorization_url(&self, state: &str, nonce: &str) -> Result<ProviderAuthRequest, String>;
    async fn verify_code_and_get_user(
        &self,
        code: &str,
        state: &str,
        nonce: &str,
    ) -> Result<AuthUser, String>;
    fn provider_name(&self) -> &'static str;
    fn exchange_user_payload(&self, payload: Value) -> Result<AuthUser, String>;
}
```

The Google provider in `backend/src/auth/google.rs` is the best template to copy.

### Practical steps

For a provider such as GitHub, Discord, Twitch, or Microsoft:

- Create `backend/src/auth/github.rs` or similar
- Implement `authorization_url()` to build the OAuth authorize URL
- Implement `verify_code_and_get_user()` to exchange the code and decode the identity payload
- Map the external identity to the local normalized shape:

```rust
AuthUser {
    id: external_subject_or_user_id,
    email: email_or_fallback,
    name: display_name_or_username,
}
```

Then, in the callback route, use the same session creation flow:

```rust
let session_id = Uuid::new_v4().to_string();
let user_key = format!("user:{provider_name}_{id}");
let session_key = format!("session:{session_id}");

store.set(&user_key, json!({
    "id": user.id,
    "email": user.email,
    "name": user.name,
})).await?

store.set(&session_key, json!({
    "id": user.id,
    "email": user.email,
    "name": user.name,
})).await?
```

### Frontend addition

Update the login page in `frontend/src/views/Login.svelte` to add a button:

```svelte
<button onclick={goToGitHubLogin}>Continue with GitHub</button>
```

and implement:

```js
function goToGitHubLogin() {
  window.location.href = '/auth/github/login'
}
```

The backend route should redirect the user after login back to the SPA, typically `/#/login` or `/#/`.

## 5. Adding a new backend API endpoint

The project exposes endpoints under `/api` in `backend/src/main.rs`.

### Step 1: define the response type

Use a Rust `Serialize` data structure. Example:

```rust
#[derive(Serialize)]
struct PlayerStatsResponse {
    user_id: String,
    wins: i64,
    losses: i64,
}
```

### Step 2: implement the route handler

Add a new handler function to a route module (e.g. under `backend/src/routes/`) or in `backend/src/main.rs`.

Example pattern:

```rust
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::json;
use crate::guards::auth_guard::AuthenticatedUser;
use crate::db::KVStore;
use crate::LocalDbPool;

#[get("/player-stats")]
async fn player_stats(
    user: AuthenticatedUser,
    store: &State<LocalDbPool>,
) -> Result<Json<PlayerStatsResponse>, Status> {
    let key = format!("player:{}:stats", user.id);

    let stats = store.get(&key).await
        .map_err(|_| Status::InternalServerError)?
        .unwrap_or_else(|| json!({ "wins": 0, "losses": 0 }));

    Ok(Json(PlayerStatsResponse {
        user_id: user.id,
        wins: stats.get("wins").and_then(|v| v.as_i64()).unwrap_or(0),
        losses: stats.get("losses").and_then(|v| v.as_i64()).unwrap_or(0),
    }))
}
```

### Step 3: mount the route

Add it to the mounted routes in `backend/src/main.rs`:

```rust
.mount("/api", routes![get_data, get_visits, visit, me, player_stats, options_data])
```

### Step 4: call it from the frontend

Use standard `fetch` with `credentials: 'include'` so the session cookie is sent.

```js
const response = await fetch('/api/player-stats', {
  method: 'GET',
  credentials: 'include',
})

if (!response.ok) {
  throw new Error('Request failed')
}

const data = await response.json()
console.log(data.user_id, data.wins, data.losses)
```

### Protecting selected API endpoints

Use `AuthenticatedUser` as an argument to the route handler. This is the built-in guard.

Example:

```rust
#[get("/secure-data")]
async fn secure_data(user: AuthenticatedUser) -> Json<Value> {
    Json(json!({
        "user_id": user.id,
        "message": "Only authenticated users can see this",
    }))
}
```

If the caller is not logged in, Rocket will reject the request with `401 Unauthorized` before the handler executes. That is how the project enforces authenticated access.

For an even stricter pattern, you can also manually fail inside the handler, but the guard-based pattern is preferred because it is consistent with the rest of the app.

## 6. Defining new persistence record types

This project is intentionally built around small JSON records stored under namespaced keys. This is a strong match for game prototypes and multiplayer systems.

### Recommended naming pattern

Use keys that clearly encode record type and identity:

- `user:google_12345`
- `session:uuid-1234`
- `player:alice:profile`
- `match:match-42`
- `game:lobby:global`
- `guild:abc123:state`

The actual record contents are JSON maps.

### Example record shape

```rust
let user_profile = json!({
    "id": "alice",
    "name": "Alice",
    "email": "alice@example.com",
    "rank": 12,
    "xp": 2400,
})
```

Store it like this:

```rust
store.set("player:alice:profile", user_profile).await?;
```

Read it like this:

```rust
let value = store.get("player:alice:profile").await?;
```

Update it by storing a new object or by mutating serialized JSON in code.

### Best pattern for game data

For games, prefer:

- player profile by user ID
- game lobby state by lobby ID
- match state by match ID
- global counters by `counter:...`
- ephemeral session records by `session:<id>`

This keeps the records easy to inspect and easy to migrate later into a deeper relational or document model.

If the app grows beyond the starter, the same logical records can be moved into a richer SurrealDB schema later without changing the app structure dramatically.

## 7. Adding a new page to the SPA

The SPA route table is in `frontend/src/routes.js`.

### Step 1: create the page component

Add a new file under `frontend/src/views/`, for example:

- `frontend/src/views/Leaderboard.svelte`
- `frontend/src/views/Lobby.svelte`
- `frontend/src/views/Inventory.svelte`

Example component:

```svelte
<script>
  import { authState } from '../lib/auth.svelte.js'
</script>

<main class="auth-shell">
  <h1>Leaderboard</h1>
  <p>Welcome, {authState.user?.name ?? 'Player'}.</p>
</main>
```

### Step 2: register the route

In `frontend/src/routes.js`:

```js
import Leaderboard from './views/Leaderboard.svelte'

export const routes = {
  '/': Home,
  '/login': Login,
  '/leaderboard': Leaderboard,
  '/profile': wrap({
    component: Profile,
    conditions: [() => authState.isAuthenticated],
  }),
  '*': NotFound,
}
```

### Step 3: add navigation if needed

Edit `frontend/src/components/Navbar.svelte` and add a link:

```svelte
<a href="#/leaderboard">Leaderboard</a>
```

### Step 4: protect route if needed

If a page requires authentication, use `wrap()` with a condition as shown in `frontend/src/routes.js`:

```js
'/profile': wrap({
  component: Profile,
  conditions: [() => authState.isAuthenticated],
})
```

If the condition fails, the router redirects via the `conditionsFailed` handler in `App.svelte` (e.g. redirecting to `#/login`). When adding new protected routes, add their path to `handleConditionsFailed` in `frontend/src/App.svelte` as well:

```js
function handleConditionsFailed(event) {
  const detail = event?.detail
  if (detail?.route === '/profile' || detail?.route === '/leaderboard') {
    replace('/login')
  }
}
```

## 8. Typical extension workflow for a new feature

For an AI agent or developer extending the project, the default workflow should be:

1. Determine the domain concept (player, lobby, match, chat, inventory, guild, leaderboard)
2. Define a record shape and key namespace
3. Add a backend API route and a response DTO
4. Guard the route with `AuthenticatedUser` when it needs a logged-in user
5. Add the fetch call in the Svelte page
6. Add or update SPA routes and navigation
7. Reuse `authState.user` for session-aware UI

## 9. Suggested directions for a game-oriented starter

This app is designed as a foundation for multiplayer or cooperative games. Practical next additions include:

- `lobby` records for matchmaking
- `match` records for ongoing game state
- `player` or `profile` records for persistent stats
- `leaderboard` endpoints for ranking players
- `chat` or `event` logs for real-time game updates
- more auth providers for platform-specific account linking

The existing project already has the right mental model:

- user identity is normalized around `id`, `email`, and `name`
- the backend enforces auth at the route layer
- the frontend uses a single auth state object
- persistence is record-driven and namespaced

This makes it easy to evolve from a starter app into a real multi-user game platform.

## 10. Quick-reference checklist

Use this checklist when adding a new feature:

- Backend
  - Add or update a JSON record type
  - Add a `Store` key and serialization model
  - Add a route handler in a backend route module
  - Add `AuthenticatedUser` to protect private APIs
  - Mount the route in `backend/src/main.rs`

- Frontend
  - Add a Svelte view under `frontend/src/views/`
  - Register it in `frontend/src/routes.js`
  - Add navigation in `frontend/src/components/Navbar.svelte` if needed
  - Fetch `/api/...` with `credentials: 'include'`
  - Use `authState.user` for current user details

- Auth providers
  - Implement a new provider module under `backend/src/auth/`
  - Add a login and callback route
  - Store user/session records in the same shape as Google
  - Add the provider button to the Svelte login page

This project is intentionally small enough to understand quickly, but structured enough to grow into a real application foundation.
