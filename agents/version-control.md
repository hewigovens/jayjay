# Version Control Guide

Load this file before changing JJ history, splitting or describing changes, managing bookmarks, fetching, or pushing. Load [Pull Request Workflow](pull-requests.md) before creating, updating, or landing PRs.

JayJay uses Jujutsu. There is no staging area; JJ auto-snapshots the working copy, identifies changes by change ID rather than commit hash, uses `@` for the working copy and `@-` for its parent, and allows history to be rewritten.

## Command Concurrency

Never run JJ-aware commands concurrently in the same workspace. Read-only commands may snapshot the working copy, so concurrent commands that start from the same operation can create divergent commits with one change ID.

- Serialize all `jj` commands, including `jj st`, `jj log`, and `jj diff`.
- Serialize `jayjay review ...` and any script or tool that opens the repository through JJ with other JJ-aware commands.
- Parallelize only commands known not to read, snapshot, or update JJ's working-copy or operation state.
- Parallel **workspaces** may run jj at the same time; parallel jj in **one** workspace may not.
- If divergence appears, compare each divergent commit to `@` by commit ID and abandon only snapshots proven stale; never abandon every commit for the shared change ID.

## Avoid Needless Snapshots

The snapshot rules are in `AGENTS.md` (Feature Loop → Do not snapshot unless you mean to). Reads that must not snapshot the working copy:

```bash
jj --ignore-working-copy workspace list
jj --ignore-working-copy log --limit 10
jj --ignore-working-copy op log --limit 5
```

`--ignore-working-copy` is wrong for commands that should see or record the current files (`jj describe`, `jj commit`, `jj diff` of the working copy, `jj new`). Do not fetch, log, or diff "just in case" before creating a sibling workspace.

## Workspace Policy

The sibling-workspace rule and the `jj workspace add` recipe are in `AGENTS.md` (Feature Loop). Details that file leaves out:

- Why a sibling: agent commands then snapshot that working copy, not the user's current `@`.
- `git worktree`, Cursor git worktrees, and the Codex/Claude hidden-worktree pattern (`~/.codex/**`, `~/.claude/**`, or any other home-dir agent worktree) are all out; isolation belongs in a named sibling next to this repo.
- Do not create a workspace merely to keep two edits separate inside one session — that is `jj new` / `jj split`.
- Pin the new workspace to a specific change with `-r <rev>` when continuing existing work.
- When the session is complete, finish or preserve its change as requested, then `jj workspace forget <topic>` so its empty working-copy commit does not remain in the graph. Forgetting workspace metadata does not delete the sibling directory; remove files only when that cleanup is authorized.

## Workspace Build Isolation

Each JJ workspace gets its own Cargo `target/` by default. Preserve that isolation for concurrent builds; do not share a `CARGO_TARGET_DIR` because Cargo will serialize processes on the shared build-directory lock.

Preserve the configured compiler wrapper for Rust-backed commands. Kache is preferred for concurrent workspaces because it normalizes checkout paths and restores cached libraries, build outputs, and executables into each isolated target. On filesystems that support clones, restored outputs share physical storage with the cache until either copy changes.

Keep each workspace's default target when builds run concurrently. Let the Kache configuration manage incremental artifacts; do not force a shared target or override incremental settings.

Compiler caches do not replace workspace cleanup. When authorized to remove a completed sibling directory, remove its `target/` with it so old per-workspace artifacts do not accumulate. If a sandbox cannot use the configured wrapper or daemon, use `RUSTC_WRAPPER=""` for that command rather than changing the developer's global Cargo or cache configuration.

## Common Commands

Commands that snapshot the working copy (serialize these):

```bash
jj st
jj log --limit 10
jj diff
jj describe -m "summary" -m "body"
jj commit -m "summary" -m "body"
jj squash
jj split FILE -m "summary" -m "body"
jj edit <rev>
jj bookmark set <name> -r <rev>
jj git fetch
jj git push --bookmark <name>
jj fix
```

Filesets for `jj split` are positional; do not pass `--paths`.

Keep unrelated work in the current working copy unless the user asks to split or commit it. Split by behavior or responsibility, not merely by file boundaries. Use a pushed bookmark and JayJay's **Pull Request on GitHub** or **Pull Request on Cursor** action for PRs.
