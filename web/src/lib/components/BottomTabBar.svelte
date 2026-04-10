<script lang="ts">
  import { page } from '$app/stores';
  import { authToken } from '$lib/stores/auth';

  $: path = $page.url.pathname;

  function isActive(p: string) {
    if (p === '/markets') return path === '/markets' || path === '/';
    return path.startsWith(p);
  }
</script>

<nav class="btb">
  <a href="/markets" class="btb-tab" class:active={isActive('/markets')}>
    <svg class="btb-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
    <span>Markets</span>
  </a>
  <a href="/impact" class="btb-tab" class:active={isActive('/impact')}>
    <svg class="btb-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
    <span>Impact</span>
  </a>
  <a href={$authToken ? '/portfolio' : '/login'} class="btb-tab" class:active={isActive('/portfolio')}>
    <svg class="btb-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
    <span>{$authToken ? 'Profile' : 'Sign In'}</span>
  </a>
</nav>

<style>
  .btb {
    display: none; /* Show only on mobile */
    position: fixed; bottom: 0; left: 0; right: 0; z-index: 2000;
    background: rgba(8,14,24,0.95); backdrop-filter: blur(20px);
    border-top: 1px solid var(--border);
    padding: 0.4rem 0 max(0.4rem, env(safe-area-inset-bottom));
    justify-content: space-around;
  }
  @media (max-width: 768px) { .btb { display: flex; } }

  .btb-tab {
    display: flex; flex-direction: column; align-items: center; gap: 0.2rem;
    text-decoration: none; color: var(--text-muted); font-size: 0.65rem;
    font-weight: 600; padding: 0.3rem 0.75rem; border-radius: 8px;
    transition: color 0.15s;
  }
  .btb-tab.active { color: var(--teal); }
  .btb-tab:hover { color: var(--text-secondary); }

  .btb-icon { width: 22px; height: 22px; }
</style>
