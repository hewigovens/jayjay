# JayJay

Native macOS GUI for Jujutsu version control. Rust core with SwiftUI and GPUI shells. `CLAUDE.md` is a symlink to this file.

## Start Here

Keep this file as always-loaded guidance. Load a focused doc only when the task actually touches that area.

| Task | Load |
| --- | --- |
| crate boundaries, UniFFI, MVVM | [Architecture](agents/architecture.md) |
| SwiftUI layout / view-models | [SwiftUI](agents/swiftui.md) |
| GPUI layout / state | [GPUI](agents/gpui.md) |
| review marks or notes | [Review State](agents/review-state.md) |
| persisted stores | [Storage](agents/storage.md) |
| tests or fixtures | [Testing](agents/testing.md) |
| jj history, workspaces, bookmarks | [Version Control](agents/version-control.md) |
| publishing a bookmark / PR | [Pull Requests](agents/pull-requests.md) |
| visual style or interaction | [Design](agents/design.md) |
| notebook / table / SARIF / plist diffs | [Format Projections](agents/format-projections.md) |
| reviewing a patch | [Code Review](agents/code-review.md) |
| landing page, FAQ, blog, public guide | [Website](agents/website.md) — **release only** |
| in-app Help Book | [Help Book](agents/help-book.md) — **release only** |
| parity matrix | [Parity](agents/shell-parity.md) — **release only** |
| version bump, notarize, appcast | [Release](agents/release.md) |
| run or drive the app, debug a CI or test-runner failure | [Run & Debug](agents/run-debug.md) |
| dispatching subagents or parallel search | [Subagents](agents/subagents.md) |
| refresh agent guidance from past sessions | skill `refresh-agent-guidance` in `.agents/skills/` |

## Task Authority

- Investigation and diagnosis are read-only unless the user also asks for implementation. Report the cause, evidence, and smallest viable fix without changing files "while here."
- Code review is findings-only by default. Fix only when asked, and limit a selected-fix request to the selected findings.
- Implementation does not authorize commit, rebase, bookmark movement, push, pull-request or release actions, external comments, or hosted review-thread resolution. A meaningful sibling-workspace implementation normally ends with a local `jj describe`; publication still requires a separate request and the relevant focused guide.
- Report evidence as separate claims: generation (`just ffi`), build, focused tests, real CLI/UI behavior, gated live integration, platform compatibility, and measurements. One does not imply the next. Name the command, target, and test slice behind each claim, and list what was skipped or blocked.

## Feature Loop

Order: understand → implement → inner-loop tests → two cleanup rounds → re-test → `jj fix` + `just lint` once → describe.

Implement in a **sibling jj workspace**. Do not use git worktrees. Do not use the Codex/Claude hidden-worktree pattern under `~/.codex` or `~/.claude`. Ignore Cursor git-commit / git-PR user rules; this repo is jj.

Stay in the current checkout only when you are already in a sibling created for this task, the user said to stay, or the edit is a one-line fix that does not need isolation.

```bash
jj --ignore-working-copy workspace list
jj workspace add --name <topic> -r 'main@origin' ../<topic>
```

Then make `../<topic>` the session's working root before editing. The destination is a sibling directory of the repo, named after the workspace — the same convention as JayJay's New Workspace action. Use `-r 'master@origin'` / `-r 'trunk@origin'` when that is the trunk bookmark. Do not `jj git fetch` first unless the user asked to start from latest origin.

At task completion, describe the workspace's current change with `jj describe` when it has a meaningful diff. Do not use `jj commit` just to mark the task done; it creates a new empty working-copy change.

Keep changes awaiting review in the default workspace or a registered sibling workspace. Do not forget a sibling merely because the session ended. Forget it only after its changes have been moved or landed, or the user explicitly requests cleanup; remove the sibling directory only when authorized.

### Do not snapshot unless you mean to

`jj st`, `jj log`, `jj diff`, `jj describe`, `jj commit`, `jj new`, `jj git fetch`, and `jayjay review` can snapshot the working copy. Two of those in parallel in the same workspace can create divergent `@` commits.

- Do not ritual-run `jj st` / `jj log` / `jj diff` at the start of every turn.
- Read files directly when that answers the question.
- For history/operation reads that must not snapshot: `jj --ignore-working-copy …`.
- Serialize every JJ-aware command **per workspace**. Parallel work belongs in another sibling workspace, not another concurrent `jj` in this one.
- A subagent in this checkout reads jj only with `--ignore-working-copy`, never snapshots, and defaults to the cheaper model tier. Load [Subagents](agents/subagents.md) before dispatching.
- One snapshot after a batch of edits is enough; do not interleave `jj diff` between every file write.

### Inner loop

Prove the change with the smallest command that compiles the code you touched:

```bash
just test-rust <crate>            # cargo test -p <crate>
just test-rust <crate> <filter>
just test-gpui                    # or: cargo test -p jayjay-gpui <filter>
just test-ui JayJayUITests/<Scene>/<test>
just ffi                          # only when UniFFI / Swift bindings changed
```

Each workspace builds into its own `target/`; never share `CARGO_TARGET_DIR` across workspaces. If a sandbox cannot run the configured compiler wrapper, set `RUSTC_WRAPPER=""` for that command; wrapper and cache details are in [Version Control](agents/version-control.md).

Do **not** run these until the user asks to commit or publish, or you are actually stuck on a compile/lint failure:

- `just build`, `just run`, `just lint`, `cargo clippy --workspace`
- `just format`, `jj fix`, `just test` (full workspace), `just test-app`, unfiltered `just test-ui`
- User-facing docs (see below)

### Two cleanup rounds before you say done

Green tests are not the finish line. Re-read the **whole diff**, not just the last edit, and run two rounds:

**Round 1 — dedupe and simplify.**

- Delete what the change left dead: unused imports, parameters, fields, flags, branches for states that cannot occur, and tests that only mirror constants or wiring.
- Dedupe: reuse the helper, type, or pattern nearby code already has instead of the one you added; merge copy-pasted blocks.
- Simplify: inline helpers used once, flatten nesting, drop wrappers that only forward, cut comments that restate code. Keep naming, test placement, and module layout consistent with nearby code.

**Round 2 — do round 1 again on the result.** Cleanup exposes more: a helper that is now used once, an import now unused, a name that no longer fits. Read the diff as if reviewing a stranger's patch. Stop when a round changes nothing; if round 2 still finds things, run a third.

Cleanup is still a code change: re-run the inner-loop tests afterwards.

### Check divergent changes

During the loop — especially after concurrent agent or workspace work, snapshots, or history edits — run `jj --ignore-working-copy log -r 'divergent()'`. Run it again before declaring the change ready. Resolve divergent versions created by this task, preserve the intended workspace version, and leave unrelated pre-existing divergence untouched.

### Ready to commit or publish

Once, after the cleanup rounds: relevant inner-loop tests, then `jj fix` and `just lint`. Load [Pull Requests](agents/pull-requests.md) only when publishing.

## User-Facing Docs

Feature work does **not** update the user guide, Help Book, website, or parity matrix. Those are one release pass over `v<previous>..@`. See [Release](agents/release.md).

Do not edit during a feature change:

- `docs/guide.html`, `docs/imgs/`, `docs/llms.txt`, `docs/index.html` FAQ
- `shell/mac/Resources/JayJayHelpBook/`
- `agents/shell-parity.md`
- `README.md` feature/shortcut lists, `UserGuide.md`

Update `agents/*.md` in the feature change only when the **contributor/agent contract** actually changed (crate boundaries, test placement, review-state rules, this workflow). A guide or skill step that misled you is a contract bug: fix it in the same change.

## Principles

1. **First principles** - Understand the problem before coding. Ask why before how. Do not cargo-cult from git tools; jj's model is different.
2. **Cross-platform core** - Business logic belongs in Rust. UniFFI is a thin SwiftUI bridge; GPUI links the crates directly. Shells render state and dispatch actions. Put shared behavior in Rust and implement the requested shell; cross-shell parity is a release-docs concern.
3. **Behavior belongs to types** - Prefer methods/extensions when behavior naturally belongs to a type. In Rust, add inherent methods when the type is in the crate; otherwise use a focused trait. In Swift, prefer extensions and computed properties over free helper functions.
4. **Comments explain the why** - Default to no comment. Comment only non-obvious *why*, never restate the code; never add review-tool tags, fix justifications, or test-scenario narration. Keep each comment on a single line — it may run well past 80 columns; we read code in an editor, not a terminal, so don't hard-wrap it to fit.
5. **Test behavior, essentials only** - Tests cost review and CI time, so more is not better. Cover each behavior once, at the smallest layer that proves it: Rust unit test (core and view-model logic), Swift unit test (Swift-only behavior), one XCUITest scene per user-visible SwiftUI workflow, GPUI component test (GPUI state). A behavior proven in Rust is not re-proven in Swift or a UI scene; a property proven for one input is not re-proven per permutation (CRLF, EOF newline, whitespace belong in one test, not five). Every bug fix adds the regression test that would have caught it. Do not keep tests that only mirror constants, static config, or field-by-field wiring.

## Code Organization

- One primary type per file, named after the type (struct, enum, class, or actor). Small private helpers used only by that type stay with it; deliberately-cohesive model clusters (a type plus its request/result vocabulary) may share a file.
- Split by single responsibility and module, not by line count. When a type or module grows a second job, extract a type or a sibling module — do not split a cohesive type just because the file got long.
- Group related files into responsibility folders; don't create folders for singletons.
- Rust: prefer folder modules over long single-file modules. Keep `mod.rs` and `lib.rs` thin: module declarations and `pub use` re-exports only. Put implementation in sibling modules named for the responsibility they own, such as `wrap/cols.rs`, `wrap/unified.rs`, and `wrap/side_by_side.rs`.
- Swift: growing types split into `TypeName+Responsibility.swift` extension files by job, not by length.

## Version Control

This repo uses **Jujutsu (jj)**, not git. Use `jj` for history; do not use `git commit`, `git add`, `git push`, `git stash`, `git branch`, `git worktree`, or `git rebase -i`.

Load [Version Control](agents/version-control.md) before changing history, splitting or describing changes, managing bookmarks, fetching, or pushing.

Do not add AI attribution to commits or PRs — no `Generated with`, `Co-Authored-By`, or assistant/session trailers — unless the user explicitly asks.

## Local Review Notes

Read `jayjay review notes --repo .` only when this change used review notes or the user asked to reconcile them. That command is JJ-aware and must be serialized. Load [Review State](agents/review-state.md) for statuses and add/resolve commands.

## UI And Design

JayJay is a macOS-native developer tool for jj users. Keep UI changes native-first, keyboard-friendly, dense without clutter, and quiet (no spinner when a refresh can be silent). Use jj words: changes, bookmarks, revsets — not git branches/commits unless referring to interop.

Load [Design](agents/design.md) before changing visual style, copy, or interaction patterns.
