# .claude Control Center (Helios)

This folder is the Claude project control center for Helios.

## Anatomy

```text
your-project/
├── CLAUDE.md                team instructions, committed
├── CLAUDE.local.md          personal overrides, gitignored
│
└── .claude/
    ├── settings.json        permissions + config, committed
    ├── settings.local.json  personal permissions, gitignored
    │
    ├── commands/
    │   ├── review.md        → /project:review
    │   ├── fix-issue.md     → /project:fix-issue
    │   └── deploy.md        → /project:deploy
    │
    ├── rules/
    │   ├── code-style.md
    │   ├── testing.md
    │   └── api-conventions.md
    │
    ├── skills/
    │   ├── security-review/
    │   │   └── SKILL.md
    │   └── deploy/
    │       └── SKILL.md
    │
    └── agents/
        ├── code-reviewer.md
        └── security-auditor.md
```

## What Each Area Does

- `settings.json`: shared team-level Claude project defaults and permission intent.
- `settings.local.json`: local machine overrides only.
- `commands/`: reusable slash-command workflows for review/fix/deploy.
- `rules/`: modular standards for code quality, testing, and API consistency.
- `skills/`: auto-invoked workflow packs for focused domains.
- `agents/`: specialized subagent personas for targeted analysis tasks.

## Local vs Committed

- Committed: `.claude/**`, `CLAUDE.md`
- Personal only: `CLAUDE.local.md`, `.claude/settings.local.json`

## Helios-Specific Notes

- This setup is aligned with repository workflows spanning `omni-lang`, `helios-framework`, docs, and vault notes.
- Use command/rule/skill combinations to keep changes deterministic, testable, and documented.
