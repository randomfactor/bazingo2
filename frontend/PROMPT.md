# Skeleton Web Application: Rust (Rocket) + SurrealDB + Svelte SPA

## System Overview
- Backend: Rust using the Rocket web framework (async).
- Database: SurrealDB running in Key-Value / NoSQL document mode.
- Frontend: Svelte Single Page Application with client-side routing.
- Authentication: Pluggable OAuth2 architecture starting with Google Identity, structured for easy addition of Microsoft Live, Yahoo, and custom OpenID providers.

## Frontend Toolchain
- Use Node.js and npm to install dependencies and run frontend commands from `frontend/`.
- Svelte 5 provides the component framework; use `@sveltejs/vite-plugin-svelte` to integrate Svelte with Vite.
- Vite is the development server and production build tool. Use `npm run dev` for local development, `npm run build` for a production build, and `npm run preview` to preview the build.
- TypeScript is used for frontend entry points, configuration, and tests. JavaScript remains supported for existing Svelte modules and route/state files.
- `svelte-spa-router` provides client-side routing for the single-page application.
- Run `npm run check` to execute `svelte-check` and the TypeScript compiler.
- Vitest, `@testing-library/svelte`, and `jsdom` provide component testing; see the Vitest section below for test conventions.
- Keep dependencies and scripts in `frontend/package.json` and use the existing Vite and Svelte configuration files rather than introducing a second build system.

## Frontend Code Structure
- `frontend/src/main.ts`: frontend entry point; mounts the root Svelte application.
- `frontend/src/App.svelte`: root application shell and global session initialization.
- `frontend/src/routes.js`: client-side route definitions and route guards.
- `frontend/src/views/`: route-level page components such as `Home.svelte`, `Login.svelte`, `Profile.svelte`, and `About.svelte`.
- `frontend/src/components/`: reusable Svelte UI components shared by views.
- `frontend/src/lib/`: shared application logic and reactive state, including authentication state in `auth.svelte.js`.
- `frontend/src/assets/`: imported static assets.
- `frontend/src/app.css`: global styles and shared visual rules.
- Keep page-specific behavior in the relevant view, reusable UI in `components`, and cross-view state or utilities in `lib`.
- Preserve the existing Svelte 5 runes and SPA routing patterns when adding frontend code.

## Vitest
- Frontend tests use Vitest with `@testing-library/svelte` and the `jsdom` environment.
- Run all frontend tests with `npm test` from `frontend/`; run a focused test with `npm test -- --run path/to/test.ts`.
- Component tests belong beside the component under test and use `.test.ts` filenames.
- Mock browser APIs and network requests, such as `fetch`, so tests remain deterministic and do not require a running backend.
- `frontend/vitest.config.ts` resolves Svelte's browser runtime with `resolve.conditions: ['browser']` and `ssr.noExternal: ['svelte']`; retain these settings for Svelte component tests.
- Prefer accessible queries such as `getByRole` and assert only the behavior relevant to the test.

