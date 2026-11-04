import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  // Shared CI runners sometimes need more than 5 s for WebKit to import and render a change.
  expect: { timeout: 15_000 },
  fullyParallel: true,
  use: { baseURL: "http://127.0.0.1:4173", locale: "en-US" },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: {
    command: "npm run preview -- --port 4173 --strictPort",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: true,
  },
});
