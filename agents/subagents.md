# Subagents

Load this file before dispatching a worker, choosing its model, or deciding whether parallel search belongs in the parent session. Isolation rules for jj itself stay in [Version Control](version-control.md); this guide is only the dispatch contract.

The rules are the same for every harness: spawn for unknown-layout or parallel locate work; keep planning, mutations, and review on the parent. Claude Code and Codex already fan out workers. Cursor Grok must use `Task` on the spawn cases below instead of searching unknown layout in the parent.

## Roles

- **Parent** (this session): plans, evaluates, and mutates. Owns crate-boundary decisions, jj in this workspace, inner-loop tests, cleanup rounds, `jj describe`, and review judgment.
- **Worker**: a well-specified locate, inventory, or read job. Returns evidence — paths, line-level claims, and what it did not find. It does not implement the user's task and does not "finish the PR."

## When to spawn

Spawn when all of these hold:

1. The subtask is well-specified: inputs, question, and return shape.
2. The parent would otherwise spend turns mapping unknown layout, or two or more independent areas can run at once.
3. The worker will not run `jj` or `jayjay review` in this checkout.

Typical spawns:

- Unknown-layout search: where a behavior lives in core vs SwiftUI vs GPUI.
- One failing CI check, with the job name and log pinned.
- Summarize a large log or trace so the parent does not ingest it whole.
- A named read-only skill (`gpui-parity-audit` inventory, `ci-workflow-audit` table) that is not the whole user request.

Do not spawn when:

- The query is a needle: known path, one symbol, one URL, one file you can Read. Grep or Read in the parent.
- You would only forward the user's whole task to a general-purpose worker.
- The next step is a write, `jj`, or `jayjay review` in this checkout.
- The worker would need the parent conversation to make sense.

## Isolation

A worker in this checkout is not a sibling workspace.

- Workers here may read files, grep, and run non-jj commands. They must not invoke `jj` or `jayjay review`.
- Parallel jj still means another sibling workspace, with one agent serializing jj there. Two workers in one checkout that both talk to jj are the divergence bug in [Version Control](version-control.md#command-concurrency).
- If a subtask must run jj (rebase, describe, tests that snapshot), create the sibling first and treat that agent as the parent of that checkout.

## Prompts

Workers do not see the parent conversation. The prompt must stand alone:

- Repo root, the question, which focused guide to load, and the return shape.
- Stop conditions: do not edit, do not run `jj` or `jayjay review`, do not run `just test` / `just lint` / `just build`.
- Ask for a short evidence report, not a dump of file bodies the parent will re-read.
- Do not attach host MCP catalogs or SaaS tool schemas. The interface is this repo's files and CLI (`just`, `gh`).

## Models

- Parent keeps the session's stronger model for planning, architecture, jj mutations, and review.
- Worker default: the cheaper or faster tier the harness exposes. Locate, inventory, log fetch, and "where does X live" do not need frontier reasoning.
- Give a worker the parent's model only when the subtask itself is architecture or adversarial review.
- Do not hard-code vendor model slugs in this guide; they rot. Use the harness's cheaper/faster vs default/frontier tiers.

## Cursor

Cursor dispatches through `Task`. Use:

| Job | `subagent_type` |
| --- | --- |
| Unknown-layout search across crates or shells | `explore` |
| One failing PR check, already identified | `ci-investigator` |
| Drive the running app | `computerUse` |

Do not use `generalPurpose` to re-delegate the whole user task. When the tool lets you pick a model, pick the cheaper/faster tier for locate and inventory. Launch independent explores in one turn; do not serialize three searches the parent could have fanned out. Load [Run & Debug](run-debug.md) before `computerUse` or `ci-investigator`.

If the harness has no worker tool, search in-process with Grep/Read. That is still better than concurrent `jj` in this checkout.

## Anti-patterns

- Spawning because another harness would. Spawn because the search is unknown-layout or parallelizable.
- Three workers grepping the same symbol.
- A worker that returns fifty files for the parent to re-read.
- A worker running `just test`, `just lint`, `jj`, or `jayjay review` in this checkout.
- Loading extra MCP servers so a worker can "also do GitHub." Use `gh` / `just` / files.
