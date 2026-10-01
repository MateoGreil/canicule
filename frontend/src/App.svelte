<script lang="ts">
  import { onMount } from 'svelte'
  import { Map, NavigationControl } from 'maplibre-gl'
  import { BASEMAPS, type BasemapId } from './lib/basemap'
  import { viewState } from './lib/state.svelte'
  import { FALLBACK_CONFIG, fetchConfig, showResolutionBadge, type AppConfig } from './lib/config'
  import { addHeatmapLayer, updateHeatmapTiles } from './lib/map'
  import { DEBOUNCE_MS, formatHash, parseHash } from './lib/urlHash'
  import ControlPanel from './components/ControlPanel.svelte'
  import ResolutionBadge from './components/ResolutionBadge.svelte'
  import AboutModal from './components/AboutModal.svelte'

  let container: HTMLDivElement
  let map: Map | undefined
  let config = $state<AppConfig>(FALLBACK_CONFIG)
  let configLoaded = false
  let styleReady = false
  let aboutOpen = $state(false)
  let locationError = $state('')
  let hashTimer: ReturnType<typeof setTimeout> | undefined

  function locate() {
    if (!navigator.geolocation) {
      locationError = 'Geolocation is not available in this browser.'
      return
    }
    navigator.geolocation.getCurrentPosition(
      (position) => {
        locationError = ''
        map?.flyTo({ center: [position.coords.longitude, position.coords.latitude], zoom: Math.max(map.getZoom(), 13) })
      },
      () => { locationError = 'Could not get your location. Check browser permissions.' },
      { timeout: 10_000 },
    )
  }

  function scheduleHashSync() {
    clearTimeout(hashTimer)
    hashTimer = setTimeout(() => {
      if (!map) return
      const center = map.getCenter()
      history.replaceState(null, '', formatHash({
        z: map.getZoom(), lat: center.lat, lng: center.lng,
        sport: viewState.sport, color: viewState.color,
      }))
    }, DEBOUNCE_MS)
  }

  function basemapStyle(id: BasemapId): string {
    return BASEMAPS[id].styleUrl
  }

  function switchBasemap(id: BasemapId) {
    styleReady = false
    viewState.basemap = id
    map?.setStyle(basemapStyle(id))
  }

  $effect(() => {
    const opacity = viewState.opacity
    if (map?.getLayer('heatmap')) map.setPaintProperty('heatmap', 'raster-opacity', opacity)
  })

  $effect(() => {
    const sport = viewState.sport
    const color = viewState.color
    if (map?.getSource('heatmap')) updateHeatmapTiles(map, sport, color)
    if (map) scheduleHashSync()
  })

  onMount(() => {
    const initial = parseHash(location.hash)
    if (initial) {
      viewState.sport = initial.sport
      viewState.color = initial.color
    }
    map = new Map({
      container,
      style: basemapStyle(viewState.basemap),
      center: initial ? [initial.lng, initial.lat] : [2.35, 48.85],
      zoom: initial?.z ?? 10,
    })
    map.on('moveend', scheduleHashSync)
    map.addControl(new NavigationControl(), 'top-right')
    map.on('style.load', () => {
      styleReady = true
      if (map && configLoaded) addHeatmapLayer(map, config, viewState)
    })
    fetchConfig().then((loaded) => {
      config = loaded
      configLoaded = true
      if (map && styleReady) addHeatmapLayer(map, config, viewState)
    })
    return () => {
      clearTimeout(hashTimer)
      map?.remove()
    }
  })
</script>

<div class="map" bind:this={container}></div>
<ControlPanel {config} view={viewState} />
<div class="app-actions">
  <button type="button" onclick={locate} aria-label="Locate me">⌖ Locate me</button>
  <button type="button" onclick={() => aboutOpen = true}>About</button>
</div>
{#if locationError}<p class="location-error" role="alert">{locationError}</p>{/if}
{#if showResolutionBadge(config)}
  <ResolutionBadge onabout={() => aboutOpen = true} />
{/if}
{#if aboutOpen}
  <AboutModal onclose={() => aboutOpen = false} />
{/if}
<div class="basemap-switch" aria-label="Basemap">
  {#each Object.entries(BASEMAPS) as [id, basemap]}
    <button
      type="button"
      class:active={viewState.basemap === id}
      onclick={() => switchBasemap(id as BasemapId)}
    >{basemap.name}</button>
  {/each}
</div>
