/** @jsxRuntime automatic */
import type { Overview } from "../../model/review";

// Joins terminal-wrapped commit message lines into paragraphs, keeping list items, indented lines and blank lines.
function reflow(text: string): string {
  return text
    .split(/\n\s*\n/)
    .map((paragraph) =>
      paragraph.split("\n").reduce((out, line) => {
        const keep = /^(\s|[-*+] |\d+[.)] )/.test(line);
        return out === "" ? line : keep ? `${out}\n${line}` : `${out} ${line.trim()}`;
      }, ""),
    )
    .join("\n\n");
}

export function ChangeDescription({
  overview,
  className,
}: {
  overview: Overview;
  className: string;
}) {
  return overview.descriptionHtml ? (
    <div
      className={`${className} jj-markdown`}
      dangerouslySetInnerHTML={{ __html: overview.descriptionHtml }}
    />
  ) : (
    <p className={className}>{reflow(overview.description ?? "")}</p>
  );
}
