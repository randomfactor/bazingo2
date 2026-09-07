# Skeleton Web Application: Rust (Rocket) + SurrealDB + Svelte SPA

## System Overview
- Backend: Rust using the Rocket web framework (async).
- Database: SurrealDB running in Key-Value / NoSQL document mode.
- Frontend: Svelte Single Page Application with client-side routing.
- Authentication: Pluggable OAuth2 architecture starting with Google Identity, structured for easy addition of Microsoft Live, Yahoo, and custom OpenID providers.

## Routing & Views Specification
1. Unauthenticated Visitor: Directed to the `Welcome.svelte` landing page with an option to navigate to `Login.svelte`.
2. Authenticated Visitor: Upon successful login, the user is redirected to the `Home.svelte` page.
3. Visit Counter: The Home page must fetch and display `Total Number of Visits: NN`.
   - Every visit to the Home page by an authenticated user must trigger an atomic increment of the shared counter key (e.g., `counter:global_home_visits`) in SurrealDB.
   - The persistence layer must use atomic operations (such as SurrealQL increment or math functions) to prevent race conditions during concurrent visits.

## Key Architecture Rules
1. Persistence: Use SurrealDB key-value paradigms. Abstract all database operations behind a generic `KVStore` trait with support for atomic numeric increments.
2. Extensible Auth: Define an `OAuthProvider` Rust trait exposing `get_authorization_url()` and `verify_code_and_get_user()`.
3. SPA Fallback: Rocket must serve the compiled Svelte `index.html` on unhandled GET requests to support browser client-side routing.
4. Agent Context: Reference `.agents/skills/*.md` before modifying database schemas or Rocket request guards.

## Rust Toolchain
   - Look in `/home/<user>/.cargo/bin`
   - Common executables include:
     - `cargo`
     - `rustc`
     - `rustup`

