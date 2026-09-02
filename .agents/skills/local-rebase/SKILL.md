---
name: local-rebase
description: Rebase a local feature stack onto updated main without publishing. Use for "rebase onto main", "keep it local", or "no need to push".
argument-hint: "[feature-bookmark]"
disable-model-invocation: true
---

# Local Rebase

Moves a feature stack onto current `main` and leaves the remote bookmark and any pull request untouched. Load [Version Control](../../../agents/version-control.md) first; publication needs a separate request per `AGENTS.md` Task Authority.

## When to use

- The user names a feature bookmark and asks to rebase it onto `main` without pushing.
- Not for updating a pull request or rewriting a shared remote bookmark.

## Inputs

1. `jj --ignore-working-copy workspace list`; if the stack lives in a sibling workspace, work there.
2. Topology and remote state, without snapshotting:

   ```bash
   jj --ignore-working-copy log -r 'main | <topic> | ancestors(<topic>, 2)' --limit 8
   jj --ignore-working-copy bookmark list <topic> --all-remotes
   ```

3. Note the behavioral slices in the stack and where they integrate with `main` (shared modules, imports, registrations).

## Procedure

1. Fetch only if the user asked to start from latest origin: `jj git fetch`.
2. Rebase the roots of the stack: `jj rebase -s 'roots(main..<topic>)' -d main`.
3. Resolve only real conflicts. At an integration seam keep both sides (imports, module declarations, registrations) rather than taking a file wholesale, then compile the affected slice.
4. `jj fix`, the focused tests for the affected slices, then `just lint`.
5. Verify:

   ```bash
   jj --ignore-working-copy resolve --list
   jj --ignore-working-copy log -r 'divergent()' --no-graph
   jj --ignore-working-copy bookmark list <topic> --all-remotes
   ```

## Pitfalls

- Editing conflict markers is not resolution: `jj resolve --list` must be empty and the slice must compile.
- A local bookmark rewrite is not authorization to push; the remote bookmark must be unchanged at handoff.

## Report

Final head and base, conflicts resolved, tests and lint run, and an explicit statement that the remote bookmark and pull request were not changed.
