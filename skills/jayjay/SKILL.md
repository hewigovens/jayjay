---
name: jayjay
description: Work with a person who reviews jj (Jujutsu) changes in the JayJay app. Use the `jayjay review` CLI to read and resolve their line-anchored review notes, pre-mark straightforward files as agent-reviewed, leave notes for critical risks, print jj diff/merge tool config with `jayjay config`, create sibling workspaces or check out pull requests with `jayjay workspace` and `jayjay import`, and on macOS drive the running app with AppleScript (open a repository, set the revset, select a change, read the selection). Use when the user mentions JayJay, review notes, review marks, asks you to address their review feedback in a jj repository, or asks you to show something in JayJay.
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

## Sibling workspaces and pull requests

When the user wants a change worked on in its own checkout, or a pull request checked out locally, use these instead of hand-built `jj workspace` and fetch commands:

```bash
jayjay workspace add <name> --repo .          # ../<name> on trunk(); -r <rev> picks another start
jayjay import <PR or MR URL> --repo .         # fetches the head into ../pr-<number>; --name/--dest override
jayjay workspace prune <name>... --repo .     # or --all; deletes only workspaces whose changes are in trunk() or empty
```

`import` reports the existing workspace instead of importing a pull request twice. `prune` keeps any workspace with pending work and says how many changes it holds; ask before pruning a workspace you did not create.

## Show something in the app (macOS)

On macOS, JayJay is scriptable through AppleScript, so you can put a repository, a revset, or a change in front of the user, or read what they have selected. This controls the window the user is looking at: do it when they ask ("show me that change in JayJay", "what do I have selected?"), not as a side effect. It only navigates; change the repository with `jj` as usual.

```bash
osascript <<'EOF'
tell application "JayJay"
    set r to open repository "/abs/path/to/workspace"
    set revset of r to "trunk()..@"
    select change "@-" in r
    show file "src/main.rs" in r
    get selected change ids of r
end tell
EOF
```

| Script | Does |
| --- | --- |
| `open repository "<abs path>"` | Opens or activates the window for the jj workspace containing that path and returns the repository |
| `front repository`, `repositories`, `repository id "<workspace root>"` | The frontmost repository (`missing value` if none), all open ones front first, or one by its root |
| `path of r`, `revset of r` | The workspace root; the revset shown in the graph |
| `set revset of r to "<revset>"` | Reloads the graph with that revset; `""` resets to the default |
| `select change "<revision>" in r` | Selects one change and shows its diff; `in r` defaults to the front repository |
| `show file "<path>" in r` | Selects that file of the selected change and shows its diff; the path is relative to the workspace root |
| `selected change ids of r` | Full change IDs of the selection, as a list; `{}` when nothing is selected |
| `refresh r` | Snapshots and reloads, for example after you ran `jj` |
| `error message of r`, `notice of r` | The error or the latest notice the window shows; `missing value` when there is none |
| `window id of r` | The window number, for `screencapture -x -o -l <id> <file.png>` to capture that window without focusing it |

- Each command returns once the window has finished loading, so run them back to back with no `delay`.
- `<revision>` is any jj revision expression that resolves to exactly one change (`@`, a change ID or unique prefix, a bookmark). The change must already be in the graph: if it is not, set a revset that includes it first.
- `show file` needs one selected change that contains the file; it cannot jump to a line yet.
- To act on what the user selected, read `selected change ids` and pass those IDs to `jj`:

  ```bash
  osascript -e 'tell application "JayJay" to get selected change ids of front repository'
  ```

- Errors come back as AppleScript errors with a message: `-1728` means no such repository, change or file (not open, unknown, ambiguous, or outside the current revset), `-10000` means the app tried and failed (for example an invalid revset), `-1700` a malformed argument such as a relative path, `-1712` that the window did not appear within 30 seconds.
- The first script from a terminal makes macOS ask the user to allow it to control JayJay. If `osascript` hangs or fails with `-1743`, ask the user to approve the prompt (System Settings ▸ Privacy & Security ▸ Automation); do not retry in a loop.
- If JayJay answers that it does not understand a command, the installed version predates scripting; say so instead of falling back to simulated clicks or keystrokes.

## Use JayJay as jj's diff and merge tool

`jayjay config` prints a `[merge-tools.jayjay]` table for jj's config. When the user asks to set this up, show it to them or append it to their jj config (`jj config path --user`) with their approval; then `jj diff --tool jayjay`, `jj diffedit --tool jayjay`, and `jj resolve --tool jayjay` open JayJay. Those are for the user to run, per the ground rules.
