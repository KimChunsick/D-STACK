document: claude/skills/dstack-develop/SKILL.md
Follow runtime.md's coordinator boundary, compact receipt and interruptible wait protocol.
Retained secondary attribution.
`docs/explanation/the-phase-loop.md`: "Each executor gets a fresh 200k-token context window loaded with exactly what it needs"
`skills/gsd-execute-phase/SKILL.md`: "Orchestrator stays lean: discover plans, analyze dependencies, group into waves, spawn subagents, collect results."
| 5 | `dstack review --scope plan --plan P3` | independent review |
| 6 | `dstack review seal --from <output>` | seal |
| 7 | `dstack plan done P3` | close |
