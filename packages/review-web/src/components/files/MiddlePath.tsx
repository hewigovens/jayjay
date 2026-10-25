/** @jsxRuntime automatic */

/** A path truncated in the middle like SwiftUI's `.truncationMode(.middle)`: the directory shrinks first and the basename stays whole. */
export function MiddlePath({ path, className = "" }: { path: string; className?: string }) {
  const split = path.lastIndexOf("/") + 1;
  return <span className={`middle-path ${className}`} title={path}>
    {split > 0 && <span className="middle-path-head">{path.slice(0, split)}</span>}
    <span className="middle-path-tail">{path.slice(split)}</span>
  </span>;
}
