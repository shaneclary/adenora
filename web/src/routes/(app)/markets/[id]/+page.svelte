<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';
  import { authToken } from '$lib/stores/auth';

  let market: any = null;
  let book: any = null;
  let loading = true;

  // Order form
  let side = 'yes';
  let action = 'buy';
  let price = 50;
  let quantity = 1;
  let mode = 'people';
  let orderResult: any = null;

  $: id = $page.params.id;

  onMount(async () => {
    try {
      [market, book] = await Promise.all([
        api.getMarket(id),
        api.getOrderbook(id),
      ]);
    } catch (e) {}
    loading = false;
  });

  async function placeOrder() {
    try {
      orderResult = await api.placeOrder({
        market_id: id, side, action,
        price_cents: price, quantity, mode,
      });
    } catch (e: any) {
      orderResult = { error: e.error || 'Order failed' };
    }
  }
</script>

<svelte:head><title>{market?.question || 'Market'} — Adenora</title></svelte:head>

{#if loading}
  <p class="text-muted">{$t('common.loading')}</p>
{:else if !market}
  <p class="text-muted">Market not found</p>
{:else}
  <div class="market-detail">
    <div class="market-header bg-card">
      <span class="badge badge-gold">{market.category}</span>
      <h1>{market.question}</h1>
      <p class="text-muted">{market.description}</p>
      <div class="market-info">
        <span>Closes: {new Date(market.closes_at).toLocaleString()}</span>
        <span class="badge badge-success">{market.status}</span>
      </div>
    </div>

    <div class="grid grid-2">
      <!-- Order Book -->
      <div class="bg-card">
        <h3>Order Book</h3>
        <div class="book-modes">
          <button class="btn btn-teal" class:active={mode==='people'} on:click={() => mode='people'}>
            {$t('market.people_mode')}
          </button>
          <button class="btn btn-crimson" class:active={mode==='bot'} on:click={() => mode='bot'}>
            {$t('market.bot_mode')}
          </button>
        </div>

        {#if book}
          {@const b = mode === 'people' ? book.people_book : book.bot_book}
          <div class="book-summary">
            <div class="bid-side">
              <span class="text-success">Best Bid: {b.best_bid ?? '-'}c</span>
            </div>
            <div class="spread">Spread: {b.spread ?? '-'}c</div>
            <div class="ask-side">
              <span class="text-error">Best Ask: {b.best_ask ?? '-'}c</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- Trading Panel -->
      <div class="bg-card">
        <h3>{$t('market.place_order')}</h3>
        {#if $authToken}
          <div class="trade-form">
            <div class="side-toggle">
              <button class="btn" class:btn-teal={side==='yes'} class:btn-outline={side!=='yes'} on:click={() => side='yes'}>
                {$t('market.yes')}
              </button>
              <button class="btn" class:btn-crimson={side==='no'} class:btn-outline={side!=='no'} on:click={() => side='no'}>
                {$t('market.no')}
              </button>
            </div>
            <div class="action-toggle">
              <button class="btn" class:btn-gold={action==='buy'} class:btn-outline={action!=='buy'} on:click={() => action='buy'}>
                {$t('market.buy')}
              </button>
              <button class="btn" class:btn-outline={action!=='sell'} class:btn-navy={action==='sell'} on:click={() => action='sell'}>
                {$t('market.sell')}
              </button>
            </div>
            <label>
              <span>{$t('market.price')} (cents)</span>
              <input type="number" class="input" bind:value={price} min="1" max="99" />
            </label>
            <label>
              <span>{$t('market.quantity')}</span>
              <input type="number" class="input" bind:value={quantity} min="1" />
            </label>
            <div class="cost-preview">
              Cost: {((price * quantity) / 100).toFixed(2)} EUR
            </div>
            <button class="btn btn-teal full" on:click={placeOrder}>
              {$t('market.place_order')}
            </button>
          </div>
          {#if orderResult}
            <div class="order-result" class:success={!orderResult.error} class:error-result={orderResult.error}>
              {orderResult.error || `${orderResult.status} — ${orderResult.message || ''}`}
            </div>
          {/if}
        {:else}
          <p class="text-muted">
            <a href="/login">{$t('nav.login')}</a> to trade
          </p>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .market-header h1 { font-size: 1.3rem; margin: 0.75rem 0 0.5rem; }
  .market-info { display: flex; justify-content: space-between; margin-top: 0.75rem; font-size: 0.85rem; color: var(--text-muted); }
  .book-modes { display: flex; gap: 0.5rem; margin: 1rem 0; }
  .book-modes .active { opacity: 1; }
  .book-modes button:not(.active) { opacity: 0.5; }
  .book-summary { display: flex; justify-content: space-between; align-items: center; padding: 1rem 0; font-family: var(--font-mono); }
  .spread { color: var(--text-muted); font-size: 0.85rem; }
  .trade-form { display: flex; flex-direction: column; gap: 0.75rem; }
  .side-toggle, .action-toggle { display: flex; gap: 0.5rem; }
  .side-toggle .btn, .action-toggle .btn { flex: 1; }
  label span { display: block; font-size: 0.8rem; color: var(--text-secondary); margin-bottom: 0.25rem; }
  .cost-preview { font-family: var(--font-mono); color: var(--gold); font-weight: 600; }
  .full { width: 100%; }
  .order-result { margin-top: 0.75rem; padding: 0.5rem; border-radius: 8px; font-size: 0.85rem; }
  .success { background: rgba(46,204,113,0.1); color: var(--success); }
  .error-result { background: rgba(231,76,60,0.1); color: var(--error); }
</style>
