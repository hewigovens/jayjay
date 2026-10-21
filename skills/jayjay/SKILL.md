---
name: jayjay
description: Work with a person who reviews jj (Jujutsu) changes in the JayJay app. Use the `jayjay review` CLI to read and resolve their line-anchored review notes, pre-mark straightforward files as agent-reviewed, leave notes for critical risks, and print jj diff/merge tool config with `jayjay config`. Use when the user mentions JayJay, review notes, review marks, or asks you to address their review feedback in a jj repository.
---

# JayJay

JayJay is a desktop GUI for jj. People review the working-copy change (`@`) there, file by file and hunk by hunk, and leave notes anchored to diff lines. The `jayjay review` CLI exposes the same review state to agents, so feedback loops stay in the repository instead of in chat.

## Setup

Check that the CLI is on `PATH` with `jayjay --version`. If it is missing, ask the user to install it: on macOS, JayJay ▸ Install Command Line Tool… links it into `~/.local/bin`; the Linux packages install `/usr/bin/jayjay`. Commands take `--repo <path>` (default `.`), which may be any jj workspace; they act on that workspace's `@`.

Review state is the user's: it lives in JayJay's local store, not in the repository, and never ships with the change.

## Ground rules

- Every `jayjay review` command snapshots the working copy, like `jj st`. Run them one at a time and never concurrently with another jj command in the same workspace, or `@` can become divergent.
- Never run `jayjay tool …` or `jj diff --tool jayjay` / `jj resolve --tool jayjay` yourself. Those open an interactive window and block until a person closes it.
- Use `--format json` when you parse output; the text format is for people.

## Address review notes

Use when the user says they left notes, or asks you to address their JayJay review.

```bash
jayjay review notes --repo .                 # file:line [status] id, anchor line, body
jayjay review notes --repo . --format json   # notes[].note.{id,path,side,line,body}, notes[].status
```

1. Treat `current` notes as actionable feedback on the line shown. Re-check `stale` notes against the current diff: the anchored code changed since the note was written. `orphaned` notes lost their file or anchor; read them for intent and confirm with the user when unclear.
2. Make the change each note asks for, or explain why not when the note is a question or you disagree.
3. After addressing a note, resolve it: `jayjay review resolve-note <id> --repo .`. Leave notes you did not address unresolved and say why in your reply.
4. Run `jayjay review notes --repo .` again; it prints `No review notes.` once everything is resolved.

`--include-resolved` also lists resolved notes. Only notes on the workspace's current change can be resolved.

## Pre-mark straightforward changes

Use when asked to triage or pre-review a change so the person can start on what matters. A false "reviewed" hides code from the reviewer, so when unsure, leave a file unreviewed.

```bash
jayjay review status --repo . --format json
```

1. Read `commit_id` from the status output and inspect that exact commit: `jj --ignore-working-copy diff -r <commit_id> --stat`, then per file. Keep using that same id.
2. Mark a file whose every change is easy to verify and needs no human judgment (renames, formatting, generated code, mechanical updates):

   ```bash
   jayjay review mark --repo . --expected-commit <commit_id> --file <path>
   ```

   In a file with a mix, mark only qualifying change groups with `--line <n>`, a changed line on the new side, or `--line <n> --side old` for a deletion.
3. If `mark` fails with `Working copy changed since inspection`, the change moved. Run `status` again, inspect the new commit, and reassess before retrying. Never swap in the new id without re-inspecting.
4. Finish with `jayjay review status --repo .` and report its summary line.

Agent marks are tagged as agent-owned in the app and in `status`, and they never override a person's marks. `jayjay review unmark --repo . [--file <path>]` drops only agent marks, so it is the safe way to redo a pass.

## Leave review notes

Add a note only for a concrete, unresolved critical risk the reviewer could otherwise miss, such as data loss, a security hole, or a broken core workflow. State the trigger, the consequence, and the decision or check needed. Do not use notes for summaries, test results, or reasons a file was left unmarked; zero notes is the normal outcome.

```bash
jayjay review add-note --repo . --file <path> --line <n> [--side old] -m "…"
```

The line must be a changed line on that side of the diff. Adding a note to a line that already has an active note updates it.

## Use JayJay as jj's diff and merge tool

`jayjay config` prints a `[merge-tools.jayjay]` table for jj's config. When the user asks to set this up, show it to them or append it to their jj config (`jj config path --user`) with their approval; then `jj diff --tool jayjay`, `jj diffedit --tool jayjay`, and `jj resolve --tool jayjay` open JayJay. Those are for the user to run, per the ground rules.
