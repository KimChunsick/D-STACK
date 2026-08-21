# Codex adversarial review — Round 002

## Review scope
Re-review | serial | bundle 33385 bytes (round 001: 24264)

## GPT findings
[severity:medium][technical-correctness] The independent-grounding fix requires URL/date evidence for every probe, including internal-consistency and data-reading probes that direct inspection or recomputation can conclusively settle; demonstrable failures can therefore become `unverifiable`.
Sites: `codex/skills/socratic-audit/SKILL.md:47`, confirmed: `codex/skills/socratic-audit/SKILL.md:49`, `codex/skills/socratic-audit/SKILL.md:52`, `codex/skills/socratic-audit/SKILL.md:70`
Evidence: Implication probes compare the artifact's own claims and data-reading probes inspect recorded transformations, but the method admits only independently selected URL-cited evidence and treats web-unsettled probes as unverifiable.
Verification: A recorded local result claiming `5/10 = 80%` is directly refuted by recomputation, yet that answer has no independently selected URL or publication date and therefore fails the literal grounding rule.
Suggested direction: Permit labeled direct recomputation, local primary evidence, and formal/internal-consistency reasoning for those probe classes while retaining independent sourcing for externally checkable empirical claims.

[severity:medium][technical-correctness] The reconciliation fix lets any unexecuted deferred check cap its H without first establishing that the check is valid, relevant, and necessary; untrusted material can attach a redundant check and suppress an otherwise supported verdict.
Sites: `codex/skills/socratic-audit/SKILL.md:11`, confirmed: `codex/skills/socratic-audit/SKILL.md:62`, `codex/skills/socratic-audit/SKILL.md:77`, `codex/skills/socratic-audit/SKILL.md:95`
Evidence: The pending-check rule is unconditional and emits `unverifiable (pending check)`, despite the four-value verdict enum and the separate `unresolved checks` summary column.
Verification: An independently established H linked by the artifact to an unrelated pending comparison must still receive `unverifiable (pending check)` under the literal rule, even after the auditor determines that the comparison has no bearing on H.
Suggested direction: Cap H only when an audited pending check is necessary to establish or refute it; keep the verdict value `unverifiable` and record pending state in `unresolved checks`.

Omitted-detail: 0 low

GPT verdict: reject because the grounding and reconciliation fixes introduce concrete paths to incorrect core verdicts.

## Carried decisions
- F1-F5 (round 001): fixes stand; round 002 raised no regression against them.
- F6 (medium, over-broad independent-grounding): FIXED this round — grounding is now
  class-appropriate and labeled: independent URL-cited sources for external empirical
  claims; shown recomputation/inspection for data readings (a demonstrable arithmetic
  error is `refuted`, never `unverifiable`); formal reasoning over precisely quoted
  claims for internal-consistency probes.
- F7 (medium, unconditional pending-check cap): FIXED this round — a pending deferred
  check caps its H at `unverifiable` only after its BEARING is audited and judged
  necessary to establish or refute the H; an irrelevant pending check is recorded in
  `unresolved checks` without capping, and an artifact-asserted linkage is untrusted
  data, not a verdict instruction. The verdict enum stays four plain values; pending
  state lives in the `unresolved checks` column (the `(pending check)` annotations were
  removed).
- Cross-unit note: the residual `Out of scope:` phrasing T01's round flagged in this
  unit's task.md (frozen inside this round's bundle when found) is now fixed —
  ownership stated factually.
- Standing context: no-new-tests repo policy; the install.sh diff carries ONE
  pre-existing line from another workstream (worktree-create.sh map row), excluded from
  this unit's commit by hunk-level staging.

Consensus: disagreed
