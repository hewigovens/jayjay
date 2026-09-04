---
name: github-actions-pin-audit
description: Audit GitHub Actions `uses:` references and replace mutable tags or branches with verified full commit SHAs. Use for dependabot or security pull requests about unpinned actions.
---

# GitHub Actions Pin Audit

## When to use

- Third-party `uses:` references in `.github/workflows` that point at a tag or branch instead of a full commit SHA.
- Not for first-party workflow logic; publishing needs its own authorization.
- Unattended on a schedule: run the whole procedure; end with the pins uncommitted and the report.

## Procedure

1. Inventory every reference, not only the changed files:

   ```bash
   rg -n --glob '*.yml' 'uses:[[:space:]]*[^[:space:]]+@[^[:space:]]+' .github
   ```

2. For each mutable third-party ref, resolve the intended commit and check the annotated tag's peeled target:

   ```bash
   git ls-remote --tags https://github.com/<owner>/<action>.git 'refs/tags/<tag>*'
   ```

   `dtolnay/rust-toolchain` is pinned from upstream `master` history; keep `with: toolchain: stable`.
3. Replace the ref with the full SHA and keep the version as a trailing comment (`@<sha> # v7.0.0`) so the pin stays auditable. Preserve every `with:` input.
4. Re-run the inventory, parse the YAML (see the `ci-workflow-audit` skill), and report `actionlint` as skipped when it is not installed.
5. When superseding a bot pull request, leave it untouched and say the new change supersedes it. Pushing and opening the pull request follow [Pull Requests](../../../agents/pull-requests.md) and need explicit authorization.

## Pitfalls

- Auditing only the pull request diff leaves mutable refs elsewhere in the workflows.
- A pin silently changes behavior when an input is dropped or a non-default implementation is pinned.

## Report

Every ref inventoried, each pin's resolved SHA and source, inputs preserved, checks run, and which tools were unavailable.
