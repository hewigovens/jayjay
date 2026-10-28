/** @jsxRuntime automatic */
import { useEffect, useState } from "react";
import type { AvatarSource } from "../model/review";

const pixelSize = 96;
// One lookup per source for the page's lifetime, like the desktop AvatarStore's in-flight dedupe; images themselves use the HTTP cache.
const lookups = new Map<string, Promise<string | null>>();

async function lookup(source: AvatarSource): Promise<string | null> {
  if (source.kind === "url") return source.value;
  const gitHub = source.kind === "gitHubBot";
  const response = await fetch(gitHub
    ? `https://api.github.com/user/${encodeURIComponent(source.value)}`
    : `https://gitlab.com/api/v4/users?username=${encodeURIComponent(source.value)}`);
  if (!response.ok) return null;
  const json: unknown = await response.json();
  const user = (gitHub ? json : Array.isArray(json) ? json[0] : null) as { avatar_url?: unknown } | null;
  if (typeof user?.avatar_url !== "string") return null;
  const url = new URL(user.avatar_url);
  url.searchParams.set(gitHub ? "size" : "width", String(pixelSize));
  return url.toString();
}

function resolve(source: AvatarSource): Promise<string | null> {
  const key = `${source.kind}:${source.value}`;
  let pending = lookups.get(key);
  if (!pending) {
    pending = lookup(source).then(url => url?.startsWith("https://") ? url : null, () => null);
    lookups.set(key, pending);
  }
  return pending;
}

/** The author's avatar over their monogram, which stays visible until the image loads or if it fails. */
export function Avatar({ source, name }: { source?: AvatarSource | null; name: string }) {
  const [url, setUrl] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  useEffect(() => {
    setUrl(null); setLoaded(false);
    if (!source) return;
    let live = true;
    void resolve(source).then(resolved => { if (live) setUrl(resolved); });
    return () => { live = false; };
  }, [source?.kind, source?.value]);
  return <span className="jj-avatar" aria-hidden="true">{name.trim().charAt(0).toUpperCase()}
    {url && <img src={url} alt="" referrerPolicy="no-referrer" className={loaded ? "jj-loaded" : undefined}
      onLoad={() => setLoaded(true)} onError={() => setUrl(null)} />}</span>;
}
