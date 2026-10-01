import { describe, expect, it } from 'vitest'
import { BASEMAPS } from '../src/lib/basemap'
import { ViewState } from '../src/lib/state.svelte'

describe('OpenFreeMap basemaps', () => {
  it('provides Liberty, Light, and Bright style URLs', () => {
    expect(BASEMAPS.liberty.styleUrl).toBe('https://tiles.openfreemap.org/styles/liberty')
    expect(BASEMAPS.positron.styleUrl).toBe('https://tiles.openfreemap.org/styles/positron')
    expect(BASEMAPS.bright.styleUrl).toBe('https://tiles.openfreemap.org/styles/bright')
    expect(Object.keys(BASEMAPS).sort()).toEqual(['bright', 'liberty', 'positron'])
  })

  it('starts with a useful default map state', () => {
    const state = new ViewState()
    expect(state.sport).toBe('all')
    expect(state.color).toBe('hot')
    expect(state.opacity).toBe(0.75)
    expect(state.basemap).toBe('liberty')
  })
})
