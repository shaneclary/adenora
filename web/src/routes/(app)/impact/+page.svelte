<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import { authToken } from '$lib/stores/auth';

  let total = '0.00';
  let entries: any[] = [];
  let loading = true;
  let platformSummary: any = null;

  onMount(async () => {
    // Always load platform-wide summary
    try {
      platformSummary = await api.fundingSummary();
    } catch {}

    // Load personal feed if authenticated
    if ($authToken) {
      try {
        const res = await api.getImpactFeed();
        total = res.total_contributed_eur ?? '0.00';
        entries = res.entries ?? [];
      } catch {}
    }
    loading = false;
  });
</script>

<svelte:head><title>Your Impact — Adenora</title></svelte:head>

<div class="page container">
  <!-- Hero -->
  <div class="impact-hero">
    <div class="hero-glow"></div>
    <div class="hero-content">
      <span class="hero-label">Your Impact</span>
      {#if $authToken}
        <div class="hero-amount">{'\u20AC'}{total}</div>
        <p class="hero-sub">contributed to real-world projects through your predictions</p>
      {:else}
        <div class="hero-amount">{'\u20AC'}{platformSummary?.total_distributed ?? '0.00'}</div>
        <p class="hero-sub">distributed to projects by the Adenora community</p>
      {/if}
    </div>
  </div>

  <!-- How it works -->
  <div class="how-it-works">
    <h3>How your predictions create impact</h3>
    <div class="how-grid">
      <div class="how-step">
        <div class="how-num">1</div>
        <div>
          <strong>You predict</strong>
          <p>Pick YES or NO on any question</p>
        </div>
      </div>
      <div class="how-step">
        <div class="how-num">2</div>
        <div>
          <strong>Fees are split</strong>
          <p>40% goes directly to charity projects</p>
        </div>
      </div>
      <div class="how-step">
        <div class="how-num">3</div>
        <div>
          <strong>Projects grow</strong>
          <p>Clean water, schools, landmine removal</p>
        </div>
      </div>
      <div class="how-step">
        <div class="how-num">4</div>
        <div>
          <strong>You win (or learn)</strong>
          <p>Right or wrong, impact was made</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Personal feed or sign-up prompt -->
  {#if $authToken}
    <h3 class="section-title">Impact Feed</h3>
    {#if loading}
      {#each Array(3) as _}
        <div class="skeleton-row"></div>
      {/each}
    {:else if entries.length === 0}
      <div class="empty-state">
        <p style="font-size:2rem">&#10084;</p>
        <h3>No impact yet</h3>
        <p class="text-muted">Make your first prediction — the impact starts immediately.</p>
        <a href="/markets" class="btn-primary">Browse Markets</a>
      </div>
    {:else}
      <div class="feed">
        {#each entries as entry}
          <div class="feed-card">
            <div class="feed-left">
              <div class="feed-amount">{'\u20AC'}{entry.amount_eur}</div>
              <div class="feed-project">{entry.project_name}</div>
            </div>
            <div class="feed-right">
              <div class="feed-desc">{entry.description}</div>
              <div class="feed-date">{new Date(entry.date).toLocaleDateString('en', { month: 'short', day: 'numeric' })}</div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="cta-card">
      <h3>Start making an impact</h3>
      <p class="text-muted">Create an account and every prediction you make contributes to real-world projects.</p>
      <div class="cta-buttons">
        <a href="/register" class="btn-primary">Get Started</a>
        <a href="/charity" class="btn-secondary">See All Projects</a>
      </div>
    </div>
  {/if}

  <!-- Platform totals -->
  {#if platformSummary}
    <div class="platform-stats">
      <h3 class="section-title">Platform Impact</h3>
      <div class="stat-grid">
        <div class="stat-card">
          <span class="stat-value">{'\u20AC'}{platformSummary.total_distributed ?? '0'}</span>
          <span class="stat-label">Total Distributed</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{platformSummary.total_projects ?? 0}</span>
          <span class="stat-label">Active Projects</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{platformSummary.total_entries ?? 0}</span>
          <span class="stat-label">Ledger Entries</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .page { padding: 1.5rem 0 5rem; }
  .container { max-width: 700px; margin: 0 auto; padding: 0 1.25rem; }

  .impact-hero {
    position: relative; border-radius: 20px; overflow: hidden;
    padding: 2.5rem 1.5rem; margin-bottom: 2rem; text-align: center;
  }
  .hero-glow {
    position: absolute; inset: 0;
    background: radial-gradient(ellipse at 50% 50%, rgba(232,50,74,0.15) 0%, transparent 70%),
                rgba(13,27,46,0.9);
    border: 1px solid rgba(232,50,74,0.2);
    border-radius: 20px;
  }
  .hero-content { position: relative; }
  .hero-label { font-size: 0.72rem; font-weight: 700; color: var(--crimson); text-transform: uppercase; letter-spacing: 0.15em; }
  .hero-amount { font-size: clamp(2.5rem, 8vw, 4rem); font-weight: 900; font-family: var(--font-mono); color: var(--text-primary); margin: 0.5rem 0; }
  .hero-sub { color: var(--text-muted); font-size: 0.9rem; margin: 0; }

  .how-it-works { margin-bottom: 2rem; }
  .how-it-works h3 { font-size: 1rem; margin: 0 0 1rem; }
  .how-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 0.75rem; }
  @media (max-width: 500px) { .how-grid { grid-template-columns: 1fr; } }
  .how-step {
    display: flex; gap: 0.75rem; align-items: flex-start;
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 12px; padding: 1rem;
  }
  .how-num {
    width: 28px; height: 28px; border-radius: 50%; flex-shrink: 0;
    background: rgba(232,50,74,0.15); color: var(--crimson);
    display: flex; align-items: center; justify-content: center;
    font-size: 0.8rem; font-weight: 800;
  }
  .how-step strong { display: block; font-size: 0.85rem; margin-bottom: 0.15rem; }
  .how-step p { font-size: 0.78rem; color: var(--text-muted); margin: 0; }

  .section-title { font-size: 1rem; margin: 0 0 1rem; }

  .feed { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 2rem; }
  .feed-card {
    display: flex; justify-content: space-between; align-items: center;
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 10px; padding: 0.85rem 1rem;
  }
  .feed-left { display: flex; flex-direction: column; gap: 0.1rem; }
  .feed-amount { font-size: 1rem; font-weight: 700; font-family: var(--font-mono); color: var(--crimson); }
  .feed-project { font-size: 0.78rem; color: var(--text-secondary); }
  .feed-right { text-align: right; }
  .feed-desc { font-size: 0.75rem; color: var(--text-muted); max-width: 200px; }
  .feed-date { font-size: 0.68rem; color: var(--text-muted); margin-top: 0.2rem; }

  .cta-card {
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 16px; padding: 2rem; text-align: center; margin-bottom: 2rem;
  }
  .cta-card h3 { margin: 0 0 0.5rem; }
  .cta-buttons { display: flex; gap: 0.75rem; justify-content: center; margin-top: 1.25rem; flex-wrap: wrap; }

  .btn-primary {
    display: inline-block; padding: 0.7rem 1.5rem; border-radius: 10px;
    background: linear-gradient(135deg, var(--crimson), var(--gold)); color: #fff;
    font-weight: 700; text-decoration: none; font-size: 0.9rem;
  }
  .btn-secondary {
    display: inline-block; padding: 0.7rem 1.5rem; border-radius: 10px;
    background: none; border: 1px solid var(--border); color: var(--text-secondary);
    font-weight: 600; text-decoration: none; font-size: 0.9rem;
  }

  .platform-stats { margin-bottom: 2rem; }
  .stat-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; }
  .stat-card {
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 12px; padding: 1rem; text-align: center;
  }
  .stat-value { display: block; font-size: 1.25rem; font-weight: 800; font-family: var(--font-mono); color: var(--text-primary); }
  .stat-label { display: block; font-size: 0.68rem; color: var(--text-muted); text-transform: uppercase; margin-top: 0.25rem; }

  .empty-state { text-align: center; padding: 2rem; }
  .empty-state h3 { margin: 0.5rem 0; }
  .text-muted { color: var(--text-muted); }
  .text-crimson { color: var(--crimson); }

  .skeleton-row {
    height: 60px; border-radius: 10px; margin-bottom: 0.5rem;
    background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%);
    background-size: 200% 100%; animation: shimmer 1.5s infinite;
  }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
