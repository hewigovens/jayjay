/** @jsxRuntime automatic */
import type { Labels } from "../../model/diff";
import type { ReviewNote } from "../../model/review";
import { NoteBubble } from "./NoteBubble";

export function NotesBanner({
  items,
  labels,
  disabled,
  resolve,
  locate,
}: {
  items: ReviewNote[];
  labels: Labels;
  disabled: boolean;
  resolve: (id: string) => void;
  locate: (item: ReviewNote) => void;
}) {
  if (!items.length) return null;
  return (
    <section className="jj-notes-banner" aria-label={labels.notes}>
      {items.map((item) => (
        <NoteBubble
          key={item.note.id}
          item={item}
          labels={labels}
          disabled={disabled}
          resolve={resolve}
          locate={() => locate(item)}
        />
      ))}
    </section>
  );
}
