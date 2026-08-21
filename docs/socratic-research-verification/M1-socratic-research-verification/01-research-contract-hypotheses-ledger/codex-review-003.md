# Codex adversarial review — Round 003

## Review scope
Re-review | serial | bundle 35542 bytes (round 002: 29917)

## GPT findings
[severity:medium][technical correctness] The closed-format fix remains incomplete: refusal applies only when a schema can carry none of the blocks, while "as that shape allows" permits schemas containing only one or two blocks to omit the remainder silently.
Sites: Primary: `codex/skills/adversarial-research/SKILL.md:64-73`; confirmed: `codex/skills/adversarial-research/SKILL.md:84-86`.
Evidence: The contract requires all three blocks but explicitly rejects only schemas with zero corresponding fields and otherwise uses whichever fields are provided.
Verification: A closed schema exposing only `hypotheses` does not meet the zero-field refusal condition, yet cannot encode the mandatory ledger or deferred-check blocks.
Suggested direction: Treat any format lacking an encoding for any required block as incomplete and require complete encoding or explicit refusal.

[severity:low][software structure] The task record is stale after the round-3 fix: it still describes two added sections and says omitted blocks are always appended as Markdown headings.
Sites: Primary: `task.md:20-42`; confirmed: `task.md:49-55,65-67`.
Evidence: The contract adds four substantive sections and now uses schema fields for structured outputs rather than universally appended headings.
Verification: Comparing the task record with `SKILL.md:22-73` shows both discrepancies.

Omitted-detail: 0 low

GPT verdict: reject — the reachability fix is effective, but partially compatible closed schemas can still silently lose mandatory research blocks.

## Carried decisions
- F1-F8: fixes stand; round 003 verified F8 (reachability) effective and raised one
  residual of F7 plus one doc-staleness low.
- F11 (medium, partial-schema silent drop): FIXED this round — a format lacking an
  encoding for ANY of the three blocks (none or only some) yields an incomplete
  artifact: the carried blocks are encoded and each missing one is flagged as the
  caller's defect; the format rule now says "flagging any block the shape cannot
  encode".
- F12 (low, stale task record): FIXED this round — the task doc now describes the three
  added sections + amended rule and the per-shape encoding accurately.
- F9 stands as accepted-residual (immutable P3 record; contract supersedes).
- Standing context: no-new-tests repo policy; caller file (pinned section list) is
  declared task T03's work.

Consensus: disagreed
