<script lang="ts">
  import type { AppConfig } from '../lib/config'
  import { BASEMAPS } from '../lib/basemap'
  import type { ViewState } from '../lib/state.svelte'

  let { config, view }: { config: AppConfig; view: ViewState } = $props()
  let expanded = $state(true)

  const sportNames: Record<string, string> = {
    all: 'All activities', ride: 'Ride', run: 'Run', water: 'Water', winter: 'Winter',
  }
  const colorNames: Record<string, string> = {
    hot: 'Hot', mobileblue: 'Blue', gray: 'Gray', bluered: 'Blue / Red', purple: 'Purple', orange: 'Orange',
  }
</script>

<section class="panel" class:dark={BASEMAPS[view.basemap].theme === 'dark'} aria-label="Heatmap controls">
  <header>
    <h1>canicule <span aria-hidden="true">🔥</span></h1>
    <button class="toggle" type="button" aria-label="Toggle controls" aria-expanded={expanded} onclick={() => expanded = !expanded}>{expanded ? '−' : '+'}</button>
  </header>
  {#if expanded}
    <label for="sport">Sport</label>
    <select id="sport" bind:value={view.sport}>
      {#each config.sports as sport}
        <option value={sport}>{sportNames[sport] ?? sport}</option>
      {/each}
    </select>
    <label for="color">Color</label>
    <select id="color" bind:value={view.color}>
      {#each config.colors as color}
        <option value={color}>{colorNames[color] ?? color}</option>
      {/each}
    </select>
    <label for="opacity">Opacity · {Math.round(view.opacity * 100)}%</label>
    <input id="opacity" type="range" min="0" max="1" step="0.05" bind:value={view.opacity} />
  {/if}
</section>

<style>
  .panel { position: absolute; top: 12px; left: 12px; width: 210px; padding: 16px; background: #fff; color: #222; border-radius: 8px; box-shadow: 0 2px 12px #0004; display: grid; gap: 8px; }
  .panel.dark { background: #222; color: #fff; }
  header { display: flex; align-items: center; justify-content: space-between; }
  h1 { margin: 0; font-size: 1.2rem; }
  .toggle { border: 0; background: transparent; color: inherit; font-size: 1.4rem; cursor: pointer; }
  label { font-size: 0.82rem; font-weight: 600; }
  select { padding: 7px; border: 1px solid #bbb; border-radius: 4px; background: white; }
  input[type='range'] { width: 100%; }
  @media (max-width: 640px) {
    .panel { position: fixed; top: auto; bottom: 0; left: 0; right: 0; width: auto; border-radius: 12px 12px 0 0; padding: 10px 16px; max-height: 60vh; overflow: auto; }
  }
</style>
