# Rocket Web Framework (v0.5) Guidelines

## Core Principles
- Rocket 0.5 is fully asynchronous; route handlers and guards must use `async`/`await`.
- Use the `#[launch]` or `#[rocket::main]` macros to initialize and configure Rocket.

## Managed State & Dependencies
- Inject database pools, configs, or KV stores using `.manage(service)`.
- Retrieve state in route handlers using `&State<T>`. Never initialize connections inside route handlers.

## Routing & Request Guards
- Dynamic route parameters use `<param>` syntax.
- Protected routes must use custom Request Guards implementing `rocket::request::FromRequest<'r>`:
  - Return `Outcome::Success(user)` on valid session/token.
  - Return `Outcome::Error((Status::Unauthorized, Error))` on missing/invalid credentials.
- Set explicit ranks (`rank = 1`, `rank = 2`) on routes when ambiguity exists, especially with fallback catch-all routes.

## Static SPA Serving
- To serve a Svelte SPA without breaking browser deep-linking, serve static files first and provide a fallback handler with low rank returning `index.html` on unhandled GET routes.

## Anti-Patterns (Do NOT Use)
- Do NOT use legacy Rocket 0.4 `rocket::ignite()` syntax (use `rocket::build()`).
- Do NOT use synchronous blocking I/O inside route handlers; use async equivalents or `rocket::tokio::task::spawn_blocking`.