/** @jsxRuntime automatic */
export function DiffStats({ added, removed }: { added: number; removed: number }) {
  if (!added && !removed) return null;
  return <span className="diff-stats">{added > 0 && <span className="added">+{added}</span>}{removed > 0 && <span className="removed">−{removed}</span>}</span>;
}
