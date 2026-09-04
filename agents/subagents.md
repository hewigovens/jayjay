# Subagents Guide

Load this file before dispatching a subagent or deciding whether a search belongs in the parent session. How to spawn is the harness's business; what to spawn, what a worker may touch, and which model tier it gets are this repo's. jj isolation rules stay in [Version Control](version-control.md).

## Roles

- **Parent**: the session talking to the user. It plans, owns crate-boundary decisions, runs jj in this workspace, runs the inner loop and cleanup rounds, judges review findings, and writes `jj describe`.
- **Worker**: a bounded read job with a stated question and return shape. It locates, inventories, summarizes a log or diff, or runs a named read-only skill, and returns evidence: paths, line-level claims, and what it did not find.
- **Delegate**: a subagent that implements. Concurrent work gets a sibling workspace it owns alone, where it follows the Feature Loop as the parent of that checkout. A delegate working in this checkout runs serially: the parent waits and runs nothing jj-aware until it returns.

## When to spawn

Spawn when the subtask is well specified and the parent would otherwise spend turns mapping unknown layout, or when independent areas can run at once.

- Where a behavior lives across core, SwiftUI, and GPUI.
- One failing CI job, with the job name and log pinned.
- A log, trace, or diff too large for the parent to ingest whole.
- A named read-only skill such as `gpui-parity-audit` or `ci-workflow-audit` that is not the whole user request.
- Mechanical implementation or a cleanup round, as a delegate.

Do not spawn when:

- The query is a needle: a known path, one symbol, one file. Grep or read it in the parent.
- You would only forward the user's whole task to a general-purpose subagent.
- The subagent would need the parent conversation to make sense.

## Isolation

A worker in this checkout is not a sibling workspace.

- It may read files, grep, and run commands that do not touch jj state. jj reads only with `--ignore-working-copy`; no snapshot, no mutation, no `jayjay review`.
- It does not edit files and does not run `just test`, `just lint`, `just build`, or `xcodebuild`; those contend with the parent's build and test session.
- Two agents that both snapshot one checkout are the divergence bug in [Version Control](version-control.md#command-concurrency). Anything that must snapshot gets a sibling workspace first.

## Prompts

Subagents do not see the parent conversation. The prompt must stand alone:

- Repository root, the question, which focused guide to load, and the return shape.
- The stop conditions from Isolation above.
- Ask for a short evidence report, not file bodies the parent will re-read.
- Give it this repo's files and CLI: `jj --ignore-working-copy`, `just`, `gh`. Do not attach extra tool catalogs or MCP servers so it can "also do GitHub"; `gh` already does, and every attached schema is paid for on every request.

## Models

- The parent keeps the session's strongest tier for planning, jj mutations, and review judgment.
- Workers and delegates default to the cheaper or faster tier the harness offers. Locate, inventory, log summary, mechanical implementation, and cleanup rounds do not need frontier reasoning.
- Give a subagent the parent's tier only when the subtask itself is architecture or adversarial review.
- Name tiers, not vendor model ids; ids rot.

## Anti-patterns

- Spawning because the task looks big, not because the search is unknown-layout or parallel.
- Three workers grepping the same symbol.
- A worker that returns fifty files for the parent to re-read.
- A background subagent driving the app or a test session; take it over in the foreground per [Run & Debug](run-debug.md#test-runner-recovery).
- Fan-out without a cap: a workflow or loop that spawns per item validates its input and bounds the count before the first spawn.
