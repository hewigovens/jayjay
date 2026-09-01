# Code Review Guide

Load this file when reviewing local changes, pull requests, or proposed patches.

Use this alongside the agent's native review workflow and output format. Review by default; fix issues only when the user explicitly asks.

The focused docs remain the source of truth. This guide points review attention to JayJay-specific risks.

## Review Setup

1. Read `AGENTS.md`.
2. Load only the focused guide for the changed area, from the `AGENTS.md` Start Here table.
3. Inspect the diff (`jj --ignore-working-copy diff` when you must not snapshot). Read the full changed files and nearby patterns before judging the patch. Do not ritual-run `jj st`.
4. Flag direct edits to generated files, bindings, fixtures, release outputs, or documentation assets unless they trace back to source inputs. Feature PRs should not include user-guide / Help Book / parity-matrix churn.
5. Identify the changed behavior, affected user path, verification that would catch a regression, and whether the changed path needs the risk review below.

## Scope And Convergence

- Review the requested behavior and changed trust boundaries. If the patch adds an unrelated feature or subsystem, recommend removing or deferring it before demanding completeness.
- On later rounds, re-check the full patch for regressions but report only unresolved material defects and defects introduced by the fixes. Do not expand the feature contract; optional improvements are non-blocking.
- Prefer the smallest fix that restores the intended invariant. Stop when the requested behavior is correct, focused tests pass, and no material defect remains.

## Core Checks

- Verify requested behavior, not just compilation.
- Review jj operations using jj's model: changes, bookmarks, revsets, working-copy snapshots, mutable history, and `@`/`@-`. Git branch/commit assumptions are only valid for git interop.
- Check the changed mutating repo path for target revision, snapshot timing, bookmark movement, conflict handling, stale state, cancellation, and recovery.
- Keep business logic in Rust core, UniFFI bindings thin, and SwiftUI/GPUI shells focused on rendering state and dispatching actions.
- Preserve review-state invariants: content-based identity, per-file invalidation, hunk/file promotion, and local persistence.
- Keep UI changes native, keyboard-friendly, quiet, and jj-native in wording. Use repo-level presentation types instead of ad hoc alerts or booleans.
- Match nearby patterns. Keep patches focused, avoid speculative abstractions, prefer structured parsers/APIs, and comment only non-obvious why.
- Look for what the `AGENTS.md` cleanup rounds should have removed: once-used helpers, unused parameters/flags/imports, forwarding wrappers, copy-pasted blocks, restating comments, and tests that only mirror wiring.

## Risk Review

Use adversarial review when the patch changes a trust boundary, destructive repo mutation, persistence format, review-state invariant, or release/update integrity. Keep it scoped to the changed path; do not turn an ordinary UI or state change into a repository-wide audit.

- For the affected boundary, look for command injection, path traversal, unsafe URL/HTML/Markdown rendering, accidental execution of repo-controlled content, and cross-repo or cross-window state leaks.
- Confirm external commands pass structured arguments instead of concatenated shell strings unless a clear escaping boundary exists.
- For release/update changes, preserve asset integrity and avoid leaking credentials, signing details, or unnecessary private paths.

## Tests and Verification

- Use the smallest test layer that proves behavior: Rust unit/integration, Swift unit, XCUITest scene, or GPUI component test.
- Bug fixes should include the regression test that would have caught the bug.
- UI tests that mutate repo state need isolated fixtures; GPUI tests should use hermetic `jj-test` fixtures and assert behavior, not pixels.
- Report the checks that were actually relevant: `just test-rust <crate>`, a filtered `just test-ui`, `just test-gpui`, `just test-app`, `just lint` on publish, or a release command. Do not imply `just build` or workspace-wide `just test` ran unless they did.

## Reporting

Report findings first, ordered by severity, with file and line references. State the concrete bug or risk, impact, and smallest reasonable fix.

If no issues are found, say that clearly and list residual risk or checks not run. Include exact verification commands that were run, skipped, or blocked.

Use the agent's native severity labels when available. Otherwise use:

- **Critical**: data loss, destructive repo mutation, command execution or credential exposure, release/update integrity failure, or a security issue that can compromise users.
- **High**: broken build, likely user-facing workflow regression, incorrect jj operation, corrupted review state, invalid generated binding, broken release or PR flow.
- **Medium**: edge-case correctness bug, stale state, brittle error handling, meaningful missing test coverage, or maintainability issue that makes future changes risky.
- **Low**: optional small cleanup that does not block the change or require another review round.

When asked to fix issues, make the smallest scoped patch, rerun the affected checks, and re-review the resulting diff before handing it back.
