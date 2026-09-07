<script>
  import { onMount } from 'svelte'
  import { authState, logout } from '../lib/auth.svelte.js'

  let totalVisits = 0

  async function loadVisits() {
    try {
      const response = await fetch('/api/visit', {
        method: 'GET',
        credentials: 'include',
      })

      if (!response.ok) {
        return
      }

      const data = await response.json()
      totalVisits = Number(data?.total ?? 0)
    } catch (error) {
      console.warn('Failed to fetch visit count:', error)
    }
  }

  onMount(() => {
    void loadVisits()
  })

  function handleLogout() {
    void logout()
  }
</script>

{#if !authState.isAuthenticated}
  <main class="auth-shell">
    <section class="card">
      <p class="eyebrow">Welcome</p>
      <h1>Rust / Svelte Skeleton Application</h1>
      <ul class="description-list">
        <li>Rocket web server</li>
        <li>SurrealDB persistence layer</li>
        <li>OpenConnect user login</li>
      </ul>
      <a class="primary-button" href="#/login">Sign In</a>
    </section>
  </main>
{:else}
  <main class="auth-shell">
    <section class="card dashboard">
      <p class="eyebrow">Site Name</p>
      <h1>Welcome back, {authState.user?.name ?? 'friend'}!</h1>
      <p class="description">{authState.user?.email ?? 'You are signed in.'}</p>

      <p class="description">
        The number of visits for all logged-in users is tracked over time in a value maintained in
        the SurrealDB database for the application.
      </p>
      <p class="description">
        This code demonstrates how to use the SurrealDB Rust client to modify a value in the 
        database and get it from the frontend for display.
      </p>

      <div class="feature-list">
        <h2>Total visits: {totalVisits}</h2>
      </div>

      <button class="secondary-button" onclick={handleLogout}>Logout</button>
    </section>
  </main>
{/if}

<style>
  .auth-shell {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #f5f3ff 0%, #eef6ff 100%);
    padding: 2rem;
  }

  .card {
    width: min(100%, 620px);
    background: rgba(255, 255, 255, 0.9);
    border: 1px solid rgba(148, 163, 184, 0.25);
    border-radius: 20px;
    padding: 2.5rem;
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.08);
  }

  .dashboard {
    display: grid;
    gap: 1rem;
  }

  .eyebrow {
    margin: 0 0 0.75rem;
    color: #6d28d9;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0;
    font-size: clamp(2rem, 3vw, 3rem);
    line-height: 1.1;
    color: #111827;
  }

  .description {
    margin: 0;
    color: #4b5563;
    font-size: 1.05rem;
  }

  .primary-button,
  .secondary-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    padding: 0.9rem 1.4rem;
    font-weight: 700;
    text-decoration: none;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    cursor: pointer;
    border: none;
  }

  .primary-button {
    margin-top: 1.5rem;
    background: linear-gradient(135deg, #7c3aed 0%, #2563eb 100%);
    color: white;
    box-shadow: 0 16px 28px rgba(124, 58, 237, 0.25);
  }

  .secondary-button {
    margin-top: 0.75rem;
    background: #111827;
    color: white;
  }

  .primary-button:hover,
  .secondary-button:hover {
    transform: translateY(-1px);
  }

  .feature-list {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 16px;
    padding: 1.25rem 1.5rem;
  }

  .feature-list h2 {
    margin: 0 0 0.75rem;
    font-size: 1.05rem;
    color: #111827;
  }
</style>
