/** @jsxRuntime automatic */
export function MiddlePath({ path, className = "" }: { path: string; className?: string }) {
  const split = path.lastIndexOf("/") + 1;
  return (
    <span className={`jj-middle-path ${className}`} title={path}>
      {split > 0 && <span className="jj-middle-path-head">{path.slice(0, split)}</span>}
      <span className="jj-middle-path-tail">{path.slice(split)}</span>
    </span>
  );
}
