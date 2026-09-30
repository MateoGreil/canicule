import { describe, expect, it } from 'vitest'
import { DEBOUNCE_MS, formatHash, parseHash } from '../src/lib/urlHash'

describe('map URL hash', () => {
  it('round-trips location and layer settings', () => {
    const hash = '#11/48.85/2.35/ride/hot'
    expect(formatHash(parseHash(hash)!)).toBe(hash)
    expect(parseHash(hash)).toEqual({ z: 11, lat: 48.85, lng: 2.35, sport: 'ride', color: 'hot' })
    expect(DEBOUNCE_MS).toBe(300)
  })

  it.each([
    '#11/48.85', '#abc', '#11/99/2.35/ride/hot',
    '#11/48.85/2.35/swim/hot', '#11/48.85/2.35/ride/unknown',
    '#11/48.85/200/ride/hot', '#42/48.85/2.35/ride/hot',
  ])('rejects malformed location %s', (hash) => {
    expect(parseHash(hash)).toBeNull()
  })

  it('rounds moving-map coordinates to stable, shareable values', () => {
    expect(formatHash({ z: 11.123, lat: 48.856611, lng: 2.352222, sport: 'all', color: 'purple' }))
      .toBe('#11.12/48.85661/2.35222/all/purple')
  })
})
