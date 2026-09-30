import type { Map, RasterTileSource } from 'maplibre-gl'
import type { AppConfig } from './config'
import type { ViewState } from './state.svelte'

export function heatmapTileUrl(sport: string, color: string): string {
  return `/api/tiles/${sport}/${color}/{z}/{x}/{y}.png`
}

export function addHeatmapLayer(map: Map, config: AppConfig, view: ViewState): void {
  if (map.getLayer('heatmap')) map.removeLayer('heatmap')
  if (map.getSource('heatmap')) map.removeSource('heatmap')
  map.addSource('heatmap', {
    type: 'raster',
    tiles: [heatmapTileUrl(view.sport, view.color)],
    tileSize: 256,
    maxzoom: config.maxZoom,
    attribution: 'Heatmap © Strava',
  })
  map.addLayer({
    id: 'heatmap',
    type: 'raster',
    source: 'heatmap',
    paint: { 'raster-opacity': view.opacity },
  })
}

export function updateHeatmapTiles(map: Map, sport: string, color: string): void {
  map.getSource<RasterTileSource>('heatmap')?.setTiles([heatmapTileUrl(sport, color)])
}
