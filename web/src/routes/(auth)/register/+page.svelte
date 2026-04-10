<script lang="ts">
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';
  import { authToken, currentUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let email = '';
  let password = '';
  let display_name = '';
  let date_of_birth = '';
  let country_code = 'XK';
  let error = '';
  let loading = false;

  const countries = [
    { code: 'XK', name: 'Kosovo' },
    { code: 'AL', name: 'Albania' },
    { code: 'MK', name: 'North Macedonia' },
    { code: 'RS', name: 'Serbia' },
    { code: 'BA', name: 'Bosnia' },
    { code: 'ME', name: 'Montenegro' },
    { code: 'HR', name: 'Croatia' },
    { code: 'BG', name: 'Bulgaria' },
    { code: 'RO', name: 'Romania' },
    { code: 'GR', name: 'Greece' },
    { code: 'TR', name: 'Turkey' },
    { code: 'HU', name: 'Hungary' },
  ];

  async function handleRegister() {
    error = '';
    loading = true;
    try {
      const res = await api.register({ email, password, display_name, date_of_birth, country_code });
      authToken.set(res.access_token);
      currentUser.set({ id: res.user_id });
      goto('/markets');
    } catch (e: any) {
      error = e.error || 'Registration failed';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Sign Up — Adenora</title></svelte:head>

<div class="auth-page">
  <div class="auth-card bg-card">
    <h1>ADENORA</h1>
    <p class="auth-tagline">{$t('app.cta')}</p>

    <form on:submit|preventDefault={handleRegister}>
      <label>
        <span>{$t('auth.name')}</span>
        <input type="text" class="input" bind:value={display_name} required />
      </label>
      <label>
        <span>{$t('auth.email')}</span>
        <input type="email" class="input" bind:value={email} required />
      </label>
      <label>
        <span>{$t('auth.password')}</span>
        <input type="password" class="input" bind:value={password} required minlength="8" />
      </label>
      <label>
        <span>{$t('auth.dob')}</span>
        <input type="date" class="input" bind:value={date_of_birth} required />
      </label>
      <label>
        <span>{$t('auth.country')}</span>
        <select class="input" bind:value={country_code}>
          {#each countries as c}
            <option value={c.code}>{c.name}</option>
          {/each}
        </select>
      </label>
      {#if error}<p class="error">{error}</p>{/if}
      <button type="submit" class="btn btn-teal full" disabled={loading}>
        {loading ? '...' : $t('auth.register')}
      </button>
    </form>

    <p class="auth-switch">
      {$t('auth.have_account')} <a href="/login">{$t('nav.login')}</a>
    </p>
  </div>
</div>

<style>
  .auth-page { display: flex; justify-content: center; align-items: center; min-height: 70vh; }
  .auth-card { max-width: 440px; width: 100%; text-align: center; }
  .auth-card h1 { font-size: 1.8rem; letter-spacing: 0.2em; margin-bottom: 0.25rem; }
  .auth-tagline { color: var(--gold); font-weight: 600; margin-bottom: 2rem; }
  label { display: block; text-align: left; margin-bottom: 0.75rem; }
  label span { display: block; font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 0.25rem; }
  .full { width: 100%; margin-top: 0.5rem; }
  .error { color: var(--error); font-size: 0.85rem; margin-bottom: 0.5rem; }
  .auth-switch { margin-top: 1.5rem; font-size: 0.85rem; color: var(--text-muted); }
</style>
