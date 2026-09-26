# Video Guide

Load this file to record a video of JayJay or to write a scene for one. A scene is a JSON script; `director` (the Swift package in `video/`) builds the scene's fixture, launches the debug app, plays the scene through the Accessibility API, and captures the window. Neither the app nor a test target changes when a scene does.

## Example

`video/examples/hero.json` records the clip that the website's hero plays on the screenshot fixture: it opens a change's diff, marks a working-copy file reviewed, and returns to the opening frame. `hero-dark.json` extends it and only switches the appearance:

```json
{
  "extends": "hero.json",
  "defaults": { "jayjay.appearanceMode": "dark" }
}
```

The base file sets the fixture, a 1280×650 window (2560×1300 pixels, the hero's aspect ratio), the fixture's `env`, and the launch `defaults` the release screenshots use. Its steps name graph rows by subject and files by path:

```json
{ "click": { "row": "prefer the fastest available route" }, "label": "loop" },
{ "click": { "file": "src/routes.rs" } },
{ "beat": 2 },
{ "click": { "idPrefix": "dag.row.", "index": 0 } },
{ "click": "file.review.src/stations.rs" }
```

Record both with:

```bash
just shell::record video/examples/hero.json video/examples/hero-dark.json
```

`just shell::site-tour` publishes both takes from 1.0 s on: `director web` converts each to H.264, the recipe writes the first frame to `docs/imgs/tour*.webp`, and uploads the clip to the `jayjay-media` R2 bucket (served at `media.hewig.dev`) under a content-hashed name. Put the printed URLs and click cues into `docs/js/tour.js`.

## Recording

`just shell::record` builds the debug app and `director`, then records every scene in `video/scenes`, or the scene names and `.json` paths it is given. Each take writes to `build/recordings/`:

- `<scene>.mov`: the window at the display's pixel scale in HEVC, constant 60 fps, without the pointer.
- `<scene>.cues.jsonl`: the time of every click, key, and paste in seconds, and each click's position as fractions of the clip, for drawing the pointer in the edit.
- `<scene>.log`: the app's error output.

A failed take is kept as `<scene>.failed.mov` and never replaces a good clip; `--no-capture` leaves every output alone. Ctrl-C stops after the current step and finishes the clip as `<scene>.failed.mov`, releases held modifiers, restores the clipboard, and quits the app; a second Ctrl-C quits at once and leaves an unfinished `<scene>.partial.mov`.

A take owns the screen. It launches the app in the background, waits until the keyboard and mouse have been idle for 2 s, brings the app forward once, and then stops, without clicking or typing, if another window, including a sheet or second window of the app, covers a target, or the app loses focus at any point; such a stop also ends the run, so the next scene never opens in front of someone. After a paste it restores the clipboard, unless something was copied since. Record only when nobody, including another agent session, is using the screen. The terminal needs Screen Recording and Accessibility.

## Scenes

Top-level keys, all optional; `extends` names a base file whose keys a scene overrides, merging `env` and `defaults` key by key. Relative paths resolve against the directory `director` runs in.

| Key | Meaning |
| --- | --- |
| `setup` | Command run before launch; `${app}` is the app bundle path |
| `repo` | Repository the app opens (`--repo`) |
| `rows` | The fixture's `rows.tsv`, for `row` and `lane` targets |
| `window` | `[x, y, width, height]` of the captured window, in points |
| `env` | Environment for the app |
| `defaults` | Launch arguments; strings are quoted for the argument domain automatically, and `${window}` becomes `window` as an AppKit frame string for the app's window-frame defaults |
| `prelude` | Steps played before `steps`, usually from the base file |
| `steps` | The take |

A step is one action plus an optional `label` (the cue name, on clicks, `menu`, `key`, and `paste`) and `timeout` (seconds to wait for the target, default 10, on clicks, `menu`, `wait`, and `waitGone`):

| Action | Does |
| --- | --- |
| `click`, `doubleClick`, `rightClick` | Click the target's center, or its `x`/`y` offset from the top-left |
| `menu` | Click the open menu's item with this title |
| `key` | Press a combination such as `cmd+shift+o`, `escape`, `cmd+down` |
| `paste` | Put text on the pasteboard and press ⌘V; typing goes through the input method |
| `wait`, `waitGone` | Wait until the target is on screen, or gone |
| `beat` | Hold for the edit, in seconds |

A target is a string (an accessibility identifier from `AID` in `shell/mac/Sources/JayJay/Shared/AccessibilityIdentifiers.swift`) or an object whose matchers must all hold: `id`, `idPrefix`, `label` (title, description, placeholder, or value), `role` (`button` or `AXButton`), `menuItem`, `row` (a graph row by subject), `lane` (an Overview lane by its head's text in the rows file: the subject, or a workspace name such as `default@` when the fixture writes working-copy names there; a divergent head has no lane name), `file` (a file-list row by path), and `index` for the nth match. `line` is a regular expression that picks the first matching line of the element's text, for clicks such as a diff gutter row.

## Writing a Scene

1. Write the steps you know, then play them without capture and leave the app open: `video/.build/release/director record --keep-open --no-capture <scene>.json`.
2. `video/.build/release/director inspect` prints the open app's elements with role, identifier, text, and frame.
3. Add steps and repeat; record for real once the scene plays through.

Graph rows and Overview lanes expose no text, so name them with `row` and `lane`. Pace with `beat`s: the edit cuts around them.

