# Version Control Guide

Load this file before changing JJ history, splitting or describing changes, managing bookmarks, fetching, or pushing. Load [Pull Request Workflow](pull-requests.md) before creating, updating, or landing PRs.

JayJay uses Jujutsu. There is no staging area; JJ auto-snapshots the working copy, identifies changes by change ID rather than commit hash, uses `@` for the working copy and `@-` for its parent, and allows history to be rewritten.

## Command Concurrency

Never run JJ-aware commands concurrently in the same workspace. Read-only commands may snapshot the working copy, so concurrent commands that start from the same operation can create divergent commits with one change ID.

- Serialize all `jj` commands, including `jj st`, `jj log`, and `jj diff`.
- Serialize `jayjay review ...` and any script or tool that opens the repository through JJ with other JJ-aware commands.
- Parallelize only commands known not to read, snapshot, or update JJ's working-copy or operation state.
- If divergence appears, compare each divergent commit to `@` by commit ID and abandon only snapshots proven stale; never abandon every commit for the shared change ID.

## Workspace Policy

Use the current JJ workspace for normal and focused work. Create a sibling workspace only for a large or long-running session where isolation is materially useful, or when the user explicitly requests one. Do not create a workspace merely to keep a routine change separate.

When a temporary session is complete, finish or preserve its change as requested, then run `jj workspace forget <name>` so its empty working-copy commit does not remain in the graph. Forgetting workspace metadata does not delete the sibling directory; remove files only when that cleanup is authorized.

## Common Commands

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

Keep unrelated work in the current working copy unless the user asks to split or commit it. Split by behavior or responsibility, not merely by file boundaries. Use a pushed bookmark and JayJay's **Pull Request on GitHub** or **Pull Request on Codeberg** action for PRs.
