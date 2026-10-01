import { expect, test } from '@playwright/test'

const tile = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/ZV8AAAAASUVORK5CYII=', 'base64')

async function mockApi(page: import('@playwright/test').Page) {
  await page.route('**/api/config', (route) => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      authenticated: false, maxZoom: 11,
      sports: ['all', 'ride', 'run', 'water', 'winter'],
      colors: ['hot', 'mobileblue', 'gray', 'bluered', 'purple', 'orange'],
    }),
  }))
  await page.route('**/api/tiles/**', (route) => route.fulfill({ status: 200, contentType: 'image/png', body: tile }))
}

test('the map renders and changing sport updates tile requests and URL hash', async ({ page }) => {
  await mockApi(page)
  await page.goto('/')
  await expect(page.locator('canvas.maplibregl-canvas')).toBeVisible()
  const mapBox = await page.locator('.map').boundingBox()
  expect(mapBox?.height).toBe(page.viewportSize()?.height)
  await expect(page.getByText('Low resolution · zoom 11 max')).toBeVisible()
  const nextTile = page.waitForRequest((request) => request.url().includes('/api/tiles/run/'))
  await page.getByLabel('Sport').selectOption('run')
  await nextTile
  await expect.poll(() => page.evaluate(() => location.hash)).toContain('/run/')
  await page.getByRole('button', { name: 'About', exact: true }).click()
  await expect(page.getByRole('dialog', { name: 'About canicule' })).toBeVisible()
})

test('color, opacity, basemap and geolocation stay usable', async ({ page, context }) => {
  await context.grantPermissions(['geolocation'])
  await context.setGeolocation({ latitude: 48.86, longitude: 2.34 })
  await mockApi(page)
  await page.goto('/')
  await expect(page.locator('canvas.maplibregl-canvas')).toBeVisible()
  const purpleTile = page.waitForRequest((request) => request.url().includes('/api/tiles/all/purple/'))
  await page.getByLabel('Color').selectOption('purple')
  await purpleTile
  await page.getByLabel(/^Opacity/).fill('0.5')
  await expect(page.getByLabel(/^Opacity/)).toHaveValue('0.5')
  await page.getByRole('button', { name: 'Dark', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Dark', exact: true })).toHaveClass(/active/)
  await page.getByRole('button', { name: 'Locate me' }).click()
  await expect.poll(() => page.evaluate(() => location.hash)).toContain('/48.86/2.34/')
})

test('the controls can collapse on a narrow screen', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 720 })
  await mockApi(page)
  await page.goto('/')
  await page.getByRole('button', { name: 'Toggle controls' }).click()
  await expect(page.getByLabel('Sport')).toHaveCount(0)
})
