import { expect, test, vi } from "vitest";
import worker from "./index";
import { GitHub } from "./github/GitHub";
import type { PullManifest } from "../src/github/PullManifest";
import { samples } from "../src/github/samples";
import { reviewPath } from "../src/github/ReviewRef";

test.each(["/o/r/pull/7", reviewPath(samples[0])])(
  "refresh bypasses cached manifests for %s",
  async (path) => {
    const stale = { title: "Cached diff" };
    const fresh = { title: "Latest diff" } as PullManifest;
    vi.stubGlobal("caches", {
      default: { match: async () => Response.json(stale), put: async () => {} },
    });
    const manifest = vi.spyOn(GitHub.prototype, "manifest").mockResolvedValue(fresh);
    const env = {
      STORE: { get: async () => JSON.stringify(stale), put: async () => {} },
    } as unknown as Parameters<typeof worker.fetch>[1];
    const context = { waitUntil: () => {} } as unknown as ExecutionContext;
    try {
      const url = `https://diff.hewig.dev/api${path}`;
      expect(
        await (
          await worker.fetch(new Request(url) as Parameters<typeof worker.fetch>[0], env, context)
        ).json(),
      ).toEqual(stale);
      const request = new Request(url, { headers: { "cache-control": "no-cache" } }) as Parameters<
        typeof worker.fetch
      >[0];
      expect(await (await worker.fetch(request, env, context)).json()).toEqual(fresh);
      expect(manifest).toHaveBeenCalledTimes(1);
    } finally {
      vi.restoreAllMocks();
      vi.unstubAllGlobals();
    }
  },
);
