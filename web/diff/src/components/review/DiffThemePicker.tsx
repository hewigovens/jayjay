import { useId, useRef, useState } from "react";
import { Dialog, ToolButton } from "@jayjay/review-web/react";
import { diffThemes } from "@jayjay/review-web/themes";
import type { Labels } from "@jayjay/review-web/types";
import {
  diffFontRange,
  interfaceFontRange,
  type AppearanceSettings,
} from "../../AppearanceSettings";
import { FontSizeStepper } from "./FontSizeStepper";
import { AppearanceSelect } from "./AppearanceSelect";
import { ThemeIcon, AutoAppearanceIcon, LightAppearanceIcon, DarkAppearanceIcon } from "../icons";

export function DiffThemePicker({
  labels,
  themes,
  select,
}: {
  labels: Labels;
  themes: AppearanceSettings;
  select: (themes: AppearanceSettings) => void;
}) {
  const group = useId();
  const [open, setOpen] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const close = () => {
    setOpen(false);
    button.current?.focus();
  };
  return (
    <>
      <ToolButton ref={button} label={labels.appearance} onClick={() => setOpen(true)}>
        <ThemeIcon />
      </ToolButton>
      {open && (
        <Dialog className="diff-theme-picker" label={labels.appearance} onClose={close}>
          <h2>{labels.appearance}</h2>
          <fieldset className="appearance-options" aria-label={labels.appearance}>
            {(
              [
                ["system", labels.appearanceSystem, AutoAppearanceIcon],
                ["light", labels.appearanceLight, LightAppearanceIcon],
                ["dark", labels.appearanceDark, DarkAppearanceIcon],
              ] as const
            ).map(([appearance, label, AppearanceIcon]) => (
              <label key={appearance}>
                <input
                  type="radio"
                  name={group}
                  value={appearance}
                  checked={(themes.appearance ?? "system") === appearance}
                  onChange={() => select({ ...themes, appearance })}
                />
                <span>
                  <AppearanceIcon />
                  {label}
                </span>
              </label>
            ))}
          </fieldset>
          {(["light", "dark"] as const).map((side) => (
            <AppearanceSelect
              key={side}
              label={side === "dark" ? labels.darkTheme : labels.lightTheme}
              value={themes[side]}
              onChange={(value) => select({ ...themes, [side]: value })}
              options={diffThemes
                .filter((theme) => theme.dark === (side === "dark"))
                .map((theme) => ({
                  value: theme.id,
                  label:
                    theme.id === "jayjay-light"
                      ? labels.jayjayLight
                      : theme.id === "jayjay-dark"
                        ? labels.jayjayDark
                        : theme.name,
                }))}
            />
          ))}
          <section className="font-settings" aria-labelledby={`${group}-font`}>
            <h2 id={`${group}-font`}>{labels.fontSection}</h2>
            <FontSizeStepper
              label={labels.fontInterface}
              value={themes.interfaceFontSize}
              {...interfaceFontRange}
              labels={labels}
              onChange={(interfaceFontSize) => select({ ...themes, interfaceFontSize })}
            />
            <FontSizeStepper
              label={labels.fontDiff}
              value={themes.fontSize}
              {...diffFontRange}
              labels={labels}
              onChange={(fontSize) => select({ ...themes, fontSize })}
            />
          </section>
          <form method="dialog">
            <button className="diff-theme-close">{labels.close}</button>
          </form>
        </Dialog>
      )}
    </>
  );
}
