<script>
  import { authState, logout } from '../lib/auth.svelte.js'

  function handleLogout() {
    void logout()
    window.location.hash = '/'
  }
</script>

<nav class="navbar">
  <div class="brand-wrap">
    <a href="#/" class="brand">rskel</a>
  </div>

  <div class="nav-links">
    <a href="#/">Home</a>
    <a href="#/about">About</a>

    {#if authState.isAuthenticated}
      <a href="#/profile">Profile</a>
      <button class="nav-button" onclick={handleLogout}>Logout</button>
    {:else}
      <a href="#/login">Login</a>
    {/if}
  </div>
</nav>

<style>
  .navbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 2rem;
    background: rgba(15, 23, 42, 0.96);
    border-bottom: 1px solid rgba(148, 163, 184, 0.2);
  }

  .brand {
    color: white;
    text-decoration: none;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .nav-links a,
  .nav-button {
    color: #e2e8f0;
    text-decoration: none;
    background: transparent;
    border: none;
    padding: 0.5rem 0.8rem;
    border-radius: 999px;
    font: inherit;
    cursor: pointer;
  }

  .nav-links a:hover,
  .nav-button:hover {
    background: rgba(148, 163, 184, 0.15);
  }
</style>
