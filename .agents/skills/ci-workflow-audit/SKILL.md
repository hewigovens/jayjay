---
name: ci-workflow-audit
description: Audit GitHub Actions overlap, timing, path filters, or gates, and make bounded workflow edits without breaking the AppImage release contract. Use for "review github workflows" or "duplicate clippy checks".
argument-hint: "[workflow-or-suspected-overlap]"
---

# CI Workflow Audit

## When to use

- Questions about duplicate jobs, CI cost, path filters, lint or test gates, or AppImage triggers.
- Not for release behavior changes or a CI redesign without explicit authorization.

## Inputs

1. Read `.github/workflows/ci.yml`, `gpui-ci.yml`, `appimage.yml`, [Release](../../../agents/release.md), and `shell/release.just` before judging any job as duplicate.
2. For a cost claim, get real timings: `gh run list --workflow <file> --limit 5`, then `gh run view <run-id> --json jobs`.

## Procedure

1. Build one table: job, OS, command, trigger, path filter, cache, release role. Platform-specific compilation (macOS, Linux, Windows) is coverage, not duplication, until timings say otherwise.
2. Quantify the overlap before proposing a change; compare clippy, test, build, and cache-restore costs.
3. Keep the AppImage contract unless the release guide changes: `v*` tag pushes build and retain artifacts, `just shell::publish` creates the release, and `release: published` uploads the AppImages to it.
4. If edits are authorized, make only the bounded change and leave it uncommitted unless asked.
5. Validate locally, and report `actionlint` as skipped when it is not installed:

   ```bash
   ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].sort.each { |p| YAML.parse_file(p); puts "parsed #{p}" }'
   just --summary
   ```

6. Keep the jj version the workflows install aligned with the `jj-lib` pin in `Cargo.toml`; the mismatch is silent until a fixture depends on newer jj behavior.

## Pitfalls

- Removing the tag trigger as redundant with the release trigger breaks artifact retention; both trigger classes are intentional.
- Two OS jobs with identical command text are not duplicates when they compile different platform code.
- YAML parsing and `just --summary` are static checks; report remote CI state separately.

## Report

Findings table, quantified overlap with run evidence, edits made (uncommitted), and what was validated locally versus on CI.
