---
name: local-rebase
description: Rebase a local jj feature stack onto the requested trunk without publishing. Use when a local rebase is requested; "no need to push" alone is not a rebase request.
argument-hint: "[feature-bookmark]"
disable-model-invocation: true
---

# Local Rebase

Moves a feature stack onto the requested trunk revision and leaves the remote bookmark and any pull request untouched. Examples use `main`; resolve the actual destination (`main`, `main@origin`, or another requested trunk) before mutating history. Load [Version Control](../../../agents/version-control.md) first; publication needs a separate request per `AGENTS.md` Task Authority.

## When to use

- The user asks to rebase a feature bookmark or an identifiable current stack without pushing.
- Not for updating a pull request or rewriting a shared remote bookmark.

## Inputs

1. `jj --ignore-working-copy workspace list`; if the stack lives in a sibling workspace, work there.
2. Topology and remote state, without snapshotting:

   ```bash
   jj --ignore-working-copy log -r 'main | <topic> | ancestors(<topic>, 2)' --limit 8
   jj --ignore-working-copy bookmark list <topic> --all-remotes
   ```

3. Identify the stack roots, destination, and immutable head/base IDs. Inspect enough ancestry to establish the whole requested stack; the limited log above is only an initial view. Note the affected integration points and preserve unrelated working-copy edits.

## Procedure

1. Fetch only if the user asked to start from latest origin: `jj git fetch`. After a fetch, refresh the destination and record remote-tracking bookmark targets before the rebase.
2. Rebase the identified roots onto the resolved destination. For a stack based on `main`, the pattern is `jj rebase -s 'roots(main..<topic>)' -d main`; adapt the revset to the verified topology, and do not guess when unrelated changes would be included.
3. Resolve only real conflicts. At an integration seam keep both sides (imports, module declarations, registrations) rather than taking a file wholesale, then validate the affected slice.
4. Run focused checks for affected integration points. Apply the [validation policy](../../../AGENTS.md#inner-loop): a local rebase alone does not require repository-wide formatting or lint; broaden checks when conflict resolution or failures warrant it.
5. Confirm the intended stack descends from the resolved destination, no conflicts remain, and remote-tracking bookmark targets match the recorded pre-rebase state. Snapshot conflict-resolution edits before the read-only checks below:

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
