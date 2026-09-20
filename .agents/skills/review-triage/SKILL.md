---
name: review-triage
description: Pre-mark straightforward files and hunks of a change as reviewed through the jayjay review CLI, and flag only critical risks before a human review starts.
argument-hint: "[workspace-path]"
disable-model-invocation: true
---

# Review Triage

Clears straightforward changes so the reviewer starts on what matters. Marks written here carry an agent tag in the app and in `jayjay review status`; a false "reviewed" hides a change from the reviewer, so when unsure leave the file unreviewed. Unreviewed status already directs human attention; it does not need an explanatory note. Load [Review State](../../../agents/review-state.md) first.

## When to use

- The user asks to triage, pre-review, or clear the trivial files of a change before reviewing it.
- Complements human review; leave changes requiring human judgment unreviewed.

## Inputs

1. The workspace holding the change (`jj --ignore-working-copy workspace list`); the CLI acts on that workspace's `@`.
2. What is already marked, without touching marks a person made:

   ```bash
   jayjay review status --repo <workspace> --format json
   jayjay review notes --repo <workspace>
   ```

3. Capture `commit_id` from status and inspect that immutable commit: `jj --ignore-working-copy diff -r <commit-id> <path>`; `--stat` first for the shape. Keep this same ID for every mark based on that inspection.

## Procedure

1. Mark changes agent-reviewed when their correctness is straightforward to verify and human review would add little value. Leave changes requiring judgment unreviewed. Mechanical changes are examples, not an exhaustive category. A whole-file mark requires every changed group to qualify.
2. Mark qualifying files with `jayjay review mark --repo <workspace> --expected-commit <commit-id> --file <path>`. In a long file with a mix, mark only the qualifying groups with `--line <n>` (a changed new-side line, or `--side old` for a deletion) and leave the rest.
3. Leave unmarked files and groups without notes by default. Add a note only for a concrete, unresolved critical risk that the reviewer could otherwise miss, such as data loss, a security vulnerability, or a broken core workflow. State the trigger, consequence, and specific decision or check needed; anchor it to the relevant changed line with `jayjay review add-note --repo <workspace> --file <path> --line <n> -m "…"`. Use one note per distinct risk, checking existing notes to avoid duplicates. Do not add code summaries, test results, implementation explanations, generic "please review" requests, or reasons a file was not marked. Zero notes is the normal outcome.
4. Finish with `jayjay review status --repo <workspace>` and report it.

## Pitfalls

- `mark`, `unmark`, `status`, `notes`, and `add-note` snapshot the working copy. Run them one at a time, never in parallel with other jj commands in the same workspace.
- A stale-commit rejection means the inspected basis changed. Refresh status, inspect the new commit, and reassess before retrying with its ID. Never replace the ID without reinspection.
- Do not use `unmark` to redo a pass on top of a person's marks; it only drops agent marks, so `unmark --repo <workspace>` is the safe reset.

## Report

Report the final `status` summary and any critical notes added. Do not produce a file-by-file explanation of ordinary unreviewed code.
