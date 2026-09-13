---
name: review-triage
description: Pre-mark mechanical files and hunks of a change as reviewed through the jayjay review CLI, and leave notes where a person must look, before a human review starts.
argument-hint: "[workspace-path]"
disable-model-invocation: true
---

# Review Triage

Clears the mechanical part of a change so the person reviewing it starts on what matters. Marks written here carry an agent tag in the app and in `jayjay review status`; a false "reviewed" hides a change from the reviewer, so when unsure leave the file unreviewed and say why in a note. Load [Review State](../../../agents/review-state.md) first.

## When to use

- The user asks to triage, pre-review, or clear the trivial files of a change before reviewing it.
- Not for judging whether the change is correct; that stays with the person.

## Inputs

1. The workspace holding the change (`jj --ignore-working-copy workspace list`); the CLI acts on that workspace's `@`.
2. What is already marked, without touching marks a person made:

   ```bash
   jayjay review status --repo <workspace> --format json
   jayjay review notes --repo <workspace>
   ```

3. Capture `commit_id` from status and inspect that immutable commit: `jj --ignore-working-copy diff -r <commit-id> <path>`; `--stat` first for the shape. Keep this same ID for every mark based on that inspection.

## Procedure

1. Run the mechanical checks before any judgment call. A file qualifies for a whole-file mark only when every change group is one of:
   - a pure rename or move: `jj diff` pairs it and the content diff is empty, or a deleted file reappears verbatim elsewhere;
   - whitespace or formatting only: `jj --ignore-working-copy diff -r <commit-id> --ignore-all-space <path>` is empty;
   - `mod.rs`, `lib.rs`, or export lines that only declare or re-export the modules the change adds or moves;
   - call sites of a rename that keep the same shape and arity, including test fixtures that only follow the rename.
2. Mark qualifying files with `jayjay review mark --repo <workspace> --expected-commit <commit-id> --file <path>`. In a long file with a mix, mark only the mechanical groups with `--line <n>` (a changed new-side line, or `--side old` for a deletion) and leave the rest.
3. For every file or group you skipped because it needs a person, leave one note on its first changed line saying what to look at: `jayjay review add-note --repo <workspace> --file <path> --line <n> -m "…"`. One note per file; do not narrate the diff.
4. Finish with `jayjay review status --repo <workspace>` and report it.

## Pitfalls

- `mark`, `unmark`, `status`, `notes`, and `add-note` snapshot the working copy. Run them one at a time, never in parallel with other jj commands in the same workspace.
- A stale-commit rejection means the inspected basis changed. Refresh status, inspect the new commit, and reassess before retrying with its ID. Never replace the ID without reinspection.
- A rename plus edits is not a pure rename; mark the rename-only groups by line and leave the edited ones.
- A test file that changes assertions or fixtures beyond following a rename is not mechanical.
- Do not use `unmark` to redo a pass on top of a person's marks; it only drops agent marks, so `unmark --repo <workspace>` is the safe reset.

## Report

Files marked (whole or by group), files left with a note and why, and the final `status` summary line.
