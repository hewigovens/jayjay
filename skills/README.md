# Agent skills

Skills for coding agents working alongside JayJay, in the [Agent Skills](https://agentskills.io) format.

- [`jayjay`](jayjay/SKILL.md): read and resolve JayJay review notes, pre-mark straightforward files, leave notes for critical risks, and set JayJay up as jj's diff and merge tool.

Install with the [`skills`](https://github.com/vercel-labs/skills) CLI, or copy the folder into your agent's skills directory (for example `~/.claude/skills/jayjay` or `~/.codex/skills/jayjay`):

```bash
npx skills add hewigovens/jayjay --skill jayjay
```

The skill needs the `jayjay` command on `PATH`; on macOS use JayJay ▸ Install Command Line Tool….
