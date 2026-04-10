<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api/client';

  let tab: 'arena' | 'leaderboard' | 'register' = 'arena';
  let bots: any[] = [];
  let matches: any[] = [];
  let loading = true;
  let wsConnected = false;
  let feed: string[] = [];
  let ws: WebSocket | null = null;

  // Register form
  let regName = '';
  let regDesc = '';
  let regSubmitting = false;
  let regSuccess = '';
  let regError = '';

  onMount(async () => {
    try {
      const res = await api.botLeaderboard();
      bots = res.bots ?? [];
    } catch (e) {}
    loading = false;
    connectSpectator();
  });

  onDestroy(() => ws?.close());

  function connectSpectator() {
    try {
      const proto = location.protocol === 'https:' ? 'wss' : 'ws';
      ws = new WebSocket(`${proto}://${location.host}/ws`);
      ws.onopen = () => {
        wsConnected = true;
        ws!.send(JSON.stringify({ type: 'subscribe', channel: 'bot_arena' }));
      };
      ws.onmessage = (e) => {
        try {
          const msg = JSON.parse(e.data);
          if (msg.type === 'bot_trade') {
            feed = [`${msg.buyer} vs ${msg.seller} — ${msg.quantity}x @ ${msg.price_cents}¢`, ...feed].slice(0, 50);
          } else if (msg.type === 'cage_match') {
            matches = [msg, ...matches].slice(0, 20);
          }
        } catch {}
      };
      ws.onclose = () => { wsConnected = false; };
    } catch {}
  }

  async function submitReg() {
    if (!regName.trim()) { regError = 'Name required'; return; }
    regSubmitting = true; regError = ''; regSuccess = '';
    try {
      await api.post('/api/v1/bots/register', { name: regName, description: regDesc });
      regSuccess = `Bot "${regName}" registered! API key sent to your account.`;
      regName = ''; regDesc = '';
    } catch (e: any) {
      regError = e?.message ?? 'Registration failed';
    }
    regSubmitting = false;
  }

  function rankColor(rank: number) {
    if (rank === 1) return '#F0B429';
    if (rank === 2) return '#A0AEC0';
    if (rank === 3) return '#CD7F32';
    return 'var(--text-muted)';
  }
</script>

<svelte:head><title>Bot Arena — Adenora</title></svelte:head>

<!-- Hero banner -->
<div class="arena-hero">
  <div class="arena-hero-bg"></div>
  <div class="arena-hero-content">
    <div class="arena-badge">BOTS ONLY</div>
    <h1 class="arena-title">Bot <span class="grad-gold">Arena</span></h1>
    <p class="arena-sub">Pure algorithmic combat. No humans. No mercy. Speed is the only edge.</p>
    <div class="arena-stats">
      <div class="astat"><span class="astat-val">{bots.length}</span><span class="astat-lbl">Registered Bots</span></div>
      <div class="astat"><span class="astat-val">0ms</span><span class="astat-lbl">Min Latency</span></div>
      <div class="astat"><span class="astat-val">∞</span><span class="astat-lbl">Rate Limit</span></div>
      <div class="astat">
        <span class="astat-val ws-dot" class:connected={wsConnected}></span>
        <span class="astat-lbl">{wsConnected ? 'Live Feed' : 'Connecting'}</span>
      </div>
    </div>
  </div>
</div>

<!-- Tabs -->
<div class="tabs">
  <button class="tab" class:active={tab === 'arena'} on:click={() => tab = 'arena'}>Spectator Feed</button>
  <button class="tab" class:active={tab === 'leaderboard'} on:click={() => tab = 'leaderboard'}>Leaderboard</button>
  <button class="tab" class:active={tab === 'register'} on:click={() => tab = 'register'}>Register Bot</button>
</div>

<!-- ── SPECTATOR FEED ── -->
{#if tab === 'arena'}
  <div class="feed-grid">
    <!-- Live trade tape -->
    <div class="glass-card feed-card">
      <div class="feed-header">
        <h3>Live Trade Feed</h3>
        <span class="live-pill" class:active={wsConnected}>{wsConnected ? 'LIVE' : 'OFFLINE'}</span>
      </div>
      {#if feed.length === 0}
        <div class="feed-empty">
          <div class="pulse-ring"></div>
          <p class="text-muted">Waiting for bot trades…</p>
        </div>
      {:else}
        <ul class="feed-list">
          {#each feed as line, i}
            <li class="feed-line" style="opacity:{Math.max(0.3, 1 - i * 0.015)}">
              <span class="feed-dot"></span>{line}
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <!-- Active cage matches -->
    <div class="glass-card matches-card">
      <h3>Cage Matches</h3>
      {#if matches.length === 0}
        <div class="feed-empty">
          <p class="text-muted">No active cage matches</p>
          <p class="text-muted" style="font-size:0.75rem">Matches start automatically when 2+ bots enter the same market</p>
        </div>
      {:else}
        {#each matches as m}
          <div class="match-card">
            <div class="match-bots">
              <span class="bot-tag teal">{m.bot_a}</span>
              <span class="vs">vs</span>
              <span class="bot-tag crimson">{m.bot_b}</span>
            </div>
            <div class="match-meta">
              <span>{m.market_title ?? m.market_id}</span>
              <span class="text-muted">{m.trades ?? 0} trades</span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Rules card -->
  <div class="glass-card rules-card">
    <h3>Arena Rules</h3>
    <div class="rules-grid">
      <div class="rule"><span class="rule-icon">🤖</span><strong>Bots only</strong><p>Human accounts cannot submit orders in Arena markets. Bot API key required.</p></div>
      <div class="rule"><span class="rule-icon">⚡</span><strong>Continuous matching</strong><p>No batch windows. Orders match the instant a counterpart rests. Microseconds matter.</p></div>
      <div class="rule"><span class="rule-icon">∞</span><strong>No rate limits</strong><p>Submit as fast as your infrastructure allows. The exchange imposes zero throttling.</p></div>
      <div class="rule"><span class="rule-icon">📡</span><strong>Public spectator feed</strong><p>All trades stream live via WebSocket. The arena is transparent — your edge is not hidden.</p></div>
    </div>
  </div>
{/if}

<!-- ── LEADERBOARD ── -->
{#if tab === 'leaderboard'}
  {#if loading}
    <div class="glass-card">
      {#each Array(5) as _}
        <div class="skeleton" style="height:48px;margin-bottom:0.5rem;border-radius:6px"></div>
      {/each}
    </div>
  {:else if bots.length === 0}
    <div class="glass-card empty-state">
      <p style="font-size:2.5rem">🤖</p>
      <h3>No bots registered yet</h3>
      <p class="text-muted">Register yours and claim the top spot.</p>
      <button class="btn btn-gold" on:click={() => tab = 'register'}>Register Bot</button>
    </div>
  {:else}
    <div class="glass-card leaderboard-card">
      <div class="lb-header">
        <h3>Global Bot Leaderboard</h3>
        <span class="text-muted" style="font-size:0.8rem">Updated every 5 min</span>
      </div>
      <table class="lb-table">
        <thead>
          <tr>
            <th>#</th>
            <th>Bot</th>
            <th>PnL (EUR)</th>
            <th>Win Rate</th>
            <th>Trades</th>
            <th>Sharpe</th>
            <th>Max DD</th>
          </tr>
        </thead>
        <tbody>
          {#each bots as bot, i}
            <tr class="lb-row">
              <td class="rank-col" style="color:{rankColor(bot.rank ?? i+1)}">
                {bot.rank ?? i+1}
                {#if (bot.rank ?? i+1) === 1}👑{/if}
              </td>
              <td class="bot-col">
                <span class="bot-avatar">{(bot.name ?? '?')[0]}</span>
                <div>
                  <div class="bot-name">{bot.name}</div>
                  {#if bot.owner}<div class="bot-owner text-muted">{bot.owner}</div>{/if}
                </div>
              </td>
              <td class="pnl-col" class:pos={parseFloat(bot.total_pnl) > 0} class:neg={parseFloat(bot.total_pnl) < 0}>
                {parseFloat(bot.total_pnl) > 0 ? '+' : ''}{bot.total_pnl}
              </td>
              <td>{((bot.win_rate ?? 0) * 100).toFixed(1)}%</td>
              <td>{(bot.total_trades ?? 0).toLocaleString()}</td>
              <td class:pos={parseFloat(bot.sharpe_ratio) > 1}>{(bot.sharpe_ratio ?? 0).toFixed(2)}</td>
              <td class="neg">{bot.max_drawdown ? `-${bot.max_drawdown}%` : '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/if}

<!-- ── REGISTER BOT ── -->
{#if tab === 'register'}
  <div class="reg-layout">
    <div class="glass-card reg-form-card">
      <h3>Register Your Bot</h3>
      <p class="text-muted" style="margin-bottom:1.5rem">Get an API key to compete in the Arena. Your bot will be visible on the leaderboard.</p>

      {#if regSuccess}
        <div class="alert alert-success">{regSuccess}</div>
      {/if}
      {#if regError}
        <div class="alert alert-error">{regError}</div>
      {/if}

      <div class="form-group">
        <label class="form-label">Bot Name</label>
        <input class="form-input" bind:value={regName} placeholder="AlphaBot_v2" maxlength="40" />
      </div>
      <div class="form-group">
        <label class="form-label">Description <span class="text-muted">(optional)</span></label>
        <textarea class="form-input" bind:value={regDesc} placeholder="Market-making bot using VWAP signals…" rows="3"></textarea>
      </div>
      <button class="btn btn-gold" style="width:100%" on:click={submitReg} disabled={regSubmitting}>
        {regSubmitting ? 'Registering…' : 'Register Bot'}
      </button>
    </div>

    <div class="reg-info">
      <div class="glass-card info-card">
        <h4>API Access</h4>
        <p class="text-muted">After registration, your API key is sent to your account email. Use it in the <code>X-Bot-Key</code> header on all requests.</p>
      </div>
      <div class="glass-card info-card">
        <h4>Order Format</h4>
        <pre class="code-block">{`POST /api/v1/orders
X-Bot-Key: your-api-key

{
  "market_id": "uuid",
  "mode": "unlimited",
  "side": "yes",
  "action": "buy",
  "price_cents": 55,
  "quantity": 100,
  "time_in_force": "ioc"
}`}</pre>
      </div>
      <div class="glass-card info-card">
        <h4>WebSocket Feed</h4>
        <pre class="code-block">{`ws://host/ws
→ {"type":"subscribe","channel":"bot_arena"}
← {"type":"bot_trade","buyer":…,"price_cents":…}`}</pre>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Hero */
  .arena-hero { position: relative; border-radius: 16px; overflow: hidden; margin-bottom: 2rem; padding: 3rem 2rem; }
  .arena-hero-bg {
    position: absolute; inset: 0;
    background: radial-gradient(ellipse at 20% 50%, rgba(0,194,224,0.15) 0%, transparent 60%),
                radial-gradient(ellipse at 80% 50%, rgba(232,50,74,0.15) 0%, transparent 60%),
                rgba(13,27,46,0.9);
    border: 1px solid rgba(232,50,74,0.3);
  }
  .arena-hero-content { position: relative; }
  .arena-badge {
    display: inline-block; background: rgba(232,50,74,0.2); border: 1px solid var(--crimson);
    color: var(--crimson); font-size: 0.7rem; font-weight: 700; letter-spacing: 0.15em;
    padding: 0.25rem 0.75rem; border-radius: 4px; margin-bottom: 1rem;
  }
  .arena-title { font-size: clamp(2rem, 5vw, 3.5rem); font-weight: 800; margin: 0 0 0.5rem; }
  .arena-sub { color: var(--text-muted); margin-bottom: 2rem; max-width: 480px; }
  .arena-stats { display: flex; gap: 2rem; flex-wrap: wrap; }
  .astat { display: flex; flex-direction: column; gap: 0.25rem; }
  .astat-val { font-size: 1.5rem; font-weight: 700; color: var(--text); }
  .astat-lbl { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; }
  .ws-dot { width: 10px; height: 10px; border-radius: 50%; background: var(--text-muted); display: inline-block; }
  .ws-dot.connected { background: #22c55e; box-shadow: 0 0 8px rgba(34,197,94,0.6); animation: pulse 2s infinite; }
  @keyframes pulse { 0%,100% { opacity:1; } 50% { opacity:0.5; } }

  /* Tabs */
  .tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; border-bottom: 1px solid var(--border); padding-bottom: 0; }
  .tab {
    background: none; border: none; color: var(--text-muted); padding: 0.6rem 1rem;
    font-size: 0.9rem; cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -1px;
    transition: color 0.2s;
  }
  .tab.active { color: var(--crimson); border-bottom-color: var(--crimson); }
  .tab:hover:not(.active) { color: var(--text); }

  /* Glass cards */
  .glass-card {
    background: var(--bg-glass); border: 1px solid var(--border);
    backdrop-filter: blur(12px); border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem;
  }
  .glass-card h3 { font-size: 1rem; font-weight: 700; margin: 0 0 1rem; }

  /* Feed */
  .feed-grid { display: grid; grid-template-columns: 1fr 340px; gap: 1rem; margin-bottom: 1rem; }
  @media (max-width: 768px) { .feed-grid { grid-template-columns: 1fr; } }
  .feed-card, .matches-card { margin-bottom: 0; }
  .feed-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  .feed-header h3 { margin: 0; }
  .live-pill {
    font-size: 0.65rem; font-weight: 700; letter-spacing: 0.12em;
    padding: 0.2rem 0.6rem; border-radius: 4px;
    background: rgba(100,100,100,0.2); color: var(--text-muted);
  }
  .live-pill.active { background: rgba(34,197,94,0.15); color: #22c55e; }
  .feed-empty { text-align: center; padding: 2rem 0; }
  .pulse-ring {
    width: 40px; height: 40px; border-radius: 50%;
    border: 3px solid var(--crimson); margin: 0 auto 1rem;
    animation: pulse-ring 1.5s ease-out infinite;
  }
  @keyframes pulse-ring { 0% { transform: scale(0.8); opacity:1; } 100% { transform: scale(1.6); opacity:0; } }
  .feed-list { list-style: none; margin: 0; padding: 0; font-family: var(--font-mono); font-size: 0.78rem; }
  .feed-line { display: flex; align-items: center; gap: 0.5rem; padding: 0.3rem 0; border-bottom: 1px solid rgba(255,255,255,0.04); color: var(--text); }
  .feed-dot { width: 6px; height: 6px; background: var(--crimson); border-radius: 50%; flex-shrink: 0; }
  .match-card { background: rgba(255,255,255,0.03); border-radius: 8px; padding: 0.75rem; margin-bottom: 0.5rem; }
  .match-bots { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.4rem; }
  .bot-tag { padding: 0.15rem 0.5rem; border-radius: 4px; font-size: 0.78rem; font-weight: 600; }
  .bot-tag.teal { background: rgba(0,194,224,0.15); color: var(--teal); }
  .bot-tag.crimson { background: rgba(232,50,74,0.15); color: var(--crimson); }
  .vs { font-size: 0.7rem; color: var(--text-muted); font-weight: 700; }
  .match-meta { display: flex; justify-content: space-between; font-size: 0.75rem; color: var(--text-muted); }

  /* Rules */
  .rules-card { }
  .rules-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-top: 0; }
  .rule { }
  .rule-icon { font-size: 1.5rem; display: block; margin-bottom: 0.4rem; }
  .rule strong { display: block; margin-bottom: 0.25rem; }
  .rule p { font-size: 0.8rem; color: var(--text-muted); margin: 0; }

  /* Leaderboard */
  .lb-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  .lb-header h3 { margin: 0; }
  .lb-table { width: 100%; border-collapse: collapse; }
  .lb-table th { text-align: left; padding: 0.5rem; font-size: 0.75rem; color: var(--text-muted); border-bottom: 1px solid var(--border); }
  .lb-table td { padding: 0.6rem 0.5rem; font-size: 0.85rem; border-bottom: 1px solid rgba(255,255,255,0.04); }
  .lb-row:hover { background: rgba(255,255,255,0.02); }
  .rank-col { font-weight: 700; font-size: 0.9rem; }
  .bot-col { display: flex; align-items: center; gap: 0.75rem; }
  .bot-avatar {
    width: 32px; height: 32px; border-radius: 8px;
    background: linear-gradient(135deg, var(--teal), var(--crimson));
    display: flex; align-items: center; justify-content: center;
    font-weight: 700; font-size: 0.85rem; flex-shrink: 0;
  }
  .bot-name { font-weight: 600; }
  .bot-owner { font-size: 0.72rem; }
  .pnl-col { font-family: var(--font-mono); font-weight: 600; }
  .pos { color: #22c55e; }
  .neg { color: var(--crimson); }
  .empty-state { text-align: center; padding: 3rem; }
  .empty-state p { margin-bottom: 1rem; }

  /* Register */
  .reg-layout { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; align-items: start; }
  @media (max-width: 768px) { .reg-layout { grid-template-columns: 1fr; } }
  .reg-form-card { margin-bottom: 0; }
  .reg-info { display: flex; flex-direction: column; gap: 1rem; }
  .info-card { padding: 1rem; margin-bottom: 0; }
  .info-card h4 { font-size: 0.85rem; margin: 0 0 0.5rem; }
  .info-card p { font-size: 0.8rem; color: var(--text-muted); margin: 0; }
  .info-card code { color: var(--teal); font-family: var(--font-mono); }
  .code-block {
    font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-muted);
    background: rgba(0,0,0,0.3); border-radius: 6px; padding: 0.75rem;
    overflow-x: auto; margin: 0.5rem 0 0; white-space: pre;
  }
  .form-group { margin-bottom: 1rem; }
  .form-label { display: block; font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.4rem; }
  .form-input {
    width: 100%; background: rgba(255,255,255,0.04); border: 1px solid var(--border);
    border-radius: 8px; color: var(--text); padding: 0.6rem 0.75rem; font-size: 0.9rem;
    box-sizing: border-box; font-family: inherit;
  }
  .form-input:focus { outline: none; border-color: var(--crimson); }
  textarea.form-input { resize: vertical; }
  .alert { padding: 0.75rem 1rem; border-radius: 8px; font-size: 0.85rem; margin-bottom: 1rem; }
  .alert-success { background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); color: #22c55e; }
  .alert-error { background: rgba(232,50,74,0.1); border: 1px solid rgba(232,50,74,0.3); color: var(--crimson); }

  /* Shared */
  .btn { display: inline-flex; align-items: center; justify-content: center; gap: 0.4rem; padding: 0.65rem 1.25rem; border-radius: 8px; font-weight: 600; font-size: 0.9rem; border: none; cursor: pointer; transition: all 0.2s; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-gold { background: var(--gold); color: #000; }
  .btn-gold:hover:not(:disabled) { background: #ffc844; box-shadow: 0 0 20px rgba(240,180,41,0.4); }
  .grad-gold { background: linear-gradient(135deg, var(--gold), var(--crimson)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
  .text-muted { color: var(--text-muted); }
  .skeleton { background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%); background-size: 200% 100%; animation: shimmer 1.5s infinite; }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
