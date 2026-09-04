# Version Control Guide

Load this file before changing JJ history, splitting or describing changes, managing bookmarks, fetching, or pushing. Load [Pull Request Workflow](pull-requests.md) before creating, updating, or landing PRs.

JayJay uses Jujutsu. There is no staging area; JJ auto-snapshots the working copy, identifies changes by change ID rather than commit hash, uses `@` for the working copy and `@-` for its parent, and allows history to be rewritten.

## Command Concurrency

Never run JJ-aware commands concurrently in the same workspace. Read-only commands may snapshot the working copy, so concurrent commands that start from the same operation can create divergent commits with one change ID.

- Serialize all `jj` commands, including `jj st`, `jj log`, and `jj diff`.
- Serialize `jayjay review ...` and any script or tool that opens the repository through JJ with other JJ-aware commands.
- Parallelize only commands known not to read, snapshot, or update JJ's working-copy or operation state.
- Parallel **workspaces** may run jj at the same time; parallel jj in **one** workspace may not.
- A subagent in this checkout is not a sibling workspace. It must not run `jj` or `jayjay review`. Load [Subagents](subagents.md) before dispatching workers.
- Running JayJay instances (SwiftUI or GPUI) that watch the repository snapshot on every filesystem event and fork the working-copy change while files churn. Quit them before divergence cleanup or history rewrites; otherwise operation reconciliation resurrects the stale siblings.
- If divergence appears, compare each divergent commit to `@` by commit ID and abandon only snapshots proven stale; never abandon every commit for the shared change ID.

## Avoid Needless Snapshots

The snapshot rules are in `AGENTS.md` (Feature Loop → Do not snapshot unless you mean to). Reads that must not snapshot the working copy:

```bash
jj --ignore-working-copy workspace list
jj --ignore-working-copy log --limit 10
jj --ignore-working-copy op log --limit 5
```

`--ignore-working-copy` is wrong for commands that should see or record the current files (`jj describe`, `jj commit`, `jj diff` of the working copy, `jj new`). Do not fetch, log, or diff "just in case" before creating a sibling workspace.

Keep scratch output outside the workspace tree — temporary `HOME` directories, test artifacts, agent logs — because the next snapshot adds every untracked file to the change.

## Workspace Policy

The sibling-workspace rule and the `jj workspace add` recipe are in `AGENTS.md` (Feature Loop). Details that file leaves out:

- Why a sibling: agent commands then snapshot that working copy, not the user's current `@`.
- `git worktree`, Cursor git worktrees, and the Codex/Claude hidden-worktree pattern (`~/.codex/**`, `~/.claude/**`, or any other home-dir agent worktree) are all out; isolation belongs in a named sibling next to this repo.
- Do not create a workspace merely to keep two edits separate inside one session — that is `jj new` / `jj split`.
- Pin the new workspace to a specific change with `-r <rev>` when continuing existing work.
- Sibling workspaces are not colocated (no `.git`), so `gh` cannot infer the repository there. Set `GH_REPO=hewigovens/jayjay` for `gh` commands and `just shell::publish` outside the main checkout.
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

Keep unrelated work in the current working copy unless the user asks to split or commit it. Split by behavior or responsibility, not merely by file boundaries. Use a pushed bookmark and JayJay's **Pull Request on GitHub** or **Pull Request on Cursor** action for PRs.

## Scripting jj

- Filesets for `jj split` are positional; do not pass `--paths`.
- Scripted `jj describe`, `jj commit`, and `jj squash` must pass `-m` (or `--use-destination-message`); without it jj waits on an editor forever.
- Never pipe a mutating jj command through `head` or another early-exiting filter: SIGPIPE can kill jj after it prints the result but before the operation is persisted, so the command looks successful and did nothing. Redirect to a file instead.
- To see what `jj fix` rewrote, list the working copy's evolog commit ids (the template exposes `commit`, not a top-level `commit_id`) and diff two of them:

```bash
jj --ignore-working-copy evolog -r @ --no-graph -T 'commit.commit_id().short() ++ "\n"'
jj --ignore-working-copy diff --from <older> --to <newer> --git
```
