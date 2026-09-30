<script lang="ts">
  import { onMount } from 'svelte'
  let { onclose }: { onclose: () => void } = $props()
  let dialog: HTMLDialogElement

  onMount(() => {
    dialog.showModal()
  })
</script>

<dialog bind:this={dialog} onclose={onclose} aria-label="About canicule">
  <div class="content">
    <header>
      <h2>About canicule</h2>
      <button type="button" aria-label="Close" onclick={() => dialog.close()}>×</button>
    </header>
    <p>canicule is a self-hostable viewer for Strava's Global Heatmap. You can explore the public heatmap without a Strava account.</p>
    <p>Anonymous mode shows the heatmap up to zoom 11. Self-hosters can set <code>STRAVA_SESSION_COOKIE</code> on the server to enable high-resolution tiles up to zoom 15. The session cookie is never sent to visitors.</p>
    <p>Map data © OpenStreetMap contributors · basemap © CARTO · heatmap © Strava. Please respect Strava's terms of use: no bulk scraping, personal and small-scale use only.</p>
    <p><a href="https://git.greil.fr/mat/canicule" target="_blank" rel="noopener noreferrer">Source code and setup instructions</a></p>
  </div>
</dialog>

<style>
  dialog { border: 0; border-radius: 10px; box-shadow: 0 10px 40px #0006; width: min(440px, calc(100vw - 36px)); padding: 0; color: #222; }
  dialog::backdrop { background: #0009; }
  .content { padding: 24px; }
  header { display: flex; justify-content: space-between; align-items: center; }
  h2 { margin: 0; }
  button { border: 0; background: transparent; cursor: pointer; font-size: 1.8rem; }
  p { line-height: 1.5; }
</style>
