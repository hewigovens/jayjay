# Code Review Guide

Load this file when reviewing local changes, pull requests, or proposed patches.

Use this alongside the agent's native review workflow and output format. [Task Authority](../AGENTS.md#task-authority) defines whether the pass is findings-only or includes fixes.

The focused docs remain the source of truth. This guide points review attention to JayJay-specific risks.

## Review Setup

1. Read `AGENTS.md`.
2. Load only the focused guide for the changed area, from the `AGENTS.md` Start Here table.
3. Resolve the exact review target before reading the diff: workspace, base and head revisions, and immutable commit IDs when change IDs may be divergent. For a live pull request, record the head and verify final checks, review state, and mergeability against that same head. If it moves, refresh the affected evidence or mark it stale.
4. Inspect that fixed range (`jj --ignore-working-copy diff` when you must not snapshot). Read the full changed files and nearby patterns before judging the patch. Do not ritual-run `jj st`.
5. Flag direct edits to generated files, bindings, fixtures, release outputs, or documentation assets unless they trace back to source inputs. Feature PRs should not include user-guide / Help Book / parity-matrix churn.
6. Identify the changed behavior, affected user path, verification that would catch a regression, and whether the changed path needs the risk review below.

## Scope And Convergence

- Review the requested behavior and changed trust boundaries. If the patch adds an unrelated feature or subsystem, recommend removing or deferring it before demanding completeness.
- On later rounds, re-check the full patch for regressions but report only unresolved material defects and defects introduced by the fixes. Do not expand the feature contract; optional improvements are non-blocking.
- Prefer the smallest fix that restores the intended invariant. Stop when the requested behavior is correct, focused tests pass, and no material defect remains.

## Core Checks

- Verify requested behavior, not just compilation.
- Review jj operations using jj's model: changes, bookmarks, revsets, working-copy snapshots, mutable history, and `@`/`@-`. Git branch/commit assumptions are only valid for git interop.
- Check the changed mutating repo path for target revision, snapshot timing, bookmark movement, conflict handling, stale state, cancellation, and undo/recovery.
- For operations on rendered selections or line numbers, validate acceptance against the rendered basis: a reload between render and confirm must abort or reselect, never apply the old selection to new content.
- Prefer one validate-then-act step immediately before a mutating jj operation over stacked TOCTOU guards (op-id pins, retry loops, quarantine moves). When a review round asks for another guard on top of the last one, question the contract instead of layering; see the [Repository Operation Contracts](architecture.md#repository-operation-contracts).
- Keep business logic in Rust core, UniFFI bindings thin, and SwiftUI/GPUI shells focused on rendering state and dispatching actions.
- Preserve review-state invariants: content-based identity, per-file invalidation, hunk/file promotion, and local persistence.
- Keep UI changes native, keyboard-friendly, quiet, and jj-native in wording. Use repo-level presentation types instead of ad hoc alerts or booleans.
- Match nearby patterns. Keep patches focused, avoid speculative abstractions, prefer structured parsers/APIs, and comment only non-obvious why.
- Look for what the `AGENTS.md` cleanup rounds should have removed: once-used helpers, unused parameters/flags/imports, forwarding wrappers, copy-pasted blocks, restating comments, and tests that only mirror wiring.

## Risk Review

Use adversarial review when the patch changes a trust boundary, destructive repo mutation, persistence format, review-state invariant, or release/update integrity. Keep it scoped to the changed path; do not turn an ordinary UI or state change into a repository-wide audit.

- For the affected boundary, look for command injection, path traversal, unsafe URL/HTML/Markdown rendering, accidental execution of repo-controlled content, and cross-repo or cross-window state leaks.
- Confirm external commands pass structured arguments instead of concatenated shell strings, and that repository-controlled paths and names reach `jj`/`git`/`gh` only through the operand rules in [Repository Operation Contracts](architecture.md#repository-operation-contracts) — structured argv does not stop the tool's own option or fileset parsing.
- For release/update changes, preserve asset integrity and avoid leaking credentials, signing details, or unnecessary private paths.

## Tests and Verification

- Use the smallest test layer that proves behavior: Rust unit/integration, Swift unit, XCUITest scene, or GPUI component test.
- Bug fixes should include the regression test that would have caught the bug.
- UI tests that mutate repo state need isolated fixtures; GPUI tests should use hermetic `jj-test` fixtures and assert behavior, not pixels.
- Report the checks that actually ran, per the evidence rule in [Task Authority](../AGENTS.md#task-authority); do not imply `just build` or workspace-wide `just test` ran unless they did.
- For crucial changes — security fixes, destructive or mutating repo operations, review-state invariants, release/update integrity — include a mini test matrix: a compact table mapping the key scenarios (normal, boundary, and hostile/adversarial input) to expected behavior and the test that covers each. Flag any uncovered row as missing coverage.

## Reporting

Report findings first, ordered by severity, with file and line references. State the concrete bug or risk, impact, and smallest reasonable fix.

If no issues are found, say that clearly and list residual risk or checks not run.

Use the agent's native severity labels when available. Otherwise use:

- **Critical**: data loss, destructive repo mutation, command execution or credential exposure, release/update integrity failure, or a security issue that can compromise users.
- **High**: broken build, likely user-facing workflow regression, incorrect jj operation, corrupted review state, invalid generated binding, broken release or PR flow.
- **Medium**: edge-case correctness bug, stale state, brittle error handling, meaningful missing test coverage, or maintainability issue that makes future changes risky.
- **Low**: optional small cleanup that does not block the change or require another review round.

When asked to fix issues, make the smallest scoped patch, rerun the affected checks, and re-review the resulting diff before handing it back.
