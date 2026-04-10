<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';

  let mounted = false;
  let tickerMarkets = [
    { q: 'Albania EU accession before 2027?', yes: 67 },
    { q: 'EUR/ALL exceed 120 in Q4 2026?',   yes: 34 },
    { q: 'Kosovo join UEFA before 2028?',     yes: 81 },
    { q: 'N. Macedonia early elections?',     yes: 52 },
    { q: 'Western Balkans Schengen by 2026?', yes: 23 },
  ];

  // Duplicate for seamless loop
  const ticker = [...tickerMarkets, ...tickerMarkets];

  let stats = [
    { value: '0%',   label: 'Deposit Fees',      color: 'teal',    target: 0 },
    { value: '500ms',label: 'Batch Auctions',     color: 'gold',    target: 500 },
    { value: '40%',  label: 'Fees to Charity',    color: 'crimson', target: 40 },
    { value: '11',   label: 'Jury Members',       color: 'teal',    target: 11 },
  ];

  onMount(() => { mounted = true; });
</script>

<svelte:head>
  <title>Adenora — Win. Grow. Give.</title>
  <meta name="description" content="Human-first event markets with transparent project funding and advanced execution on eligible markets." />
</svelte:head>

<!-- ── TICKER ── -->
<div class="ticker-wrap">
  <div class="ticker-track">
    {#each ticker as item}
      <span class="ticker-item">
        <span class="ticker-q">{item.q}</span>
        <span class="ticker-yes" class:high={item.yes >= 60} class:low={item.yes < 40}>
          YES {item.yes}¢
        </span>
        <span class="ticker-sep">·</span>
      </span>
    {/each}
  </div>
</div>

<!-- ── HERO ── -->
<section class="hero" class:mounted>
  <div class="hero-bg">
    <div class="orb orb-teal"></div>
    <div class="orb orb-gold"></div>
    <div class="orb orb-crimson"></div>
    <div class="grid-lines"></div>
  </div>

  <div class="hero-inner container">
    <div class="hero-badge animate-fadeup">
      <span class="live-dot"></span>
      Human-first event markets · Western Balkans
    </div>

    <h1 class="hero-title animate-fadeup delay-1">
      Where Value<br/>
      <span class="grad-wgg">Flows</span>
    </h1>

    <p class="hero-sub animate-fadeup delay-2">
      Trade human-first event markets, opt into ultimate markets with API bot access on eligible events,<br class="hide-mobile"/>
      and fund humanitarian and infrastructure projects through defined platform fee flows.
    </p>

    <div class="hero-cta animate-fadeup delay-3">
      <a href="/register" class="btn btn-primary btn-xl">Start Trading Free</a>
      <a href="/markets" class="btn btn-outline btn-xl">Explore Markets</a>
    </div>

    <!-- Pillars -->
    <div class="pillars animate-fadeup delay-4">
      <a href="/markets" class="pillar pillar-grow">
        <div class="pillar-icon-wrap gold-glow">📈</div>
        <div class="pillar-body">
          <span class="pillar-label text-gold">Human</span>
          <span class="pillar-desc">Default event markets with 500ms batch auctions</span>
        </div>
        <span class="pillar-arrow">→</span>
      </a>
      <a href="/unlimited" class="pillar pillar-win">
        <div class="pillar-icon-wrap teal-glow">⚡</div>
        <div class="pillar-body">
          <span class="pillar-label text-teal">Ultimate</span>
          <span class="pillar-desc">Opt-in advanced execution with API bot access</span>
        </div>
        <span class="pillar-arrow">→</span>
      </a>
      <a href="/charity" class="pillar pillar-give">
        <div class="pillar-icon-wrap crimson-glow">❤</div>
        <div class="pillar-body">
          <span class="pillar-label text-crimson">Give</span>
          <span class="pillar-desc">Defined fee routing supports approved campaigns and projects</span>
        </div>
        <span class="pillar-arrow">→</span>
      </a>
    </div>
  </div>
</section>

<!-- ── STATS ── -->
<section class="stats-section">
  <div class="container">
    <div class="stats-grid">
      {#each stats as stat, i}
        <div class="stat-card animate-fadeup" style="animation-delay:{0.1*i}s">
          <div class="stat-value text-{stat.color}">{stat.value}</div>
          <div class="stat-label">{stat.label}</div>
        </div>
      {/each}
    </div>
  </div>
</section>

<!-- ── HOW IT WORKS ── -->
<section class="section container">
  <div class="section-label">How it works</div>
  <h2 class="section-title">Fairer Markets.<br/>Real Impact.</h2>

  <div class="how-grid">
    <div class="how-card animate-fadeup delay-1">
      <div class="how-num teal">01</div>
      <h3>Human Markets</h3>
      <p>500ms batch auctions reduce speed advantages and keep the default experience fair for mainstream users.</p>
    </div>
    <div class="how-card animate-fadeup delay-2">
      <div class="how-num gold">02</div>
      <h3>Ultimate Markets</h3>
      <p>Eligible markets can open an opt-in advanced book with continuous matching, where API bot access lives alongside informed human participation.</p>
    </div>
    <div class="how-card animate-fadeup delay-3">
      <div class="how-num crimson">03</div>
      <h3>Project Funding</h3>
      <p>A defined share of platform economics routes to approved campaigns, with allocations published through the charity ledger.</p>
    </div>
    <div class="how-card animate-fadeup delay-4">
      <div class="how-num teal">04</div>
      <h3>Resolution And Review</h3>
      <p>Markets resolve against documented criteria, with evidence review and an 11-member jury path when outcomes are disputed.</p>
    </div>
  </div>
</section>

<!-- ── LIVE MARKETS PREVIEW ── -->
<section class="section container">
  <div class="section-header">
    <div>
      <div class="section-label">Illustrative</div>
      <h2 class="section-title-sm">Launch Market Themes</h2>
    </div>
    <a href="/markets" class="btn btn-outline">View all →</a>
  </div>

  <div class="market-preview-grid">
    {#each tickerMarkets as m, i}
      <a href="/markets" class="market-preview-card card animate-fadeup" style="animation-delay:{0.08*i}s">
        <div class="mp-header">
          <span class="badge badge-gold">Politics</span>
          <span class="live-dot"></span>
        </div>
        <p class="mp-question">{m.q}</p>
        <div class="mp-prob">
          <div class="prob-bar">
            <div class="prob-bar-fill" style="width:{m.yes}%"></div>
          </div>
          <div class="mp-prob-labels">
            <span class="text-teal text-mono font-bold">{m.yes}¢ YES</span>
            <span class="text-muted text-mono">{100 - m.yes}¢ NO</span>
          </div>
        </div>
      </a>
    {/each}
  </div>
</section>

<!-- ── TRUST BAR ── -->
<section class="trust-bar">
  <div class="container">
    <div class="trust-inner">
      <div class="trust-item">
        <span class="trust-icon">🏦</span>
        <span>Jurisdiction-led launch</span>
      </div>
      <div class="trust-divider"></div>
      <div class="trust-item">
        <span class="trust-icon">🔒</span>
        <span>Argon2 Security</span>
      </div>
      <div class="trust-divider"></div>
      <div class="trust-item">
        <span class="trust-icon">⚖️</span>
        <span>KYC workflow</span>
      </div>
      <div class="trust-divider"></div>
      <div class="trust-item">
        <span class="trust-icon">🌍</span>
        <span>Country-aware access</span>
      </div>
      <div class="trust-divider"></div>
      <div class="trust-item">
        <span class="trust-icon">💶</span>
        <span>EUR · ALL · MKD · USDC</span>
      </div>
    </div>
  </div>
</section>

<style>
  /* ── Ticker ── */
  .ticker-wrap {
    background: rgba(0, 194, 224, 0.05);
    border-bottom: 1px solid rgba(0, 194, 224, 0.12);
    overflow: hidden; height: 36px;
    display: flex; align-items: center;
    position: relative; z-index: 10;
  }
  .ticker-track {
    display: flex; white-space: nowrap;
    animation: ticker 40s linear infinite;
    will-change: transform;
  }
  .ticker-item {
    display: inline-flex; align-items: center; gap: 0.5rem;
    padding: 0 1.5rem; font-size: 0.78rem; font-weight: 500;
    color: var(--text-secondary);
  }
  .ticker-yes { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; }
  .ticker-yes.high { color: var(--teal); }
  .ticker-yes.low  { color: var(--crimson); }
  .ticker-sep { color: var(--text-muted); }
  .ticker-q { color: var(--text-secondary); }

  /* ── Hero ── */
  .hero {
    position: relative; overflow: hidden;
    padding: 6rem 0 4rem;
    min-height: 90vh;
    display: flex; align-items: center;
  }
  .hero-bg {
    position: absolute; inset: 0;
    overflow: hidden; pointer-events: none;
  }
  .orb {
    position: absolute; border-radius: 50%;
    filter: blur(80px); opacity: 0;
    transition: opacity 1.5s ease;
  }
  .hero.mounted .orb { opacity: 1; }
  .orb-teal {
    width: 600px; height: 600px;
    background: radial-gradient(circle, rgba(0,194,224,0.12) 0%, transparent 70%);
    top: -200px; left: -100px;
    animation: float 8s ease-in-out infinite;
  }
  .orb-gold {
    width: 500px; height: 500px;
    background: radial-gradient(circle, rgba(240,180,41,0.08) 0%, transparent 70%);
    top: 100px; right: -150px;
    animation: float 10s ease-in-out infinite reverse;
  }
  .orb-crimson {
    width: 400px; height: 400px;
    background: radial-gradient(circle, rgba(232,50,74,0.06) 0%, transparent 70%);
    bottom: -100px; left: 40%;
    animation: float 12s ease-in-out infinite;
  }
  .grid-lines {
    position: absolute; inset: 0;
    background-image:
      linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px);
    background-size: 60px 60px;
    mask-image: radial-gradient(ellipse 80% 80% at 50% 50%, black 0%, transparent 100%);
  }

  .hero-inner { position: relative; z-index: 2; }
  .hero-badge {
    display: inline-flex; align-items: center; gap: 0.5rem;
    padding: 0.4rem 1rem; border-radius: var(--r-full);
    background: rgba(0,194,224,0.08);
    border: 1px solid rgba(0,194,224,0.2);
    color: var(--teal); font-size: 0.8rem; font-weight: 600;
    letter-spacing: 0.03em; margin-bottom: 1.5rem;
  }
  .hero-title {
    font-size: clamp(3rem, 7vw, 5.5rem);
    font-weight: 900;
    line-height: 1.0;
    letter-spacing: -0.04em;
    margin-bottom: 1.5rem;
    color: var(--text-primary);
  }
  .hero-sub {
    font-size: clamp(1rem, 2vw, 1.2rem);
    color: var(--text-secondary);
    line-height: 1.7;
    max-width: 600px;
    margin-bottom: 2.5rem;
  }
  .hero-cta {
    display: flex; gap: 1rem; flex-wrap: wrap;
    margin-bottom: 4rem;
  }

  /* ── Pillars ── */
  .pillars {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem; max-width: 900px;
  }
  .pillar {
    display: flex; align-items: center; gap: 1rem;
    padding: 1.25rem 1.5rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    backdrop-filter: blur(12px);
    text-decoration: none;
    transition: all var(--t-med) var(--ease-out);
    position: relative; overflow: hidden;
  }
  .pillar::before {
    content: ''; position: absolute;
    inset: 0; opacity: 0;
    transition: opacity var(--t-med);
    border-radius: var(--r-lg);
  }
  .pillar-grow::before { background: radial-gradient(ellipse at 0% 50%, rgba(240,180,41,0.08), transparent 70%); }
  .pillar-win::before  { background: radial-gradient(ellipse at 0% 50%, rgba(0,194,224,0.08), transparent 70%); }
  .pillar-give::before { background: radial-gradient(ellipse at 0% 50%, rgba(232,50,74,0.08), transparent 70%); }
  .pillar:hover::before { opacity: 1; }
  .pillar:hover { transform: translateY(-3px); box-shadow: var(--shadow-lg); }
  .pillar-grow:hover { border-color: var(--border-gold); }
  .pillar-win:hover  { border-color: var(--border-teal); }
  .pillar-give:hover { border-color: var(--border-crimson); }

  .pillar-icon-wrap {
    font-size: 1.5rem; width: 44px; height: 44px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--r-md); flex-shrink: 0;
  }
  .teal-glow    { background: rgba(0,194,224,0.1); }
  .gold-glow    { background: rgba(240,180,41,0.1); }
  .crimson-glow { background: rgba(232,50,74,0.1); }

  .pillar-body { flex: 1; min-width: 0; }
  .pillar-label { display: block; font-weight: 700; font-size: 0.95rem; margin-bottom: 0.2rem; }
  .pillar-desc  { display: block; font-size: 0.75rem; color: var(--text-muted); line-height: 1.4; }
  .pillar-arrow { color: var(--text-muted); font-size: 1rem; transition: transform var(--t-fast); }
  .pillar:hover .pillar-arrow { transform: translateX(4px); color: var(--text-secondary); }

  /* ── Stats ── */
  .stats-section {
    padding: 3rem 0;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(0,194,224,0.03) 0%, transparent 100%);
  }
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    overflow: hidden;
  }
  .stat-card {
    background: var(--bg-surface);
    padding: 2rem;
    text-align: center;
    transition: background var(--t-fast);
  }
  .stat-card:hover { background: var(--bg-raised); }
  .stat-value {
    font-size: 2.5rem; font-weight: 900;
    font-family: var(--font-mono);
    line-height: 1; margin-bottom: 0.5rem;
    letter-spacing: -0.03em;
  }
  .stat-label { font-size: 0.8rem; color: var(--text-muted); font-weight: 500; letter-spacing: 0.04em; text-transform: uppercase; }

  /* ── Section ── */
  .section { padding: 5rem 0; }
  .section-label {
    font-size: 0.75rem; font-weight: 700;
    letter-spacing: 0.1em; text-transform: uppercase;
    color: var(--teal); margin-bottom: 0.75rem;
  }
  .section-title {
    font-size: clamp(1.8rem, 4vw, 2.8rem);
    font-weight: 900; letter-spacing: -0.03em;
    line-height: 1.1; margin-bottom: 3rem;
  }
  .section-title-sm {
    font-size: 1.5rem; font-weight: 800;
    letter-spacing: -0.02em;
  }
  .section-header {
    display: flex; justify-content: space-between;
    align-items: flex-end; margin-bottom: 2rem;
  }

  /* ── How grid ── */
  .how-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1.5rem;
  }
  .how-card {
    padding: 2rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    backdrop-filter: blur(8px);
    transition: all var(--t-med) var(--ease-out);
  }
  .how-card:hover { border-color: rgba(255,255,255,0.12); transform: translateY(-4px); box-shadow: var(--shadow-lg); }
  .how-num {
    font-size: 3rem; font-weight: 900;
    font-family: var(--font-mono);
    opacity: 0.15; line-height: 1;
    margin-bottom: 1rem; letter-spacing: -0.05em;
  }
  .how-num.teal { color: var(--teal); }
  .how-num.gold { color: var(--gold); }
  .how-num.crimson { color: var(--crimson); }
  .how-card h3 { font-size: 1rem; font-weight: 700; margin-bottom: 0.75rem; }
  .how-card p  { font-size: 0.85rem; color: var(--text-secondary); line-height: 1.6; }

  /* ── Market preview ── */
  .market-preview-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }
  .market-preview-card {
    padding: 1.5rem;
    text-decoration: none; color: inherit;
    cursor: pointer;
  }
  .market-preview-card:hover { border-color: var(--border-gold); }
  .mp-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
  .mp-question { font-size: 0.9rem; font-weight: 600; line-height: 1.4; margin-bottom: 1rem; color: var(--text-primary); }
  .mp-prob { display: flex; flex-direction: column; gap: 0.5rem; }
  .mp-prob-labels { display: flex; justify-content: space-between; font-size: 0.8rem; }

  /* ── Trust bar ── */
  .trust-bar {
    padding: 1.5rem 0;
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
  }
  .trust-inner {
    display: flex; align-items: center; justify-content: center;
    gap: 2rem; flex-wrap: wrap;
  }
  .trust-item {
    display: flex; align-items: center; gap: 0.5rem;
    font-size: 0.8rem; color: var(--text-muted); font-weight: 500;
  }
  .trust-icon { font-size: 1rem; }
  .trust-divider { width: 1px; height: 20px; background: var(--border); }

  /* ── Responsive ── */
  @media (max-width: 1024px) {
    .pillars { grid-template-columns: 1fr; max-width: 480px; }
    .how-grid { grid-template-columns: repeat(2, 1fr); }
    .market-preview-grid { grid-template-columns: repeat(2, 1fr); }
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
  }
  @media (max-width: 768px) {
    .hero { padding: 4rem 0 3rem; min-height: auto; }
    .hero-title { font-size: 2.8rem; }
    .market-preview-grid { grid-template-columns: 1fr; }
    .trust-inner { gap: 1rem; }
    .trust-divider { display: none; }
    .how-grid { grid-template-columns: 1fr; }
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
  }
</style>
