<main class="page-shell">
  <section class="card">
    <p class="eyebrow">About</p>
    <h3>Efficient modern UI</h3>
    <p>
      This app combines a Rust + Rocket backend with a Svelte frontend to deliver a secure,
      OAuth-enabled experience that is easy to extend with additional providers and features.
    </p>
    <a href="#/" class="back-link">Return home</a>
  </section>
</main>

<style>
  .page-shell {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: linear-gradient(180deg, #f8fafc 0%, #f5f3ff 100%);
    padding: 2rem;
  }

  .card {
    width: min(100%, 560px);
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 20px;
    box-shadow: 0 18px 48px rgba(15, 23, 42, 0.08);
    padding: 2rem;
  }

  .eyebrow {
    margin: 0 0 0.5rem;
    color: #7c3aed;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h3 {
    margin: 0 0 1rem;
    color: #111827;
  }

  p {
    margin: 0 0 1.25rem;
    color: #374151;
    line-height: 1.7;
  }

  .back-link {
    color: #4f46e5;
    text-decoration: none;
    font-weight: 600;
  }
</style>
