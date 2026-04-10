<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';

  let markets: any[] = [];
  let loading = true;
  let selectedMarket: any = null;
  let orderBook: any = null;
  let bookLoading = false;

  // Order form
  let side = 'yes';
  let action = 'buy';
  let priceCents = 50;
  let quantity = 10;
  let tif = 'gtc';
  let submitting = false;
  let orderResult: any = null;
  let orderError = '';

  // Disclaimer acknowledgement
  let acknowledged = false;

  onMount(async () => {
    try {
      const res = await api.listMarkets();
      markets = (res.markets ?? []).map(enrich);
    } catch {}
    loading = false;
  });

  function enrich(m: any) {
    return {
      ...m,
      yes_price: m.yes_price ?? Math.floor(Math.random() * 70 + 15),
      volume: m.volume ?? Math.floor(Math.random() * 50000 + 1000),
      traders: m.traders ?? Math.floor(Math.random() * 300 + 10),
    };
  }

  async function selectMarket(m: any) {
    selectedMarket = m;
    orderBook = null;
    bookLoading = true;
    orderResult = null;
    orderError = '';
    try {
      const res = await api.getOrderbook(m.id);
      orderBook = res?.bot_book ?? res;
    } catch {}
    bookLoading = false;
  }

  async function placeOrder() {
    if (!selectedMarket) return;
    submitting = true; orderResult = null; orderError = '';
    try {
      const res = await api.placeOrder({
        market_id: selectedMarket.id,
        mode: 'ultimate',
        side, action,
        price_cents: priceCents,
        quantity,
        time_in_force: tif,
      });
      orderResult = res;
    } catch (e: any) {
      orderError = e?.message ?? 'Order failed';
    }
    submitting = false;
  }

  function yesColor(p: number) {
    if (p >= 70) return 'var(--teal)';
    if (p <= 30) return 'var(--crimson)';
    return 'var(--gold)';
  }

  $: impliedNo = 100 - priceCents;
</script>

<svelte:head><title>Ultimate Markets — Adenora</title></svelte:head>

<!-- Header -->
<div class="page-header">
  <div>
    <div class="mode-tag">ULTIMATE MARKETS</div>
    <h1>Ultimate <span class="grad-gold">Execution</span></h1>
    <p class="text-muted">Continuous matching on eligible event markets. Humans can opt in here, while bot participation is API-first by design.</p>
  </div>
  <div class="mode-badges">
    <span class="mbadge teal">Continuous</span>
    <span class="mbadge gold">Opt-in</span>
    <span class="mbadge neutral">API Bot Access</span>
  </div>
</div>

<p class="page-note text-muted">Volume, trader counts, and implied prices on this screen are preview analytics until live market telemetry is wired into the frontend.</p>

<!-- Disclaimer banner — must acknowledge once per session -->
{#if !acknowledged}
  <div class="disclaimer-banner">
    <div class="disclaimer-icon">⚠</div>
    <div class="disclaimer-body">
      <strong>You are entering Ultimate Markets</strong>
      <p>Ultimate Markets use <em>continuous matching</em> on the advanced book. API bot access lives here, and humans join only when they want this faster execution environment.</p>
    </div>
    <button class="btn btn-gold" on:click={() => acknowledged = true}>I Understand — Enter</button>
  </div>
{/if}

{#if acknowledged}
<div class="trading-layout">
  <!-- Left: market list -->
  <div class="market-list-col">
    <div class="glass-card" style="padding:0">
      <div class="list-header">
        <span>Markets</span>
        <span class="text-muted" style="font-size:0.75rem">{markets.length} active</span>
      </div>
      {#if loading}
        {#each Array(6) as _}
          <div class="skeleton" style="height:60px;margin:0.5rem;border-radius:8px"></div>
        {/each}
      {:else if markets.length === 0}
        <p class="text-muted" style="padding:1rem">No active markets</p>
      {:else}
        {#each markets as m}
          <button
            class="market-row"
            class:selected={selectedMarket?.id === m.id}
            on:click={() => selectMarket(m)}
          >
            <div class="mrow-title">{m.question ?? m.title}</div>
            <div class="mrow-meta">
              <span class="yes-price" style="color:{yesColor(m.yes_price)}">{m.yes_price}¢</span>
              <span class="text-muted">{(m.volume/1000).toFixed(1)}K vol</span>
            </div>
            <div class="prob-bar-small">
              <div class="prob-fill-small" style="width:{m.yes_price}%;background:{yesColor(m.yes_price)}"></div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Center: order book -->
  <div class="book-col">
    {#if !selectedMarket}
      <div class="glass-card book-empty">
        <p style="font-size:2rem">📈</p>
        <p class="text-muted">Select a market to see the order book</p>
      </div>
    {:else}
      <div class="glass-card book-card">
        <div class="book-header">
          <div>
            <h3 class="book-title">{selectedMarket.question ?? selectedMarket.title}</h3>
            <div class="book-badges">
              <span class="mbadge gold" style="font-size:0.65rem">ULTIMATE</span>
              <span class="mbadge neutral" style="font-size:0.65rem">{selectedMarket.yes_price}¢ YES</span>
            </div>
          </div>
          <div class="book-stats">
            <div class="bstat"><span>{selectedMarket.traders}</span><span class="text-muted">traders</span></div>
            <div class="bstat"><span>€{(selectedMarket.volume/100).toFixed(0)}</span><span class="text-muted">volume</span></div>
          </div>
        </div>

        <!-- Spread view -->
        <div class="spread-bar">
          <div class="spread-yes" style="width:{selectedMarket.yes_price}%">YES {selectedMarket.yes_price}¢</div>
          <div class="spread-no" style="width:{100-selectedMarket.yes_price}%">NO {100-selectedMarket.yes_price}¢</div>
        </div>

        {#if bookLoading}
          <div class="book-skeleton">
            {#each Array(8) as _}
              <div class="skeleton" style="height:20px;margin-bottom:4px;border-radius:4px"></div>
            {/each}
          </div>
        {:else}
          <div class="book-grid">
            <div class="book-side asks">
              <div class="book-side-label">Asks (Sell)</div>
              {#if orderBook?.asks?.length}
                {#each orderBook.asks.slice(0,8).reverse() as ask}
                  <div class="book-row ask-row">
                    <span class="book-price ask-price">{ask.price}</span>
                    <span class="book-qty">{ask.quantity}</span>
                    <div class="book-depth" style="width:{Math.min(ask.quantity/10,100)}%;background:rgba(232,50,74,0.15)"></div>
                  </div>
                {/each}
              {:else}
                <p class="text-muted" style="font-size:0.78rem;padding:0.5rem">No asks</p>
              {/if}
            </div>
            <div class="book-spread-line">
              <span class="spread-label">Spread: {Math.abs((orderBook?.best_ask ?? selectedMarket.yes_price+2) - (orderBook?.best_bid ?? selectedMarket.yes_price-1))}¢</span>
            </div>
            <div class="book-side bids">
              <div class="book-side-label">Bids (Buy)</div>
              {#if orderBook?.bids?.length}
                {#each orderBook.bids.slice(0,8) as bid}
                  <div class="book-row bid-row">
                    <span class="book-price bid-price">{bid.price}</span>
                    <span class="book-qty">{bid.quantity}</span>
                    <div class="book-depth" style="width:{Math.min(bid.quantity/10,100)}%;background:rgba(0,194,224,0.15)"></div>
                  </div>
                {/each}
              {:else}
                <p class="text-muted" style="font-size:0.78rem;padding:0.5rem">No bids</p>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Right: order entry -->
  <div class="order-col">
    <div class="glass-card order-card">
      <h3>Place Order</h3>

      {#if !selectedMarket}
        <p class="text-muted" style="font-size:0.85rem">Select a market first</p>
      {:else}
        <!-- Side toggle -->
        <div class="seg-ctrl">
          <button class="seg" class:active-yes={side==='yes'} on:click={() => side='yes'}>YES</button>
          <button class="seg" class:active-no={side==='no'} on:click={() => side='no'}>NO</button>
        </div>

        <!-- Action toggle -->
        <div class="seg-ctrl" style="margin-top:0.5rem">
          <button class="seg" class:active-buy={action==='buy'} on:click={() => action='buy'}>Buy</button>
          <button class="seg" class:active-sell={action==='sell'} on:click={() => action='sell'}>Sell</button>
        </div>

        <div class="form-group" style="margin-top:1rem">
          <label class="form-label" for="ultimate-price">Price (¢) — implied: NO {impliedNo}¢</label>
          <input id="ultimate-price" type="range" min="1" max="99" bind:value={priceCents} class="price-slider" />
          <div class="price-display" style="color:{yesColor(priceCents)}">{priceCents}¢</div>
        </div>

        <div class="form-group">
          <label class="form-label" for="ultimate-quantity">Quantity (contracts)</label>
          <input id="ultimate-quantity" type="number" min="1" max="10000" bind:value={quantity} class="form-input" />
        </div>

        <div class="form-group">
          <label class="form-label" for="ultimate-tif">Time in Force</label>
          <select id="ultimate-tif" class="form-input" bind:value={tif}>
            <option value="ioc">IOC — Immediate or Cancel</option>
            <option value="gtc">GTC — Good Till Cancelled</option>
            <option value="fok">FOK — Fill or Kill</option>
          </select>
        </div>

        <div class="order-summary">
          <div class="os-row"><span class="text-muted">Cost estimate</span><span>€{((priceCents * quantity)/100).toFixed(2)}</span></div>
          <div class="os-row"><span class="text-muted">Mode</span><span class="mbadge gold" style="font-size:0.7rem">ULTIMATE</span></div>
          <div class="os-row"><span class="text-muted">Matching</span><span>Continuous</span></div>
        </div>

        {#if orderResult}
          <div class="alert alert-success">
            {#if orderResult.status === 'executed'}
              Filled {orderResult.trades} trade(s) — order {orderResult.order_id?.slice(0,8)}
            {:else}
              Order submitted: {orderResult.order_id?.slice(0,8)}
            {/if}
          </div>
        {/if}
        {#if orderError}
          <div class="alert alert-error">{orderError}</div>
        {/if}

        <button
          class="btn btn-trade"
          class:btn-yes={side==='yes' && action==='buy'}
          class:btn-no={side==='no' && action==='buy'}
          class:btn-sell={action==='sell'}
          on:click={placeOrder}
          disabled={submitting}
          style="width:100%"
        >
          {submitting ? 'Submitting…' : `${action === 'buy' ? 'Buy' : 'Sell'} ${side.toUpperCase()} @ ${priceCents}¢`}
        </button>
      {/if}
    </div>

    <!-- Mode explainer -->
    <div class="glass-card explainer">
      <h4>About Ultimate Markets</h4>
      <ul class="explainer-list">
        <li><strong>Continuous matching</strong> — no 500ms batch windows</li>
        <li><strong>Opt-in participation</strong> — humans join knowingly</li>
        <li><strong>API bot access</strong> — advanced automation lives here</li>
        <li><strong>Same fee model</strong> — identical parabolic fee schedule</li>
        <li><strong>Same resolution</strong> — identical oracle and jury process</li>
      </ul>
      <p class="text-muted" style="font-size:0.75rem;margin-top:0.75rem">
        Want the default fair-auction surface? Switch to <a href="/markets" style="color:var(--teal)">Human Markets</a>.
      </p>
    </div>
  </div>
</div>
{/if}

<style>
  .page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1.5rem; flex-wrap: wrap; gap: 1rem; }
  .page-note { margin: -0.5rem 0 1.5rem; font-size: 0.78rem; }
  .mode-tag { display: inline-block; background: rgba(240,180,41,0.15); border: 1px solid var(--gold); color: var(--gold); font-size: 0.65rem; font-weight: 700; letter-spacing: 0.15em; padding: 0.2rem 0.6rem; border-radius: 4px; margin-bottom: 0.5rem; }
  .page-header h1 { font-size: clamp(1.5rem, 3vw, 2.5rem); font-weight: 800; margin: 0 0 0.25rem; }
  .mode-badges { display: flex; flex-wrap: wrap; gap: 0.4rem; align-items: flex-start; padding-top: 0.5rem; }

  /* Disclaimer */
  .disclaimer-banner {
    display: flex; align-items: flex-start; gap: 1rem;
    background: rgba(240,180,41,0.06); border: 1px solid rgba(240,180,41,0.3);
    border-radius: 12px; padding: 1.25rem 1.5rem; margin-bottom: 1.5rem; flex-wrap: wrap;
  }
  .disclaimer-icon { font-size: 1.5rem; flex-shrink: 0; color: var(--gold); }
  .disclaimer-body { flex: 1; }
  .disclaimer-body strong { display: block; margin-bottom: 0.4rem; }
  .disclaimer-body p { font-size: 0.85rem; color: var(--text-muted); margin: 0; }
  .disclaimer-body em { color: var(--gold); font-style: normal; }

  /* Trading layout */
  .trading-layout { display: grid; grid-template-columns: 240px 1fr 280px; gap: 1rem; align-items: start; }
  @media (max-width: 1100px) { .trading-layout { grid-template-columns: 1fr 1fr; } .order-col { grid-column: 1/-1; } }
  @media (max-width: 700px) { .trading-layout { grid-template-columns: 1fr; } }

  /* Market list */
  .glass-card { background: var(--bg-glass); border: 1px solid var(--border); backdrop-filter: blur(12px); border-radius: 12px; margin-bottom: 1rem; }
  .list-header { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; border-bottom: 1px solid var(--border); font-weight: 600; font-size: 0.85rem; }
  .market-row { display: block; width: 100%; text-align: left; background: none; border: none; border-bottom: 1px solid rgba(255,255,255,0.04); padding: 0.75rem 1rem; cursor: pointer; transition: background 0.15s; }
  .market-row:hover, .market-row.selected { background: rgba(255,255,255,0.04); }
  .market-row.selected { border-left: 2px solid var(--gold); }
  .mrow-title { font-size: 0.8rem; font-weight: 500; color: var(--text); margin-bottom: 0.3rem; line-height: 1.3; }
  .mrow-meta { display: flex; justify-content: space-between; font-size: 0.73rem; margin-bottom: 0.3rem; }
  .yes-price { font-weight: 700; font-family: var(--font-mono); }
  .prob-bar-small { height: 3px; background: rgba(255,255,255,0.08); border-radius: 2px; }
  .prob-fill-small { height: 100%; border-radius: 2px; transition: width 0.3s; }

  /* Order book */
  .book-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 300px; gap: 0.5rem; }
  .book-card { padding: 1.25rem; }
  .book-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem; }
  .book-title { font-size: 0.95rem; font-weight: 700; margin: 0 0 0.4rem; }
  .book-badges { display: flex; gap: 0.4rem; }
  .book-stats { display: flex; gap: 1.5rem; }
  .bstat { display: flex; flex-direction: column; align-items: flex-end; gap: 0.1rem; font-size: 0.8rem; }
  .bstat span:first-child { font-weight: 700; }
  .spread-bar { display: flex; height: 28px; border-radius: 6px; overflow: hidden; margin-bottom: 1rem; }
  .spread-yes { background: rgba(0,194,224,0.25); display: flex; align-items: center; justify-content: center; font-size: 0.72rem; font-weight: 700; color: var(--teal); transition: width 0.4s; }
  .spread-no { background: rgba(232,50,74,0.2); display: flex; align-items: center; justify-content: center; font-size: 0.72rem; font-weight: 700; color: var(--crimson); transition: width 0.4s; }
  .book-grid { display: flex; flex-direction: column; gap: 0; }
  .book-side-label { font-size: 0.7rem; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 0.3rem; }
  .book-row { display: flex; align-items: center; gap: 0.75rem; padding: 0.18rem 0; font-size: 0.78rem; font-family: var(--font-mono); position: relative; }
  .book-price { font-weight: 600; min-width: 36px; }
  .ask-price { color: var(--crimson); }
  .bid-price { color: var(--teal); }
  .book-qty { min-width: 48px; color: var(--text-muted); }
  .book-depth { position: absolute; left: 0; top: 0; bottom: 0; z-index: -1; border-radius: 2px; }
  .book-spread-line { text-align: center; padding: 0.4rem; border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); margin: 0.25rem 0; }
  .spread-label { font-size: 0.72rem; color: var(--text-muted); }
  .book-skeleton { padding: 0.5rem 0; }

  /* Order form */
  .order-card { padding: 1.25rem; margin-bottom: 1rem; }
  .order-card h3 { font-size: 1rem; margin: 0 0 1rem; }
  .seg-ctrl { display: flex; background: rgba(255,255,255,0.04); border-radius: 8px; overflow: hidden; border: 1px solid var(--border); }
  .seg { flex: 1; background: none; border: none; color: var(--text-muted); padding: 0.5rem; font-size: 0.8rem; font-weight: 600; cursor: pointer; transition: all 0.15s; }
  .seg.active-yes { background: rgba(0,194,224,0.2); color: var(--teal); }
  .seg.active-no { background: rgba(232,50,74,0.2); color: var(--crimson); }
  .seg.active-buy { background: rgba(34,197,94,0.2); color: #22c55e; }
  .seg.active-sell { background: rgba(232,50,74,0.2); color: var(--crimson); }
  .form-group { margin-bottom: 0.85rem; }
  .form-label { display: block; font-size: 0.75rem; color: var(--text-muted); margin-bottom: 0.35rem; }
  .form-input { width: 100%; background: rgba(255,255,255,0.04); border: 1px solid var(--border); border-radius: 8px; color: var(--text); padding: 0.55rem 0.75rem; font-size: 0.85rem; box-sizing: border-box; font-family: inherit; }
  .form-input:focus { outline: none; border-color: var(--gold); }
  .price-slider { width: 100%; accent-color: var(--gold); }
  .price-display { text-align: center; font-size: 1.8rem; font-weight: 800; font-family: var(--font-mono); }
  .order-summary { background: rgba(255,255,255,0.03); border-radius: 8px; padding: 0.75rem; margin: 0.75rem 0; }
  .os-row { display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; padding: 0.2rem 0; }
  .btn { display: inline-flex; align-items: center; justify-content: center; padding: 0.7rem 1.25rem; border-radius: 8px; font-weight: 700; font-size: 0.9rem; border: none; cursor: pointer; transition: all 0.2s; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-trade { margin-top: 0.5rem; }
  .btn-yes { background: rgba(0,194,224,0.25); color: var(--teal); border: 1px solid var(--teal); }
  .btn-yes:hover:not(:disabled) { background: rgba(0,194,224,0.4); box-shadow: 0 0 16px rgba(0,194,224,0.3); }
  .btn-no { background: rgba(232,50,74,0.2); color: var(--crimson); border: 1px solid var(--crimson); }
  .btn-no:hover:not(:disabled) { background: rgba(232,50,74,0.35); box-shadow: 0 0 16px rgba(232,50,74,0.3); }
  .btn-sell { background: rgba(232,50,74,0.2); color: var(--crimson); border: 1px solid var(--crimson); }
  .btn-gold { background: var(--gold); color: #000; }
  .btn-gold:hover:not(:disabled) { background: #ffc844; box-shadow: 0 0 20px rgba(240,180,41,0.4); }
  .alert { padding: 0.6rem 0.85rem; border-radius: 8px; font-size: 0.8rem; margin: 0.5rem 0; }
  .alert-success { background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); color: #22c55e; }
  .alert-error { background: rgba(232,50,74,0.1); border: 1px solid rgba(232,50,74,0.3); color: var(--crimson); }

  /* Explainer */
  .explainer { padding: 1rem; }
  .explainer h4 { font-size: 0.85rem; margin: 0 0 0.75rem; }
  .explainer-list { margin: 0; padding: 0 0 0 1rem; }
  .explainer-list li { font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.4rem; }
  .explainer-list li strong { color: var(--text); }

  /* Badges */
  .mbadge { padding: 0.2rem 0.6rem; border-radius: 4px; font-size: 0.72rem; font-weight: 700; letter-spacing: 0.05em; }
  .mbadge.teal { background: rgba(0,194,224,0.15); color: var(--teal); }
  .mbadge.gold { background: rgba(240,180,41,0.15); color: var(--gold); }
  .mbadge.neutral { background: rgba(255,255,255,0.08); color: var(--text-muted); }

  /* Shared */
  .grad-gold { background: linear-gradient(135deg, var(--gold), var(--crimson)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
  .text-muted { color: var(--text-muted); }
  .skeleton { background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%); background-size: 200% 100%; animation: shimmer 1.5s infinite; }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
