# Codex adversarial review — Round 002

## Review scope
Re-review of Round 001's repairs. Bundle: this unit's `task.md`, round 001, and the scoped
diffs of `claude/CLAUDE.md` and `claude/skills/full-cycle/tests/skill-schema.test.sh`.

## GPT findings
[severity:medium][real Why] The prior startup-routing blocker remains: exploratory frontend work simultaneously matches unconditional frontend delegation and mandatory orchestrator retention.
Sites: `claude/CLAUDE.md:11-17`; confirmed: `task.md` “What was done,” `codex-review-001.md` first finding.
Evidence: The revised copy says frontend work is delegated “unconditionally,” then says all exploratory work stays with the orchestrator.
Verification: An exploratory frontend task still receives opposite ownership depending on which adjacent rule is applied.
Suggested direction: Explicitly state that the frontend rule takes precedence over otherwise applicable task-shape retention rules.

[severity:medium][technical correctness] The repaired guard still uses raw text as schema data, accepting comment-only null keys and potentially rejecting semantically valid negated PARALLEL text.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh:118-141`; confirmed: `task.md` “Direct verification.”
Evidence: `-z` treats an indented comment as content, while `grep -qi PARALLEL` also matches comments; Ruby parses both comment-only keys as `nil`.
Verification: With comment-only bodies, the exact extractor reported both blocks nonempty and satisfied both placement outcomes, while `YAML.safe_load` returned `{"requires"=>nil, "parallel-when"=>nil}`.
Suggested direction: Use the existing Ruby YAML parse to require nonempty arrays and inspect their actual entries for PARALLEL placement.

Omitted-detail: 0 low

GPT verdict: reject — Two concrete medium blockers remain: startup guidance still gives contradictory ownership and the schema guard can pass null gate definitions.

## Bundle size (the ratchet, recorded)

R1 11,193 · **R2 13,457** bytes. Violated, by 2,264 bytes. Same cause as T01's and T02's: round 001
joins the bundle and the task record grew with the provenance the round demanded. Already filed as
F-02 — the ratchet rule assumes growth comes from carried maintainer prose, and in this Goal it
comes from the reviewed artifacts themselves. Not fixed in the round that broke it.

## Round outcome

Two mediums, both real, both fixed. Blocking count 2 (R1) → 2 (R2).

The second finding is the third consecutive time this guard has been wrong, and each version failed
for a reason the previous version's control could not see. Text-matching one phrase missed the live
wording. Scoping by indentation with awk read comments as content. Both were "check a string", and
the question is about schema shape, which strings cannot answer. The check now reads the parsed
document. Recording that progression because the lesson is not about this file: a control that
exercises anything other than the live artifact proves nothing about the live artifact.

Round 003 re-reviews the repairs.

Consensus: disagreed
