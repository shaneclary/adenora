<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api/client';
  import { authToken } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let campaign: any = null;
  let loading = true;
  let error = '';

  // Vote state
  let selectedProposal: string = '';
  let voteAmount = '';
  let voteComment = '';
  let isPublic = true;
  let submitting = false;
  let voteResult: any = null;
  let voteError = '';

  const PRESETS = [1, 5, 10, 25, 50];

  onMount(async () => {
    try {
      campaign = await api.getVoteCampaign($page.params.id);
    } catch (e: any) {
      error = e?.error || 'Failed to load';
    }
    loading = false;
  });

  $: proposals = campaign?.proposals ?? [];
  $: totalVotes = campaign?.total_votes ?? 0;
  $: totalFunds = campaign?.total_funds ?? 0;
  $: amountNum = parseFloat(voteAmount) || 0;
  $: votesYouGet = campaign?.vote_mode === 'one_person' ? 1
    : campaign?.vote_mode === 'capped' ? Math.min(Math.floor(amountNum), campaign?.vote_cap ?? 100)
    : Math.floor(amountNum);

  function modeExplainer(mode: string, cap: number) {
    if (mode === 'one_person') return 'One person, one vote. KYC verified.';
    if (mode === 'capped') return `\u20AC1 = 1 vote, max ${cap} votes per person. Extra funds still contribute.`;
    return '\u20AC1 = 1 vote, no cap. Conviction determines influence.';
  }

  async function submitVote() {
    if (!$authToken) { goto('/login'); return; }
    if (!selectedProposal || amountNum < 0.01) return;
    submitting = true; voteResult = null; voteError = '';
    try {
      voteResult = await api.castVote($page.params.id, {
        proposal_id: selectedProposal,
        amount_eur: amountNum,
        comment: voteComment || undefined,
        is_public: isPublic,
      });
      // Refresh campaign data
      campaign = await api.getVoteCampaign($page.params.id);
    } catch (e: any) {
      voteError = e?.error || 'Vote failed';
    }
    submitting = false;
  }
</script>

<svelte:head><title>{campaign?.title ?? 'Vote'} — Adenora</title></svelte:head>

<div class="page container">
  {#if loading}
    <div class="skeleton-hero"></div>
    {#each Array(3) as _}<div class="skeleton-card"></div>{/each}
  {:else if error}
    <div class="empty-state">
      <p class="text-crimson">{error}</p>
      <a href="/vote" class="link">Back to votes</a>
    </div>
  {:else if campaign}
    <!-- Header -->
    <div class="vote-header">
      <a href="/vote" class="back-link">&larr; All votes</a>
      <div class="vote-meta">
        <span class="vote-badge">{campaign.vote_mode === 'one_person' ? 'One Person One Vote' : campaign.vote_mode === 'capped' ? 'Capped Conviction' : 'Open Conviction'}</span>
        <span class="vote-time">{campaign.time_left} left</span>
      </div>
    </div>

    <h1 class="vote-title">{campaign.title}</h1>
    {#if campaign.description}
      <p class="vote-desc">{campaign.description}</p>
    {/if}

    <!-- Rules bar -->
    <div class="rules-bar">
      <span>{modeExplainer(campaign.vote_mode, campaign.vote_cap)}</span>
      <span class="rules-split">
        {#if campaign.split}
          {campaign.split.winner_pct}% winner
          {#if campaign.split.charity_pct > 0} · {campaign.split.charity_pct}% charity{/if}
          {#if campaign.split.pool_pct > 0} · {campaign.split.pool_pct}% pool{/if}
        {/if}
      </span>
    </div>

    <!-- Stats -->
    <div class="stats-bar">
      <div class="stat"><span class="stat-val">{totalVotes}</span><span class="stat-lbl">total votes</span></div>
      <div class="stat"><span class="stat-val">{'\u20AC'}{totalFunds}</span><span class="stat-lbl">total funds</span></div>
      {#if campaign.charity_project}
        <div class="stat"><span class="stat-val">{campaign.charity_project}</span><span class="stat-lbl">charity partner</span></div>
      {/if}
    </div>

    <!-- Proposals -->
    <h2 class="section-title">Proposals</h2>
    <div class="proposal-list">
      {#each proposals as p (p.id)}
        <button
          class="proposal-card"
          class:selected={selectedProposal === p.id}
          class:winner={p.is_winner}
          on:click={() => selectedProposal = p.id}
        >
          <div class="prop-header">
            <h3 class="prop-title">{p.title}</h3>
            {#if p.is_winner}<span class="winner-badge">WINNER</span>{/if}
          </div>
          {#if p.description}
            <p class="prop-desc">{p.description}</p>
          {/if}
          <!-- Vote bar -->
          <div class="prop-bar">
            <div class="prop-fill" style="width:{p.vote_pct}%"></div>
          </div>
          <div class="prop-stats">
            <span class="prop-votes">{p.vote_count} votes ({p.vote_pct}%)</span>
            <span class="prop-funds">{'\u20AC'}{p.fund_total}</span>
            <span class="prop-voters">{p.voter_count} voters</span>
          </div>
        </button>
      {/each}
    </div>

    <!-- Vote form -->
    {#if campaign.status === 'open' || campaign.status === 'voting'}
      <div class="vote-form glass-card">
        <h3>Cast Your Vote</h3>

        {#if !selectedProposal}
          <p class="text-muted">Select a proposal above to vote</p>
        {:else}
          <div class="selected-label">
            Voting for: <strong>{proposals.find(p => p.id === selectedProposal)?.title}</strong>
          </div>

          <div class="vf-section-label">How much conviction?</div>
          <div class="vf-presets">
            {#each PRESETS as p}
              <button class="vf-preset" class:active={voteAmount===String(p)} on:click={() => voteAmount=String(p)}>
                {'\u20AC'}{p}
              </button>
            {/each}
          </div>
          <input class="vf-input" type="number" min="0.01" step="0.01" placeholder="Custom amount" bind:value={voteAmount} />

          {#if amountNum > 0}
            <div class="vf-info">
              <span>Your votes: <strong>{votesYouGet}</strong></span>
              {#if campaign.vote_mode === 'capped' && amountNum > campaign.vote_cap}
                <span class="vf-note">{'\u20AC'}{(amountNum - campaign.vote_cap).toFixed(2)} contributes as funding (beyond vote cap)</span>
              {/if}
            </div>
          {/if}

          <input class="vf-input" type="text" maxlength="500" placeholder="Why this one? (optional)" bind:value={voteComment} />

          <label class="vf-public">
            <input type="checkbox" bind:checked={isPublic} />
            <span>Show my vote publicly</span>
          </label>

          {#if voteResult}
            <div class="alert-success">{voteResult.message}</div>
          {/if}
          {#if voteError}
            <div class="alert-error">{voteError}</div>
          {/if}

          <button class="vf-submit" on:click={submitVote} disabled={submitting || amountNum < 0.01}>
            {submitting ? 'Voting...' : `Vote with ${votesYouGet} conviction for \u20AC${amountNum.toFixed(2)}`}
          </button>
        {/if}
      </div>
    {/if}

    <!-- Conviction feed -->
    {#if campaign.feed?.length > 0}
      <h2 class="section-title">Conviction Feed</h2>
      <div class="feed-list">
        {#each campaign.feed as f}
          <div class="feed-item">
            <div class="feed-who">
              <strong>{f.name}</strong> voted for <span class="feed-proposal">{f.proposal}</span>
            </div>
            <div class="feed-meta">
              <span class="feed-amount">{'\u20AC'}{f.amount_eur}</span>
              <span class="feed-votes">{f.votes} votes</span>
              {#if f.comment}<span class="feed-comment">"{f.comment}"</span>{/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { padding: 1.5rem 0 5rem; }
  .container { max-width: 700px; margin: 0 auto; padding: 0 1.25rem; }

  .back-link { font-size: 0.8rem; color: var(--text-muted); text-decoration: none; display: inline-block; margin-bottom: 0.75rem; }
  .back-link:hover { color: var(--teal); }
  .vote-meta { display: flex; gap: 0.75rem; align-items: center; margin-bottom: 0.5rem; }
  .vote-badge { font-size: 0.68rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: 6px; background: rgba(0,194,224,0.12); color: var(--teal); }
  .vote-time { font-size: 0.78rem; color: var(--text-muted); }
  .vote-title { font-size: clamp(1.3rem, 4vw, 1.8rem); font-weight: 800; margin: 0 0 0.5rem; }
  .vote-desc { color: var(--text-muted); font-size: 0.9rem; margin: 0 0 1rem; line-height: 1.5; }

  .rules-bar {
    display: flex; justify-content: space-between; flex-wrap: wrap; gap: 0.5rem;
    font-size: 0.78rem; color: var(--text-muted); padding: 0.75rem 1rem;
    background: rgba(240,180,41,0.06); border: 1px solid rgba(240,180,41,0.15);
    border-radius: 10px; margin-bottom: 1rem;
  }
  .rules-split { color: var(--gold); font-weight: 600; }

  .stats-bar { display: flex; gap: 2rem; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .stat { display: flex; flex-direction: column; gap: 0.1rem; }
  .stat-val { font-size: 1.1rem; font-weight: 700; font-family: var(--font-mono); }
  .stat-lbl { font-size: 0.68rem; color: var(--text-muted); text-transform: uppercase; }

  .section-title { font-size: 1rem; font-weight: 700; margin: 0 0 0.75rem; }

  .proposal-list { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.5rem; }
  .proposal-card {
    display: block; width: 100%; text-align: left;
    background: var(--bg-glass); border: 2px solid var(--border);
    border-radius: 12px; padding: 1rem; cursor: pointer; transition: all 0.15s;
  }
  .proposal-card:hover { border-color: rgba(255,255,255,0.12); }
  .proposal-card.selected { border-color: var(--teal); background: rgba(0,194,224,0.04); }
  .proposal-card.winner { border-color: var(--gold); }
  .prop-header { display: flex; justify-content: space-between; align-items: center; }
  .prop-title { font-size: 0.95rem; font-weight: 700; margin: 0 0 0.3rem; }
  .winner-badge { font-size: 0.6rem; font-weight: 800; color: var(--gold); background: rgba(240,180,41,0.15); padding: 0.15rem 0.4rem; border-radius: 4px; }
  .prop-desc { font-size: 0.8rem; color: var(--text-muted); margin: 0 0 0.6rem; }
  .prop-bar { height: 8px; background: rgba(255,255,255,0.06); border-radius: 4px; overflow: hidden; margin-bottom: 0.4rem; }
  .prop-fill { height: 100%; background: linear-gradient(90deg, var(--teal), var(--gold)); border-radius: 4px; transition: width 0.5s; }
  .prop-stats { display: flex; gap: 1rem; font-size: 0.75rem; color: var(--text-muted); }
  .prop-votes { font-weight: 600; color: var(--text-secondary); }
  .prop-funds { font-family: var(--font-mono); }

  .glass-card { background: var(--bg-glass); border: 1px solid var(--border); border-radius: 14px; padding: 1.25rem; margin-bottom: 1.5rem; }
  .vote-form h3 { margin: 0 0 1rem; font-size: 1rem; }
  .selected-label { font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 1rem; }
  .selected-label strong { color: var(--teal); }
  .vf-section-label { font-size: 0.72rem; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 0.4rem; }
  .vf-presets { display: flex; gap: 0.4rem; margin-bottom: 0.5rem; }
  .vf-preset {
    flex: 1; padding: 0.55rem 0; border-radius: 10px; border: 1px solid var(--border);
    background: rgba(255,255,255,0.03); color: var(--text-secondary);
    font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.15s;
  }
  .vf-preset.active { background: rgba(240,180,41,0.15); border-color: var(--gold); color: var(--gold); }
  .vf-input {
    width: 100%; background: rgba(255,255,255,0.04); border: 1px solid var(--border);
    border-radius: 10px; color: var(--text-primary); padding: 0.6rem 0.75rem;
    font-size: 0.85rem; box-sizing: border-box; font-family: inherit; margin-bottom: 0.6rem;
  }
  .vf-input:focus { outline: none; border-color: var(--gold); }
  .vf-info { font-size: 0.82rem; color: var(--text-secondary); margin-bottom: 0.6rem; }
  .vf-info strong { color: var(--gold); }
  .vf-note { display: block; font-size: 0.75rem; color: var(--text-muted); margin-top: 0.2rem; }
  .vf-public { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.75rem; cursor: pointer; }
  .vf-public input { accent-color: var(--gold); }
  .vf-submit {
    width: 100%; padding: 0.85rem; border-radius: 12px; border: none;
    background: linear-gradient(135deg, var(--teal), var(--gold));
    color: #000; font-size: 0.95rem; font-weight: 800; cursor: pointer; transition: all 0.2s;
  }
  .vf-submit:disabled { opacity: 0.4; cursor: not-allowed; }
  .vf-submit:hover:not(:disabled) { box-shadow: 0 0 24px rgba(0,194,224,0.4); }

  .alert-success { padding: 0.6rem 0.85rem; border-radius: 8px; font-size: 0.82rem; margin-bottom: 0.75rem; background: rgba(34,197,94,0.1); border: 1px solid rgba(34,197,94,0.3); color: #22c55e; }
  .alert-error { padding: 0.6rem 0.85rem; border-radius: 8px; font-size: 0.82rem; margin-bottom: 0.75rem; background: rgba(232,50,74,0.1); border: 1px solid rgba(232,50,74,0.3); color: var(--crimson); }

  .feed-list { display: flex; flex-direction: column; gap: 0.4rem; }
  .feed-item { background: rgba(255,255,255,0.02); border-radius: 8px; padding: 0.6rem 0.85rem; border-bottom: 1px solid rgba(255,255,255,0.04); }
  .feed-who { font-size: 0.82rem; margin-bottom: 0.2rem; }
  .feed-who strong { color: var(--text-primary); }
  .feed-proposal { color: var(--teal); font-weight: 600; }
  .feed-meta { display: flex; gap: 0.75rem; font-size: 0.75rem; color: var(--text-muted); flex-wrap: wrap; }
  .feed-amount { color: var(--gold); font-weight: 700; font-family: var(--font-mono); }
  .feed-comment { font-style: italic; }

  .text-muted { color: var(--text-muted); }
  .text-crimson { color: var(--crimson); }
  .empty-state { text-align: center; padding: 3rem; }
  .empty-state h3 { margin: 0.5rem 0; }
  .link { color: var(--teal); text-decoration: none; }
  .skeleton-hero { height: 180px; border-radius: 20px; margin-bottom: 1.5rem; background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%); background-size: 200% 100%; animation: shimmer 1.5s infinite; }
  .skeleton-card { height: 100px; border-radius: 12px; margin-bottom: 0.75rem; background: linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.04) 75%); background-size: 200% 100%; animation: shimmer 1.5s infinite; }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
</style>
