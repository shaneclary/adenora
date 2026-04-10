<script lang="ts">
  export let market: any;
  export let onPredict: (market: any, side: string) => void = () => {};

  $: chance = market.chance_yes_pct ?? 50;
  $: timeLeft = market.time_left ?? '';
  $: charity = market.charity_project ?? '';

  function categoryIcon(cat: string) {
    const icons: Record<string, string> = {
      geopolitics: 'G', economy: 'E', sports: 'S',
      eu_policy: 'EU', tech: 'T', weather: 'W',
      entertainment: 'E', politics: 'P', finance: 'F',
    };
    return icons[cat?.toLowerCase()] || '?';
  }
</script>

<div class="opinion-card">
  <div class="oc-header">
    <span class="oc-cat">{categoryIcon(market.category)}</span>
    <span class="oc-time">{timeLeft}</span>
  </div>

  <h3 class="oc-question">{market.question}</h3>

  <div class="oc-bar-wrap">
    <div class="oc-bar">
      <div class="oc-fill-yes" style="width:{chance}%"></div>
    </div>
    <div class="oc-bar-labels">
      <span class="oc-yes-pct">{chance}% YES</span>
      <span class="oc-no-pct">{100 - chance}% NO</span>
    </div>
  </div>

  {#if charity}
    <div class="oc-charity">Fees fund <strong>{charity}</strong></div>
  {/if}

  <div class="oc-actions">
    <button class="oc-btn oc-btn-yes" on:click={() => onPredict(market, 'yes')}>YES</button>
    <button class="oc-btn oc-btn-no" on:click={() => onPredict(market, 'no')}>NO</button>
  </div>
</div>

<style>
  .opinion-card {
    background: var(--bg-glass); border: 1px solid var(--border);
    backdrop-filter: blur(12px); border-radius: 16px;
    padding: 1.25rem; transition: border-color 0.2s, transform 0.2s;
  }
  .opinion-card:hover { border-color: rgba(255,255,255,0.12); transform: translateY(-2px); }

  .oc-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
  .oc-cat {
    width: 28px; height: 28px; border-radius: 8px;
    background: rgba(240,180,41,0.15); color: var(--gold);
    display: flex; align-items: center; justify-content: center;
    font-size: 0.7rem; font-weight: 800;
  }
  .oc-time { font-size: 0.72rem; color: var(--text-muted); }

  .oc-question { font-size: 0.95rem; font-weight: 600; line-height: 1.4; margin: 0 0 1rem; color: var(--text-primary); }

  .oc-bar-wrap { margin-bottom: 0.75rem; }
  .oc-bar { height: 6px; background: rgba(232,50,74,0.2); border-radius: 3px; overflow: hidden; }
  .oc-fill-yes { height: 100%; background: var(--teal); border-radius: 3px; transition: width 0.4s; }
  .oc-bar-labels { display: flex; justify-content: space-between; margin-top: 0.35rem; }
  .oc-yes-pct { font-size: 0.75rem; font-weight: 700; color: var(--teal); font-family: var(--font-mono); }
  .oc-no-pct { font-size: 0.75rem; font-weight: 700; color: var(--crimson); font-family: var(--font-mono); }

  .oc-charity { font-size: 0.72rem; color: var(--text-muted); margin-bottom: 0.75rem; }
  .oc-charity strong { color: var(--crimson); }

  .oc-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; }
  .oc-btn {
    padding: 0.7rem; border-radius: 10px; border: none;
    font-size: 0.9rem; font-weight: 800; letter-spacing: 0.05em;
    cursor: pointer; transition: all 0.15s;
  }
  .oc-btn-yes { background: rgba(0,194,224,0.15); color: var(--teal); border: 1px solid rgba(0,194,224,0.3); }
  .oc-btn-yes:hover { background: rgba(0,194,224,0.3); box-shadow: 0 0 20px rgba(0,194,224,0.2); }
  .oc-btn-no { background: rgba(232,50,74,0.12); color: var(--crimson); border: 1px solid rgba(232,50,74,0.3); }
  .oc-btn-no:hover { background: rgba(232,50,74,0.25); box-shadow: 0 0 20px rgba(232,50,74,0.2); }
</style>
