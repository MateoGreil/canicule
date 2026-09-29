# canicule 🔥

An alternative frontend to browse [Strava's Global Heatmap](https://www.strava.com/maps/global-heatmap) **without a Strava account**.

- **Rust proxy** (axum): serves the embedded frontend and caches heatmap tiles
- **Svelte 5 + MapLibre GL frontend**: sport selection, 6 color palettes, opacity control, light/dark basemaps
- **Zero config**: works out of the box in low resolution (zoom ≤ 11), no account needed
- **With `STRAVA_SESSION_COOKIE` set**: high resolution (zoom 15), CloudFront cookies refreshed automatically

> [!NOTE]
> Work in progress — see the design spec in `docs/superpowers/specs/`.
