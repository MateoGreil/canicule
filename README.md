# canicule 🔥

**Explore [Strava's Global Heatmap](https://www.strava.com/maps/global-heatmap) without a Strava account.** canicule is a self-hostable, single-binary map viewer with a Rust tile proxy and a Svelte + MapLibre frontend. It works anonymously at zoom levels up to 11; an optional server-side Strava session cookie unlocks high-resolution tiles up to zoom 15. Visitors never receive that cookie.

This is an unofficial project, not affiliated with Strava.

## Quick start

```bash
git clone ssh://git@git.greil.fr/mat/canicule.git
cd canicule
docker compose up -d --build
```

Open **http://localhost:8080**. No account or configuration is needed for low-resolution tiles. Change the host port with `CANICULE_PORT=3000 docker compose up -d --build`.

To enable high resolution, sign in to Strava in your browser, open developer tools → **Application / Storage → Cookies → www.strava.com**, and copy the value of `_strava4_session`. Keep it secret: it grants access to your Strava session.

```bash
STRAVA_SESSION_COOKIE='your-session-cookie' docker compose up -d
```

For a persistent installation, put `STRAVA_SESSION_COOKIE=...` in a local `.env` file (ignored by Git) and run `docker compose up -d`. The proxy obtains short-lived CloudFront cookies from that session and refreshes them automatically. If the Strava session expires, copy a new one. Never put the cookie in a committed file or a public issue.

To run the binary without Docker:

```bash
make build
CACHE_DIR=./cache ./target/release/canicule
```

## Map controls

- Choose **All, Ride, Run, Water, or Winter** activities.
- Pick one of six heatmap palettes: **Hot, Blue, Gray, Blue / Red, Purple, Orange**.
- Adjust heatmap opacity; switch between **Voyager, Light, and Dark** Carto basemaps.
- Use browser geolocation to center the map on your position.
- Copy the URL to preserve map position, zoom, sport, and color. The heatmap remains visible beyond its native maximum zoom by overzooming existing tiles; no new detail is added.

## Configuration

| Variable | Default | Description |
|---|---:|---|
| `PORT` | `8080` | HTTP listening port inside the container |
| `STRAVA_SESSION_COOKIE` | unset | `_strava4_session` value; enables high-resolution mode |
| `CACHE_DIR` | unset | Persistent tile cache directory; Compose sets `/var/cache/canicule` in a named volume |
| `CACHE_TTL_DAYS` | `14` | Tile freshness period in days |
| `CACHE_MAX_ENTRIES` | `10000` | In-memory LRU capacity (tiles) |
| `UPSTREAM_TIMEOUT_SECS` | `10` | Timeout for requests to Strava |
| `CANICULE_PORT` | `8080` | Host port in Docker Compose only |

The cache keeps tiles fresh for **14 days** to avoid repeatedly fetching unchanged heatmap data. Empty tiles have a one-hour negative cache. Concurrent requests for the same tile share one upstream fetch; no more than eight different upstream requests run at once. With `CACHE_DIR` set, an expired tile is served as a fallback when Strava is unavailable.

The Rust server serves both the frontend and `/api/tiles/{sport}/{color}/{z}/{x}/{y}.png` from the same origin. No CORS configuration or browser-side Strava credentials are needed.

## Development

Requires Node.js 22+, Rust (stable), and optionally Docker and Google Chrome for browser tests.

```bash
npm --prefix frontend ci
make dev     # Vite :5173 + Rust proxy :8080
make test    # cargo tests, Vitest, typecheck, Playwright
make build   # target/release/canicule with embedded frontend
```

In environments where port 8080 is occupied, run `PORT=18080 VITE_PROXY_TARGET=http://127.0.0.1:18080 make dev`.

## Verification checklist

- [ ] `docker compose up -d --build` loads a map at http://localhost:8080 without `STRAVA_SESSION_COOKIE`.
- [ ] `/api/config` reports `authenticated: false`, `maxZoom: 11`; a real anonymous tile at `/api/tiles/all/hot/6/32/21.png` returns PNG.
- [ ] A second request for the same tile returns `X-Cache: hit`; zoom 12 in anonymous mode is rejected locally.
- [ ] Five sports, six colors, opacity, three basemaps, geolocation, mobile controls, and URL hash restore work in the browser.
- [ ] `make test` passes Rust, Vitest, typechecking, and Playwright; `make build` produces a binary serving the actual frontend bundle.
- [ ] If a valid `STRAVA_SESSION_COOKIE` is available: `/api/config` reports `authenticated: true`, `maxZoom: 15`, and a high-resolution tile loads. **This live authentication path has not yet been verified**; the automated cookie-flow tests use a mock Strava server.

## Attribution and fair use

Heatmap data © Strava · basemaps © CARTO · map data © OpenStreetMap contributors. Attribution is also shown on the map.

Please respect Strava's terms of use. Do not bulk-scrape tiles. This tool is intended for personal and small-scale use; a public instance needs its own legal and rate-limit assessment. Strava can change its tile authentication or block access at any time. The cache reduces requests but does not guarantee availability.
