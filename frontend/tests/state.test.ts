import { describe, expect, it } from 'vitest'
import { ViewState } from '../src/lib/state.svelte'
import { heatmapTileUrl } from '../src/lib/map'

describe('heatmap controls', () => {
  it('builds a MapLibre tile URL for the selected sport and color', () => {
    expect(heatmapTileUrl('ride', 'hot')).toBe('/api/tiles/ride/hot/{z}/{x}/{y}.png')
  })

  it('updates the selected sport in state', () => {
    const state = new ViewState()
    state.sport = 'winter'
    expect(state.sport).toBe('winter')
  })
})
