# JayJay information hierarchy

The principles a JayJay window layout should follow, derived from jj's own model and from macOS conventions. Principles are numbered P1–P10 and conventions M1–M9 so reviews and PRs can cite them.

## The model, outside in

jj's own structure already says what contains what:

```
Repo                      store: all changes, bookmarks, remotes, operation log
├─ Operation log          repo time: every view the repo has had (undo lives here)
├─ Bookmarks / remotes    named labels pointing at changes; remote tracking state
└─ Workspace (1..n)       a working-copy folder on disk + its @ change
   └─ (history graph, seen through a revset)
      └─ Change           stable change id; a description; author; parents
         ├─ Evolog        change time: each rewritten version (commit) of this change
         └─ Diff vs parents
            └─ File       path, kind (added/modified/moved/deleted), +/- counts
               └─ Hunks   the lines
```

Three things in this picture are easy to get wrong:

1. **Bookmarks are labels, not places.** In git you are "on a branch" and the branch defines the work. In jj you are on a change (`@`), and bookmarks are tags that point at some changes. They belong next to the changes they label, plus a summary of their sync state. They don't frame the whole window.
2. **There are two time axes.** The op log is repo time (what happened to the repo). The evolog is change time (how this one change evolved). Each belongs at its own level: the op log at the window, the evolog inside the change.
3. **The working copy is just a change.** `@` is the change whose files happen to live in the workspace folder. It has the same anatomy as every other change: description, metadata, files. It can be edited, but so can any mutable change.

## Principles

<a id="p1"></a>
1. **P1. Containment decides placement.** Scope narrows from left to right and top to bottom:
   - window chrome = repo + workspace
   - sidebar = history of the repo
   - detail header = one change
   - file column = that change's files
   - diff = one file

   If a piece of information belongs to a level, it sits inside that level's region, never beside it or above it.
<a id="p2"></a>
2. **P2. Each fact appears once, at its level.** The repo path, the bookmark, the file name and the IDs each have one home. Repeating them across regions adds noise without adding information.
<a id="p3"></a>
3. **P3. Actions sit with what they act on.**
   - repo actions (fetch, push, refresh): window toolbar
   - change actions (describe, commit, split): change header
   - file actions: file rows and the diff toolbar
<a id="p4"></a>
4. **P4. No status bar by default; each status fact goes to its own level.**
   - Last operation (with undo): window level, next to the op log entry point.
   - Working-copy file counts: already on the `@` change and in the file overview.
   - Conflict and divergence counts: badges or revset presets in the change list.
   - SSH connection: the title subtitle.
   - PR state: the bookmark it belongs to.

   A status bar comes back only if something has no better home.
<a id="p5"></a>
5. **P5. Viewing is the default, workflows are opt-in.** Review marks, message editing and diff editing are modes you enter, not chrome on every screen. Empty states read as calm, not as prompts.
<a id="p6"></a>
6. **P6. Identity is compact, detail on demand.** IDs show 8 characters with the unique prefix emphasised, as in `jj log`. Full IDs and emails appear on hover or copy, not in the layout.
<a id="p7"></a>
7. **P7. Colour means kind, the same everywhere, and follows jj's defaults:**
   - change IDs and bookmarks: magenta
   - commit IDs: blue
   - `@` and workspaces: green
   - added lines: green; removed lines: red
   - conflicts: red; divergence: orange

   Chrome stays neutral, and the accent colour is reserved for selection and focus. Colour is never decoration.
<a id="p8"></a>
8. **P8. Surfaces nest; they never continue into each other.**
   - Repo-scope navigation: the system sidebar.
   - The change: the detail area, on the reading background.
     - Change identity: the header, raised slightly above the detail's background.
     - File list and diffs: the reading background itself.

   Adjacent scopes never share a colour. If the sidebar and the header match, they read as one surface and the nesting is lost.
<a id="p9"></a>
9. **P9. Show content, not summaries of content.** The detail area shows the change's actual diffs: every file stacked when nothing is selected, one file when something is. The file list is the index and can be hidden with one toggle. A second list of the same files (a stat summary) just repeats the index.
<a id="p10"></a>
10. **P10. Alternate views replace content at the same level.** Evolog and annotate swap in for the diff inside the change. The op log swaps in for the history in the sidebar, or opens as a sheet. They never stack beside it.

## macOS conventions that reinforce this

These come from the HIG and from well-made Mac apps (Finder, Mail, Xcode, Notes). They answer "where" and "how it looks" with patterns people already know:

<a id="m1"></a>
1. **M1. The sidebar is a source list for navigation, not a workspace.** It holds the things you choose between (here: changes, filtered by revset), with quiet rows and a single accent-coloured selection. Forms and editors don't belong in it, which is why the commit box moved out.
<a id="m2"></a>
2. **M2. Use the system sidebar and let the OS style it.** Tahoe (macOS 26) floats it as an inset glass panel. Golden Gate (macOS 27) brings back full-height, edge-to-edge sidebars with a darker background, coloured icons and semi-bold selection. Apple says apps on the standard `NavigationSplitView` / `NSSplitViewController` pick this up with no code changes ([WWDC26 session 289](https://developer.apple.com/videos/play/wwdc2026/289/); [Six Colors](https://sixcolors.com/post/2026/07/first-look-macos-golden-gate-public-beta/), [MacStories](https://www.macstories.net/stories/macos-27-golden-gate-the-macstories-public-beta-preview/), [MacRumors](https://www.macrumors.com/2026/06/09/macos-golden-gate-liquid-glass/), [Michael Tsai](https://mjtsai.com/blog/2026/06/09/golden-gate-sidebars-and-toolbars/)). A custom flat sidebar would freeze today's look and miss those changes. On Tahoe, the undocumented `NSSplitViewItemSidebarDefaultsToFloatingAppearance = false` default already gives the full-height form. Don't add custom margins, rounded panels or a painted sidebar background. Golden Gate also has a user transparency slider, so don't assume a fixed level of glass.
<a id="m3"></a>
3. **M3. The window title names the document; the subtitle gives context.** Native: title = repo, subtitle = workspace (and the nearest bookmark, quietly). This avoids a custom picker label carrying everything. Use `navigationTitle` / `navigationSubtitle`, with the switcher as the title's menu.
<a id="m4"></a>
4. **M4. Toolbar layout is fixed by convention.**
   - Leading: navigation (sidebar toggle, at the sidebar's trailing edge).
   - Centre: the title, or one dominant control such as search or the revset field.
   - Trailing: a few frequent actions as monochrome SF Symbols with tooltips and menu equivalents.

   Rarely used actions live in menus. Settings is never a toolbar button.
<a id="m5"></a>
5. **M5. Every toolbar action and context-menu item also exists in the menu bar,** with its shortcut shown there. Toolbar buttons are a convenience, not the only route. That is why Finder and Settings could go.
<a id="m6"></a>
6. **M6. Metadata goes in a header or an inspector.** Mac apps show an item's properties either in a header above its content (Mail) or in a trailing inspector (Xcode, Finder Get Info). Change metadata fits the header today. The evolog, parents and bookmark details could later move into a toggled inspector rather than growing the header.
<a id="m7"></a>
7. **M7. Use standard controls for modes.**
   - Unified/split view: a segmented control.
   - Tree/flat list: a toggle in the list's header or the View menu.
   - Empty states: `ContentUnavailableView`, short and calm.
   - Edit: an "Edit" button that swaps in a form with Cancel/Save (or Done), Return to save, Escape to cancel.
<a id="m8"></a>
8. **M8. A status bar is unusual on the Mac.** Where apps have one (Xcode, BBEdit), it carries counts and transient progress, never identity or paths. Most Mac apps put that in the title subtitle, toolbar progress, or badges on the items themselves.
<a id="m9"></a>
9. **M9. Keep chrome quiet.** One accent colour (the system accent) for selection and focus. Body text in the primary colour, supporting text secondary, separators hairline. Colour comes from semantics (principle 7), not from chrome.
