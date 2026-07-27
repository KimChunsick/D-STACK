# Codex adversarial review — Round 006

## Review scope
Batch pass 1 (consolidated round) | `REVIEW_MODE=serial` | bundle 43361 bytes. Allowlist: the review-unit folder, `claude/skills/full-cycle/SKILL.md`.

## GPT findings

[severity:high][technical correctness] P6 still lacks one exact review-unit set: its caller always registers task-depth paths, milestone mode compares only counts, and task mode collapses duplicate scaffold IDs with `uniq`.
Sites: `claude/skills/full-cycle/SKILL.md` — P6 recipe; confirmed: `claude/skills/full-cycle/check-registration.sh` — identity comparison; task document — derived-set claim.
Evidence: Correct milestone units are never registered by the recipe, while milestones M1/M2 can be substituted with M1/M9 without detection.
Verification: Bash and zsh returned rc=0 for declared M1/M2 versus scaffolded M1/M9; a declared T01 with both `01-a` and `01-b` produced no missing or extra identity.
Suggested direction: Derive exact review-unit paths once and use that same list for registration and status verification.

[severity:high][technical correctness] The registration checker does not parse the same declaration source as the scheduler: fenced examples can override both granularity and task identities.
Sites: `claude/skills/full-cycle/check-registration.sh` — granularity and declaration parsing; confirmed: `claude/skills/full-cycle/check-parallel.sh` — global fence handling; `claude/skills/full-cycle/SKILL.md` — GOAL template and parser-equivalence claim.
Evidence: Its `grep | head` accepts a granularity line anywhere, and its AWK ignores an opening fence before `## Milestones & tasks`; the required granularity field is also absent from the GOAL template.
Verification: The exact AWK selected fenced M9/T99 and skipped real M1/T01; the exact grep selected fenced `milestone` over the later real `task` declaration.
Suggested direction: Add one mandatory granularity field to the template and reuse a single globally fence-aware declaration grammar.

[severity:medium][technical correctness] Terminal-record-gated scratch cleanup is still broken in the review wrapper because `RUNDIR="$RD"` executes before `RD` is defined.
Sites: `claude/skills/codex-review/SKILL.md` — background launch recipe; confirmed: `claude/skills/full-cycle/SKILL.md` — cleanup guarantee; `claude/skills/codex-research/SKILL.md` — stale unconditional-cleanup caveat.
Evidence: With ordinary unset `RD`, the EXIT trap watches `/exit` and leaks scratch after successful rounds; an inherited stale `RD` can instead authorize cleanup using another run’s terminal record.
Verification: The current assignment order produced `RUNDIR=<>` under both Bash and zsh after `RD` was subsequently set to `/actual/run`.
Suggested direction: Define the current `RD` and `RUNDIR` before installing traps, then align the research residual prose with its conditional trap.

[severity:medium][technical correctness] Concrete-MEDIUM cap closure still has two transitions: P9 escalates all “blockers,” while autonomy and the review skill permit a human stop only for concrete HIGH findings.
Sites: `claude/skills/full-cycle/SKILL.md` — P9 and `autonomy.stops`; confirmed: `claude/skills/codex-review/SKILL.md` — cap closure.
Evidence: High and medium findings are both classified as blockers, but the review skill records concrete mediums as follow-ups and seals `Consensus: resolved`.
Verification: Tracing round five with one concrete MEDIUM reaches both user escalation and unattended resolved closure.
Suggested direction: Make P9 defer explicitly to autonomy and reserve cap escalation for concrete HIGH findings.

[severity:medium][technical correctness] The claimed exit-2 fail-loud guarantee is false when identity-check dependencies fail; `comm`, `sed`, or `uniq` errors can erase both deltas and allow success.
Sites: `claude/skills/full-cycle/SKILL.md` — exit-code guarantee; confirmed: `claude/skills/full-cycle/check-registration.sh` — unchecked transformation substitutions.
Evidence: The script uses only `set -u`; failed identity command substitutions are not checked before empty `missing` and `extra` values are accepted.
Verification: A `comm` returning 127 yielded empty missing/extra values and final rc=0 under both Bash and zsh.
Suggested direction: Preflight every required utility and make each identity transformation’s status part of the gate.

Omitted-detail: 0 low

GPT verdict: reject — P6 can still certify the wrong review-unit set, while cleanup and review-cap control flow retain concrete unattended-execution failures.

## Carried decisions
- **P6's registration loop READS the depth, it does not write it.** The fence iterated a literal
  `<Mn>/<NN-task>/task.md`, so it registered task-depth documents even for a Goal that declared
  milestone granularity — the exact misregistration the granularity table above it warns about,
  baked into the recipe that implements it. It now calls `check-registration.sh --depth`, which
  returns 3 or 2 from the same GOAL.md parse the check itself uses, so the two cannot disagree.
- **The checker parses GOAL.md exactly as `check-parallel.sh` does.** Fences tracked globally from
  line one, task rows at column zero with the `-` marker, a repeated section heading keeping the
  section open. Two parsers that disagree about what a declaration is are worse than one that is
  wrong, because the disagreement is invisible — measured on a fixture with a fenced example above
  the section, the old parser read the FAKE `per milestone` granularity and the FAKE task rows and
  none of the real ones.
- **Cap closure has ONE authority, and it is `codex-review` §4.** P9 said to escalate to the user
  when the budget is reached with `blockers` still open; `blockers` means high AND medium, so it
  demanded a human for exactly the case §4 and `autonomy.stops` both close without one. Two
  transitions out of one state is how an unattended run stalls on a medium nobody needs to see.
- **`reclaim` has no autonomous carve-out, because the case it named does not exist.** The stop
  entry allowed it for a "provably orphaned" handoff whose owner is this session — but `reg` returns
  0 for a document this session already owns, so that state is never reached, and every other case
  is unprovable without a liveness signal. The other half, "or the user says so", is not autonomy.
- **The `RUNDIR` ordering defect is real and was fixed where it lives**, in `codex-review`'s Step 2
  fence rather than here; this file's `waits.external` describes the contract, not the recipe.

Consensus: disagreed
