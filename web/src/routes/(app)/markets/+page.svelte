<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import OpinionCard from '$lib/components/OpinionCard.svelte';
  import PredictModal from '$lib/components/PredictModal.svelte';

  let markets: any[] = [];
  let loading = true;
  let error = '';
  let filter = 'all';

  // Predict modal state
  let predicting: any = null;
  let predictSide = 'yes';

  // Enrich with preview analytics when API doesn't have real data
  function enrich(m: any) {
    const hash = m.id ? m.id.charCodeAt(0) + m.id.charCodeAt(1) : 50;
    return {
      ...m,
      chance_yes_pct: m.chance_yes_pct ?? (20 + (hash % 60)),
      time_left: m.time_left ?? '—',
      charity_project: m.charity_project ?? null,
    };
  }

  onMount(async () => {
    try {
      // Try the human-friendly cards endpoint first
      const res = await api.listMarketCards();
      markets = (res.cards || []).map(enrich);
    } catch {
      try {
        // Fall back to raw market list
        const res = await api.listMarkets();
        markets = (res.markets || []).map((m: any) => ({
          ...m,
          question: m.question || m.title,
          chance_yes_pct: m.yes_price ?? (20 + ((m.id?.charCodeAt(0) ?? 50) % 60)),
          time_left: '—',
          charity_project: null,
        }));
      } catch (e: any) {
        error = 'Unable to load markets. Please refresh.';
      }
    }
    loading = false;
  });

  $: filtered = filter === 'all' ? markets : markets.filter(m => m.category?.toLowerCase() === filter);

  function openPredict(market: any, side: string) {
    predicting = market;
    predictSide = side;
  }

  function closePredict() {
    predicting = null;
  }

  const FILTERS = ['all', 'geopolitics', 'economy', 'sports', 'tech'];
</script>

<svelte:head><title>Markets — Adenora</title></svelte:head>

<div class="page">
  <!-- Header -->
  <div class="page-header container">
    <div>
      <h1 class="page-title">What do you think?</h1>
      <p class="page-desc">Browse questions, pick a side, put your money where your mouth is.</p>
    </div>
    <a href="/unlimited" class="adv-link">Advanced Trading</a>
  </div>

  <!-- Filters -->
  <div class="filter-bar container">
    {#each FILTERS as f}
      <button
        class="filter-pill"
        class:active={filter === f}
        on:click={() => filter = f}
      >{f === 'all' ? 'All' : f.charAt(0).toUpperCase() + f.slice(1)}</button>
    {/each}
  </div>

  <!-- Markets grid -->
  <div class="container">
    {#if loading}
      <div class="card-grid">
        {#each Array(6) as _}
          <div class="skeleton-card"></div>
        {/each}
      </div>
    {:else if error}
      <div class="empty-state">
        <p class="error-msg">{error}</p>
        <button class="btn-refresh" on:click={() => location.reload()}>Refresh</button>
      </div>
    {:else if filtered.length === 0}
      <div class="empty-state">
        <p style="font-size:2rem">&#128200;</p>
        <h3>No markets yet</h3>
        <p class="text-muted">Check back soon or propose one.</p>
      </div>
    {:else}
      <div class="card-grid">
        {#each filtered as market (market.id)}
          <OpinionCard {market} onPredict={openPredict} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Predict modal -->
{#if predicting}
  <PredictModal market={predicting} initialSide={predictSide} onClose={closePredict} />
{/if}

<style>
  .page { padding: 1.5rem 0 5rem; }
  .page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1.25rem; flex-wrap: wrap; gap: 0.75rem; }
  .page-title { font-size: clamp(1.5rem, 4vw, 2.2rem); font-weight: 800; margin: 0 0 0.25rem; }
  .page-desc { color: var(--text-muted); font-size: 0.9rem; margin: 0; max-width: 420px; }
  .adv-link {
    font-size: 0.78rem; color: var(--text-muted); text-decoration: none;
    padding: 0.4rem 0.8rem; border: 1px solid var(--border); border-radius: 8px;
    transition: all 0.15s; margin-top: 0.5rem;
  }
  .adv-link:hover { color: var(--gold); border-color: rgba(240,180,41,0.3); }

  .filter-bar { display: flex; gap: 0.4rem; margin-bottom: 1.5rem; overflow-x: auto; padding-bottom: 0.25rem; }
  .filter-pill {
    padding: 0.4rem 1rem; border-radius: 999px; border: 1px solid var(--border);
    background: rgba(255,255,255,0.03); color: var(--text-muted);
    font-size: 0.8rem; font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: all 0.15s;
  }
  .filter-pill.active { background: rgba(240,180,41,0.15); border-color: var(--gold); color: var(--gold); }
  .filter-pill:hover:not(.active) { background: rgba(255,255,255,0.06); }

  .card-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem; }
  @media (max-width: 640px) { .card-grid { grid-template-columns: 1fr; } }

  .skeleton-card {
    height: 220px; border-radius: 16px;
    background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%);
    background-size: 200% 100%; animation: shimmer 1.5s infinite;
  }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }

  .empty-state { text-align: center; padding: 3rem; }
  .empty-state h3 { margin: 0.5rem 0; }
  .error-msg { color: var(--crimson); margin-bottom: 1rem; }
  .btn-refresh {
    padding: 0.5rem 1.25rem; border-radius: 8px; border: 1px solid var(--border);
    background: none; color: var(--text-secondary); cursor: pointer;
  }
  .text-muted { color: var(--text-muted); }
  .container { max-width: 1100px; margin: 0 auto; padding: 0 1.25rem; }
</style>
