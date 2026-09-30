import { FALLBACK_CONFIG } from './config'

export interface HashView {
  z: number
  lat: number
  lng: number
  sport: string
  color: string
}

export const DEBOUNCE_MS = 300

export function parseHash(hash: string): HashView | null {
  const parts = hash.replace(/^#/, '').split('/')
  if (parts.length !== 5 || parts.slice(0, 3).some((value) => value.trim() === '')) return null
  const [z, lat, lng] = parts.slice(0, 3).map(Number)
  const [sport, color] = parts.slice(3)
  if (
    !Number.isFinite(z) || z < 0 || z > 22 ||
    !Number.isFinite(lat) || lat < -85.0511 || lat > 85.0511 ||
    !Number.isFinite(lng) || lng < -180 || lng > 180 ||
    !FALLBACK_CONFIG.sports.includes(sport) || !FALLBACK_CONFIG.colors.includes(color)
  ) return null
  return { z, lat, lng, sport, color }
}

export function formatHash(view: HashView): string {
  const short = (n: number, digits: number) => String(Number(n.toFixed(digits)))
  return `#${short(view.z, 2)}/${short(view.lat, 5)}/${short(view.lng, 5)}/${view.sport}/${view.color}`
}
