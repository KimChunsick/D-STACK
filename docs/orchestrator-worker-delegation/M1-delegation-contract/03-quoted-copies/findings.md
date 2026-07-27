# Finding ledger — T03 (quoted copies)

Blocking findings per round: **R1 2 · R2 2 · R3 1 · R4 3 · R5 3 · R6 2**
Bundle bytes: R1 11,193 · R2 13,457 · R3 19,117 · R4 23,041 · R5 29,197 · R6 38,536 (ratchet
VIOLATED every round since R1 — F-02)

Closed at R6 by the non-convergence rule — 1, 3, 3 is not strictly decreasing. Every concrete
finding raised in six rounds was fixed before its round sealed; no finding was a false positive.
Five of the six rounds landed on the same object, the regression guard, each time in a place the
previous repair had opened.

## Open

- [low][DX] the `AGENTS.md` citation in `task.md` names a policy whose text is not in the bundle.
  Adding it would grow the allowlist, which the ratchet rule forbids mid-loop. Follow-up.
- [low][DX] round 005's sealed round file states how this loop would close. Sealed rounds are the
  audit trail and were not rewritten; recorded in round 006 instead.

## Closed in round 6

- [medium][correctness] token pins carry no polarity — "a verdict of PARALLEL must never permit
  concurrent execution" and "`frontend-dev` never OUTRANKS orchestrator retention" both passed;
  pins are now whole normalized decisions, one equality and five full canonical sentences
- [medium][correctness] `safe_load` returns only the first document of a stream, so a second
  document in one fence carried a regressed node past every check — one document per fence, or fail

## Closed in round 5

- [medium][correctness] duplicate `worker-fanout:` keys inside ONE mapping collapse during
  `safe_load`, so counting parsed nodes could not see the first — duplicate mapping keys are now
  rejected at the Psych AST, before loading
- [medium][correctness] types and a bare `PARALLEL` token are not semantics: `delegate-when:
  [a task exists]` and `parallel-when: [PARALLEL must never be used]` passed while inverting the
  contract — each key must now state its decision, read off the parsed field
- [medium][security] round 004 converted only the new parser; the pre-existing parse-validity loop
  still ran `load_file` over the same content — both passes now use `safe_load(File.read(...))`

## Closed in round 4

- [medium][correctness] `keep-in-the-orchestrator` decides delegation too and was not in the
  forbidden-location set — all three gating lists are iterated, and the failure names which one
- [medium][correctness] `- # commented out` parses as `[nil]` and passed as non-empty; a duplicate
  `worker-fanout` node silently overwrote the first — entries must be non-blank strings, and
  exactly one node is required
- [medium][security] `YAML.load_file` constructs tagged Ruby objects before any structural check
  runs — the new parser uses `safe_load` over `File.read`
- [low][security] the record asserted "no TDD, no new tests" and a review-timing placeholder with no
  governing artifact cited — `AGENTS.md` and its section are now named

## Closed in round 3

- [medium][correctness] the parsed guard rejected PARALLEL from `requires` only, so the same line
  under `delegate-when` gated delegation while the check reported it did not; `honest-scope` was
  still matched globally — the gating set is now iterated and both prose keys read off the parsed
  node

## Closed in round 2

- [medium][real Why] exploratory frontend work matched both the frontend rule and the
  orchestrator-retention rule with no precedence stated — section 0 now says 0.2 outranks the list
- [medium][correctness] the guard still read raw text as schema data: a comment-only `requires:`
  body was non-empty to `-z`, matched `grep PARALLEL`, and loads as `nil` — the placement check now
  reads the parsed `scheduling.modes.worker-fanout` node

## Closed in round 1

- [medium][real Why] `claude/CLAUDE.md` section 0 subjected frontend work to the task-shape gate and
  claimed every review fix stays with the orchestrator — both exceptions now stated
- [medium][correctness] the regression guard text-matched a phrase that was not the live wording, so
  it missed the real regression and false-failed a correct placement — now block-scoped over
  `requires:` vs `parallel-when:`, with empty extraction treated as failure
- [low][DX] duplicate `Files changed (where / why)` section with a `<pending>` placeholder
