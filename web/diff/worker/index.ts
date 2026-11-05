import { parseReviewRef, reviewPath } from "../src/github/ReviewRef";
import { GitHub, GitHubError } from "./github/GitHub";
import { Exports } from "./Exports";
import { GitHubLogin } from "./github/GitHubLogin";
import { samples } from "../src/github/samples";

interface Env {
  GITHUB_TOKEN?: string;
  GITHUB_CLIENT_ID: string;
  STORE: KVNamespace;
}

const prefix = "/api/";
const samplePaths = new Set(samples.map(reviewPath));

function json(body: unknown, status: number, maxAge = 0) {
  return Response.json(body, {
    status,
    headers: { "cache-control": maxAge ? `public, max-age=${maxAge}` : "no-store" },
  });
}

async function login(request: Request, env: Env, pathname: string) {
  const github = new GitHubLogin(env.GITHUB_CLIENT_ID);
  const body = (await request.json().catch(() => null)) as {
    deviceCode?: unknown;
    write?: unknown;
  } | null;
  if (pathname === "/api/login/device") {
    const code = await github.start(body?.write === true);
    return code ? json(code, 200) : json({ error: "unavailable" }, 502);
  }
  if (typeof body?.deviceCode !== "string") return json({ error: "unavailable" }, 400);
  return json(await github.poll(body.deviceCode), 200);
}

export default {
  async fetch(request, env, context) {
    const { pathname } = new URL(request.url);
    if (
      request.method === "POST" &&
      (pathname === "/api/login/device" || pathname === "/api/login/token")
    )
      return login(request, env, pathname);
    if (request.method === "POST" && pathname === "/api/exports")
      return new Exports(env.STORE).create(request);
    if (request.method === "GET" && pathname.startsWith("/api/exports/"))
      return new Exports(env.STORE).read(pathname.slice("/api/exports/".length));
    const target = pathname.startsWith(prefix)
      ? parseReviewRef(pathname.slice(prefix.length))
      : null;
    if (!target) return json({ error: "not-found" }, 404);
    if (request.method !== "GET") return json({ error: "not-found" }, 405);
    // Samples are kept once fetched, so the landing page's links open even when the rate limit is spent.
    const sample = samplePaths.has(reviewPath(target)) ? `sample:${reviewPath(target)}` : undefined;
    const fresh = request.headers
      .get("cache-control")
      ?.split(",")
      .some((value) => value.trim() === "no-cache");
    const kept = !fresh && sample && (await env.STORE.get(sample));
    if (kept)
      return new Response(kept, {
        headers: { "content-type": "application/json", "cache-control": "public, max-age=3600" },
      });
    const cached = !fresh && (await caches.default.match(request));
    if (cached) return cached;
    // A signed-in reader spends their own rate limit; no scope it asks for reads private repositories, so what it reads is cacheable.
    const token =
      /^Bearer (\S+)$/.exec(request.headers.get("authorization") ?? "")?.[1] ?? env.GITHUB_TOKEN;
    try {
      // A short edge cache absorbs reloads and shared links; new pushes show up within a minute.
      const manifest = await new GitHub(token).manifest(target);
      if (sample) context.waitUntil(env.STORE.put(sample, JSON.stringify(manifest)));
      const response = json(manifest, 200, 60);
      context.waitUntil(caches.default.put(request, response.clone()));
      return response;
    } catch (error) {
      if (error instanceof GitHubError) return json({ error: error.code }, error.status);
      throw error;
    }
  },
} satisfies ExportedHandler<Env>;
