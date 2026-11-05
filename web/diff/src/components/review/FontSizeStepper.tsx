import { useId } from "react";
import type { Labels } from "@jayjay/review-web/types";

export function FontSizeStepper({
  label,
  value,
  min,
  max,
  onChange,
  labels,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  labels: Labels;
}) {
  const id = useId();
  return (
    <div className="diff-theme-row">
      <span id={id}>{label}</span>
      <div className="font-size-stepper" role="group" aria-labelledby={id}>
        <button
          aria-label={labels.decreaseFontSize}
          disabled={value <= min}
          onClick={() => onChange(value - 1)}
        >
          −
        </button>
        <output aria-live="polite">{value} px</output>
        <button
          aria-label={labels.increaseFontSize}
          disabled={value >= max}
          onClick={() => onChange(value + 1)}
        >
          +
        </button>
      </div>
    </div>
  );
}
