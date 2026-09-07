---
id: "04-svelte-router-guards"
title: "Configure svelte-spa-router with Route Preconditions and Guarding"
status: "queued"
priority: 4
depends_on: ["03-svelte-auth-store"]
skills_required:
  - ".agents/skills/svelte.md"
verification:
  - "cd frontend && npm run build"
---

### Objective
Install and configure `svelte-spa-router` to manage client-side routing, protected route guards, public routes, and redirect unauthenticated users to `/login`.

### Requirements & Constraints
1. **Router Dependency**: Install `svelte-spa-router` inside `frontend/`.
2. **Route Definitions (`frontend/src/routes.js`)**:
   - `/`: `Home.svelte` (unprotected: shows public or member view depending on auth state).
   - `/login`: `Login.svelte` (public: auth provider options).
   - `/about`: `About.svelte` (unprotected public route).
   - `/profile`: A protected route wrapped using `wrap({ component: Profile, conditions: [...] })`.
   - `*`: A NotFound fallback component.
3. **Guard Logic**:
   - For protected routes, the condition function must check `authState.isAuthenticated`.
   - If condition evaluates to `false`, reject navigation.
4. **App Integration (`frontend/src/App.svelte`)**:
   - On component mount / initial effect, call `checkSession()` from `auth.svelte.js`.
   - Render a loading spinner or placeholder while `authState.isLoading` is true to avoid route flicker.
   - Listen for the `conditionsFailed` event on the Router:
     - If triggered, use `replace('/login')` or `window.location.hash = '/login'` to redirect the visitor.
   - Provide a shared `Navbar.svelte` displaying dynamic navigation links (`Home`, `About`, `Profile` [if authenticated], `Login` [if unauthenticated], `Logout` [if authenticated]).

### Acceptance Criteria
- [ ] `svelte-spa-router` installed and routing functional.
- [ ] Visiting `/` displays generic info when logged out, or detailed info when logged in.
- [ ] Direct navigation to protected routes (e.g., `#/profile`) redirects to `#/login` when unauthenticated.
- [ ] Unprotected routes (`#/`, `#/about`) are visitable in all states.
- [ ] `npm run build` succeeds in `frontend/` without Svelte compiler warnings.