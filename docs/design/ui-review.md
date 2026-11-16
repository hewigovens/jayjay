# JayJay macOS UI review

A review of the repo window: what feels off today, the principles behind each proposed fix, and a suggested order to land them. Each recommendation notes where it touches an earlier decision, with the reasoning for revisiting it.

- Working prototype: the commits on the discussion branch, one per idea. They build and run; the unit and UI test suites have not been run.
- Theory: [information-hierarchy.md](information-hierarchy.md). It holds the containment model and the principles, including the macOS and Golden Gate notes.
- Before/after screenshots and clips: the discussion PR body.

## 1. The core problem

The old window was assembled feature by feature, so information sat wherever it was first added:

- The path appeared in the status bar.
- The bookmark appeared in the status bar.
- The commit editor sat at the bottom of the sidebar.
- Full IDs and emails were inline in the header.
- The file list sat beside the header instead of under it.
- Review circles appeared on every mutable change.
- The Settings and Finder buttons were in the toolbar.

Each placement was defensible on its own. Together they gave the window no information hierarchy, so it read as a developer tool rather than a Mac app.

The fix is to place everything by jj's own containment:

```
repo (op log, bookmarks, remotes)
└─ workspace (folder, @)
   └─ history (revset) → change → files → hunks
```

Scope narrows from left to right and top to bottom. Each fact appears once, at its own level. Actions sit next to what they act on. Viewing is the default, and workflows are modes you enter. Use the system's controls and let the OS style them. The full principles and reasons are in [information-hierarchy.md](information-hierarchy.md).

## 2. Recommendations, in landing order

The maintainer's `agents/design.md:56` asks for a mockup or annotated screenshot with rationale for any change to hierarchy, palette or control placement. These could land as small PRs, each with its before/after pair from the discussion PR as that mockup.

### A. Toolbar: repo scope only (low risk)

- **Drop the Finder and Settings buttons.** Settings lives in ⌘, and the app menu, and Finder has a menu item with a shortcut. HIG: toolbar buttons are conveniences, the menu bar is the full set.
- **Show the configured editor's own app icon** on the open-in button. The `{}` glyph said nothing about which app opens.
- **Pull/push:** circle-arrow glyphs read as next/previous. Use tray-with-arrow glyphs (`tray.and.arrow.down` / `.up`).
- **Add an op log button** (`clock.arrow.trianglehead.counterclockwise.rotate.90`). Its tooltip names the current operation, and it toggles the op log as the window's trailing `.inspector` panel instead of a modal sheet. The op log is repo time, so it sits beside the content at window level; Restore leaves the panel open.
- **Earlier decisions this touches:** issue #273 accepted a toolbar "grouped by scope" with editor/terminal/settings. Keep menu, palette and shortcut equivalents and say so in the PR (`design.md:28`).

### B. Title and subtitle; remove the status bar (low risk)

- **Title = repo name; subtitle = workspace (when there are several) · SSH host and state · nearest tracked bookmark with sync state.** This is the native title/subtitle pattern, and bookmarks are context, not a place you're "on".
- **Remove the status bar.** Each item moves to its own level:

  | Item | New home |
  | --- | --- |
  | Last op | Op log button tooltip |
  | Working-copy line counts | Already on `@` and the change header |
  | Conflict / divergence counts | Filter buttons in the change-list header |
  | SSH | Subtitle, plus a toolbar Reconnect button when disconnected |
  | PR | Toolbar link |
  | Path | Title tooltip; Finder stays in the menu |

  A status bar is unusual on the Mac, and here it duplicated other regions.
- **Gaps to close before landing:**
  - The SSH connection sheet (`modal = .sshConnection`) and "copy SSH path" were reached from the status bar. Put both in the title picker menu.
  - `agents/swiftui.md` (lines 14, 22, 37, 61) still describes `StatusBar/`. `StatusBarSnapshot` is still loaded by the view model. Rename it or slim it to what's still used.
  - `docs/guide.html` mentions the status bar. It's release-pass only, per `AGENTS.md`.

### C. System sidebar and nested surfaces (medium risk)

- **Use `NavigationSplitView` for the sidebar and let the OS style it.**
  - Golden Gate (macOS 27) returns to full-height, edge-to-edge sidebars with a darker background, coloured icons and semi-bold selection. Apps on the standard split view get this with no code changes (WWDC26 session 289; Six Colors, MacStories, MacRumors and Michael Tsai coverage of the betas).
  - On Tahoe, the undocumented `NSSplitViewItemSidebarDefaultsToFloatingAppearance = false` gives the full-height form.
  - A custom painted sidebar freezes today's look. The prototype tried one and dropped it.
- **Nest surfaces; adjacent scopes never share a colour:**
  - sidebar = system
  - detail = reading background
  - change header = reading background raised slightly (`Color.primary.opacity(0.05)`)
  - files and diffs = reading background
- **Sidebar minimum width: 320pt.** Refresh/pull/push need the room, and narrower widths truncated every row.
- **Maintainer invariants to keep** (`22b622a7`, #281):
  - The hidden sidebar keeps its width, scroll, selection, revset and drafts.
  - The Tab cycle skips the hidden pane.
  - No toolbar glass flash when hiding.

  Verify each with the native split. The prototype persists width with a settle delay (`onGeometryChange`), which is fragile; find a better hook or accept losing the stored width.
- **Earlier decisions this touches:** issue #417 complained that dividers are hard to grab. Check the native divider hit area.

### D. Change header: one scope, read-first (medium risk)

- **The header spans the file list and the diffs.** They belong to the change, so the change's identity sits above both.
- **Read-only by default for every change, including `@`.** A small Edit button opens the editor in place:
  - the working copy gets Describe/Commit/AI with Done in the same row;
  - other mutable changes get Save/Cancel.

  This also removes the commit box from the sidebar, which is a source list and shouldn't hold forms.
- **Undescribed changes** show a quiet "No description · ✎ Add description".
- **Details are expanded by default.** When wide, they lay out as author/date | change/commit/parents | bookmarks/line counts.
- **Scrolling the diffs collapses the header** to the one-line byline, and returning to the top restores it. There's no expand/collapse button.
  - Caveat 1: this only works in the stacked-diffs view, because a single file's diff scrolls inside its own NSView. Rendering a single selection through the same stacked scroll view (one file) would unify the behaviour.
  - Caveat 2: the collapsed state is verified by screenshot, but the scroll trigger is not.
- **IDs are 8+ characters** with the unique prefix in colour. Full IDs, emails and bookmark names copy via right-click "Copy …", the Mac convention, replacing a copy button beside every value.
- **Change title at 17pt semibold** (was 20).
- **Earlier decisions this touches:**
  - `0f31c134` built the always-visible Summary + Description box on purpose for stacked PRs.
  - Draft reseed rules (`swiftui.md:61`; issues #187 and #353) must survive the move. Test split, divergent-sibling and workspace-switch cases.
  - #242 wants the expanded height remembered.

### E. Files and diffs: content, not summaries (low–medium risk)

- **No auto-selected first file.** With nothing selected, the detail stacks every file's actual diff in one scroll, each under its own header with +/- counts. Selecting a file shows only that file. Escape or clicking the file count returns to all files.
  - A stat summary was tried and dropped, because it repeated the file list.
- **The file list is the index.** It can be hidden from its own header with a files icon (`list.bullet.rectangle.portrait`, deliberately not the sidebar glyph). While hidden, the toggle moves to the change header. The choice is remembered.
- **One unified/side-by-side control** for the whole change, as a segmented control in the change header. It was a button repeated on every file.
  - Lost: the per-file "side by side unavailable" hint. Add a quiet "unified only" note to that file's header.
- **File rows:** name, folder only (`src/detail/`), and the same +/- badge on every change. Tree mode drops the path line, and single-line rows use natural height.
  - #274 wants the secondary path readable, which still holds at 10pt monospaced. Consider 11pt.
- **An empty change shows one empty state;** the file column hides.
- **Earlier decisions this touches:**
  - The review flow assumes "the next unreviewed file opens by itself" (`6a8a7094`). Keep that when review marks are on.
  - Stacked diffs need the lazy loading and flicker work from #343 and #346. The prototype uses `LazyVStack` and `DiffSection(fitsContent: true)` (the converge card's mode). Measure a 100-file change before landing.

### F. Review is a mode, not chrome (higher impact)

- **Today, marks show on every mutable change.** Viewing your own working copy, the most common thing a jj GUI shows, draws an empty circle on every file, and the change-type badge shrinks to a dot to make room. Outside a review the circles read as radio buttons.
- **Review is the product's headline** (launch post, README, agent marks), so it must stay one step away, not buried in a setting.
- **Proposed: a Review toggle in the change header** (and the Repository menu). While it's on, each file's diff header carries Mark Reviewed and Space works as before; file rows never show circles. Review notes always show.
- **Worth discussing:** turning review on automatically when a change already has reviewed or agent-marked files, so an agent's review is visible without a click.

### G. Colour (low risk)

- **Follow jj's defaults, the same everywhere:**
  - change IDs and bookmarks: magenta
  - commit IDs: blue
  - `@` and workspaces: green
  - added/removed lines: green/red
  - conflicts: red; divergence: orange

  The accent colour is for selection and focus only. Colour is never decoration.
- **Done so far:**
  - The header bookmark chip uses the theme bookmark colour, like the graph.
  - The change-ID prefix is magenta in core (`repo_colors.rs`: dark `0xd86bf2`, light `0x9635c9`, seeded `seed.magenta`).
  - Theme tests pass (`just test-rust jayjay-core theme`: 6 passed).
- Colour is owned by core (`design.md:39`), so the change belongs in core. The maintainer hasn't ruled on magenta change IDs.

## 3. Type and icon consistency (not done; do as its own pass)

The audit found nine text sizes in the main views (8, 9, 10, 11, 12, 13, 14, 18, 20pt). Icons get fixed sizes independent of the text beside them. That is most of the "inconsistent" feel.

**Adopt a small scale that maps to macOS text styles:**

| Role | Size and weight | Used for |
| --- | --- | --- |
| Title | 17 semibold | Change summary |
| Body | 13 | Rows, file names, header values |
| Secondary | 12 | Counts, labels, metadata |
| Caption | 11 | Folder lines, chips, relative times |
| Mono | 12 for code and IDs; 11 in dense rows | |

Nothing below 10pt. Chips and badges use the caption size, not 9 or 8.

**Icons follow their text:**

- Set the font on the label and use `.imageScale(.small/.medium)` rather than `jayjayFont(14)` on bare images.
- Toolbar items use standard toolbar sizing; don't hand-size them.
- In the prototype, the file-list header icons are now all 13pt.

**Glyph vocabulary, one meaning each:**

| Concept | Glyph |
| --- | --- |
| Sidebar | `sidebar.leading` |
| File list | `list.bullet.rectangle.portrait` |
| Tree / flat list | `list.bullet.indent` / `list.bullet` |
| Filter | `magnifyingglass` |
| Edit | `pencil` |
| Bookmark | `bookmark` |
| Op log | clock arrow |
| Pull / push | trays |

Audit `FileRow` (8 and 9pt badges), `DAGRow` (8–13pt), `BookmarkPicker` and the diff header for violations.

## 4. Other areas that need the same treatment

1. **Change list rows.** Each row has three lines (IDs, summary, commit ID/author/time) and repeats the author's name on every row. Closer to `jj log`: line 1 = change ID · bookmarks/@ · relative time; line 2 = summary. Show the author only when it isn't you, and the commit ID on hover or selection. This roughly doubles visible history.
2. **The sidebar header** ("Bookmarks 4 ▾ … 81 changes"). The bookmark picker is a filter on the history and reads well. The count is repeated information: hide it unless filtered, and keep conflict/divergence buttons.
3. **The revset field** is the dominant title-bar control, as HIG wants (one central control), but it shows the raw default revset. Show a friendly name for presets ("Default view") and the expression on focus.
4. **Diff file header.** It shows status pill, line counts, Preview, Edit Diff and Edit File. Keep file-scoped actions there, but group them as one quiet trailing set and move rarely used ones into a ⋯ menu.
5. **Evolog** is change time, so it belongs inside the change. Use an alternate view of the detail, like annotate. The trailing inspector now holds the op log (repo time); if the evolog also goes there, give the inspector a segmented switch (Operations | Evolution) rather than two panels.
6. **Light mode, larger text sizes and Reduce Transparency** haven't been checked. Golden Gate's transparency slider means no layout can assume a fixed glass level.

## 5. Validation still owed (none of this has been run)

- **Unit:** `just test-rust jayjay-core` passed only the theme slice. Run the Swift unit tests (`just test-app`).
- **UI scenes likely broken by the prototype:**
  - `SidebarToggleScene` (divider and toggle IDs)
  - `RepoListWindowScene` (toolbar sidebar toggle)
  - `ReleaseScreenshots` (`AID.Diff.layoutToggle` is now a segmented picker, not a button)
  - Any scene that relies on the first file being auto-selected
  - Commit box scenes (`AID.CommitBox.*`, now behind Edit)
  - Review-mark scenes (now off by default)
- **Strings:** new `settings-show-review-marks`, `repo-show-file-list` and `repo-hide-file-list` in four locales. The translations are best-effort and need native review.
- **Docs:** update `agents/swiftui.md` for removed files; user guide and Help Book in the release pass only.
- **Before pushing:** `jj fix`, `just lint`, and an independent review per `agents/code-review.md`.

## 6. How this relates to existing guidance and decisions

- `agents/design.md`: native first, keyboard-driven, dense not cluttered, quiet; "Is this fact already on screen? Say it once, where it is used"; "Propose the option you think is best, not the safest"; past decisions can be reopened by a better idea. Most of this review applies those rules.
- [#270 macOS visual polish](https://github.com/hewigovens/jayjay/issues/270): "incremental polish… keep the three-pane layout, jj vocabulary, native controls, and current density".
  - This review keeps three panes: history, files, diff. It keeps the density, and moves to more native controls.
  - It's meant as polish in small PRs, not a redesign.
- [#274](https://github.com/hewigovens/jayjay/issues/274) and [#275](https://github.com/hewigovens/jayjay/issues/275): the description is the detail pane's heading with Edit at its trailing side, and the diff is the working surface that should dominate. That matches D and E.
- [#242](https://github.com/hewigovens/jayjay/issues/242): avoid pushing the diff down. Scroll-collapse serves that directly.
- [#281](https://github.com/hewigovens/jayjay/issues/281) and `22b622a7`: sidebar hide/show invariants. See C.
- Review marks are the headline. See F for keeping them one step away.
