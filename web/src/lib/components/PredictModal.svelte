<script lang="ts">
  import { api } from '$lib/api/client';
  import { authToken } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  export let market: any;
  export let initialSide: string = 'yes';
  export let onClose: () => void = () => {};

  let side = initialSide;
  let amount = '';
  let isPublic = false;
  let comment = '';
  let step: 'amount' | 'confirm' | 'success' | 'error' = 'amount';
  let submitting = false;
  let result: any = null;
  let error = '';

  const PRESETS = [1, 5, 10, 20, 50];

  $: amountNum = parseFloat(amount) || 0;
  $: chance = side === 'yes' ? (market.chance_yes_pct ?? 50) : (market.chance_no_pct ?? 50);
  $: payout = chance > 0 ? (amountNum / (chance / 100)).toFixed(2) : '0.00';
  $: profit = (parseFloat(payout) - amountNum).toFixed(2);

  function selectAmount(p: number) {
    amount = String(p);
  }

  function flipSide() {
    side = side === 'yes' ? 'no' : 'yes';
  }

  function proceed() {
    if (!$authToken) {
      goto('/login');
      return;
    }
    if (amountNum < 0.01) return;
    step = 'confirm';
  }

  async function submit() {
    submitting = true;
    error = '';
    try {
      result = await api.predict({
        market_id: market.id,
        side,
        amount_eur: amountNum,
        public: isPublic,
        comment: isPublic ? comment : undefined,
      });
      step = 'success';
    } catch (e: any) {
      error = e?.error || e?.message || 'Something went wrong';
      step = 'error';
    }
    submitting = false;
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="pm-overlay" on:click|self={onClose} on:keydown={(e) => e.key === 'Escape' && onClose()} role="dialog" aria-modal="true">
  <div class="pm-sheet">
    <!-- Close -->
    <button class="pm-close" on:click={onClose}>x</button>

    {#if step === 'amount'}
      <!-- STEP 1: pick side + amount -->
      <div class="pm-question">{market.question}</div>

      <div class="pm-side-toggle">
        <button class="pm-side" class:active={side==='yes'} class:yes={side==='yes'} on:click={() => side='yes'}>YES</button>
        <button class="pm-side" class:active={side==='no'} class:no={side==='no'} on:click={() => side='no'}>NO</button>
      </div>

      <div class="pm-chance">
        You think <strong class:text-teal={side==='yes'} class:text-crimson={side==='no'}>{side.toUpperCase()}</strong> — crowd says {chance}%
      </div>

      <div class="pm-section-label">How much?</div>
      <div class="pm-presets">
        {#each PRESETS as p}
          <button class="pm-preset" class:active={amount===String(p)} on:click={() => selectAmount(p)}>
            {'\u20AC'}{p}
          </button>
        {/each}
      </div>
      <input class="pm-input" type="number" min="0.01" step="0.01" placeholder="Custom amount" bind:value={amount} />

      {#if amountNum > 0}
        <div class="pm-payout-card">
          <div class="pm-payout-row">
            <span class="pm-payout-label">You could win</span>
            <span class="pm-payout-amount">{'\u20AC'}{payout}</span>
          </div>
          <div class="pm-payout-row pm-profit">
            <span class="pm-payout-label">Profit if right</span>
            <span class="pm-payout-amount text-teal">+{'\u20AC'}{profit}</span>
          </div>
        </div>
      {/if}

      <!-- Public prediction toggle -->
      <label class="pm-public-toggle">
        <input type="checkbox" bind:checked={isPublic} />
        <span>Make it public — show your conviction</span>
      </label>

      {#if isPublic}
        <input class="pm-input" type="text" maxlength="280" placeholder="Why do you think {side.toUpperCase()}? (optional)" bind:value={comment} />
      {/if}

      <button class="pm-submit" disabled={amountNum < 0.01} on:click={proceed}>
        {amountNum >= 0.01 ? `Predict ${side.toUpperCase()} for \u20AC${amountNum.toFixed(2)}` : 'Pick an amount'}
      </button>

    {:else if step === 'confirm'}
      <!-- STEP 2: confirm -->
      <div class="pm-confirm-icon">{side === 'yes' ? '?' : '?'}</div>
      <div class="pm-question" style="font-size:1rem">{market.question}</div>

      <div class="pm-summary">
        <div class="pm-sum-row"><span>Your call</span><strong class:text-teal={side==='yes'} class:text-crimson={side==='no'}>{side.toUpperCase()}</strong></div>
        <div class="pm-sum-row"><span>Amount</span><strong>{'\u20AC'}{amountNum.toFixed(2)}</strong></div>
        <div class="pm-sum-row"><span>Crowd says</span><strong>{chance}%</strong></div>
        <div class="pm-sum-row"><span>You could win</span><strong class="text-teal">{'\u20AC'}{payout}</strong></div>
        {#if isPublic}
          <div class="pm-sum-row"><span>Visibility</span><strong>Public</strong></div>
        {/if}
      </div>

      <button class="pm-submit pm-submit-go" on:click={submit} disabled={submitting}>
        {submitting ? 'Submitting...' : 'Confirm Prediction'}
      </button>
      <button class="pm-back" on:click={() => step = 'amount'}>Go back</button>

    {:else if step === 'success'}
      <!-- STEP 3: success -->
      <div class="pm-success-icon">&#10003;</div>
      <h3 class="pm-success-title">You predicted {side.toUpperCase()}!</h3>
      <p class="pm-success-sub">{'\u20AC'}{result?.amount_eur} on "{market.question}"</p>

      {#if result?.charity_contribution_eur && result.charity_contribution_eur !== '0'}
        <div class="pm-charity-callout">
          {'\u20AC'}{result.charity_contribution_eur} just went to <strong>{result.charity_project}</strong>
        </div>
      {/if}

      {#if isPublic}
        <div class="pm-public-callout">Your prediction is visible on the debate feed</div>
      {/if}

      <div class="pm-success-actions">
        <button class="pm-submit" on:click={onClose}>Keep Browsing</button>
        <a href="/portfolio" class="pm-link">See Your Predictions</a>
      </div>

    {:else if step === 'error'}
      <div class="pm-error-icon">!</div>
      <h3 class="pm-error-title">Something went wrong</h3>
      <p class="pm-error-msg">{error}</p>
      <button class="pm-submit" on:click={() => step = 'amount'}>Try Again</button>
    {/if}
  </div>
</div>

<style>
  .pm-overlay {
    position: fixed; inset: 0; z-index: 9999;
    background: rgba(0,0,0,0.7); backdrop-filter: blur(8px);
    display: flex; align-items: flex-end; justify-content: center;
  }
  @media (min-width: 640px) { .pm-overlay { align-items: center; } }

  .pm-sheet {
    background: var(--bg-surface); border: 1px solid var(--border);
    border-radius: 20px 20px 0 0; padding: 1.5rem; width: 100%;
    max-width: 420px; max-height: 90vh; overflow-y: auto;
    position: relative;
  }
  @media (min-width: 640px) { .pm-sheet { border-radius: 20px; } }

  .pm-close {
    position: absolute; top: 1rem; right: 1rem;
    background: rgba(255,255,255,0.06); border: none; color: var(--text-muted);
    width: 32px; height: 32px; border-radius: 50%; cursor: pointer;
    font-size: 0.85rem; display: flex; align-items: center; justify-content: center;
  }
  .pm-close:hover { background: rgba(255,255,255,0.12); color: var(--text-primary); }

  .pm-question { font-size: 1.1rem; font-weight: 700; line-height: 1.4; margin-bottom: 1rem; color: var(--text-primary); padding-right: 2rem; }

  .pm-side-toggle { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-bottom: 0.75rem; }
  .pm-side {
    padding: 0.8rem; border-radius: 12px; border: 2px solid var(--border);
    background: none; color: var(--text-muted); font-size: 1rem; font-weight: 800;
    letter-spacing: 0.08em; cursor: pointer; transition: all 0.15s;
  }
  .pm-side.active.yes { background: rgba(0,194,224,0.15); border-color: var(--teal); color: var(--teal); }
  .pm-side.active.no { background: rgba(232,50,74,0.15); border-color: var(--crimson); color: var(--crimson); }

  .pm-chance { font-size: 0.85rem; color: var(--text-muted); margin-bottom: 1.25rem; text-align: center; }

  .pm-section-label { font-size: 0.75rem; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 0.5rem; }

  .pm-presets { display: flex; gap: 0.4rem; margin-bottom: 0.5rem; }
  .pm-preset {
    flex: 1; padding: 0.6rem 0; border-radius: 10px; border: 1px solid var(--border);
    background: rgba(255,255,255,0.03); color: var(--text-secondary);
    font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: all 0.15s;
  }
  .pm-preset.active { background: rgba(240,180,41,0.15); border-color: var(--gold); color: var(--gold); }
  .pm-preset:hover:not(.active) { background: rgba(255,255,255,0.06); }

  .pm-input {
    width: 100%; background: rgba(255,255,255,0.04); border: 1px solid var(--border);
    border-radius: 10px; color: var(--text-primary); padding: 0.7rem 0.85rem;
    font-size: 0.9rem; box-sizing: border-box; font-family: inherit; margin-bottom: 0.75rem;
  }
  .pm-input:focus { outline: none; border-color: var(--gold); }

  .pm-payout-card {
    background: rgba(255,255,255,0.03); border-radius: 12px;
    padding: 0.85rem 1rem; margin-bottom: 1rem;
  }
  .pm-payout-row { display: flex; justify-content: space-between; align-items: center; padding: 0.2rem 0; }
  .pm-payout-label { font-size: 0.85rem; color: var(--text-muted); }
  .pm-payout-amount { font-size: 1.3rem; font-weight: 800; font-family: var(--font-mono); color: var(--text-primary); }
  .pm-profit .pm-payout-amount { font-size: 0.9rem; }

  .pm-public-toggle {
    display: flex; align-items: center; gap: 0.5rem; font-size: 0.82rem;
    color: var(--text-muted); margin-bottom: 0.75rem; cursor: pointer;
  }
  .pm-public-toggle input[type="checkbox"] { accent-color: var(--gold); }

  .pm-submit {
    width: 100%; padding: 0.85rem; border-radius: 12px; border: none;
    background: linear-gradient(135deg, var(--teal), var(--gold));
    color: #000; font-size: 1rem; font-weight: 800; cursor: pointer;
    transition: all 0.2s;
  }
  .pm-submit:disabled { opacity: 0.4; cursor: not-allowed; }
  .pm-submit:hover:not(:disabled) { box-shadow: 0 0 24px rgba(0,194,224,0.4); transform: translateY(-1px); }
  .pm-submit-go { background: linear-gradient(135deg, var(--gold), var(--crimson)); }

  .pm-back { width: 100%; margin-top: 0.5rem; padding: 0.6rem; background: none; border: 1px solid var(--border); border-radius: 10px; color: var(--text-muted); font-size: 0.85rem; cursor: pointer; }

  .pm-summary { background: rgba(255,255,255,0.03); border-radius: 12px; padding: 1rem; margin-bottom: 1rem; }
  .pm-sum-row { display: flex; justify-content: space-between; padding: 0.3rem 0; font-size: 0.9rem; }
  .pm-sum-row span { color: var(--text-muted); }
  .pm-confirm-icon { text-align: center; font-size: 2.5rem; margin-bottom: 0.75rem; }

  .pm-success-icon { text-align: center; font-size: 3rem; color: var(--teal); margin-bottom: 0.5rem; }
  .pm-success-title { text-align: center; font-size: 1.25rem; margin: 0 0 0.5rem; }
  .pm-success-sub { text-align: center; font-size: 0.85rem; color: var(--text-muted); margin-bottom: 1rem; }
  .pm-charity-callout {
    text-align: center; padding: 0.75rem; border-radius: 10px;
    background: rgba(232,50,74,0.08); border: 1px solid rgba(232,50,74,0.2);
    font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 0.75rem;
  }
  .pm-charity-callout strong { color: var(--crimson); }
  .pm-public-callout {
    text-align: center; font-size: 0.78rem; color: var(--gold); margin-bottom: 1rem;
  }
  .pm-success-actions { display: flex; flex-direction: column; gap: 0.5rem; }
  .pm-link { text-align: center; color: var(--teal); font-size: 0.85rem; text-decoration: none; padding: 0.5rem; }

  .pm-error-icon { text-align: center; font-size: 3rem; color: var(--crimson); margin-bottom: 0.5rem; }
  .pm-error-title { text-align: center; margin: 0 0 0.5rem; }
  .pm-error-msg { text-align: center; color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem; }

  .text-teal { color: var(--teal); }
  .text-crimson { color: var(--crimson); }
</style>
