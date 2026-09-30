export interface AppConfig {
  authenticated: boolean
  maxZoom: number
  sports: string[]
  colors: string[]
}

export const FALLBACK_CONFIG: AppConfig = {
  authenticated: false,
  maxZoom: 11,
  sports: ['all', 'ride', 'run', 'water', 'winter'],
  colors: ['hot', 'mobileblue', 'gray', 'bluered', 'purple', 'orange'],
}

export function showResolutionBadge(config: AppConfig): boolean {
  return !config.authenticated
}

export async function fetchConfig(): Promise<AppConfig> {
  try {
    const response = await fetch('/api/config')
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const config: unknown = await response.json()
    if (
      typeof config !== 'object' || config === null ||
      !('authenticated' in config) || typeof config.authenticated !== 'boolean' ||
      !('maxZoom' in config) || typeof config.maxZoom !== 'number' ||
      !('sports' in config) || !Array.isArray(config.sports) ||
      !('colors' in config) || !Array.isArray(config.colors)
    ) throw new Error('Invalid config response')
    return config as AppConfig
  } catch {
    return FALLBACK_CONFIG
  }
}
