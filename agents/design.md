# Design Guide

Load this file before changing visual style, copy, interaction patterns, or user-facing workflows.

## Users

JayJay users are developers who use jj for version control. They value keyboard-driven workflows, fast iteration, and tools that stay out of the way. Many come from git, so familiar patterns are useful only when adapted to jj's model.

## Brand

Clean, modern, approachable. The blue jaybird mascot adds personality without making the tool feel childish.

## Visual Direction

- The default treatment is incremental polish of the three-pane developer workflow: keep its density, native controls, and jj vocabulary. Redesign only when the task asks for one.
- Playful jaybird theme: blue gradient `#3B82F6` to `#1E3A8A`, orange accent `#F59E0B`, light blue `#93C5FD`. Brand colors are accents; selection and meaningful state stay distinguishable from decoration.
- Register: technical but polished. Anti-reference: cluttered enterprise tools and generic SaaS dashboards.
- Platform baseline: Apple's [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos) and the HIG topics under it. Depart from it when the developer workflow gives a reason, and say the reason.
- Both shells are the product: SwiftUI on macOS, GPUI on Linux. This guide applies to each. GPUI shares the hierarchy, vocabulary, and palette, and follows the host desktop's conventions where the HIG is macOS-specific.

## Interaction Principles

1. **Native first** - Follow the host platform's controls, menu conventions, shortcuts, and typography: standard controls and SF Symbols in SwiftUI, the shared `ui/` widgets behaving the way the desktop expects in GPUI.
2. **Keyboard-driven** - Keep high-frequency workflows keyboard accessible.
3. **Dense, not cluttered** - Optimize for scanning, comparison, and repeated developer workflows.
4. **Performance is UX** - Prefer quiet refreshes over loading spinners where possible.
5. **Jujutsu-native** - Embrace changes, bookmarks, revsets, and working-copy semantics rather than forcing git branch/commit mental models.
6. **Palette, not terminal** - The command palette runs raw jj (`jj …` or `!…` prefix). When copy, help, or issue replies suggest a jj command, point to the palette rather than "the terminal".

## Invariants

A visual change that breaks one of these is a bug, whatever the design.

- Text honors the configured font family and size; each shell has shared helpers for it ([SwiftUI](swiftui.md#conventions), [GPUI](gpui.md#conventions)). Changing a default never overwrites a saved preference.
- Light and dark follow the system preference and are designed together. Judge contrast on the rendered backgrounds, including selection and tinted surfaces, and with the platform's increased-contrast setting where it has one.
- Color never carries alone a distinction the user must make. Pair it with text, shape, or a symbol, and supply an accessibility label where the shell supports one.
- Layout and visibility changes keep selection, scroll position, pane widths, and keyboard reachability. Animate them to preserve orientation, and respect the platform's reduced-motion setting.
- A color, string, or layout rule owned by the Rust core (diff colors, the theme seed, change-ID prefix color, lane layout, status strings) changes in core so both shells get it.
- Polish leaves behavior alone: graph topology, node meanings, hit testing, and shortcuts.

## Judgment

Everything else is a design decision. Make it; do not look it up. Past decisions live in the code and in the issue where they were made, not here, and a better idea can reopen them. Questions worth asking:

- What is the user scanning for in this pane? That leads; identifiers and metadata stay subordinate but readable.
- Is this an action or a status? They should not look alike.
- Is this fact already on screen? Say it once, where it is used.
- How loud is this cue relative to how often it matters?
- Would a standard control, an existing SwiftUI [presentation surface](swiftui.md#presentation-surfaces), or a GPUI [`ui/` widget](gpui.md#file-layout) do?

Propose the option you think is best, not the safest. Spend the boldness in one place and keep everything around it quiet.

## Visual Change Workflow

1. **Calibrate.** A constant, a truncation rule, or an alignment fix goes straight to implementation. A change to hierarchy, palette, or control placement starts with a mockup or annotated screenshot and its rationale. Mockups suggest direction; the running app with real repository data is the authority.
2. **One visible hypothesis per change.** Land geometry, typography, palette, and interaction separately when they can be judged separately, and hold the rest constant. Do not add theme machinery or feature flags just to compare variants.
3. **Matched before and after.** Same repository, selected change, window size, font settings, and tint setting on both sides. Show light and dark, the full window, and an original-resolution crop of the affected area.
4. **Stress it.** Check 12 pt and a larger size, a non-system font, and long names and paths. For interaction changes, exercise keyboard navigation and state preservation; use a recording to assess motion.
5. **Report what you looked at**, separately from builds and automated tests. One shell or one appearance proves only that. Implement in the requested shell; when the other shell has the same defect, say so rather than fixing it unasked.
