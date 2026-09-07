<script>
  import { onMount } from 'svelte'
  import Router, { replace } from 'svelte-spa-router'
  import { authState, checkSession } from './lib/auth.svelte.js'
  import { routes } from './routes.js'
  import Navbar from './components/Navbar.svelte'

  function handleConditionsFailed(event) {
    const detail = event?.detail
    if (detail?.route === '/profile') {
      replace('/login')
    }
  }

  onMount(() => {
    void checkSession()
  })
</script>

{#if authState.isLoading}
  <main class="loading-shell">
    <div class="spinner" aria-live="polite" aria-label="Loading session"></div>
  </main>
{:else}
  <div class="app-shell">
    <Navbar />
    <Router {routes} on:conditionsFailed={handleConditionsFailed} />
  </div>
{/if}

<style>
  .app-shell {
    min-height: 100vh;
    background: #f8fafc;
  }

  .loading-shell {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: linear-gradient(180deg, #f8fafc 0%, #eef2ff 100%);
  }

  .spinner {
    width: 52px;
    height: 52px;
    border: 4px solid #e2e8f0;
    border-top-color: #7c3aed;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
