# Storage Guide

Load this file before adding, removing, migrating, or changing persisted JayJay data. For ownership boundaries also load [Architecture Guide](architecture.md); for marks and notes load [Review State Guide](review-state.md).

## Storage Boundaries

- Data that SwiftUI, GPUI, or the CLI must agree on belongs in a canonical Rust store. SwiftUI reaches it through UniFFI; GPUI and the CLI link it directly.
- Shell preferences and repository history stay shell-local. SwiftUI uses its own `AppSettings` backed by `UserDefaults`; it does not consume GPUI's Rust `AppConfig` TOML schema. Do not silently merge these schemas.
- Repository contents and jj metadata remain owned by the repository. JayJay does not put app preferences, pins, or review notes inside `.jj`.
- Temporary render artifacts are derived caches. They must be safe to delete and must never be the only copy of user-authored data.

Rust stores resolve platform-native directories through `directories::ProjectDirs::from("dev", "hewig", "jayjay")`. On macOS, the shared config directory is `~/Library/Application Support/dev.hewig.jayjay/`. Use `ProjectDirs` in code instead of reconstructing these paths.

## Persistent Data Inventory

| Data | Owner and consumers | Storage | Contents |
| --- | --- | --- | --- |
| Pinned repositories | `jayjay-core`; SwiftUI via UniFFI; GPUI directly | `repositories.json` in the shared config directory | An ordered `repositories` array of canonical absolute UTF-8 repository paths. New pins are inserted first; empty paths and exact duplicates are removed on load. |
| Review marks and notes | `jayjay-review`; SwiftUI via UniFFI; GPUI and CLI directly | `review_store.json` in the shared config directory | File/hunk review marks keyed by `change_id|path`, content identities, hunk-baseline fingerprints and group states, and local review notes including path, side, line, anchor context, body, timestamps, and resolution state. |
| SwiftUI settings and history | SwiftUI-only `AppSettings` | `UserDefaults` for bundle `dev.hewig.jayjay` | Appearance and font, diff options, layout, confirmations, onboarding, editor/terminal choices, update channel, sponsorship state, up to 12 recent repositories, and the last opened repository. |
| SwiftUI auxiliary state | SwiftUI components | The same `UserDefaults` domain | Command-palette position; window frames per scene (`jayjay.windowFrame.<scene id>`, applied to the first window of that scene); pane widths (`jayjay.sidebarWidth` and `jayjay.secondaryPaneWidth`, which falls back to the legacy `jayjay.fileColumnWidth` when unset; both fitted to the window when shown). UI tests mask the frame and pane keys through launch arguments; `scripts/ui-test-fixtures.sh` deletes them per run. |
| GPUI settings and history | GPUI-only Rust `AppConfig` | `config.toml` in the platform config directory | Appearance and font, diff options, layout, tools, feature confirmations, onboarding, update channel, window bounds/maximized state, and up to 12 recent repositories. |

Recent repositories are history, not projects. Each shell owns its own recent list. Pins are persistent projects and are intentionally shared by both shells.

## Shared JSON Stores

### Pinned repositories

The canonical implementation is `crates/jayjay-core/src/repositories.rs`.

```json
{"repositories":["/Users/example/work/project-a","/Users/example/work/project-b"]}
```

- Canonicalize paths before lookup or mutation; never pin non-UTF-8 paths (the JSON/UniFFI contract cannot round-trip them).
- Reads fingerprint the file contents and reparse when another process changed them, including equal-length replacements on coarse-timestamp filesystems; every mutation refreshes from disk first, then writes via temp file + rename. A failed write must not report the unsaved mutation as persisted.
- Deleting the file empties long-lived readers; the next mutation starts from empty instead of restoring old pins.
- Malformed JSON is renamed to `repositories.json.corrupt`; the store starts empty without overwriting that file.
- `JAYJAY_REPOSITORIES_PATH` overrides the path for tests.

### Review marks and notes

The canonical implementation is `crates/jayjay-review/src/store/`. See [Review State Guide](review-state.md) for reconciliation behavior and CLI operations.

```json
{
  "reviewed": {
    "change-id|src/main.rs": {
      "identity": "content-identity",
      "state": {
        "kind": "groups",
        "algorithm_version": 1,
        "groups": [{"digest": "fingerprint-hex", "state": "reviewed"}],
        "removed_reviewed": []
      }
    }
  },
  "notes": []
}
```

- Same refresh-before-mutate and atomic temp-file/rename rules as the pin store. Malformed JSON is preserved as `review_store.json.corrupt`.
- Unknown top-level fields, unknown note entries, and unknown fields inside parseable notes and review entries survive a save so mixed JayJay/CLI versions can share the store. Entries in the pre-tag `file_marked`/`hunks` shape migrate to file/hunk states on load; anything else unreadable and the obsolete `review_baselines` map are dropped.
- One tagged review entry per `(change_id, path)`: a whole-file mark, snapshot-less hunk indices, or fingerprinted groups. Persist hashes and review state only — never whole-file contents.
- `JAYJAY_REVIEW_STORE_PATH` overrides the path for tests.

Neither JSON store is a synchronization service. The refresh-before-mutate contract prevents ordinary cross-process lost updates, but simultaneous writes are still last-rename-wins. Keep mutations short and route all writes through the Rust store.

## Shell-local Preferences

### SwiftUI

`shell/mac/Sources/JayJay/App/Config/AppSettings.swift` reads all values at initialization and writes each changed property immediately to `UserDefaults`. Repository-history paths are standardized lexically without filesystem access, deduplicated, ordered most-recent-first, and capped at 12.

Apple owns the physical preferences-file lifecycle. Code and tests should address values by key through `UserDefaults`, not read or edit the plist directly. Tests must inject an isolated `UserDefaults` suite.

### GPUI

`shell/gpui/src/app/config/` owns the serde-backed TOML schema. `AppConfig::load()` returns defaults when the file is absent or malformed; `config::update` clones the current configuration, applies one mutation, saves the complete TOML document, and publishes the new process-global snapshot.

Missing TOML fields use defaults and unknown fields are ignored. Tests must install `AppConfigStore::new_ephemeral`, which updates in memory without touching the user's real file.

## Telemetry Data

Telemetry is enabled by a shell-local preference and disabled in debug builds. Local storage contains a random installation secret and the last successfully sent UTC day. The secret never leaves the device: it derives separate SHA-256 identifiers scoped to the current UTC day and month.

Activity requests contain platform, app version/build, OS/version, architecture, and the rotating identifiers. They do not contain repository paths, file names, review content, commands, or a permanent installation identifier.

## Disposable Files

- Materialized image diffs: the system temporary directory under `jayjay-images/`.
- GPUI SVG previews: the system temporary directory under `jayjay-svg-previews/`.
- GPUI tool-test/process artifacts: process-specific directories under the system temporary directory.

These files are content-derived or process-scoped. Callers must tolerate their absence and regenerate them as needed.

## Privacy and Deletion

- `repositories.json`, both recent lists, and last-opened state reveal local absolute repository paths.
- `review_store.json` may contain repository-relative file paths, line excerpts, surrounding context, and user-authored note bodies. Treat it as private source-review data.
- Deleting pins or recent history does not delete repositories. Deleting review storage does not change jj history; it only removes JayJay's local review state. Settings › Diff › Review › Clear does the same in-app through `ReviewStore::clear_all`, keeping unknown top-level fields.
- Do not log full storage documents or transmit them in telemetry.

## Adding or Changing Storage

1. Decide whether the data is shared domain state, shell-local preference/history, or disposable derived data.
2. Put shared domain state in Rust with one canonical path and API. Add UniFFI functions only as a thin Swift bridge.
3. Define missing-file, malformed-file, unknown-field, and migration behavior before shipping the schema.
4. Use atomic replacement for user-authored shared files and refresh from disk before mutating a long-lived snapshot.
5. Add a path override or in-memory constructor so tests cannot write production data.
6. Test round trips, malformed input, duplicate/normalization rules, and two independently loaded writers when the store is cross-process.
7. Update this inventory when storage ownership or shell-sharing behavior changes. Refresh [Shell Feature Parity Guide](shell-parity.md) in the release shipped-docs pass if a user-visible workflow changed.
