<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';

  let lotteries: any[] = [];
  let loading = true;

  onMount(async () => {
    try { const res = await api.listLotteries(); lotteries = res.lotteries; }
    catch (e) {}
    loading = false;
  });
</script>

<svelte:head><title>{$t('lottery.title')} — Adenora</title></svelte:head>

<div class="page-header">
  <div>
    <h1><span class="dot dot-give"></span> {$t('lottery.title')}</h1>
    <p class="text-muted">{$t('lottery.subtitle')}</p>
  </div>
</div>

{#if loading}
  <p class="text-muted">{$t('common.loading')}</p>
{:else if lotteries.length === 0}
  <div class="empty bg-card">
    <p class="hero-emoji">&#127922;</p>
    <h3>Coming Soon</h3>
    <p class="text-muted">Lottery games are being set up. Check back soon to Donate and Play.</p>
  </div>
{:else}
  <div class="grid grid-3">
    {#each lotteries as lottery}
      <div class="lottery-card bg-card">
        <span class="badge badge-crimson">{lottery.game_type}</span>
        <h3>{lottery.name}</h3>
        <p class="text-muted">{lottery.description}</p>
        <div class="lottery-meta">
          <span class="price">{lottery.ticket_price} {lottery.currency}</span>
          <span class="pct">{lottery.prize_pct}% to prizes</span>
        </div>
        <button class="btn btn-crimson full">{$t('lottery.buy_ticket')}</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .page-header { margin-bottom: 1.5rem; }
  .page-header h1 { display: flex; align-items: center; gap: 0.5rem; font-size: 1.5rem; }
  .dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
  .dot-give { background: var(--pillar-give); }
  .empty { text-align: center; padding: 3rem; }
  .hero-emoji { font-size: 3rem; margin-bottom: 1rem; }
  .lottery-card h3 { margin: 0.75rem 0 0.5rem; }
  .lottery-meta { display: flex; justify-content: space-between; margin: 1rem 0; font-size: 0.85rem; }
  .price { font-weight: 700; color: var(--gold); font-family: var(--font-mono); }
  .pct { color: var(--text-muted); }
  .full { width: 100%; }
</style>
