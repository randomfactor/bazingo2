<script>
  import { authState } from '../lib/auth.svelte.js'
</script>

<main class="page-shell">
  <section class="card">
    <p class="eyebrow">Profile</p>
    <h1>{authState.user?.name ?? 'Member'}</h1>
    <p><strong>Email:</strong> {authState.user?.email ?? 'Not available'}</p>
    <p>
      This route is protected and only reachable while the session is authenticated.
    </p>
  </section>
</main>

<style>
  .page-shell {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: linear-gradient(180deg, #f8fafc 0%, #ecfeff 100%);
    padding: 2rem;
  }

  .card {
    width: min(100%, 540px);
    background: white;
    border: 1px solid #dbeafe;
    border-radius: 20px;
    box-shadow: 0 18px 48px rgba(14, 116, 144, 0.08);
    padding: 2rem;
  }

  .eyebrow {
    margin: 0 0 0.5rem;
    color: #0f766e;
    font-size: 0.76rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0 0 1rem;
    color: #111827;
  }

  p {
    color: #374151;
    line-height: 1.7;
  }
</style>
