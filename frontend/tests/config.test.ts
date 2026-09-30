import { afterEach, describe, expect, it, vi } from 'vitest'
import { FALLBACK_CONFIG, fetchConfig, showResolutionBadge } from '../src/lib/config'

afterEach(() => vi.unstubAllGlobals())

describe('resolution badge', () => {
  it('is visible in anonymous mode and hidden in authenticated mode', () => {
    expect(showResolutionBadge(FALLBACK_CONFIG)).toBe(true)
    expect(showResolutionBadge({ ...FALLBACK_CONFIG, authenticated: true, maxZoom: 15 })).toBe(false)
  })
})

describe('fetchConfig', () => {
  it('falls back to anonymous mode when the proxy is unreachable', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')))
    expect(await fetchConfig()).toEqual(FALLBACK_CONFIG)
    expect(FALLBACK_CONFIG.maxZoom).toBe(11)
    expect(FALLBACK_CONFIG.authenticated).toBe(false)
  })

  it('returns the authenticated config from the proxy', async () => {
    const remote = { authenticated: true, maxZoom: 15, sports: ['all'], colors: ['hot'] }
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => remote }))
    expect(await fetchConfig()).toEqual(remote)
  })

  it('falls back on a non-successful response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }))
    expect(await fetchConfig()).toEqual(FALLBACK_CONFIG)
  })
})
