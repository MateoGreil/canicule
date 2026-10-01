export type BasemapId = 'liberty' | 'positron' | 'bright'

type Basemap = {
  name: string
  styleUrl: string
  theme: 'light' | 'dark' | 'mixed'
}

export const BASEMAPS: Record<BasemapId, Basemap> = {
  liberty: { name: 'Liberty', styleUrl: 'https://tiles.openfreemap.org/styles/liberty', theme: 'mixed' },
  positron: { name: 'Light', styleUrl: 'https://tiles.openfreemap.org/styles/positron', theme: 'light' },
  bright: { name: 'Bright', styleUrl: 'https://tiles.openfreemap.org/styles/bright', theme: 'light' },
}
