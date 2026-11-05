import { defineConfig } from "vitest/config";

// Separate from vite.config.ts so tests skip the Cloudflare plugin's Worker runtime.
export default defineConfig({ test: { include: ["tests/**/*.test.ts", "worker/**/*.test.ts"] } });
