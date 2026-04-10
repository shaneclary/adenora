<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import { authToken } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let predictions: any[] = [];
  let summary = { total_invested_eur: '0.00', total_potential_win_eur: '0.00', total_charity_contributed_eur: '0.00' };
  let loading = true;
  let error = '';
  let tab: 'active' | 'resolved' = 'active';

  onMount(async () => {
    if (!$authToken) { goto('/login'); return; }
    try {
      const res = await api.getPredictions();
      predictions = res.predictions ?? [];
      summary = res;
    } catch (e: any) {
      // Fall back to raw positions
      try {
        const res = await api.getPositions();
        predictions = (res.positions ?? []).map((p: any) => ({
          ...p,
          question: p.market_id?.slice(0, 12) + '...',
          your_side: p.side,
          amount_spent_eur: p.value,
          potential_win_eur: p.quantity,
          status: 'active',
          time_left: '—',
        }));
      } catch {
        error = 'Unable to load predictions';
      }
    }
    loading = false;
  });

  $: filtered = predictions.filter(p => {
    if (tab === 'active') return p.status === 'active' || p.status === 'pending';
    return p.status === 'resolved' || p.status === 'won' || p.status === 'lost';
  });
</script>

<svelte:head><title>Your Predictions — Adenora</title></svelte:head>

<div class="page container">
  <h1 class="page-title">Your Predictions</h1>

  <!-- Summary cards -->
  <div class="summary-grid">
    <div class="sum-card">
      <span class="sum-label">Invested</span>
      <span class="sum-value">{'\u20AC'}{summary.total_invested_eur}</span>
    </div>
    <div class="sum-card">
      <span class="sum-label">Potential Winnings</span>
      <span class="sum-value text-teal">{'\u20AC'}{summary.total_potential_win_eur}</span>
    </div>
    <div class="sum-card">
      <span class="sum-label">Impact</span>
      <span class="sum-value text-crimson">{'\u20AC'}{summary.total_charity_contributed_eur}</span>
      <a href="/impact" class="sum-link">See full impact</a>
    </div>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    <button class="tab" class:active={tab==='active'} on:click={() => tab='active'}>Active</button>
    <button class="tab" class:active={tab==='resolved'} on:click={() => tab='resolved'}>Resolved</button>
  </div>

  {#if loading}
    {#each Array(3) as _}
      <div class="skeleton-row"></div>
    {/each}
  {:else if error}
    <div class="empty-state">
      <p class="text-crimson">{error}</p>
    </div>
  {:else if filtered.length === 0}
    <div class="empty-state">
      <p style="font-size:2rem">&#128064;</p>
      <h3>{tab === 'active' ? 'No active predictions' : 'No resolved predictions yet'}</h3>
      <p class="text-muted">
        {tab === 'active' ? 'Browse markets and make your first prediction.' : 'Your resolved predictions will appear here.'}
      </p>
      {#if tab === 'active'}
        <a href="/markets" class="btn-primary">Browse Markets</a>
      {/if}
    </div>
  {:else}
    <div class="pred-list">
      {#each filtered as pred (pred.id)}
        <a href="/markets/{pred.market_id}" class="pred-card">
          <div class="pred-top">
            <span class="pred-badge" class:yes={pred.your_side === 'yes'} class:no={pred.your_side === 'no'}>
              You said {pred.your_side?.toUpperCase()}
            </span>
            <span class="pred-time">{pred.time_left}</span>
          </div>
          <div class="pred-question">{pred.question}</div>
          <div class="pred-bottom">
            <div class="pred-stat">
              <span class="pred-stat-label">Spent</span>
              <span class="pred-stat-value">{'\u20AC'}{pred.amount_spent_eur}</span>
            </div>
            <div class="pred-stat">
              <span class="pred-stat-label">Win if right</span>
              <span class="pred-stat-value text-teal">{'\u20AC'}{pred.potential_win_eur}</span>
            </div>
            {#if pred.charity_project}
              <div class="pred-stat">
                <span class="pred-stat-label">Funded</span>
                <span class="pred-stat-value text-crimson">{pred.charity_project}</span>
              </div>
            {/if}
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { padding: 1.5rem 0 5rem; }
  .page-title { font-size: 1.5rem; font-weight: 800; margin: 0 0 1.25rem; }
  .container { max-width: 700px; margin: 0 auto; padding: 0 1.25rem; }

  .summary-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; margin-bottom: 1.5rem; }
  @media (max-width: 500px) { .summary-grid { grid-template-columns: 1fr; } }
  .sum-card {
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 12px; padding: 1rem; display: flex; flex-direction: column; gap: 0.2rem;
  }
  .sum-label { font-size: 0.72rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .sum-value { font-size: 1.25rem; font-weight: 800; font-family: var(--font-mono); }
  .sum-link { font-size: 0.72rem; color: var(--crimson); text-decoration: none; margin-top: 0.25rem; }

  .tabs { display: flex; gap: 0.25rem; margin-bottom: 1rem; border-bottom: 1px solid var(--border); }
  .tab {
    background: none; border: none; color: var(--text-muted); padding: 0.6rem 1rem;
    font-size: 0.85rem; font-weight: 600; cursor: pointer; border-bottom: 2px solid transparent;
    margin-bottom: -1px; transition: color 0.15s;
  }
  .tab.active { color: var(--teal); border-bottom-color: var(--teal); }

  .pred-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .pred-card {
    display: block; text-decoration: none; color: var(--text-primary);
    background: var(--bg-glass); border: 1px solid var(--border);
    border-radius: 12px; padding: 1rem; transition: all 0.15s;
  }
  .pred-card:hover { border-color: rgba(255,255,255,0.12); transform: translateY(-1px); }

  .pred-top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem; }
  .pred-badge {
    font-size: 0.72rem; font-weight: 700; padding: 0.2rem 0.6rem; border-radius: 6px;
    text-transform: uppercase; letter-spacing: 0.06em;
  }
  .pred-badge.yes { background: rgba(0,194,224,0.15); color: var(--teal); }
  .pred-badge.no { background: rgba(232,50,74,0.12); color: var(--crimson); }
  .pred-time { font-size: 0.72rem; color: var(--text-muted); }
  .pred-question { font-size: 0.9rem; font-weight: 600; line-height: 1.4; margin-bottom: 0.75rem; }
  .pred-bottom { display: flex; gap: 1.25rem; flex-wrap: wrap; }
  .pred-stat { display: flex; flex-direction: column; gap: 0.1rem; }
  .pred-stat-label { font-size: 0.68rem; color: var(--text-muted); }
  .pred-stat-value { font-size: 0.9rem; font-weight: 700; font-family: var(--font-mono); }

  .text-teal { color: var(--teal); }
  .text-crimson { color: var(--crimson); }
  .text-muted { color: var(--text-muted); }

  .empty-state { text-align: center; padding: 3rem 1rem; }
  .empty-state h3 { margin: 0.5rem 0; }
  .btn-primary {
    display: inline-block; margin-top: 1rem; padding: 0.7rem 1.5rem; border-radius: 10px;
    background: linear-gradient(135deg, var(--teal), var(--gold)); color: #000;
    font-weight: 700; text-decoration: none;
  }

  .skeleton-row {
    height: 100px; border-radius: 12px; margin-bottom: 0.75rem;
    background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%);
    background-size: 200% 100%; animation: shimmer 1.5s infinite;
  }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
