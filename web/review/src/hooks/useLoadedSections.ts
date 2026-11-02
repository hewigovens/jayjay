import { useLayoutEffect, useRef, useState } from "react";
import type { ReviewResult } from "../model/review";
import { toggled } from "../components/toggled";

export interface LoadedFile {
  file: number;
  data: ReviewResult;
  key: string;
}

export function useLoadedSections(
  result: ReviewResult | null,
  revision: number,
  renderFile: (file: number) => Promise<ReviewResult>,
) {
  const [loaded, setLoaded] = useState<Map<string, LoadedFile>>(() => new Map());
  const loading = useRef(new Set<number>());
  const [failed, setFailed] = useState<Set<number>>(() => new Set());
  const path = result?.overview.files[result.file]?.path;
  const cacheKey = result ? `${result.session}|${revision}|${JSON.stringify(result.options)}` : "";
  const liveKey = useRef(cacheKey);
  // The newest result the review itself produced for each file; a lazy render that started before it is stale.
  const reviewed = useRef(new Map<number, ReviewResult>());

  // Layout effects, because a section's load effect runs next and must see this render's guards.
  useLayoutEffect(() => {
    liveKey.current = cacheKey;
    reviewed.current.clear();
    loading.current.clear();
    setFailed(new Set());
    // Sections stay until their new render arrives, so a host's refresh doesn't collapse the list under the reader.
    setLoaded(
      (previous) =>
        new Map(
          [...previous].filter(([path]) =>
            result?.overview.files.some((entry) => entry.path === path),
          ),
        ),
    );
  }, [cacheKey]);
  useLayoutEffect(() => {
    if (!result) return;
    reviewed.current.set(result.file, result);
    if (path)
      setLoaded((previous) =>
        new Map(previous).set(path, { file: result.file, data: result, key: cacheKey }),
      );
  }, [result]);

  function load(index: number) {
    if (loading.current.has(index) || !result) return;
    loading.current.add(index);
    setFailed((previous) => (previous.has(index) ? toggled(previous, index) : previous));
    const key = cacheKey;
    const latest = reviewed.current.get(index);
    renderFile(index).then(
      (next) => {
        if (key !== liveKey.current) return;
        loading.current.delete(index);
        if (reviewed.current.get(index) !== latest) return;
        if (`${next.session}|${revision}|${JSON.stringify(next.options)}` === key)
          setLoaded((previous) =>
            new Map(previous).set(next.overview.files[index].path, {
              file: index,
              data: next,
              key,
            }),
          );
      },
      () => {
        if (key !== liveKey.current) return;
        loading.current.delete(index);
        setFailed((previous) => toggled(previous, index));
      },
    );
  }

  return { cacheKey, loaded, failed, load };
}
