# Codex adversarial review — Round 004

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 36815 bytes (r1 12593, r2 23695, r3 33441);
round 001 compacted. Allowlist unchanged: the review-unit folder,
`claude/skills/full-cycle/SKILL.md`.

## GPT findings

[severity:high][technical correctness] The P6 fence still proves only its self-declared `DOCS` array, not the complete review-unit set; duplicate entries can conceal an omitted unit while preserving the expected count.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 schema and registration fence; confirmed: `claude/bin/dstack` `cmd_reg`, `claude/hooks/fullcycle-gate.sh` registry scan.
Evidence: Same-session registration is idempotent, every array element is checked independently, and the Stop hook discovers only registered records rather than expected review units.
Verification: The current fence under bash and zsh accepted `GOAL,u1,u1`, printing “3 documents” although only two unique records existed and required `u2` was absent.
Suggested direction: Derive a unique expected review-unit set from P5’s durable decomposition and granularity, then compare registration against that exact set instead of using `DOCS` as both assertion and proof.

[severity:high][technical correctness] P6 still presents foreign ownership as recoverable by applying `reclaim`, contradicting the autonomy stop and enabling automatic theft of a live session’s gate.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 failure prose and `autonomy.stops`; confirmed: `claude/bin/dstack` `cmd_reclaim`, `claude/hooks/fullcycle-gate.sh` owner filtering.
Evidence: P6 groups foreign ownership with recoverable failures and says to apply `reclaim`; the stop table says it must not run autonomously because liveness is unknown.
Verification: `cmd_reclaim` replaces the owner with `mv -f`, and the hook skips records whose owner differs; Round 002’s direct probe observed the original session continue ungated.
Suggested direction: Remove the recovery outcome from P6 and defer foreign ownership exclusively to the stop-table branch requiring proven orphanhood or user authorization.

[severity:medium][technical correctness] Review failure and cap transitions still have competing authorities, so unavailable models and concrete-medium closure do not select a unique next state.
Sites: `claude/skills/full-cycle/SKILL.md` — `autonomy` and P9 prose; confirmed: `claude/skills/codex-review/SKILL.md` Steps 2a, 2, and 4.
Evidence: Step 2a reruns every nonzero result while model unavailability says stop; P9 escalates “blockers” at the cap while autonomy and Step 4 escalate only concrete highs and record-close concrete mediums.
Verification: `codex exec --definitely-not-a-real-flag` returned 2, demonstrating a diagnosed permanent nonzero reaches the unconditional rerun rule; tracing round five with one concrete medium reaches both user escalation and `Consensus: resolved`.
Suggested direction: Make Step 2a and P9 defer to one precedence table: retry only diagnosed transient failures, and escalate cap closure only for concrete highs.

[severity:medium][technical correctness] The acknowledged SIGPROF orphan path can still delete a live Codex process’s scratch directory because both invoked wrappers install unconditional EXIT cleanup.
Sites: `claude/skills/full-cycle/SKILL.md` — `waits.external-residuals`; confirmed: `claude/skills/codex-review/SKILL.md` Step 2, `claude/skills/codex-research/SKILL.md` Step 2.
Evidence: Full-cycle says SIGPROF can orphan the child and cleanup must be avoided, but both recipes remove `SCRATCH` on EXIT; the review recipe additionally cleans in its signal handlers.
Verification: On deployed bash 3.2.57, SIGPROF killed the supervisor with rc=155 while its child remained alive, and the outer EXIT handler observed that child alive when cleanup ran.
Suggested direction: Clean scratch only after a terminal run record proves process-group quiescence; leave it intact for nonterminal captures.

[severity:low][software structure] The task record still contradicts the current instruction and its parsed schema.
Sites: `docs/autonomous-goal-loop/M2-unattended-contract/04-full-cycle-unattended-contract/task.md`; confirmed: `claude/skills/full-cycle/SKILL.md`.
Evidence: It retains “nothing else in that call” and records autonomy keys without `internal-recoveries`.
Verification: Direct Ruby YAML parsing returned `rule, internal-recoveries, stops, bounds, notify`, while the task records only four keys.

Omitted-detail: 0 low

GPT verdict: reject — P6 can still certify an omitted review unit or steal a live owner’s gate, while review transitions and orphan cleanup retain reproducible unattended-execution failures.

## Carried decisions
- **A hand-listed expected set is its own proof and proves nothing.** `DOCS=(GOAL u1 u1)` printed
  "3 documents" with a required unit absent, because same-session `reg` is idempotent and every
  element was checked independently. The set is now DERIVED — `find` at the depth the granularity
  fixes, since P6 scaffolds exactly one `task.md` per review unit and the filesystem cannot omit or
  duplicate. Verified across six scenarios.
- **`find` proves what was SCAFFOLDED, not what was DECOMPOSED**, so the count is cross-checked
  against GOAL.md's task rows — the same section the parallelism checker parses. Without it a unit
  whose folder was never created is invisible and the fence confirms the smaller set. Verified in
  both directions: 2 scaffolded vs 3 rows BLOCKS, 4 scaffolded vs 3 rows BLOCKS.
- **P6 names no failure outcomes of its own; `scheduling.autonomy` decides.** Prose here that also
  routed failures is what kept `reclaim` alive as a "recovery" for foreign ownership after the stop
  table already forbade it. Third repair of the same defect, and the fix is to remove the second
  authority rather than to reword it.
- **Scratch cleanup is CONDITIONAL on `<run-dir>/exit` existing.** `dstack run` publishes that file
  only after confirming its child's process group is gone, so it is the quiescence proof. An
  unconditional EXIT trap deletes a live `codex exec`'s cwd whenever `dstack` died to something it
  cannot trap — reproduced with `SIGPROF`, which kills the supervisor at rc=155 while the child
  lives.
- **The precedence table overrides any per-skill retry text.** `codex-review` Step 2a says to re-run
  every nonzero result; a missing dependency or a rejected model pin is a stop, and retrying it
  burns rounds and changes nothing.

Consensus: disagreed
