You are tasked with implementing the static asset serving and Single Page Application (SPA) HTML5 history fallback in a Rust Rocket (v0.5) web server.

### Objective
Configure Rocket to:
1. Serve pre-compiled static frontend assets (JS, CSS, images, icons) located in the directory `frontend/dist/`[cite: 1].
2. Provide an SPA catch-all fallback route that returns `frontend/dist/index.html` for any unmatched GET request[cite: 1].
3. Ensure no static route intercepts or collides with API endpoints (`/api/*`) or authentication flows (`/auth/*`)[cite: 1].

### Requirements & Constraints

1. **Framework Version**: Rocket 0.5 (async idioms, `rocket::fs` module)[cite: 1].
2. **File Location**: Implement route handlers in `backend/src/routes/static_files.rs` and wire them into `backend/src/main.rs`.
3. **Route Priority & Ranking**:
   - API endpoints (`/api/*`) and Auth endpoints (`/auth/*`) must have higher precedence[cite: 1].
   - Static asset files must resolve if the requested file exists on disk[cite: 1].
   - The fallback route must use a low rank (e.g., `rank = 10` or higher) and match multi-segment paths using `<_path..>`[cite: 1].
4. **SPA Fallback Logic**:
   - For all unhandled GET requests outside `/api/*` and `/auth/*`, read and stream `frontend/dist/index.html` using `rocket::fs::NamedFile`[cite: 1].
   - If `frontend/dist/index.html` is missing from the disk (e.g., frontend has not been built yet), the route should return an `Option<NamedFile>` resolving to `None` (triggering Rocket's 404 catcher) rather than panicking[cite: 1].
5. **Asset Serving**:
   - Mount `FileServer::from(relative!("frontend/dist"))` or configure explicit paths to serve hashed bundles (such as `/assets/<file..>`) efficiently with correct MIME types and cache headers[cite: 1].

### Implementation Details

1. Create `backend/src/routes/static_files.rs`:
   - Export an `spa_fallback` handler decorated with `#[get("/<_path..>", rank = 10)]`[cite: 1].
   - Define a function `pub fn routes() -> Vec<rocket::Route>` or specify mount paths for `rocket::fs::FileServer`[cite: 1].
   - Ensure the path to `frontend/dist` resolves accurately both in local debug runs and in production environments.

2. Update `backend/src/main.rs`:
   - Mount the static file server and fallback route on the root path `/`[cite: 1].
   - Mount `/api` routes and `/auth` routes separately to verify routing isolation[cite: 1].

Do not implement business logic for OAuth or database storage in this task; focus strictly on static file serving, route rankings, and SPA fallback behavior[cite: 1].