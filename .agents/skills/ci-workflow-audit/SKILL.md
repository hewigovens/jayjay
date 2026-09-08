---
name: ci-workflow-audit
description: Audit GitHub Actions overlap, timing, path filters, or gates, and make bounded workflow edits without breaking the AppImage release contract. Use for "review github workflows" or "duplicate clippy checks".
argument-hint: "[workflow-or-suspected-overlap]"
---

# CI Workflow Audit

## When to use

- Questions about duplicate jobs, CI cost, path filters, lint or test gates, or AppImage triggers.
- Not for release behavior changes or a CI redesign without explicit authorization.
- Audit requests are findings-only. Make workflow edits only when the user or saved task explicitly authorizes them; a schedule does not add authority.

## Inputs

1. Read the workflows in scope and any workflow or recipe that shares their commands or triggers. For AppImage or release overlap, include `.github/workflows/appimage.yml`, [Release](../../../agents/release.md), and `shell/release.just`.
2. For a cost claim, get real timings: `gh run list --workflow <file> --limit 5`, then `gh run view <run-id> --json jobs`.

## Procedure

1. Build one table: job, OS, command, trigger, path filter, cache, release role. Platform-specific compilation (macOS, Linux, Windows) provides distinct coverage; timing alone does not make it redundant.
2. Quantify the overlap before proposing a change; compare clippy, test, build, and cache-restore costs.
3. Keep the AppImage contract unless the release guide changes: `release: published` builds and uploads the AppImages, `just shell::publish` creates that release, and `workflow_dispatch` on a tag builds retained CI artifacts for a pre-publish check. Tag pushes do not build.
4. If edits are authorized, make only the bounded change and leave it uncommitted unless asked.
5. For workflow edits, run `actionlint` when available and parse the changed YAML. Check `just --summary` when recipe calls changed. Report missing validators rather than installing tools or treating parsing as equivalent validation. Example static checks:

   ```bash
   ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].sort.each { |p| YAML.parse_file(p); puts "parsed #{p}" }'
   just --summary
   ```

6. If the changed jobs install jj or run jj fixtures, compare their jj version with the `jj-lib` pin in `Cargo.toml`. Report a mismatch; change pins only within the authorized scope.

## Pitfalls

- Do not re-add a `v*` tag-push build: it shares the tag's concurrency group with the `release: published` run that follows within minutes, so it was always cancelled after spending its build time (two 16-minute builds on v0.3.17-beta.2).
- Two OS jobs with identical command text are not duplicates when they compile different platform code.
- YAML parsing and `just --summary` are static checks; report remote CI state separately.

## Report

Findings table, quantified overlap with run evidence, edits made (uncommitted), and what was validated locally versus on CI.
