import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./browser",
  expect: { timeout: 15_000 },
  fullyParallel: true,
  use: { baseURL: "http://127.0.0.1:5174", locale: "en-US" },
  projects: [
    { name: "chromium", use: { ...devices["Pixel 7"] } },
    { name: "webkit", use: { ...devices["iPhone 13"] } },
  ],
  webServer: {
    command: "npx vite --host 127.0.0.1 --port 5174 --strictPort",
    url: "http://127.0.0.1:5174",
    reuseExistingServer: true,
  },
});
