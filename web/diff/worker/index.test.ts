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

test.each([
  ["/o/r/pull/7", "public, max-age=60"],
  [`/o/r/pull/7/commits/${"a".repeat(40)}`, "public, max-age=86400"],
  [`/o/r/commit/${"a".repeat(40)}`, "public, max-age=86400"],
  ["/o/r/commit/aaaaaaa", "public, max-age=60"],
])("%s is cached for as long as it can stay true", async (path, expected) => {
  vi.stubGlobal("caches", { default: { match: async () => undefined, put: async () => {} } });
  vi.spyOn(GitHub.prototype, "manifest").mockResolvedValue({ title: "T" } as PullManifest);
  const env = { STORE: { get: async () => null } } as unknown as Parameters<typeof worker.fetch>[1];
  const context = { waitUntil: () => {} } as unknown as ExecutionContext;
  try {
    const request = new Request(`https://diff.hewig.dev/api${path}`) as Parameters<
      typeof worker.fetch
    >[0];
    expect((await worker.fetch(request, env, context)).headers.get("cache-control")).toBe(expected);
  } finally {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  }
});
