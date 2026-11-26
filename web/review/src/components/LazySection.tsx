/** @jsxRuntime automatic */
import { useEffect, useRef, useState, type ReactNode, type RefObject } from "react";

export function LazySection({
  index,
  version,
  loaded,
  load,
  root,
  children,
}: {
  index: number;
  version: string;
  /** The scroller holding the sections; the window when omitted. */
  root?: RefObject<HTMLElement | null>;
  loaded: boolean;
  load: (index: number) => void;
  children: ReactNode;
}) {
  const element = useRef<HTMLElement>(null);
  const [near, setNear] = useState(false);
  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => setNear(entry.isIntersecting), {
      root: root?.current,
      rootMargin: "800px 0px",
    });
    if (element.current) observer.observe(element.current);
    return () => observer.disconnect();
  }, [root]);
  useEffect(() => {
    if (near && !loaded) load(index);
  }, [near, loaded, index, version]);
  return (
    <section className="jj-file-section" data-file={index} tabIndex={-1} ref={element}>
      {children}
    </section>
  );
}
