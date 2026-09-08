---
name: github-actions-pin-audit
description: Audit GitHub Actions `uses:` references and replace mutable tags or branches with verified full commit SHAs. Use for dependabot or security pull requests about unpinned actions.
---

# GitHub Actions Pin Audit

## When to use

- Third-party `uses:` references in `.github/workflows` that point at a tag or branch instead of a full commit SHA.
- Not for first-party workflow logic; publishing needs its own authorization.
- Audit requests are findings-only. Replace pins only when the user or saved task authorizes fixes; a schedule does not add authority.

## Procedure

1. Inventory references in the requested scope. For a repository-wide audit, include all workflows; for a selected PR or finding, keep edits scoped and report unrelated mutable refs separately:

   ```bash
   rg -n --glob '*.yml' 'uses:[[:space:]]*[^[:space:]]+@[^[:space:]]+' .github
   ```

2. For each mutable third-party ref, resolve the intended commit and check the annotated tag's peeled target:

   ```bash
   git ls-remote --tags https://github.com/<owner>/<action>.git 'refs/tags/<tag>*'
   ```

   `dtolnay/rust-toolchain` is pinned from upstream `master` history; keep `with: toolchain: stable`.
3. When fixes are authorized, replace the ref with the full SHA and keep the version as a trailing comment (`@<sha> # v7.0.0`) so the pin stays auditable. Preserve every `with:` input.
4. After edits, re-run the inventory, parse the changed YAML, and run `actionlint` when available. Report unavailable validators. For a syntax check, use `ruby -e 'require "yaml"; ARGV.each { |p| YAML.parse_file(p) }' <changed-workflow.yml>`; parsing does not validate action inputs or runtime behavior.
5. When superseding a bot pull request, leave it untouched and say the new change supersedes it. Pushing and opening the pull request follow [Pull Requests](../../../agents/pull-requests.md) and need explicit authorization.

## Pitfalls

- A selected PR audit does not establish that the whole repository is pinned; state the inspected scope.
- A pin silently changes behavior when an input is dropped or a non-default implementation is pinned.

## Report

Every ref inventoried, each pin's resolved SHA and source, inputs preserved, checks run, and which tools were unavailable.
