export type BasemapId = 'voyager' | 'positron' | 'dark_matter'

type Basemap = {
  name: string
  urls: string[]
  theme: 'light' | 'dark' | 'mixed'
}

function urls(id: BasemapId): string[] {
  return ['a', 'b', 'c'].map((subdomain) =>
    `https://${subdomain}.basemaps.cartocdn.com/${id}/{z}/{x}/{y}.png`,
  )
}

export const BASEMAPS: Record<BasemapId, Basemap> = {
  voyager: { name: 'Voyager', urls: urls('voyager'), theme: 'mixed' },
  positron: { name: 'Light', urls: urls('positron'), theme: 'light' },
  dark_matter: { name: 'Dark', urls: urls('dark_matter'), theme: 'dark' },
}
