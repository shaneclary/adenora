<script lang="ts">
  import { t, locale, locales } from '$lib/i18n';
  import { logout } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  function handleLogout() { logout(); goto('/'); }
  function setLocale(code: string) { locale.set(code); }
</script>

<svelte:head><title>{$t('nav.account')} — Adenora</title></svelte:head>

<h1>{$t('nav.account')}</h1>

<div class="grid grid-2">
  <div class="bg-card">
    <h3>Language</h3>
    <div class="locale-grid">
      {#each locales as loc}
        <button
          class="btn"
          class:btn-teal={$locale === loc.code}
          class:btn-outline={$locale !== loc.code}
          on:click={() => setLocale(loc.code)}
        >
          {loc.name}
        </button>
      {/each}
    </div>
  </div>

  <div class="bg-card">
    <h3>Responsible Gaming</h3>
    <p class="text-muted">Set deposit limits and self-exclusion periods to stay in control.</p>
    <div class="limits">
      <label>
        <span>Daily deposit limit (EUR)</span>
        <input type="number" class="input" placeholder="No limit" />
      </label>
      <label>
        <span>Weekly deposit limit (EUR)</span>
        <input type="number" class="input" placeholder="No limit" />
      </label>
    </div>
    <button class="btn btn-navy">{$t('common.save')}</button>
  </div>

  <div class="bg-card">
    <h3>KYC Verification</h3>
    <p class="text-muted">Age 21+ verification required for trading and lottery.</p>
    <button class="btn btn-gold">Start Verification</button>
  </div>

  <div class="bg-card">
    <h3>Account</h3>
    <button class="btn btn-crimson" on:click={handleLogout}>Log Out</button>
  </div>
</div>

<style>
  h1 { margin-bottom: 1.5rem; }
  h3 { margin-bottom: 0.75rem; }
  .locale-grid { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .limits { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1rem; }
  .limits label span { display: block; font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.25rem; }
</style>
