# Codex adversarial review — Round 005

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 42878 bytes. Round cap for a per-task unit. Its remaining HIGH is what the maintainer's decision resolved: the P6 proof moved out of this file and into `check-registration.sh` (T06).

## GPT findings

[severity:high][technical correctness] P6 still does not derive the expected set: `GRAN` is hand-set, task mode compares only cardinality, and milestone mode never compares scaffolds with GOAL. Equal-count substitutions or omitted milestones can therefore pass while intended units remain ungated.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 registration fence; confirmed: `docs/autonomous-goal-loop/M2-unattended-contract/04-full-cycle-unattended-contract/task.md` — derivation and E2E claims.
Evidence: Neither task identities nor milestone identities are parsed from GOAL, and nothing verifies that `GRAN` matches GOAL’s declaration.
Verification: Bash 3.2 and zsh 5.9 probes returned rc=0 for three task rows backed by two intended units plus one unrelated unit, for an incomplete milestone scaffold, and for `GRAN=task` with a milestone declaration.
Suggested direction: Derive granularity and stable unit identities from GOAL, then compare exact expected and scaffolded sets for both granularities.

[severity:medium][technical correctness] The new other-depth check rejects its intended success state whenever an unregistered other-depth `task.md` exists.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 `OTHER` validation loop.
Evidence: A successful `grep` enters the blocking branch, but the desired non-match leaves the loop body and therefore the `while` pipeline with status 1; the trailing `|| exit 1` then aborts silently.
Verification: The exact loop returned rc=1 under Bash 3.2 and zsh 5.9 when `OTHER` contained one correctly unregistered path and `st` was empty.
Suggested direction: Express the lookup as an explicit conditional whose non-match branch completes successfully.

[severity:medium][technical correctness] Both `find | sort` assignments mask `find` failures, so partial unit discovery or a missed other-depth document can still be treated as complete.
Sites: `claude/skills/full-cycle/SKILL.md` — `UNITS` and `OTHER` assignments in the P6 fence.
Evidence: `set -e` observes the final `sort` status, not the failing `find`; nonempty partial output bypasses the subsequent emptiness check.
Verification: In Bash 3.2 and zsh 5.9, a `find` probe that emitted one path and returned 1 produced assignment rc=0 with that path accepted.
Suggested direction: Require pipeline failure propagation before consuming either discovery result.

[severity:medium][technical correctness] The terminal-record cleanup invariant remains contradictory at the executable recipe sites: full-cycle requires conditional cleanup, while the invoked review and research wrappers remain unconditional.
Sites: `claude/skills/full-cycle/SKILL.md` — `waits.external-residuals`; confirmed: `claude/skills/codex-review/SKILL.md` and `claude/skills/codex-research/SKILL.md` launch recipes.
Evidence: The scoped change modifies only full-cycle; the supplied Round 004 evidence confirms unconditional EXIT cleanup in both invoked wrappers.
Verification: The carried Bash 3.2 SIGPROF probe observed the child still alive when unconditional cleanup ran; neither wrapper has a current change removing that path.
Suggested direction: Apply terminal-record-gated cleanup directly to both invoked wrappers.

[severity:medium][technical correctness] Concrete-MEDIUM cap closure still has competing transitions: unchanged P9 escalates “blockers,” while `autonomy.stops` permits a human pause only for concrete HIGH and the review skill records concrete-MEDIUM closure.
Sites: `claude/skills/full-cycle/SKILL.md` — P9 and `autonomy`; confirmed: `claude/skills/codex-review/SKILL.md` — Step 4.
Evidence: The new override addresses per-skill retry text only; no supplied P9 change removes its broader escalation branch.
Verification: Tracing the fifth round with only a concrete MEDIUM still reaches both user escalation and `Consensus: resolved`.
Suggested direction: Make P9 explicitly defer to `autonomy` and reserve cap escalation for concrete HIGH findings.

Omitted-detail: 0 low

GPT verdict: reject — P6 can still certify the wrong or incomplete registration set, and multiple reproducible shell and transition failures remain unresolved.

## Carried decisions
- **The P6 proof left this file entirely.** Five rounds, five repairs, five new defects — a
  hand-listed array that was its own proof, a `find` derivation comparing counts not identities,
  `GRAN` hand-set so a milestone Goal checked at the wrong depth, `find | sort` masking a failing
  `find`, a loop returning 1 on its success path. The maintainer's decision was to move it into
  `check-registration.sh` (T06), and the fence here is now three lines that register and invoke it.
- **`find | sort` reports SORT's status.** A find that fails after emitting one path assigns rc=0
  and the partial list is accepted. Run find alone, check it, then sort.
- **`grep && { exit 1; }` inside a `while` leaves status 1 on the SUCCESS path**, so a trailing
  `|| exit 1` aborts the whole fence silently. Reproduced under bash 3.2 and zsh 5.9. Collect the
  offenders into a variable and test it, or use an explicit `if`.
- The terminal-record-gated cleanup was applied to BOTH invoked wrappers, not only stated here —
  this round could not see that because the bundle carried only this file.

Consensus: disagreed
