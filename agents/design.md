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
- Both shells are the product: SwiftUI on macOS, GPUI on Linux and Windows. This guide applies to each. GPUI shares the hierarchy, vocabulary, and palette, and follows the host desktop's conventions where the HIG is macOS-specific.

## Interaction Principles

1. **Native first** - Follow the host platform's controls, menu conventions, shortcuts, and typography: standard controls and SF Symbols in SwiftUI, the shared `ui/` widgets behaving the way the desktop expects in GPUI.
2. **Keyboard-driven** - Keep high-frequency workflows keyboard accessible.
3. **Dense, not cluttered** - Optimize for scanning, comparison, and repeated developer workflows.
4. **Performance is UX** - Prefer quiet refreshes over loading spinners where possible.
5. **Jujutsu-native** - Embrace changes, bookmarks, revsets, and working-copy semantics rather than forcing git branch/commit mental models.
6. **Palette, not terminal** - The command palette runs raw jj (`jj …` or `!…` prefix). When copy, help, or issue replies suggest a jj command, point to the palette rather than "the terminal".

## Information Hierarchy

jj's own containment decides where things go:

```
repo (operation log, bookmarks, remotes)
└─ workspace (working copy @)
   └─ history (revset) → change → files → hunks
```

Three parts of the model are easy to get wrong:

- **Bookmarks are labels, not places.** You are on a change, not on a branch. A bookmark sits next to the change it labels, with its sync state; it does not frame the window.
- **There are two time axes.** The operation log is repo time and belongs at window level. The evolog is change time and belongs inside the change.
- **The working copy is a change.** `@` has the same anatomy as any other change: description, metadata, files. Where it behaves differently, the difference is a stated exception with a reason.

- **One level per region.** Scope narrows left to right and top to bottom: window chrome (toolbar, title, menus) is repo and workspace scope, the sidebar is history, the detail pane is one change with its files and their hunks.
- **A fact appears once, at the level it describes.** When two regions could show it, one owns it and the other shows it only while the first is hidden. Conflict and divergence counts belong to the change list header, not the status bar. A file row leads with the file name and gives the folder as the secondary line; it does not repeat the name.
- **An action sits with what it acts on.** The toolbar holds frequent repo-scope actions. App-level destinations such as Settings and Show in Finder live in the menus and the palette. A file-scope action sits on the file; a choice that applies to the whole change is one control on the change, not one per file.
- **Viewing is the default; a workflow is a mode.** Outside the working copy, review marks appear only while the change is in review mode ([Review State](review-state.md)). The working copy is the exception because its marks also pick files to split or commit, and it keeps a description box in the sidebar while it has no description or an unsaved draft: describing and committing `@` is the most frequent action. Once `@` is described the box gives its room back to the change list, and Edit in the change header opens the same actions in a sheet.
- **Identity is compact, detail is on demand.** IDs show at jj's short length with the unique prefix emphasized, and keep a visible copy control. The change header spans the file list and the diff, since both belong to the change, and stays compact by default because descriptions run long; nothing in it pushes the files or the diff down.
- **An alternate view replaces content at its own level.** Evolog and annotate swap in for the diff inside the change; they do not stack beside it or open a second window.
- **Views of a wider scope sit beside the content.** History that belongs to the repo, such as the operation log, opens next to the change it would affect rather than over it, as a trailing pane inside the window, toggled from the trailing end of the toolbar; the toggle sits above the pane while it is open.
- **A row's actions live in its context menu,** opened by right click or ⌃Return on the selected row, never by a double click; restoring an operation is one.
- **Every toolbar button and context-menu item also exists in the menu bar,** with its shortcut shown there, and in the command palette. A toolbar button is a convenience, never the only route, which is what lets a rarely used one leave the toolbar.
- **The toolbar follows the platform's order:** navigation leading, one dominant control in the centre (the revset field), a few frequent actions trailing as monochrome symbols with tooltips that name what they open.
- **Standard controls carry modes.** A two-way mode is a segmented control; a list's flat or tree form is a toggle in that list's header; an empty state is one short, calm message, not a prompt; an edit form opens from an Edit button with Save and Cancel, Return to save and Escape to cancel.
- **Showing or hiding a window region is a View menu item** with a matching command, persisted app-wide like the sidebar toggle. It is not a Settings row.
- **Optional chrome is never the only path.** Anything a user would be stuck without needs an entry point outside the region they can hide: reconnecting a disconnected SSH repository has its own banner, menu item, and command, whether or not the status bar is shown.

## Color and Icons

- **Color means kind, following jj's defaults:** change IDs and bookmarks magenta, commit IDs blue, the working copy and workspaces green, conflicts red, divergence orange. A role is the same color everywhere it is drawn. The accent color is for selection and focus.
- **Themes keep their own hues.** ID prefixes, bookmarks, and syntax tokens derive from the theme's palette by lightening or darkening it, not by fading it toward the text color; check them on the row, header, and selected-row backgrounds of every built-in theme. The workspace marker, file-status colors, and the review-note accent still fade toward the text color; move a role to the first method when it reads as washed out.
- **One glyph, one meaning.** SF Symbols first on macOS. Where no symbol says it, use an Octicons glyph (the set GPUI ships) as a template image, scaled only to match the neighbouring symbols: pull and push are the download and upload tray, a mirrored pair.
- **Size a glyph by its ink, against its neighbours.** A custom or third-party mark in a 16-point box can still look a third smaller than the symbols beside it. Measure the drawn width and height of each icon in the group from a capture, then widen the artwork or scale the mark until the group reads as one set; scale a filled mark slightly smaller than a line mark.
- **Stack icons on their ink centres, not their boxes or left edges.** Symbol and icon-font glyphs carry different side margins and widths, so equal frames or flush left edges still look ragged. Give icons that sit above one another one column as wide as the reference icon's ink, centre each glyph in it, and start the text after the column so labels share an edge too. Size the glyphs to similar ink first, then check the column in a zoomed capture.
- **Another app is shown by its own mark in one color** on the same tile as every other app, tinted with the surrounding foreground. Full-color app icons and run-time masks of them do not belong in the toolbar; an app without a bundled mark gets the shell's generic code or terminal symbol (on macOS, the hammer for Xcode).

## Invariants

A visual change that breaks one of these is a bug, whatever the design.

- Equivalent UI roles share typography, spacing, alignment, and icon treatment through shared components. Check labels and control values together; fix the shared style instead of adding per-row overrides.
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
5. **Prove it and report what you looked at**, separately from builds and automated tests. A static change gets a screenshot (the matched set above for appearance changes); a workflow change gets a named passing UI test scene or a recording; interaction and motion get a recording. Link the files in the final report. One shell or one appearance proves only that. Implement in the requested shell; when the other shell has the same defect, say so rather than fixing it unasked.
