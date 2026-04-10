<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';

  let campaigns: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      const res = await api.listVoteCampaigns();
      campaigns = res.campaigns ?? [];
    } catch {}
    loading = false;
  });

  function modeLabel(mode: string) {
    const labels: Record<string, string> = {
      one_person: 'One Person One Vote',
      capped: 'Capped Conviction',
      uncapped: 'Open Conviction',
    };
    return labels[mode] || mode;
  }

  function fundLabel(mode: string) {
    const labels: Record<string, string> = {
      winner_take_all: 'Winner Take All',
      winner_impact: 'Winner + Impact',
      winner_pool: 'Winner + Community Pool',
    };
    return labels[mode] || mode;
  }
</script>

<svelte:head><title>Community Votes — Adenora</title></svelte:head>

<div class="page container">
  <!-- Hero -->
  <div class="vote-hero">
    <div class="hero-glow"></div>
    <div class="hero-content">
      <span class="hero-tag">PARTICIPATORY BUDGETING</span>
      <h1>We don't just predict the future,<br><span class="grad-wgg">we create it.</span></h1>
      <p class="hero-sub">Vote with your conviction. Fund what matters. Every euro counts — every voice matters.</p>
    </div>
  </div>

  <!-- How it works -->
  <div class="how-section">
    <div class="how-card">
      <div class="how-icon">&#128176;</div>
      <strong>Vote with euros</strong>
      <p>Your money is your conviction — put it behind the proposal you believe in</p>
    </div>
    <div class="how-card">
      <div class="how-icon">&#9878;</div>
      <strong>Fair caps</strong>
      <p>Vote caps prevent any single person from dominating — your voice matters equally</p>
    </div>
    <div class="how-card">
      <div class="how-icon">&#10084;</div>
      <strong>All money helps</strong>
      <p>Even if your pick loses, funds go to the winner, charity, or the community pool</p>
    </div>
  </div>

  <!-- Campaigns -->
  <h2 class="section-title">Active Votes</h2>

  {#if loading}
    {#each Array(3) as _}
      <div class="skeleton-card"></div>
    {/each}
  {:else if campaigns.length === 0}
    <div class="empty-state">
      <p style="font-size:2.5rem">&#128499;</p>
      <h3>No active votes yet</h3>
      <p class="text-muted">Community votes are coming soon. Want to create one?</p>
    </div>
  {:else}
    <div class="campaign-list">
      {#each campaigns as c (c.id)}
        <a href="/vote/{c.id}" class="campaign-card">
          <div class="cc-top">
            <span class="cc-badge">{modeLabel(c.vote_mode)}</span>
            <span class="cc-time">{c.time_left} left</span>
          </div>
          <h3 class="cc-title">{c.title}</h3>
          {#if c.description}
            <p class="cc-desc">{c.description.slice(0, 120)}{c.description.length > 120 ? '...' : ''}</p>
          {/if}
          <div class="cc-bottom">
            <span class="cc-fund-mode">{fundLabel(c.fund_mode)}</span>
            {#if c.seed_amount > 0}
              <span class="cc-seed">{'\u20AC'}{c.seed_amount} seeded</span>
            {/if}
            {#if c.vote_cap && c.vote_mode === 'capped'}
              <span class="cc-cap">Max {c.vote_cap} votes</span>
            {/if}
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { padding: 1.5rem 0 5rem; }
  .container { max-width: 800px; margin: 0 auto; padding: 0 1.25rem; }

  .vote-hero {
    position: relative; border-radius: 20px; overflow: hidden;
    padding: 3rem 2rem; margin-bottom: 2rem; text-align: center;
  }
  .hero-glow {
    position: absolute; inset: 0;
    background: radial-gradient(ellipse at 30% 50%, rgba(0,194,224,0.12) 0%, transparent 60%),
                radial-gradient(ellipse at 70% 50%, rgba(240,180,41,0.12) 0%, transparent 60%),
                rgba(13,27,46,0.9);
    border: 1px solid rgba(240,180,41,0.2); border-radius: 20px;
  }
  .hero-content { position: relative; }
  .hero-tag { font-size: 0.65rem; font-weight: 700; color: var(--gold); letter-spacing: 0.2em; }
  .vote-hero h1 { font-size: clamp(1.3rem, 4vw, 2rem); font-weight: 800; margin: 0.75rem 0 0.5rem; line-height: 1.3; }
  .hero-sub { color: var(--text-muted); font-size: 0.9rem; margin: 0; }
  .grad-wgg { background: linear-gradient(135deg, var(--teal) 0%, var(--gold) 50%, var(--crimson) 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }

  .how-section { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; margin-bottom: 2rem; }
  @media (max-width: 600px) { .how-section { grid-template-columns: 1fr; } }
  .how-card {
    background: var(--bg-glass); border: 1px solid var(--border); border-radius: 12px;
    padding: 1rem; text-align: center;
  }
  .how-icon { font-size: 1.5rem; margin-bottom: 0.5rem; }
  .how-card strong { display: block; font-size: 0.85rem; margin-bottom: 0.25rem; }
  .how-card p { font-size: 0.78rem; color: var(--text-muted); margin: 0; }

  .section-title { font-size: 1.1rem; font-weight: 700; margin: 0 0 1rem; }

  .campaign-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .campaign-card {
    display: block; text-decoration: none; color: var(--text-primary);
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 14px; padding: 1.25rem; transition: all 0.15s;
  }
  .campaign-card:hover { border-color: rgba(240,180,41,0.3); transform: translateY(-2px); }

  .cc-top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.6rem; }
  .cc-badge { font-size: 0.68rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: 6px; background: rgba(0,194,224,0.12); color: var(--teal); text-transform: uppercase; letter-spacing: 0.05em; }
  .cc-time { font-size: 0.72rem; color: var(--text-muted); }
  .cc-title { font-size: 1.05rem; font-weight: 700; margin: 0 0 0.3rem; line-height: 1.3; }
  .cc-desc { font-size: 0.82rem; color: var(--text-muted); margin: 0 0 0.75rem; line-height: 1.4; }
  .cc-bottom { display: flex; gap: 0.75rem; flex-wrap: wrap; }
  .cc-fund-mode { font-size: 0.7rem; padding: 0.15rem 0.5rem; border-radius: 4px; background: rgba(240,180,41,0.1); color: var(--gold); }
  .cc-seed { font-size: 0.72rem; color: var(--text-muted); font-family: var(--font-mono); }
  .cc-cap { font-size: 0.72rem; color: var(--text-muted); }

  .empty-state { text-align: center; padding: 3rem; }
  .empty-state h3 { margin: 0.5rem 0; }
  .text-muted { color: var(--text-muted); }

  .skeleton-card {
    height: 140px; border-radius: 14px; margin-bottom: 0.75rem;
    background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%);
    background-size: 200% 100%; animation: shimmer 1.5s infinite;
  }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
