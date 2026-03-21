import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  timeout: 180000,
  expect: {
    timeout: 30000,
  },
  reporter: [['html'], ['list']],
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 60000,
    navigationTimeout: 90000,
  },
  projects: [
    {
      name: 'setup',
      testMatch: /.*\.setup\.ts/,
    },
    {
      name: 'data-seed',
      testMatch: /data-seeder\.ts/,
      dependencies: ['setup'],
    },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup', 'data-seed'],
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
      dependencies: ['setup', 'data-seed'],
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
      dependencies: ['setup', 'data-seed'],
    },
    {
      name: 'journey',
      testMatch: /.*journey.*\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup', 'data-seed'],
      timeout: 300000,
    },
  ],
  webServer: {
    command: 'cd ../origa_ui && trunk serve --port 8080',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 300000,
  },
});
