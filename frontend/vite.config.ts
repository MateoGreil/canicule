import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: { '/api': process.env.VITE_PROXY_TARGET ?? 'http://127.0.0.1:8080' },
  },
  test: {
    exclude: ['e2e/**', '**/node_modules/**'],
  },
})
