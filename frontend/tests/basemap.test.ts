import { describe, expect, it } from 'vitest'
import { BASEMAPS } from '../src/lib/basemap'
import { ViewState } from '../src/lib/state.svelte'

describe('Carto basemaps', () => {
  it('provides Voyager, Positron, and Dark Matter tiles', () => {
    expect(BASEMAPS.voyager.urls[0]).toBe('https://a.basemaps.cartocdn.com/voyager/{z}/{x}/{y}.png')
    expect(Object.keys(BASEMAPS).sort()).toEqual(['dark_matter', 'positron', 'voyager'])
    expect(BASEMAPS.dark_matter.theme).toBe('dark')
  })

  it('starts with a useful default map state', () => {
    const state = new ViewState()
    expect(state.sport).toBe('all')
    expect(state.color).toBe('hot')
    expect(state.opacity).toBe(0.75)
    expect(state.basemap).toBe('voyager')
  })
})
