/** @jsxRuntime automatic */
import { useEffect, useRef, useState, type ReactNode } from "react";

export function LazySection({ index, version, loaded, load, children }: { index: number; version: string; loaded: boolean; load: (index: number) => void; children: ReactNode }) {
  const element = useRef<HTMLElement>(null);
  const [near, setNear] = useState(false);
  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => setNear(entry.isIntersecting), { rootMargin: "800px 0px" });
    if (element.current) observer.observe(element.current);
    return () => observer.disconnect();
  }, []);
  useEffect(() => { if (near && !loaded) load(index); }, [near, loaded, index, version]);
  return <section className="file-section" ref={element}>{children}</section>;
}
