/** @jsxRuntime automatic */
export function DiffStats({ added, removed }: { added: number; removed: number }) {
  if (!added && !removed) return null;
  return <span className="jj-diff-stats">{added > 0 && <span className="jj-added">+{added}</span>}{removed > 0 && <span className="jj-removed">−{removed}</span>}</span>;
}
