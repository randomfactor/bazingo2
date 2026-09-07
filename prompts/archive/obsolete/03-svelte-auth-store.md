---
id: "03-svelte-auth-store"
title: "Implement Svelte 5 Reactive Auth Store and Home/Login Views"
status: "queued"
priority: 3
depends_on: ["02-rocket-auth-routes"]
skills_required:
  - ".agents/skills/svelte.md"
verification:
  - "cd frontend && npm run build"
---

### Objective
Create the reactive client-side authentication store and the primary views (`Home.svelte`, `Login.svelte`, and an unauthenticated public view) according to the project specifications.

### Context & Rules
- Use modern Svelte 5 syntax: runes (`$state`, `$derived`, `$props`), standard HTML event handlers (`onclick` instead of `on:click`), and snippets if needed[cite: 1].
- Do NOT use legacy Svelte 3/4 stores (`writable`) or legacy syntax (`export let`, `$:`)[cite: 1].

### Implementation Details

1. **Create `frontend/src/lib/auth.svelte.js`**:
   - Export an `$state` reactive object `authState` tracking:
     - `isAuthenticated`: boolean (default `false`)
     - `isLoading`: boolean (default `true`)
     - `user`: object or null (default `null`)
   - Export an async function `checkSession()` that calls `GET /api/me` with `credentials: 'include'`[cite: 2]:
     - If response is 200 OK, parse JSON, set `authState.user = data`, `authState.isAuthenticated = true`[cite: 2].
     - If response is 401 or network fails, set `authState.user = null`, `authState.isAuthenticated = false`[cite: 2].
     - In `finally`, set `authState.isLoading = false`.
   - Export an async function `logout()` that calls `POST /auth/logout` and resets state[cite: 2].

2. **Create `frontend/src/views/Home.svelte`**:
   - Import `authState` from `../lib/auth.svelte.js`.
   - If `authState.isAuthenticated` is false:
     - Display generic welcome information describing the application.
     - Include an anchor tag or button navigating to `#/login` ("Sign In").
   - If `authState.isAuthenticated` is true:
     - Display detailed member information: user greeting, user email, and detailed dashboard features.
     - Display a Logout button that triggers `logout()`.

3. **Create `frontend/src/views/Login.svelte`**:
   - Display a clean login card titled "Sign In".
   - Include a primary button for Google Identity:
     - Clicking executes `window.location.href = '/auth/google/login'`.
   - Include placeholder buttons for future providers (Microsoft, Yahoo) marked disabled with "(Coming Soon)".
   - Include a back link to Home (`#/`).

4. **Create `frontend/src/views/About.svelte` (Public Example)**:
   - Create a basic public informational view accessible to both logged-in and unauthenticated visitors.

### Acceptance Criteria
- [ ] Auth store correctly reflects session changes via Svelte 5 runes[cite: 1].
- [ ] Home view conditionally renders generic overview vs. detailed member dashboard.
- [ ] Login view provides Google OIDC trigger and slots for future providers.