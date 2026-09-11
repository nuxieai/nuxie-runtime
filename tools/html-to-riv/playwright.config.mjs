import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './validation',
  workers: 1,
  retries: 0,
  forbidOnly: !!process.env.CI,
  timeout: 60_000,
  use: { browserName: 'chromium', deviceScaleFactor: 1, colorScheme: 'light', locale: 'en-US', reducedMotion: 'reduce' },
  reporter: [['list'], ['json', { outputFile: 'test-results/results.json' }], ['./validation/gallery-reporter.mjs']],
  projects: [{name:'native',metadata:{pixels:true}},{name:'geometry',metadata:{pixels:false}}],
});
