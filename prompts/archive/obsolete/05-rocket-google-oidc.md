---
id: "05-rocket-google-oidc"
title: "Implement Google OpenID Connect Authentication with openidconnect-rs in Rocket"
status: "queued"
priority: 2
depends_on: ["01-db-kv-counter"]
skills_required:
  - ".agents/skills/rocket.md"
  - ".agents/skills/surrealdb.md"
verification:
  - "cargo check --bin backend"
  - "cargo test --test auth_tests"
---

### Objective
Implement Google OpenID Connect (OIDC) authentication in the Rocket (v0.5) backend using `openidconnect` (v4) and `reqwest`[cite: 2]. The implementation must follow an extensible provider pattern so additional identity providers (Microsoft Live, Yahoo) can be added without architectural refactoring.

### Architecture & Pattern Constraints
1. **Trait-Based Abstraction**:
   - Define a shared async trait `OAuthProvider` in `backend/src/auth/provider.rs`.
   - Implement the trait for `GoogleOAuthProvider` in `backend/src/auth/google.rs`.
   - Initialize Google's metadata via discovery (`https://accounts.google.com/.well-known/openid-configuration`) during startup and attach client state to Rocket via `.manage()`[cite: 2].
2. **Session & Security Model**:
   - Ephemeral CSRF/PKCE state: Generate a cryptographically random `CsrfToken` and `Nonce` for the authorization request. Store them in temporary, short-lived HTTP-only cookies (`auth_csrf`, `auth_nonce`) or in SurrealDB under a TTL key[cite: 2].
   - Persistent User Session: After successfully verifying the ID token and Google JWKS signature, extract `sub` (Google user ID), `email`, and `name`.
   - Store user data in SurrealDB under key `user:google_<sub_id>`.
   - Create a session key in SurrealDB (e.g., `session:<token_uuid>`) mapping to the user record.
   - Issue a signed, secure, HTTP-only, `SameSite=Lax` session cookie (`session_id`) to the client[cite: 2].
3. **Rocket Integration**:
   - `GET /auth/google/login`: Generates the Google auth URL and redirects the browser (HTTP 303)[cite: 2].
   - `GET /auth/google/callback?code=...&state=...`: Validates CSRF/nonce, exchanges the code for tokens, verifies the ID token, establishes the session, and redirects to `/home` (or `/#/login?error=...` on failure)[cite: 2].
   - `GET /api/me`: Protected endpoint using a Rocket `AuthenticatedUser` request guard[cite: 2]. Returns `200 OK` with `{ "id": "...", "email": "...", "name": "..." }` if authenticated, or `401 Unauthorized` if invalid/missing[cite: 2].
   - `POST /auth/logout`: Invalidates the session key in SurrealDB and removes the session cookie[cite: 2].

### Implementation Steps

1. **Update `backend/Cargo.toml` Dependencies**:
   - Add `openidconnect = { version = "4.0", default-features = false, features = ["reqwest"] }`.
   - Add `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }`.
   - Ensure `uuid`, `serde`, and `serde_json` are configured.

2. **Environment Configuration (`backend/src/config.rs`)**:
   - Implement an `AuthConfig` struct loaded from environment variables:
     - `GOOGLE_CLIENT_ID`[cite: 1]
     - `GOOGLE_CLIENT_SECRET`[cite: 1]
     - `GOOGLE_REDIRECT_URL`[cite: 1]
     - `SESSION_SECRET`

3. **Core Provider Trait (`backend/src/auth/provider.rs`)**:
   - Define the `OAuthProvider` trait covering auth URL generation and code exchange with ID token verification.

4. **Google Implementation (`backend/src/auth/google.rs`)**:
   - Implement `GoogleOAuthProvider` using `openidconnect::core::CoreClient`.
   - Use `CoreProviderMetadata::discover_async` at startup against Google's issuer URL `https://accounts.google.com`.
   - Request scopes: `openid`, `email`, and `profile`.
   - Validate token signature against Google JWKS, matching the returned nonce and audience with `GOOGLE_CLIENT_ID`[cite: 1].

5. **Session Guard & KV Persistence (`backend/src/guards/auth_guard.rs`)**:
   - Implement `rocket::request::FromRequest` for `AuthenticatedUser`[cite: 2].
   - Inspect the `session_id` cookie, query SurrealDB for `session:<session_id>`, and return `Outcome::Success(user)` or `Outcome::Error((Status::Unauthorized, ()))`[cite: 2].

6. **Route Handlers (`backend/src/routes/auth.rs` and `backend/src/routes/user.rs`)**:
   - Implement `/auth/google/login`, `/auth/google/callback`, `/auth/logout`, and `/api/me`[cite: 2].
   - Mount these routes in `backend/src/main.rs` and attach the managed provider/config[cite: 2].

### Acceptance Criteria
- [ ] `GET /auth/google/login` returns a 303 Redirect targeting `accounts.google.com` with valid client ID and state[cite: 1, 2].
- [ ] Nonce and CSRF states are verified upon callback.
- [ ] Valid callback saves user/session to SurrealDB and sets an `HttpOnly` cookie[cite: 2].
- [ ] Callback failure or provider cancellation redirects gracefully to `/#/login?error=auth_failed`.
- [ ] `GET /api/me` yields 200 with user data when cookie is present, and 401 when absent[cite: 2].
- [ ] `POST /auth/logout` drops the session from SurrealDB and clears the cookie[cite: 2].