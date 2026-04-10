<script lang="ts">
  import '../app.css';
  import { t, locale, locales } from '$lib/i18n';
  import { authToken, logout } from '$lib/stores/auth';
  import { page } from '$app/stores';
  import BottomTabBar from '$lib/components/BottomTabBar.svelte';

  let mobileOpen = false;
  function setLocale(e: Event) { locale.set((e.target as HTMLSelectElement).value); }
  function isActive(path: string) { return $page.url.pathname.startsWith(path); }
  function toggleMobile() { mobileOpen = !mobileOpen; }
</script>

<div class="app">
  <!-- ── NAV ── -->
  <nav class="topbar">
    <div class="topbar-inner container-lg">

      <!-- Logo -->
      <a href="/" class="logo">
        <div class="logo-mark">
          <span class="mark-win"></span>
          <span class="mark-grow"></span>
          <span class="mark-give"></span>
        </div>
        <span class="logo-text">ADENORA</span>
      </a>

      <!-- Nav links -->
      <div class="nav-links hide-mobile">
        <a href="/markets"   class="nav-link" class:active={isActive('/markets')}>
          <span class="nav-pip pip-gold"></span>Markets
        </a>
        <a href="/unlimited" class="nav-link" class:active={isActive('/unlimited')}>
          <span class="nav-pip pip-gold"></span>Ultimate
        </a>
        <a href="/lottery"   class="nav-link" class:active={isActive('/lottery')}>
          <span class="nav-pip pip-crimson"></span>Lottery
        </a>
        <a href="/games"     class="nav-link" class:active={isActive('/games')}>
          <span class="nav-pip pip-teal"></span>Games
        </a>
        <a href="/bots"      class="nav-link" class:active={isActive('/bots')}>
          <span class="nav-pip pip-teal"></span>Bot Arena
        </a>
        <a href="/vote"      class="nav-link" class:active={isActive('/vote')}>
          <span class="nav-pip pip-gold"></span>Vote
        </a>
        <a href="/charity"   class="nav-link" class:active={isActive('/charity')}>
          <span class="nav-pip pip-crimson"></span>Give
        </a>
      </div>

      <!-- Right actions -->
      <div class="nav-right">
        <select class="locale-select hide-tablet" on:change={setLocale} aria-label="Language">
          {#each locales as loc}
            <option value={loc.code} selected={$locale === loc.code}>{loc.name}</option>
          {/each}
        </select>

        {#if $authToken}
          <a href="/portfolio" class="btn btn-ghost btn-sm hide-mobile">Portfolio</a>
          <a href="/wallet"    class="btn btn-gold  btn-sm">Wallet</a>
          <button class="btn btn-outline btn-sm" on:click={logout}>Sign out</button>
        {:else}
          <a href="/login"    class="btn btn-ghost  btn-sm hide-mobile">Sign in</a>
          <a href="/register" class="btn btn-primary btn-sm">Get Started</a>
        {/if}

        <!-- Mobile hamburger -->
        <button class="hamburger" on:click={toggleMobile} aria-label="Menu">
          <span class:open={mobileOpen}></span>
          <span class:open={mobileOpen}></span>
          <span class:open={mobileOpen}></span>
        </button>
      </div>
    </div>

    <!-- Mobile drawer -->
    {#if mobileOpen}
      <div class="mobile-menu">
        <a href="/markets"   on:click={toggleMobile}>Markets</a>
        <a href="/unlimited" on:click={toggleMobile}>Ultimate</a>
        <a href="/lottery"   on:click={toggleMobile}>Lottery</a>
        <a href="/games"     on:click={toggleMobile}>Games</a>
        <a href="/bots"      on:click={toggleMobile}>Bot Arena</a>
        <a href="/vote"      on:click={toggleMobile}>Vote</a>
        <a href="/charity"   on:click={toggleMobile}>Give</a>
        {#if $authToken}
          <a href="/portfolio" on:click={toggleMobile}>Portfolio</a>
          <a href="/wallet"    on:click={toggleMobile}>Wallet</a>
          <button class="btn btn-outline" on:click={() => { logout(); toggleMobile(); }}>Sign out</button>
        {:else}
          <a href="/login"    on:click={toggleMobile} class="btn btn-outline">Sign in</a>
          <a href="/register" on:click={toggleMobile} class="btn btn-primary">Get Started</a>
        {/if}
      </div>
    {/if}
  </nav>

  <!-- ── MAIN ── -->
  <main class="content">
    <slot />
  </main>

  <!-- ── BOTTOM TAB BAR (mobile) ── -->
  <BottomTabBar />

  <!-- ── FOOTER ── -->
  <footer class="footer">
    <div class="footer-inner container-lg">
      <div class="footer-brand">
        <div class="logo">
          <div class="logo-mark sm">
            <span class="mark-win"></span>
            <span class="mark-grow"></span>
            <span class="mark-give"></span>
          </div>
          <span class="logo-text">ADENORA</span>
        </div>
        <p class="footer-tagline">Where Value Flows</p>
        <p class="footer-copy">© 2026 GLC · Global Lottery Consultants</p>
      </div>

      <div class="footer-links">
        <div class="footer-col">
          <span class="footer-col-title">Platform</span>
          <a href="/markets">Human Markets</a>
          <a href="/unlimited">Ultimate Markets</a>
          <a href="/lottery">Lottery</a>
          <a href="/games">Games</a>
          <a href="/bots">Bot Arena</a>
        </div>
        <div class="footer-col">
          <span class="footer-col-title">Company</span>
          <a href="/charity">Give</a>
          <a href="/register">Register</a>
          <a href="/login">Sign In</a>
        </div>
        <div class="footer-col">
          <span class="footer-col-title">Legal</span>
          <a href="#">Terms</a>
          <a href="#">Privacy</a>
          <a href="#">Responsible Gaming</a>
        </div>
      </div>
    </div>

    <div class="footer-bottom container-lg">
      <span class="text-xs text-muted">Jurisdiction-led rollout · Market eligibility varies by country · Age 21+ where required</span>
      <div class="footer-pillars">
        <span class="text-teal text-xs font-bold">WIN</span>
        <span class="text-gold text-xs font-bold">GROW</span>
        <span class="text-crimson text-xs font-bold">GIVE</span>
      </div>
    </div>
  </footer>
</div>

<style>
  .app { display: flex; flex-direction: column; min-height: 100vh; }

  /* ── Topbar ── */
  .topbar {
    position: sticky; top: 0; z-index: 1000;
    background: rgba(8, 14, 24, 0.85);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-bottom: 1px solid var(--border);
    transition: background var(--t-med);
  }
  .topbar-inner {
    display: flex; align-items: center; gap: 1rem;
    height: 60px;
  }

  /* ── Logo ── */
  .logo {
    display: flex; align-items: center; gap: 0.6rem;
    text-decoration: none; flex-shrink: 0;
  }
  .logo-mark {
    position: relative; width: 28px; height: 28px;
    display: flex; align-items: center; justify-content: center;
  }
  .logo-mark span {
    position: absolute; border-radius: 50%;
    transition: transform 0.3s var(--ease-spring);
  }
  .mark-win  { width: 12px; height: 12px; background: var(--teal);    top: 0;     left: 50%; transform: translateX(-50%); }
  .mark-grow { width: 12px; height: 12px; background: var(--gold);    bottom: 2px; left: 2px; }
  .mark-give { width: 12px; height: 12px; background: var(--crimson); bottom: 2px; right: 2px; }

  .logo:hover .mark-win   { transform: translateX(-50%) translateY(-2px); }
  .logo:hover .mark-grow  { transform: translateX(-2px) translateY(2px); }
  .logo:hover .mark-give  { transform: translateX(2px)  translateY(2px); }

  .logo-mark.sm { width: 22px; height: 22px; }
  .logo-mark.sm .mark-win  { width: 9px; height: 9px; }
  .logo-mark.sm .mark-grow { width: 9px; height: 9px; }
  .logo-mark.sm .mark-give { width: 9px; height: 9px; }

  .logo-text {
    font-size: 1rem; font-weight: 900;
    letter-spacing: 0.18em; color: var(--text-primary);
  }

  /* ── Nav links ── */
  .nav-links {
    display: flex; align-items: center; gap: 0.25rem;
    margin-left: 0.5rem; flex: 1;
  }
  .nav-link {
    display: flex; align-items: center; gap: 0.35rem;
    padding: 0.4rem 0.85rem;
    border-radius: var(--r-md);
    color: var(--text-muted);
    font-size: 0.875rem; font-weight: 500;
    text-decoration: none;
    transition: all var(--t-fast);
    position: relative;
  }
  .nav-link:hover { color: var(--text-primary); background: var(--bg-glass); }
  .nav-link.active { color: var(--text-primary); background: var(--bg-glass-hover); }
  .nav-pip { width: 5px; height: 5px; border-radius: 50%; flex-shrink: 0; }
  .pip-teal    { background: var(--teal); }
  .pip-gold    { background: var(--gold); }
  .pip-crimson { background: var(--crimson); }

  /* ── Nav right ── */
  .nav-right {
    display: flex; align-items: center; gap: 0.5rem;
    margin-left: auto;
  }
  .locale-select {
    background: var(--bg-glass);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 0.3rem 0.6rem;
    font-size: 0.78rem; font-family: var(--font-sans);
    cursor: pointer;
  }

  /* ── Hamburger ── */
  .hamburger {
    display: none;
    flex-direction: column; gap: 4px;
    background: none; border: none;
    padding: 6px; cursor: pointer;
  }
  .hamburger span {
    display: block; width: 20px; height: 2px;
    background: var(--text-secondary); border-radius: 2px;
    transition: all var(--t-fast);
  }
  .hamburger span.open:nth-child(1) { transform: translateY(6px) rotate(45deg); }
  .hamburger span.open:nth-child(2) { opacity: 0; }
  .hamburger span.open:nth-child(3) { transform: translateY(-6px) rotate(-45deg); }

  /* ── Mobile menu ── */
  .mobile-menu {
    display: flex; flex-direction: column; gap: 0.5rem;
    padding: 1rem 1.5rem 1.5rem;
    border-top: 1px solid var(--border);
    background: rgba(8,14,24,0.95);
  }
  .mobile-menu a {
    padding: 0.75rem 0; font-size: 0.95rem; font-weight: 500;
    color: var(--text-secondary); border-bottom: 1px solid var(--border-subtle);
  }
  .mobile-menu a:hover { color: var(--text-primary); }

  /* ── Content ── */
  .content { flex: 1; position: relative; z-index: 1; }
  @media (max-width: 768px) { .content { padding-bottom: 70px; } }

  /* ── Footer ── */
  .footer {
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
    padding: 3rem 0 0;
    margin-top: auto;
  }
  .footer-inner {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 4rem;
    padding-bottom: 2.5rem;
  }
  .footer-brand { display: flex; flex-direction: column; gap: 0.75rem; }
  .footer-tagline { font-size: 0.85rem; color: var(--text-muted); }
  .footer-copy { font-size: 0.75rem; color: var(--text-muted); }

  .footer-links {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 2rem;
  }
  .footer-col { display: flex; flex-direction: column; gap: 0.6rem; }
  .footer-col-title {
    font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em;
    text-transform: uppercase; color: var(--text-secondary);
    margin-bottom: 0.25rem;
  }
  .footer-col a { font-size: 0.83rem; color: var(--text-muted); transition: color var(--t-fast); }
  .footer-col a:hover { color: var(--text-primary); }

  .footer-bottom {
    display: flex; justify-content: space-between; align-items: center;
    padding: 1rem 0;
    border-top: 1px solid var(--border-subtle);
  }
  .footer-pillars { display: flex; gap: 1rem; }

  /* ── Responsive ── */
  @media (max-width: 768px) {
    .hamburger { display: flex; }
    .footer-inner { grid-template-columns: 1fr; gap: 2rem; }
    .footer-links { grid-template-columns: repeat(2, 1fr); }
    .footer-bottom { flex-direction: column; gap: 0.75rem; text-align: center; }
  }
</style>
