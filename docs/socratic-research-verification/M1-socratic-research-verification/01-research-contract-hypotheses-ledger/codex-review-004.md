# Codex adversarial review — Round 004

## Review scope
Re-review | serial | bundle 34122 bytes (round 003: 35542)

## GPT findings
[severity:medium][technical correctness] The partial-schema fix still has no defined behavior when a closed schema lacks both required block fields and an error/refusal channel.
Sites: Primary: `codex/skills/adversarial-research/SKILL.md:70-74`; confirmed: `codex/skills/adversarial-research/SKILL.md:85-88`.
Evidence: The contract requires flagging every missing block "in whatever channel the format leaves," while also requiring exact schema compliance; it does not handle formats that leave no such channel.
Verification: A closed schema containing only `hypotheses` with `additionalProperties: false` permits neither the other blocks nor an explanatory flag, so every possible response violates at least one requirement.
Suggested direction: Require refusal before artifact generation whenever the requested shape cannot encode all three blocks.

[severity:low][software structure] The claimed task-record fix remains incomplete because the record still describes universal appending and later calls the change "two new contract sections."
Sites: Primary: `task.md:53-59`; confirmed: `task.md:69-71`.
Evidence: The contract now uses schema fields for structured outputs and adds more than the two sections claimed in the file summary.
Verification: Comparing these passages with `SKILL.md:22-74` reproduces both discrepancies.

Omitted-detail: 0 low

GPT verdict: reject — the contract detects partial schemas but still cannot produce or communicate a defined failure when a closed schema provides no reporting channel.

## Carried decisions
- F1-F11: fixes stand; round 004 verified F11 effective for detectable partial schemas
  and narrowed the residual to the no-channel case.
- F13 (medium, no-channel closed schema): FIXED this round — when the requested shape can
  carry neither the blocks nor a note about them, the researcher REFUSES on the first
  line instead of generating an artifact ("a refusal is visible, a silently incomplete
  artifact is not"); the format rule adds "refusing outright when the shape cannot even
  carry the flag".
- F14 (low, residual stale task-record passages): FIXED this round — the sweep passage
  now describes per-shape encoding with refusal, and the files-changed bullet counts the
  three sections + amended rule.
- F9 stands as accepted-residual (immutable P3 record; contract supersedes).
- Standing context: no-new-tests repo policy; caller file (pinned section list) is
  declared task T03's work.

Consensus: disagreed
