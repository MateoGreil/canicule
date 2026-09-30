<script lang="ts">
  let { onabout }: { onabout: () => void } = $props()
</script>

<div class="badge">
  <span>Low resolution · zoom 11 max</span>
  <button type="button" aria-label="About resolution" onclick={onabout}>ⓘ</button>
</div>

<style>
  .badge { position: absolute; left: 12px; bottom: 28px; display: flex; gap: 6px; align-items: center; background: #fff; color: #222; padding: 5px 9px; border-radius: 5px; box-shadow: 0 2px 8px #0003; font-size: 0.75rem; }
  button { border: 0; background: transparent; cursor: pointer; font-size: 1rem; padding: 0; }
  @media (max-width: 640px) { .badge { bottom: 82px; } }
</style>
