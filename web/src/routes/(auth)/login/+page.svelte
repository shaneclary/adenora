<script lang="ts">
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';
  import { authToken, currentUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleLogin() {
    error = '';
    loading = true;
    try {
      const res = await api.login({ email, password });
      authToken.set(res.access_token);
      currentUser.set({ id: res.user_id, is_bot: res.is_bot });
      goto('/markets');
    } catch (e: any) {
      error = e.error || 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Log In — Adenora</title></svelte:head>

<div class="auth-page">
  <div class="auth-card bg-card">
    <h1>ADENORA</h1>
    <p class="auth-tagline">{$t('app.cta')}</p>

    <form on:submit|preventDefault={handleLogin}>
      <label>
        <span>{$t('auth.email')}</span>
        <input type="email" class="input" bind:value={email} required />
      </label>
      <label>
        <span>{$t('auth.password')}</span>
        <input type="password" class="input" bind:value={password} required />
      </label>
      {#if error}<p class="error">{error}</p>{/if}
      <button type="submit" class="btn btn-teal full" disabled={loading}>
        {loading ? '...' : $t('auth.login')}
      </button>
    </form>

    <p class="auth-switch">
      {$t('auth.no_account')} <a href="/register">{$t('nav.register')}</a>
    </p>
  </div>
</div>

<style>
  .auth-page { display: flex; justify-content: center; align-items: center; min-height: 70vh; }
  .auth-card { max-width: 400px; width: 100%; text-align: center; }
  .auth-card h1 { font-size: 1.8rem; letter-spacing: 0.2em; margin-bottom: 0.25rem; }
  .auth-tagline { color: var(--gold); font-weight: 600; margin-bottom: 2rem; }
  label { display: block; text-align: left; margin-bottom: 1rem; }
  label span { display: block; font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 0.25rem; }
  .full { width: 100%; margin-top: 0.5rem; }
  .error { color: var(--error); font-size: 0.85rem; margin-bottom: 0.5rem; }
  .auth-switch { margin-top: 1.5rem; font-size: 0.85rem; color: var(--text-muted); }
</style>
