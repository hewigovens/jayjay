# Code Review Guide

Load this file when reviewing local changes, pull requests, or proposed patches.

Use the agent's native review workflow and output format. [Task Authority](../AGENTS.md#task-authority) defines whether the pass includes fixes; focused docs remain the source of truth for area-specific contracts.

## Review Setup

1. Read `AGENTS.md`.
2. Load only the focused guide for the changed area, from the `AGENTS.md` Start Here table.
3. Resolve the exact review target before reading the diff: workspace, base and head revisions, and immutable commit IDs when change IDs may be divergent. For a live pull request, record the head and verify final checks, review state, and mergeability against that same head. If it moves, refresh the affected evidence or mark it stale.
4. Inspect that fixed range (`jj --ignore-working-copy diff` when you must not snapshot). Read the full changed files and nearby patterns before judging the patch. Do not ritual-run `jj st`.
5. Flag direct edits to generated files, bindings, fixtures, release outputs, or documentation assets unless they trace back to source inputs. Feature PRs should not include user-guide / Help Book / parity-matrix churn.
6. Identify the changed behavior, affected user path, and whether it needs the correctness and risk review below.

## Scope And Convergence

- Review the requested behavior and changed trust boundaries. If the patch adds an unrelated feature or subsystem, recommend removing or deferring it before demanding completeness.
- On later rounds, re-check the full patch for unresolved material defects, including newly discovered defects in the original patch and regressions from fixes. Keep the original feature contract.
- Prefer the smallest fix that restores the intended invariant at every site sharing the same assumption; a fix that covers only the reported case invites the sibling case next round. Stop when the requested behavior is correct, focused tests pass, and no material defect remains.

## Correctness Review

Review as an independent maintainer: the pass runs in a fresh context that did not implement the change and starts from the diff, the task, and this guide, because an author reviewing its own patch shares its assumptions. For non-trivial behavior changes, actively try to falsify the implementation:

- Infer the contract from the issue or task, public APIs, surrounding callers, and invariants. Existing tests are evidence, not proof of correctness; their fixtures and assertions may share the implementation's wrong assumption.
- Identify the assumptions each changed path relies on and construct a reachable counterexample. Probe relevant input boundaries (empty, duplicate, malformed, numeric limits) and combinations of states: repeated operations, stale data, cancellation, partial failure, and persistence round trips.
- Probe the classes that recur in this codebase: an async completion landing on state that was replaced or superseded meanwhile (see the async conventions under [MVVM](architecture.md#mvvm)); a check made before an `await` that is stale by the time of the write; state kept per window or per view model that must be shared or discarded across windows; repository-controlled paths and names on Windows; and a regression test that still passes with the fix removed.
- Trace candidates through actual callers and guards. Check panic, unwrap, and overflow paths, and duplicated implementations of the same contract for differing behavior. Compare with the base revision to distinguish patch defects from pre-existing bugs.
- Run a focused test or minimal reproducer when useful. A code trace can establish a failure, but state whether it was reproduced.

## Core Checks

- Review jj operations using jj's model: changes, bookmarks, revsets, working-copy snapshots, mutable history, and `@`/`@-`. Git branch/commit assumptions are only valid for git interop.
- Check the changed mutating repo path for target revision, snapshot timing, bookmark movement, conflict handling, stale state, cancellation, and undo/recovery.
- For operations on rendered selections or line numbers, validate acceptance against the rendered basis: a reload between render and confirm must abort or reselect, never apply the old selection to new content.
- Prefer one validate-then-act step immediately before a mutating jj operation over stacked TOCTOU guards (op-id pins, retry loops, quarantine moves). When a review round asks for another guard on top of the last one, question the contract instead of layering; see the [Repository Operation Contracts](architecture.md#repository-operation-contracts).
- Keep business logic in Rust core, UniFFI bindings thin, and SwiftUI/GPUI shells focused on rendering state and dispatching actions.
- Preserve review-state invariants: content-based identity, per-file invalidation, hunk/file promotion, and local persistence.
- Keep UI changes native, keyboard-friendly, quiet, and jj-native in wording. Use repo-level presentation types instead of ad hoc alerts or booleans.
- Check repeated UI work for per-row whole-collection scans, stale render caches, and geometry-driven invalidation loops; apply the [SwiftUI rendering performance rules](swiftui.md#rendering-performance) or [GPUI render-cache conventions](gpui.md#conventions) for the affected shell.

## Risk Review

Use adversarial review when the patch changes a trust boundary, destructive repo mutation, persistence format, review-state invariant, or release/update integrity. Keep it scoped to the changed path; do not turn an ordinary UI or state change into a repository-wide audit.

- For the affected boundary, look for command injection, path traversal, unsafe URL/HTML/Markdown rendering, accidental execution of repo-controlled content, and cross-repo or cross-window state leaks.
- Confirm external commands pass structured arguments instead of concatenated shell strings, and that repository-controlled paths and names reach `jj`/`git`/`gh` only through the operand rules in [Repository Operation Contracts](architecture.md#repository-operation-contracts) — structured argv does not stop the tool's own option or fileset parsing.
- For release/update changes, preserve asset integrity and avoid leaking credentials, signing details, or unnecessary private paths.

## Tests and Verification

- Use the smallest test layer that proves behavior: Rust unit/integration, Swift unit, XCUITest scene, or GPUI component test.
- Bug fixes should include the regression test that would have caught the bug, shown to fail on the pre-fix code.
- Check new or changed tests against [Testing](testing.md#before-adding-a-test). Verify fixtures reach the claimed behavior and assertions would fail if its invariant broke.
- UI tests that mutate repo state need isolated fixtures; GPUI tests should use hermetic `jj-test` fixtures and assert behavior, not pixels.
- Report the checks that actually ran, per the evidence rule in [Task Authority](../AGENTS.md#task-authority); do not imply `just build` or workspace-wide `just test` ran unless they did.
- For crucial changes — security fixes, mutating repo operations, review-state invariants, release/update integrity — include a compact matrix of normal, boundary, and hostile scenarios, expected behavior, and covering tests. Record uncovered scenarios as verification gaps.

## Reporting

Report concrete defects first, ordered by severity, with file and line references. Each finding needs a specific failing scenario or minimal test case, expected versus actual behavior, impact, and the smallest reasonable fix. Do not report style nits, speculative concerns, missing tests, or duplicated logic without a supported behavioral failure. Cleanup (trivial helpers, restating comments, unused parameters) belongs to the author's cleanup rounds, not to findings.

After the findings, two separate non-blocking lists: verification gaps (a bug fix without its regression test, scenarios no test covers, checks not run) and pre-existing bugs found near the change, which are reported against the base revision, not the patch.

If no issues are found, say that clearly and still list the two lists above.

Use the agent's native severity labels when available. Otherwise use:

- **Critical**: data loss, destructive repo mutation, command execution or credential exposure, release/update integrity failure, or a security issue that can compromise users.
- **High**: broken build, likely user-facing workflow regression, incorrect jj operation, corrupted review state, invalid generated binding, broken release or PR flow.
- **Medium**: edge-case correctness bug, stale state, or incorrect error handling.
- **Low**: minor correctness bug with limited impact.

When asked to fix issues, make the smallest scoped patch that also covers the sibling sites of each finding, rerun the affected checks, and re-review the resulting diff before handing it back.
