import type { BasemapId } from './basemap'

export class ViewState {
  sport = $state('all')
  color = $state('hot')
  opacity = $state(0.75)
  basemap = $state<BasemapId>('liberty')
}

export const viewState = new ViewState()
