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
| Review marks and notes | `jayjay-review`; SwiftUI via UniFFI; GPUI and CLI directly | `review_store.json` in the shared config directory | File/hunk review marks keyed by `change_id|path`, content identities, and local review notes including path, side, line, anchor context, body, timestamps, and resolution state. |
| SwiftUI settings and history | SwiftUI-only `AppSettings` | `UserDefaults` for bundle `dev.hewig.jayjay` | Appearance and font, diff options, layout, confirmations, onboarding, editor/terminal choices, update channel, sponsorship state, up to 12 recent repositories, and the last opened repository. |
| SwiftUI auxiliary state | SwiftUI components | The same `UserDefaults` domain | Command-palette position. A legacy `jayjay.reviewedFiles` blob is imported once into the shared review store and then removed. |
| GPUI settings and history | GPUI-only Rust `AppConfig` | `config.toml` in the platform config directory | Appearance and font, diff options, layout, tools, feature confirmations, onboarding, update channel, window bounds/maximized state, and up to 12 recent repositories. |

Recent repositories are history, not projects. Each shell owns its own recent list. Pins are persistent projects and are intentionally shared by both shells.

## Shared JSON Stores

### Pinned repositories

The canonical implementation is `crates/jayjay-core/src/repositories.rs`.

```json
{"repositories":["/Users/example/work/project-a","/Users/example/work/project-b"]}
```

- Repository paths are canonicalized before lookup or mutation so aliases do not create duplicate pins or windows.
- Paths that cannot be represented as UTF-8 are not pinned, because the shared JSON and UniFFI contract cannot round-trip them; the store never writes a lossy replacement path.
- A read fingerprints the small local JSON file and reparses it when another process changes its contents, including equal-length replacements on coarse-timestamp filesystems.
- Deleting the file is a state change: long-lived readers become empty, and the next mutation starts from that empty state instead of restoring deleted pins.
- Every mutation refreshes from disk before applying its change, preventing a long-lived shell from overwriting a newer write from the other shell.
- Writes use a unique sibling temporary file followed by rename, so readers do not observe partial JSON.
- A failed write leaves the last loaded state published to the shell; an unsaved mutation is never reported as persisted.
- Malformed JSON is renamed to `repositories.json.corrupt`; the store then starts empty without overwriting the preserved file.
- `JAYJAY_REPOSITORIES_PATH` overrides the canonical path for tests and diagnostics.

### Review marks and notes

The canonical implementation is `crates/jayjay-review/src/store/`. See [Review State Guide](review-state.md) for reconciliation behavior and CLI operations.

```json
{
  "reviewed": {
    "change-id|src/main.rs": {
      "identity": "content-identity",
      "file_marked": false,
      "hunks": [0, 2]
    }
  },
  "notes": []
}
```

- Marks contain the content identity captured when a file or hunk was reviewed. Notes additionally contain their anchor, user-authored body, timestamps, and resolved state.
- Reads and mutations use the same stale-refresh and atomic temp-file/rename rules as the pin store.
- Malformed JSON is preserved as `review_store.json.corrupt` before falling back to an empty store.
- Unknown top-level fields, unknown note entries, and unknown fields inside parseable notes survive a save so different JayJay/CLI versions can safely share the file.
- `JAYJAY_REVIEW_STORE_PATH` overrides the canonical path.

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
- Deleting pins or recent history does not delete repositories. Deleting review storage does not change jj history; it only removes JayJay's local review state.
- Do not log full storage documents or transmit them in telemetry.

## Adding or Changing Storage

1. Decide whether the data is shared domain state, shell-local preference/history, or disposable derived data.
2. Put shared domain state in Rust with one canonical path and API. Add UniFFI functions only as a thin Swift bridge.
3. Define missing-file, malformed-file, unknown-field, and migration behavior before shipping the schema.
4. Use atomic replacement for user-authored shared files and refresh from disk before mutating a long-lived snapshot.
5. Add a path override or in-memory constructor so tests cannot write production data.
6. Test round trips, malformed input, duplicate/normalization rules, and two independently loaded writers when the store is cross-process.
7. Update this inventory and [Shell Feature Parity Guide](shell-parity.md) when storage ownership or shell-sharing behavior changes.
