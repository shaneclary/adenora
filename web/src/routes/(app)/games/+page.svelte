<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';

  let games: any[] = [];
  let tournaments: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      [{ games }, { tournaments }] = await Promise.all([api.listGames(), api.listTournaments()]);
    } catch (e) {}
    loading = false;
  });
</script>

<svelte:head><title>{$t('nav.games')} — Adenora</title></svelte:head>

<div class="page-header">
  <h1><span class="dot dot-win"></span> {$t('nav.games')}</h1>
</div>

{#if loading}
  <p class="text-muted">{$t('common.loading')}</p>
{:else}
  {#if games.length === 0 && tournaments.length === 0}
    <div class="empty bg-card">
      <p class="hero-emoji">&#127918;</p>
      <h3>Games Coming Soon</h3>
      <p class="text-muted">Trivia, forecasting challenges, and tournaments are on the way.</p>
    </div>
  {:else}
    {#if games.length > 0}
      <h2 class="section-title">Available Games</h2>
      <div class="grid grid-3">
        {#each games as game}
          <div class="bg-card">
            <span class="badge badge-teal">{game.game_type}</span>
            <h3>{game.name}</h3>
            <p class="text-muted">{game.description}</p>
            {#if game.is_free_to_play}
              <span class="badge badge-success">Free to play</span>
            {:else}
              <span class="entry-fee">{game.entry_fee} EUR</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if tournaments.length > 0}
      <h2 class="section-title">Tournaments</h2>
      <div class="grid">
        {#each tournaments as t}
          <div class="bg-card tournament-row">
            <div><h3>{t.name}</h3><span class="text-muted">{t.current_participants}/{t.max_participants} players</span></div>
            <div class="tournament-info">
              <span class="prize">Prize: {t.prize_pool} EUR</span>
              <span class="badge badge-gold">{t.status}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
{/if}

<style>
  .page-header { margin-bottom: 1.5rem; }
  .page-header h1 { display: flex; align-items: center; gap: 0.5rem; font-size: 1.5rem; }
  .dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
  .dot-win { background: var(--pillar-win); }
  .empty { text-align: center; padding: 3rem; }
  .hero-emoji { font-size: 3rem; margin-bottom: 1rem; }
  .section-title { font-size: 1.1rem; margin: 2rem 0 1rem; color: var(--text-secondary); }
  .entry-fee { font-family: var(--font-mono); color: var(--gold); font-weight: 600; }
  .tournament-row { display: flex; justify-content: space-between; align-items: center; }
  .tournament-info { display: flex; gap: 1rem; align-items: center; }
  .prize { font-family: var(--font-mono); color: var(--gold); font-weight: 600; }
</style>
