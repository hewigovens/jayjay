import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { cloudflare } from "@cloudflare/vite-plugin";
import { defineConfig, type Plugin } from "vite";

function serviceWorker(): Plugin {
  return {
    name: "jayjay-service-worker",
    apply: "build",
    generateBundle(_, bundle) {
      if (this.environment.name !== "client") return;
      const version = createHash("sha256")
        .update(Object.keys(bundle).sort().join("\n"))
        .digest("hex")
        .slice(0, 12);
      this.emitFile({
        type: "asset",
        fileName: "sw.js",
        source: readFileSync(new URL("sw.js", import.meta.url), "utf8").replace(
          "__VERSION__",
          version,
        ),
      });
    },
  };
}

export default defineConfig({ plugins: [cloudflare(), serviceWorker()], worker: { format: "es" } });
