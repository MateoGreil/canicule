import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  use: {
    baseURL: 'http://127.0.0.1:15174',
    browserName: 'chromium',
    channel: 'chrome',
    launchOptions: { args: ['--enable-webgl', '--use-gl=angle', '--use-angle=swiftshader'] },
  },
  webServer: {
    command: 'npm run preview -- --host 127.0.0.1 --port 15174',
    url: 'http://127.0.0.1:15174',
    reuseExistingServer: false,
  },
})
