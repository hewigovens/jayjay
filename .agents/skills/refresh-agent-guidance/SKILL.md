---
name: refresh-agent-guidance
description: Promote corrections and steering from past agent sessions into AGENTS.md, agents/*.md, and .agents/skills, and reconcile the agent's own memory with that contract. Documentation-only; never commits or pushes.
---

# Refresh Agent Guidance

Run from the repository root with any coding agent. One agent per pass: each agent reads its own memory, and the pass ends uncommitted for review.

## When to use

- On a schedule, or after a pull request that needed three or more review rounds.
- When a session ended with a correction such as "only", "no need", "findings only", "trace the full flow", "simplify", "dedupe", or "before commit".
- Never as part of a feature change.

## Inputs

Read only what you are authorized to access. Start with the current contract, then the agent's own memory, then transcripts since the previous pass.

| Source | Where |
| --- | --- |
| Current contract | `AGENTS.md`, `agents/*.md`, `.agents/skills/*/SKILL.md`, `CONTRIBUTING.md` |
| Previous passes | `git log -5 --stat -- AGENTS.md agents .agents` |
| Claude Code memory | `~/.claude/projects/<project-slug>/memory/` (`MEMORY.md` is the index; the slug is the repository path with `/` replaced by `-`) |
| Claude Code transcripts | `~/.claude/projects/<project-slug>/*.jsonl` |
| Codex memory | `~/.codex/memories/memory_summary.md`, `~/.codex/memories/rollout_summaries/`, and generated skills under `~/.codex/memories/skills/*/SKILL.md` |
| Other agents | their documented memory location, if any |

## Procedure

1. Run `git status --short` and preserve existing changes. Do not run snapshotting `jj` commands; this pass edits documentation only.
2. Inventory the contract before editing so each rule lands in the smallest canonical source and nothing is duplicated.
3. Build a candidate list from memory and transcripts. Promote lessons that are repeated, costly to rediscover, security-sensitive, or non-obvious repository architecture.
4. Reject transient paths, commit ids, versions, service status, tool/network/auth failures, credentials, unverified workarounds, duplicated rules, and personal preferences that are not team policy.
5. Verify every promoted claim against current code, scripts, and CI before writing it: file paths, function names, flags, job names.
6. Update the smallest canonical source:
   - `AGENTS.md`: routing, task authority, feature loop, and principles only. Keep it short; it is loaded into every session.
   - `agents/<area>.md`: contracts and pitfalls for that area.
   - `.agents/skills/<name>/SKILL.md`: a repeatable multi-step procedure. Create a new skill only for a coherent reusable workflow, and symlink it from `.claude/skills/<name>` so Claude Code discovers it.
7. Reconcile the agent's memory with the contract: delete or rewrite memory entries that contradict `AGENTS.md`, and mark promoted entries so they are not promoted again.
8. Re-read changed files for contradictions and duplication, confirm every relative link and heading anchor resolves, and run `git diff --check`.

## Writing rules

Write direct rules, not session stories: trigger, required action, verification boundary, important exception. No names, quotations, dates, session ids, credentials, or temporary paths. One source of truth per rule; link instead of repeating.

## Report

List what was promoted (file and rule), what was rejected and why, memory entries reconciled, checks run, overlap with existing uncommitted changes, and confirmation that no code or external state changed. Leave edits uncommitted.
