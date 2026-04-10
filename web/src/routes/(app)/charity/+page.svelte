<script lang="ts">
  import { onMount } from 'svelte';
  import { authToken } from '$lib/stores/auth';
  import { api } from '$lib/api/client';

  let campaigns: any[] = [];
  let summary: any = null;
  let ledger: any[] = [];
  let loading = true;

  // Donate modal state
  let donating: any = null;
  let donateAmount = '20';
  let donateMsg = '';
  let donateAnon = false;
  let donateSuccess = false;
  let donateError = '';

  const CATEGORY_ICONS: Record<string, string> = {
    water: '💧', education: '📚', infrastructure: '🏗️',
    clean_energy: '☀️', landmine_removal: '🕊️',
    healthcare: '🏥', disaster_relief: '🆘',
    housing: '🏠', food_security: '🌾', other: '🌍',
  };

  const MOCK_CAMPAIGNS: any[] = [
    {
      id: '1', slug: 'kosovo-desal-2026',
      title: 'Kosovo Clean Water Initiative',
      tagline: 'Fund a desalination plant for 12,000 families in northern Kosovo',
      category: 'water', status: 'active', is_featured: true,
      goal_amount: 250000, amount_raised: 87340, donor_count: 412,
      progress_pct: 35, currency: 'EUR',
      impact_metric: 'families served', impact_value: '12,000',
      location: 'Mitrovica, Kosovo', partner_org: 'H.E.L.P.',
    },
    {
      id: '2', slug: 'tirana-school-rebuild',
      title: 'Tirana School Rebuild',
      tagline: 'Rebuild earthquake-damaged classrooms for 800 students',
      category: 'education', status: 'active', is_featured: true,
      goal_amount: 120000, amount_raised: 63200, donor_count: 289,
      progress_pct: 53, currency: 'EUR',
      impact_metric: 'students served', impact_value: '800',
      location: 'Tirana, Albania', partner_org: 'Balkan Education Fund',
    },
    {
      id: '3', slug: 'ohrid-landmine-clearance',
      title: 'Ohrid Landmine Clearance',
      tagline: 'Clear 450 hectares of farmland in North Macedonia',
      category: 'landmine_removal', status: 'active', is_featured: true,
      goal_amount: 180000, amount_raised: 141000, donor_count: 531,
      progress_pct: 78, currency: 'EUR',
      impact_metric: 'hectares cleared', impact_value: '450',
      location: 'Ohrid Region, N. Macedonia', partner_org: 'Roots of Peace',
    },
    {
      id: '4', slug: 'balkans-solar-schools',
      title: 'Solar Schools Balkans',
      tagline: 'Install solar panels on 30 schools across 3 countries',
      category: 'clean_energy', status: 'active', is_featured: false,
      goal_amount: 95000, amount_raised: 12400, donor_count: 88,
      progress_pct: 13, currency: 'EUR',
      impact_metric: 'schools powered', impact_value: '30',
      location: 'Western Balkans', partner_org: 'GLC Foundation',
    },
  ];

  onMount(async () => {
    try {
      const [camRes, sumRes, ledRes] = await Promise.all([
        api.listCampaigns().catch(() => null),
        api.fundingSummary().catch(() => null),
        api.getLedger().catch(() => null),
      ]);
      campaigns = camRes?.campaigns?.length ? camRes.campaigns : MOCK_CAMPAIGNS;
      summary = sumRes;
      ledger = ledRes?.entries?.slice(0, 10) || [];
    } catch {
      campaigns = MOCK_CAMPAIGNS;
    }
    loading = false;
  });

  function fmtEur(n: number) {
    return new Intl.NumberFormat('en', { style: 'currency', currency: 'EUR', maximumFractionDigits: 0 }).format(n);
  }
  function fmtK(n: number) { return n >= 1000 ? `${(n/1000).toFixed(1)}k` : String(n); }

  function openDonate(c: any) {
    donating = c; donateAmount = '20'; donateMsg = '';
    donateAnon = false; donateSuccess = false; donateError = '';
  }
  function closeDonate() { donating = null; }

  async function submitDonate() {
    donateError = '';
    const amt = parseFloat(donateAmount);
    if (!amt || amt < 1) { donateError = 'Minimum donation is €1'; return; }
    if (!$authToken) { donateError = 'Sign in to donate'; return; }
    try {
      await api.donate({
        campaign_id: donating.id,
        amount: amt,
        currency: 'EUR',
        message: donateMsg || null,
        is_anonymous: donateAnon,
      });
      donateSuccess = true;
      donating.amount_raised = (donating.amount_raised || 0) + amt;
      donating.donor_count = (donating.donor_count || 0) + 1;
      donating.progress_pct = Math.min(100, Math.round((donating.amount_raised / donating.goal_amount) * 100));
    } catch (e: any) {
      donateError = e?.error || e?.message || 'Donation failed — check your wallet balance.';
    }
  }

  const PRESETS = [5, 20, 50, 100];
  $: featured = campaigns.filter(c => c.is_featured);
  $: rest = campaigns.filter(c => !c.is_featured);
</script>

<svelte:head><title>Give — Adenora</title></svelte:head>

<div class="page">
  <!-- Header -->
  <div class="page-header container">
    <div>
      <div class="section-label">Public Benefit</div>
      <h1 class="page-title"><span class="text-crimson">Give</span> — Cause Campaigns</h1>
      <p class="page-desc">Purpose lotteries, direct donations, and trading fees — all flowing to real projects on the ground.</p>
    </div>
    {#if summary}
      <div class="summary-pill">
        <span class="summary-val text-gold">{fmtEur(summary.total_raised || 0)}</span>
        <span class="summary-lbl">total raised</span>
      </div>
    {/if}
  </div>

  <!-- Impact row -->
  <div class="impact-bar container">
    <div class="impact-item"><span>📊</span><span><strong>40%</strong> of trading fees</span></div>
    <div class="impact-div"></div>
    <div class="impact-item"><span>🎟️</span><span><strong>70%</strong> of purpose lotto</span></div>
    <div class="impact-div"></div>
    <div class="impact-item"><span>💳</span><span><strong>100%</strong> of direct donations</span></div>
    <div class="impact-div"></div>
    <div class="impact-item"><span>🔍</span><span>Every cent <strong>on-ledger</strong></span></div>
  </div>

  <div class="container campaigns-body">
    {#if loading}
      <div class="skel-grid">{#each Array(4) as _}<div class="skel"></div>{/each}</div>
    {:else}
      <!-- Featured campaigns -->
      {#each featured as c}
        <div class="cf card animate-fadeup">
          <div class="cf-left">
            <div class="cf-top">
              <span class="cat-pill">{CATEGORY_ICONS[c.category] || '🌍'} {c.category.replace(/_/g,' ')}</span>
              <span class="badge badge-{c.status === 'active' ? 'teal' : 'gold'}">{c.status}</span>
            </div>
            <h2 class="cf-title">{c.title}</h2>
            <p class="cf-tagline">{c.tagline}</p>
            {#if c.impact_value}
              <div class="cf-impact">
                <span class="imp-num text-gold">{c.impact_value}</span>
                <span class="imp-lbl">{c.impact_metric}</span>
              </div>
            {/if}
            {#if c.location}
              <p class="cf-loc">📍 {c.location}{c.partner_org ? ` · ${c.partner_org}` : ''}</p>
            {/if}
          </div>
          <div class="cf-right">
            <div class="cf-prog">
              <div class="cf-prog-top">
                <span class="cf-raised text-teal">{fmtEur(c.amount_raised)}</span>
                <span class="text-muted">of {fmtEur(c.goal_amount)}</span>
              </div>
              <div class="prob-bar" style="height:10px; margin:0.75rem 0 0.4rem">
                <div class="prob-bar-fill" style="width:{c.progress_pct}%"></div>
              </div>
              <div class="cf-prog-bot">
                <span class="text-mono font-bold" class:text-gold={c.progress_pct>=75} class:text-teal={c.progress_pct<75}>{c.progress_pct}%</span>
                <span class="text-muted text-xs">👥 {fmtK(c.donor_count)} donors</span>
              </div>
            </div>
            <div class="cf-btns">
              <button class="btn btn-crimson btn-lg" on:click={() => openDonate(c)}>Donate Now</button>
              <a href="/lottery" class="btn btn-outline btn-lg">Buy a Ticket →</a>
            </div>
            <p class="cf-note">Purpose lottery available — 70% to this campaign</p>
          </div>
        </div>
      {/each}

      <!-- Rest of campaigns -->
      {#if rest.length}
        <h3 class="more-title">More Campaigns</h3>
        <div class="camp-grid">
          {#each rest as c}
            <div class="cc card animate-fadeup">
              <div class="cc-top">
                <span class="cat-pill sm">{CATEGORY_ICONS[c.category] || '🌍'} {c.category.replace(/_/g,' ')}</span>
              </div>
              <h3 class="cc-title">{c.title}</h3>
              <p class="cc-tag">{c.tagline}</p>
              <div class="prob-bar" style="height:4px; margin:0.75rem 0 0.4rem">
                <div class="prob-bar-fill" style="width:{c.progress_pct}%"></div>
              </div>
              <div class="cc-row">
                <span class="text-teal text-mono text-sm font-bold">{fmtEur(c.amount_raised)}</span>
                <span class="text-muted text-xs">{c.progress_pct}% of {fmtEur(c.goal_amount)}</span>
              </div>
              <div class="cc-footer">
                <span class="text-muted text-xs">👥 {fmtK(c.donor_count)}</span>
                <button class="btn btn-crimson btn-sm" on:click={() => openDonate(c)}>Donate</button>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Ledger -->
      {#if ledger.length > 0}
        <div class="ledger-section">
          <h3 class="more-title">Public Ledger — Recent Allocations</h3>
          <div class="ledger-table card">
            <table>
              <thead><tr><th>Date</th><th>Source</th><th>Amount</th><th>Description</th></tr></thead>
              <tbody>
                {#each ledger as e}
                  <tr>
                    <td class="text-mono text-xs text-muted">{new Date(e.created_at).toLocaleDateString()}</td>
                    <td><span class="badge badge-gold">{e.source.replace(/_/g,' ')}</span></td>
                    <td class="text-mono font-bold text-teal">€{e.amount}</td>
                    <td class="text-secondary text-sm">{e.description}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <!-- How it flows -->
  <div class="flow-section container">
    <div class="section-label">Transparency</div>
    <h2 class="section-title-sm">How Funds Flow</h2>
    <div class="flow-row">
      <div class="flow-card">
        <div class="flow-ic">🎯</div>
        <h4>You Trade or Play</h4>
        <p>Every prediction market trade, lottery ticket, and game entry generates fees.</p>
      </div>
      <div class="flow-arr">→</div>
      <div class="flow-card">
        <div class="flow-ic">⚖️</div>
        <h4>Automatic Split</h4>
        <p>40% of trading fees and 70% of purpose lottery revenue routes to campaigns instantly.</p>
      </div>
      <div class="flow-arr">→</div>
      <div class="flow-card">
        <div class="flow-ic">📋</div>
        <h4>On-Ledger Record</h4>
        <p>Every allocation is in the public charity ledger. Auditable by anyone, always.</p>
      </div>
      <div class="flow-arr">→</div>
      <div class="flow-card">
        <div class="flow-ic">🏗️</div>
        <h4>Real Projects Built</h4>
        <p>H.E.L.P., Roots of Peace, and local orgs execute on the ground. Updates posted here.</p>
      </div>
    </div>
  </div>
</div>

<!-- Donate modal -->
{#if donating}
  <div class="overlay" on:click|self={closeDonate} on:keydown={(e) => e.key === 'Escape' && closeDonate()} role="dialog" aria-modal="true">
    <div class="modal">
      <button class="modal-x" on:click={closeDonate}>✕</button>
      {#if donateSuccess}
        <div class="success-state">
          <div class="suc-ic">💚</div>
          <h3>Thank you.</h3>
          <p>Your contribution flows to <strong>{donating.title}</strong>.</p>
          <p class="text-muted text-sm">Recorded on the public ledger.</p>
          <button class="btn btn-outline" on:click={closeDonate}>Close</button>
        </div>
      {:else}
        <div class="modal-head">
          <span class="cat-pill">{CATEGORY_ICONS[donating.category] || '🌍'} {donating.category.replace(/_/g,' ')}</span>
          <h3 class="modal-title">{donating.title}</h3>
          <p class="text-muted text-sm">{fmtEur(donating.amount_raised)} raised · {donating.progress_pct}% funded</p>
          <div class="prob-bar" style="margin-top:0.5rem">
            <div class="prob-bar-fill" style="width:{donating.progress_pct}%"></div>
          </div>
        </div>
        <div class="modal-body">
          <label class="field-lbl">Amount (EUR)</label>
          <div class="presets">
            {#each PRESETS as p}
              <button class="preset" class:active={donateAmount===String(p)} on:click={() => donateAmount=String(p)}>€{p}</button>
            {/each}
          </div>
          <input class="input" type="number" min="1" placeholder="Custom amount" bind:value={donateAmount} />
          <label class="field-lbl" style="margin-top:1rem">Message (optional)</label>
          <input class="input" type="text" maxlength="200" placeholder="Leave a message..." bind:value={donateMsg} />
          <label class="chk-row">
            <input type="checkbox" bind:checked={donateAnon} />
            <span>Donate anonymously</span>
          </label>
          {#if donateError}<p class="d-err">{donateError}</p>{/if}
          <button class="btn btn-crimson btn-lg" style="width:100%;margin-top:1.25rem" on:click={submitDonate}>
            Donate {donateAmount ? `€${donateAmount}` : ''} →
          </button>
          {#if !$authToken}
            <p class="text-muted text-xs" style="text-align:center;margin-top:0.75rem">
              <a href="/register">Create account</a> or <a href="/login">sign in</a> to donate.
            </p>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
.page { padding: 2.5rem 0 5rem; }
.page-header { display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:2rem; gap:2rem; flex-wrap:wrap; }
.section-label { font-size:0.72rem; font-weight:700; letter-spacing:0.1em; text-transform:uppercase; color:var(--crimson); margin-bottom:0.5rem; }
.page-title { font-size:2rem; font-weight:900; letter-spacing:-0.03em; margin-bottom:0.5rem; }
.page-desc  { font-size:0.875rem; color:var(--text-muted); max-width:560px; line-height:1.6; }
.summary-pill { display:flex; flex-direction:column; align-items:center; padding:1.25rem 2rem; background:var(--bg-card); border:1px solid var(--border-gold); border-radius:var(--r-lg); backdrop-filter:blur(12px); flex-shrink:0; }
.summary-val { font-size:1.8rem; font-weight:900; font-family:var(--font-mono); letter-spacing:-0.03em; }
.summary-lbl { font-size:0.72rem; color:var(--text-muted); text-transform:uppercase; letter-spacing:0.08em; }

.impact-bar { display:flex; align-items:center; justify-content:center; gap:1.5rem; flex-wrap:wrap; padding:1rem 0 2rem; }
.impact-item { display:flex; align-items:center; gap:0.5rem; font-size:0.85rem; color:var(--text-secondary); }
.impact-item strong { color:var(--text-primary); }
.impact-div { width:1px; height:18px; background:var(--border); }

/* Featured */
.cf { display:grid; grid-template-columns:1fr 1fr; gap:3rem; padding:2.5rem; margin-bottom:1.5rem; border-color:var(--border-crimson); }
.cf:hover { border-color:rgba(232,50,74,0.5); box-shadow:0 8px 32px rgba(232,50,74,0.12); }
.cf-top { display:flex; align-items:center; gap:0.75rem; margin-bottom:1rem; }
.cf-title { font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:0.5rem; }
.cf-tagline { font-size:0.9rem; color:var(--text-secondary); line-height:1.6; margin-bottom:1.25rem; }
.cf-impact { display:flex; align-items:baseline; gap:0.5rem; margin-bottom:0.75rem; }
.imp-num { font-size:2.5rem; font-weight:900; font-family:var(--font-mono); letter-spacing:-0.04em; }
.imp-lbl { font-size:0.85rem; color:var(--text-muted); }
.cf-loc { font-size:0.8rem; color:var(--text-muted); }
.cf-prog { margin-bottom:2rem; }
.cf-prog-top { display:flex; align-items:baseline; gap:0.5rem; }
.cf-raised { font-size:1.6rem; font-weight:900; font-family:var(--font-mono); letter-spacing:-0.03em; }
.cf-prog-bot { display:flex; justify-content:space-between; }
.cf-btns { display:flex; gap:0.75rem; flex-wrap:wrap; margin-bottom:0.75rem; }
.cf-note { font-size:0.78rem; color:var(--text-muted); }

.cat-pill { display:inline-flex; align-items:center; gap:0.3rem; padding:3px 10px; border-radius:var(--r-full); background:rgba(232,50,74,0.08); border:1px solid rgba(232,50,74,0.15); color:var(--crimson); font-size:0.72rem; font-weight:700; text-transform:uppercase; letter-spacing:0.04em; }
.cat-pill.sm { font-size:0.68rem; padding:2px 8px; }

.more-title { font-size:1.1rem; font-weight:700; margin:2.5rem 0 1rem; color:var(--text-secondary); }
.camp-grid { display:grid; grid-template-columns:repeat(3,1fr); gap:1.25rem; }
.cc { padding:1.5rem; display:flex; flex-direction:column; gap:0.6rem; }
.cc:hover { border-color:var(--border-crimson); }
.cc-top { display:flex; justify-content:space-between; }
.cc-title { font-size:0.95rem; font-weight:700; line-height:1.35; }
.cc-tag { font-size:0.8rem; color:var(--text-muted); line-height:1.5; flex:1; }
.cc-row { display:flex; justify-content:space-between; }
.cc-footer { display:flex; justify-content:space-between; align-items:center; margin-top:auto; padding-top:0.75rem; border-top:1px solid var(--border-subtle); }

.skel-grid { display:grid; grid-template-columns:repeat(2,1fr); gap:1.25rem; }
.skel { height:260px; border-radius:var(--r-lg); background:linear-gradient(90deg,var(--bg-raised) 0%,var(--bg-overlay) 50%,var(--bg-raised) 100%); background-size:200%; animation:shimmer 1.5s infinite; }

.ledger-section { margin-top:2.5rem; }
.ledger-table { padding:0; overflow:hidden; }
table { width:100%; border-collapse:collapse; }
th { text-align:left; padding:0.75rem 1rem; font-size:0.72rem; color:var(--text-muted); border-bottom:1px solid var(--border); text-transform:uppercase; letter-spacing:0.05em; }
td { padding:0.75rem 1rem; font-size:0.85rem; border-bottom:1px solid var(--border-subtle); }

.flow-section { padding:4rem 0 2rem; }
.section-title-sm { font-size:1.5rem; font-weight:800; letter-spacing:-0.02em; margin-bottom:2rem; }
.flow-row { display:flex; align-items:center; gap:0; }
.flow-card { flex:1; padding:1.5rem; background:var(--bg-card); border:1px solid var(--border); border-radius:var(--r-lg); transition:all var(--t-med); }
.flow-card:hover { border-color:rgba(232,50,74,0.2); transform:translateY(-2px); }
.flow-ic { font-size:1.75rem; margin-bottom:0.75rem; }
.flow-card h4 { font-size:0.9rem; font-weight:700; margin-bottom:0.5rem; }
.flow-card p  { font-size:0.8rem; color:var(--text-muted); line-height:1.5; }
.flow-arr { padding:0 0.5rem; color:var(--text-muted); font-size:1.25rem; flex-shrink:0; }

/* Modal */
.overlay { position:fixed; inset:0; z-index:9999; background:rgba(0,0,0,0.7); backdrop-filter:blur(8px); display:flex; align-items:center; justify-content:center; padding:1rem; animation:fadeIn 0.2s ease; }
.modal { background:var(--bg-raised); border:1px solid var(--border-crimson); border-radius:var(--r-xl); padding:2rem; width:100%; max-width:480px; position:relative; box-shadow:0 24px 80px rgba(0,0,0,0.6),0 0 40px rgba(232,50,74,0.1); animation:fadeUp 0.3s var(--ease-out); }
.modal-x { position:absolute; top:1rem; right:1rem; background:var(--bg-glass); border:1px solid var(--border); color:var(--text-muted); border-radius:var(--r-full); width:28px; height:28px; display:flex; align-items:center; justify-content:center; font-size:0.75rem; cursor:pointer; transition:all var(--t-fast); }
.modal-x:hover { color:var(--text-primary); }
.modal-head { margin-bottom:1.5rem; }
.modal-title { font-size:1.2rem; font-weight:800; margin:0.5rem 0 0.25rem; }
.modal-body { display:flex; flex-direction:column; }
.field-lbl { font-size:0.78rem; font-weight:700; color:var(--text-secondary); margin-bottom:0.4rem; text-transform:uppercase; letter-spacing:0.05em; }
.presets { display:flex; gap:0.5rem; margin-bottom:0.75rem; }
.preset { flex:1; padding:0.5rem; background:var(--bg-glass); border:1px solid var(--border); border-radius:var(--r-md); color:var(--text-secondary); font-size:0.85rem; font-weight:600; cursor:pointer; transition:all var(--t-fast); }
.preset:hover { border-color:var(--crimson); color:var(--text-primary); }
.preset.active { background:rgba(232,50,74,0.15); border-color:var(--crimson); color:var(--crimson); }
.chk-row { display:flex; align-items:center; gap:0.5rem; font-size:0.85rem; color:var(--text-secondary); cursor:pointer; margin-top:0.75rem; }
.d-err { color:var(--crimson); font-size:0.82rem; margin-top:0.75rem; padding:0.5rem 0.75rem; background:rgba(232,50,74,0.08); border-radius:var(--r-sm); border:1px solid rgba(232,50,74,0.2); }
.success-state { text-align:center; padding:1rem 0; display:flex; flex-direction:column; align-items:center; gap:0.75rem; }
.suc-ic { font-size:3rem; }
.success-state h3 { font-size:1.5rem; font-weight:900; }

@media (max-width:1024px) {
  .cf { grid-template-columns:1fr; gap:1.5rem; }
  .camp-grid { grid-template-columns:repeat(2,1fr); }
  .flow-row { flex-direction:column; }
  .flow-arr { transform:rotate(90deg); }
}
@media (max-width:640px) { .camp-grid { grid-template-columns:1fr; } .skel-grid { grid-template-columns:1fr; } }
</style>
