---
name: refresh-agent-guidance
description: Refine repository agent guidance from session corrections, recurring review findings, or an explicit guidance-maintenance request. Documentation-only; memory changes require explicit authorization.
---

# Refresh Agent Guidance

Keep reusable repository lessons in the smallest canonical document. Follow [AGENTS.md](../../../AGENTS.md) for workspace isolation, task authority, validation, and local description. This workflow does not authorize code changes or publication.

## Scope and evidence

Use the requested history range or supplied corrections. Start with the relevant current guidance, then inspect only the memory and transcripts needed to support a candidate lesson. Use the active harness's documented discovery and access mechanisms; do not assume a vendor-specific path, context inheritance, or writable memory store. If history is unavailable, work from supplied evidence and report the limitation.

- Current contract: `AGENTS.md`, `CONTRIBUTING.md`, and the relevant `agents/*.md` or `.agents/skills/*/SKILL.md`.
- Previous guidance changes: `jj --ignore-working-copy log -r 'files("AGENTS.md") | files("agents") | files(".agents")' --limit 5`, then inspect the relevant changes.
- Review history: use it when the request concerns review churn or the supplied evidence points to repeated findings. A review count is a triage signal, not proof that a new rule is needed.

For a review-history pass, these read-only commands can locate merged PRs and their reviews. In a sibling workspace, set `GH_REPO=hewigovens/jayjay` so `gh` can resolve the repository:

```bash
gh pr list --state merged --limit 20 --json number,title,mergedAt
gh api "repos/{owner}/{repo}/pulls/<n>/reviews"
gh api "repos/{owner}/{repo}/pulls/<n>/comments"
```

Choose the range and pagination to cover the requested period; inspect the comments behind any claimed recurring failure. Do not run a review-history sweep for a wording-only correction.

## Refine the contract

1. Promote lessons that are repeated, costly to rediscover, security-sensitive, or non-obvious repository architecture. Look for demonstrated wasted work or errors; model tier, inherited context, and broad tests are not inherently mistakes.
2. Reject transient state, credentials, unverified workarounds, duplicated rules, and preferences that are not repository policy. Distinguish a one-task instruction from a reusable constraint.
3. Verify promoted claims against current source, scripts, and CI. Where evidence is incomplete, report a candidate rather than adding a mandatory rule.
4. Update the smallest canonical source:
   - `AGENTS.md`: routing, task authority, workflow, and shared principles.
   - `agents/<area>.md`: contracts and pitfalls for that area.
   - `.agents/skills/<name>/SKILL.md`: a coherent reusable workflow. Preserve existing discovery metadata and `.claude/skills/<name>` symlinks; new repository skills follow the same symlink convention.
5. Reconcile contradictions across the affected documents. Preserve operational invariants; express routine choices as defaults with decision criteria. Link to shared policy instead of copying it.
6. Check the full diff, relative links, heading anchors, and skill frontmatter. Use the repository's documentation validation policy; no application builds are needed for prose-only edits.

## Memory

Reading memory does not authorize editing it. Change memory only when explicitly requested and only through the active harness's supported mechanism. If memory is generated or read-only, use its designated update channel when available; otherwise report the contradiction for the user. Do not rewrite memory files directly merely because they disagree with repository guidance.

## Report

Summarize the promoted rules and their evidence, material rejected candidates, checks run, and any unresolved contradictions. Report memory edits only if authorized and actually performed. Mention review-load findings or automation candidates only when relevant to the request; implementing a script or recipe is separate code work.
