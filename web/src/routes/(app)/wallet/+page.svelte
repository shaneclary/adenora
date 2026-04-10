<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { api } from '$lib/api/client';

  let wallets: any[] = [];
  let loading = true;
  let depositAmount = '';
  let depositCurrency = 'EUR';
  let withdrawAmount = '';
  let withdrawCurrency = 'EUR';
  let withdrawDest = '';
  let message = '';

  onMount(async () => {
    try { const res = await api.getWallet(); wallets = res.wallets; }
    catch (e) {}
    loading = false;
  });

  async function handleDeposit() {
    try {
      const res = await api.deposit({ currency: depositCurrency, amount: parseFloat(depositAmount), method: 'bank_transfer' });
      message = res.message;
      depositAmount = '';
      const r = await api.getWallet(); wallets = r.wallets;
    } catch (e: any) { message = e.error || 'Deposit failed'; }
  }

  async function handleWithdraw() {
    try {
      const res = await api.withdraw({ currency: withdrawCurrency, amount: parseFloat(withdrawAmount), method: 'bank_transfer', destination: withdrawDest });
      message = res.message;
      withdrawAmount = '';
      const r = await api.getWallet(); wallets = r.wallets;
    } catch (e: any) { message = e.error || 'Withdrawal failed'; }
  }
</script>

<svelte:head><title>{$t('wallet.title')} — Adenora</title></svelte:head>

<h1>{$t('wallet.title')}</h1>

{#if loading}
  <p class="text-muted">{$t('common.loading')}</p>
{:else}
  <div class="grid grid-3 wallet-grid">
    {#each wallets as w}
      <div class="wallet-card bg-card">
        <span class="currency">{w.currency}</span>
        <div class="balance">
          <span class="label">{$t('wallet.available')}</span>
          <span class="amount">{w.available}</span>
        </div>
        <div class="balance">
          <span class="label">{$t('wallet.reserved')}</span>
          <span class="amount muted">{w.reserved}</span>
        </div>
      </div>
    {/each}
    {#if wallets.length === 0}
      <div class="bg-card"><p class="text-muted">No wallets yet. Make a deposit to get started.</p></div>
    {/if}
  </div>

  <div class="grid grid-2 actions">
    <div class="bg-card">
      <h3>{$t('wallet.deposit')}</h3>
      <p class="free-label text-success">{$t('wallet.free_deposits')}</p>
      <input type="number" class="input" placeholder="Amount" bind:value={depositAmount} />
      <button class="btn btn-gold full" on:click={handleDeposit}>{$t('wallet.deposit')}</button>
    </div>
    <div class="bg-card">
      <h3>{$t('wallet.withdraw')}</h3>
      <p class="free-label text-success">{$t('wallet.free_withdrawals')}</p>
      <input type="number" class="input" placeholder="Amount" bind:value={withdrawAmount} />
      <input type="text" class="input" placeholder="Destination (IBAN, address)" bind:value={withdrawDest} />
      <button class="btn btn-navy full" on:click={handleWithdraw}>{$t('wallet.withdraw')}</button>
    </div>
  </div>

  {#if message}
    <div class="message bg-card">{message}</div>
  {/if}
{/if}

<style>
  h1 { margin-bottom: 1.5rem; }
  .wallet-grid { margin-bottom: 2rem; }
  .wallet-card { text-align: center; }
  .currency { font-size: 1.2rem; font-weight: 700; color: var(--gold); display: block; margin-bottom: 1rem; }
  .balance { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
  .balance .label { font-size: 0.85rem; color: var(--text-muted); }
  .balance .amount { font-family: var(--font-mono); font-weight: 600; }
  .balance .muted { color: var(--text-muted); }
  .actions h3 { margin-bottom: 0.5rem; }
  .free-label { font-size: 0.8rem; margin-bottom: 0.75rem; }
  .actions .input { margin-bottom: 0.5rem; }
  .full { width: 100%; }
  .message { margin-top: 1rem; text-align: center; color: var(--teal); }
</style>
