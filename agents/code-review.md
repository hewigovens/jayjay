# Code Review Guide

Load this file when reviewing local changes, pull requests, or proposed patches.

Use this alongside the agent's native review workflow and output format. Review by default; fix issues only when the user explicitly asks.

The focused docs remain the source of truth. This guide points review attention to JayJay-specific risks.

## Review Setup

1. Read `AGENTS.md`.
2. Load the focused guide for the changed area: `agents/architecture.md`, `agents/testing.md`, `agents/design.md`, `agents/pull-requests.md`, or `agents/release.md`.
3. Inspect `jj st` and the diff, then read the full changed files and nearby patterns before judging the patch.
4. Flag direct edits to generated files, bindings, fixtures, release outputs, or documentation assets unless they trace back to source inputs.
5. Identify the changed behavior, affected user path, verification that would catch a regression, and whether the change is non-trivial enough to need adversarial review.

## Core Checks

- Verify requested behavior, not just compilation.
- Review jj operations using jj's model: changes, bookmarks, revsets, working-copy snapshots, mutable history, and `@`/`@-`. Git branch/commit assumptions are only valid for git interop.
- Check mutating repo actions for target revision, snapshot timing, bookmark movement, conflict handling, stale state, cancellation, and undo/recovery.
- Keep business logic in Rust core, UniFFI bindings thin, and SwiftUI/GPUI shells focused on rendering state and dispatching actions.
- Preserve review-state invariants: content-based identity, per-file invalidation, hunk/file promotion, and local persistence.
- Keep UI changes native, keyboard-friendly, quiet, and jj-native in wording. Use repo-level presentation types instead of ad hoc alerts or booleans.
- Match nearby patterns. Keep patches focused, avoid speculative abstractions, prefer structured parsers/APIs, and comment only non-obvious why.

## Adversarial Review

Run this section for non-trivial changes: anything that mutates repositories, crosses process/network/persistence boundaries, parses or renders external content, changes release/update behavior, touches review state, or affects shared async/cache state. For trivial copy, styling, documentation, or narrow test-only changes, skip adversarial review unless the diff introduces one of those boundaries.

When adversarial review applies, review as if repo content, remote metadata, file paths, URLs, appcast data, release notes, PR host responses, and command input may be hostile.

- Challenge every trust boundary: repo to UI, UI to jj, app to shell/tool, network to release/update, browser to PR host, and persisted state to current view.
- Look for command injection, path traversal, unsafe URL/HTML/Markdown rendering, bad shell quoting, accidental execution of repo-controlled content, and stale cache/retry paths accepting untrusted state.
- Confirm external commands pass structured arguments instead of concatenated shell strings unless a clear escaping boundary exists.
- Ask whether one repo, window, change, bookmark, review mark, cached response, or async task can influence another path it should not.
- Ensure release/update flows preserve asset integrity and errors do not leak credentials, signing details, or unnecessary private paths.

## Tests and Verification

- Use the smallest test layer that proves behavior: Rust unit/integration, Swift unit, XCUITest scene, or GPUI component test.
- Bug fixes should include the regression test that would have caught the bug.
- UI tests that mutate repo state need isolated fixtures; GPUI tests should use hermetic `jj-test` fixtures and assert behavior, not pixels.
- Report relevant checks run or missing: `just test`, `just test-app`, `just test-ui`, `just test-gpui`, `just lint`, `just build`, or release-specific commands.
- For crucial changes — security fixes, destructive or mutating repo operations, review-state invariants, release/update integrity — include a mini test matrix: a compact table mapping the key scenarios (normal, boundary, and hostile/adversarial input) to expected behavior and the test that covers each. Flag any uncovered row as missing coverage.

## Reporting

Report findings first, ordered by severity, with file and line references. State the concrete bug or risk, impact, and smallest reasonable fix.

If no issues are found, say that clearly and list residual risk or checks not run. Include exact verification commands that were run, skipped, or blocked.

Use the agent's native severity labels when available. Otherwise use:

- **Critical**: data loss, destructive repo mutation, command execution or credential exposure, release/update integrity failure, or a security issue that can compromise users.
- **High**: broken build, likely user-facing workflow regression, incorrect jj operation, corrupted review state, invalid generated binding, broken release or PR flow.
- **Medium**: edge-case correctness bug, stale state, brittle error handling, meaningful missing test coverage, or maintainability issue that makes future changes risky.
- **Low**: small style, naming, or convention issue that is safe to batch with other cleanup.

When asked to fix issues, make the smallest scoped patch, rerun the affected checks, and re-review the resulting diff before handing it back.
