import { useEffect, useState } from "react";

export function useCompact() {
  const query = "(max-width: 680px)";
  const [compact, setCompact] = useState(() => typeof matchMedia === "function" && matchMedia(query).matches);
  useEffect(() => {
    const list = matchMedia(query);
    const update = () => setCompact(list.matches);
    update();
    list.addEventListener("change", update);
    return () => list.removeEventListener("change", update);
  }, []);
  return compact;
}
