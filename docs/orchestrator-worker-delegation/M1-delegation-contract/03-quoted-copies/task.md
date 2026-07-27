# 03-quoted-copies

## Intent / Why

Two places restate the delegation gate that T01 and T02 replaced, and both are now wrong or blind.
`claude/CLAUDE.md` section 0 still tells every session that fan-out happens "only on a
`check-parallel.sh` PARALLEL verdict" — a sentence that loads at the start of every task in every
repository. The schema check pins `requires:` and the checker's filename but nothing about what
the gate keys on, so the entire PARALLEL-to-task-shape change could be reverted and the check would
stay green.

## Deployment context

`claude/CLAUDE.md` is read at the start of every session in every repository the maintainer works
in, so a wrong sentence there misdirects work that never opens the skill. `skill-schema.test.sh` is
one of the two checks `AGENTS.md` requires be run; it is plain bash, no dependencies beyond ruby
for the YAML parse.

`AGENTS.md`, section "No TDD, no new tests in this repo", is the artifact that governs this: it
calls editing an assertion inside an existing check maintenance rather than authorship when the
thing that assertion pinned was deliberately changed, and it caps the check set. No file was added.

## Design consult

Skipped — no trigger. This updates two restatements of a contract decided in T01 and T02. No new
module boundary, no API, no persistence format, no sanitization path.

## What was done (what / why)

**`claude/CLAUDE.md` section 0 stopped describing a gate that no longer exists.** Its one-line
pipeline summary said fan-out happens "only on a `check-parallel.sh` PARALLEL verdict". That
sentence loads at the start of every session in every repository, so it would have misdirected work
that never opens the skill at all. It now states the task-shape gate, says outright that a PARALLEL
verdict decides only concurrency, and names what stays with the orchestrator.

It also carries the two exceptions, which round 001 found it had flattened. Review fixes are
orchestrator-owned with one carve-out: a finding whose fix sits inside a single task's declaration
goes back to that task's worker, per the P9 attribution rule T02 wrote. And frontend code goes to
`frontend-dev` under section 0.2 regardless. Round 002 then found that stating both as facts side by
side was still not enough — exploratory frontend work matched "exploratory work stays with the
orchestrator" and "frontend is delegated" at the same time, with nothing saying which wins. Section 0
now states the precedence outright: 0.2 outranks the whole retention list. That matches
`frontend-takes-precedence` in SKILL.md, which was already explicit; the summary was not.

**The schema check gained the invariants the change is actually about.** It pinned `requires:` and
the checker's filename and nothing about what the gate keys on, so the entire PARALLEL-to-task-shape
change could have been reverted with every assertion green. Four positive `has` assertions
(`delegate-when:`, `keep-in-the-orchestrator:`, `parallel-when:`, `honest-scope:`) plus a
schema-level check of the four `worker-fanout` lists and of WHERE the PARALLEL condition sits.

The placement check is the one that matters, and it took three tries to get right. A regression
would not delete a key; it moves one line from `parallel-when` back into `requires`, and both
placements contain the same words, so no single banned string can distinguish them. Scoping by
indentation with awk was the second try and round 002 broke it: awk counts a comment as content, so
a key whose body is nothing but comments read as non-empty and still matched `PARALLEL`, while YAML
loads that key as null. The check now reads the parsed document — the harness already extracts and
parses every fenced YAML block, so this reuses that. Comments do not survive the parse, and `nil` is
not a non-empty Array.

Rounds 003 and 004 then found the same shape twice: the check was enumerating a subset of the
lists that decide delegation. It rejected PARALLEL from `requires` only, so the same line under
`delegate-when` passed while the check printed "PARALLEL is not a delegation precondition"; adding
`delegate-when` left `keep-in-the-orchestrator` open, where a PARALLEL condition recouples the two
questions just as effectively. It now iterates all three and names which list carried the violation.
Round 004 also found three ways a malformed schema could pass: `- # commented out` parses as
`[nil]`, which is non-empty and contains nothing, so entries must be non-blank strings; a second
`worker-fanout` node silently overwrote the first, so exactly one is now required; and
`YAML.load_file` constructs tagged Ruby objects during the load on Psych < 4, before any check can
inspect the result. `honest-scope` and `frontend-takes-precedence` are read off the parsed node too,
not matched globally.

Round 005 closed three more holes and forced one honest trade. Duplicate keys are now rejected at
the AST: `safe_load` collapses `worker-fanout:` declared twice in ONE mapping down to the last
value, so counting parsed nodes could never see the first — a malformed node hid behind a valid
duplicate and the count still read 1. Both YAML passes now use `safe_load` over `File.read`,
including the pre-existing parse-validity loop; round 004 had converted only the new one and left
the other, which fixes nothing when both read the same content.

The trade is on the third. Type checks and a bare `PARALLEL` token are satisfied by
`delegate-when: [a task exists]` and by `parallel-when: [PARALLEL must never be used]`, which invert
the contract and pass. Distinguishing "PARALLEL is required" from "PARALLEL is forbidden" is a
question about MEANING, and the only thing a bash check can pin is wording. So each key must state
its decision, read off the parsed field.

Round 006 showed the first attempt at that was still too loose. Pinning tokens let a negation carry
the token: `verdict of PARALLEL` is satisfied by "a verdict of PARALLEL must never permit concurrent
execution", and `OUTRANKS` by "`frontend-dev` never OUTRANKS orchestrator retention". Both pass while
saying the opposite. The pins are now whole normalized decisions in `pins.txt` — `parallel-when` must
carry an entry equal to `a checker plan verdict of PARALLEL for the exact candidate set`, the others
must contain their full canonical sentence — and a phrase that long cannot be negated without
editing the phrase, which is what should fail. Round 006 also found `YAML.safe_load` returns only
the FIRST document of a stream, so a second document inside one fence carried a regressed node past
every check; each fence must now hold exactly one document.

The cost is real and should not be discovered later: rewording a pinned decision fails the check
even when the meaning is unchanged, and updating it is then a deliberate act. That is the same trade
every other assertion in this file already makes; it is written down here because round 005 is where
it stopped being free.

## Files changed (where / why)

- `claude/CLAUDE.md` — section 0's pipeline sentence.
- `claude/skills/full-cycle/tests/skill-schema.test.sh` — assertions added inside the existing
  check, and its YAML-block extraction hoisted so both the new schema check and the existing
  parse-validity loop use one temp dir under the trap that was already there. `AGENTS.md` calls
  editing assertions whose pinned mechanism deliberately changed maintenance rather than
  authorship; the file set does not grow.

## E2E verification

Run on 2026-07-27. The unit's claim is that two restatements of the delegation gate agree with the
contract they restate, so the E2E resolves every claim `CLAUDE.md` section 0 makes about delegation
against the PARSED `worker-fanout` node in `SKILL.md` — not against its text, which is the mistake
this whole task exists to correct.

```
  declaration is complete          delegate-when              RESOLVES
  write set is determined          delegate-when              RESOLVES
  isolating it is worth the setup  delegate-when              RESOLVES
  PARALLEL decides concurrency     parallel-when              RESOLVES
  docs and skills stay put         keep-in-the-orchestrator   RESOLVES
  exploratory stays put            keep-in-the-orchestrator   RESOLVES
  review fixes stay put, except    keep-in-the-orchestrator   RESOLVES
  0.2 outranks the list            frontend-takes-precedence  RESOLVES

  skill-schema.test.sh  == all checks passed
  secret-guard.sh       green
```

Both pinned checks this repository requires are green. The revert direction was exercised separately
under «Direct verification» — nine fixtures, each moving or gutting one part of the contract, all
caught.

Scope note: the unit is not merged to a branch of its own. Repo policy runs this Goal serially in
the main checkout, so `base..HEAD` containment does not apply here; the working tree carries this
unit's two files plus the other M1 units, and `git status` above is the record of that.

## Direct verification

This repository's `AGENTS.md`, section "No TDD, no new tests in this repo", replaces the pipeline's
Red-Green-Refactor step with direct verification. No file was added to the check set. What follows
is what running the thing produced.

Run on 2026-07-27, against a patched harness (the real script hard-codes its `SKILL` path). Nine
cases; this guard has been wrong in eight distinct ways and each earned a control.

```
  r_a      FAIL PARALLEL is a delegation precondition again, under requires
  r_d      FAIL ... under delegate-when
  r_k      FAIL ... under keep-in-the-orchestrator
  r_n      FAIL worker-fanout.requires is not a list of non-blank entries (parsed as [nil])
  r_neg    FAIL worker-fanout.parallel-when no longer states (eq): a checker plan verdict of ...
  r_neg2   FAIL worker-fanout.frontend-takes-precedence no longer states (sub): it OUTRANKS ...
  r_multi  FAIL yaml fences with more than one document: b1.yaml:2
  r_dupkey FAIL duplicate yaml mapping keys: b2.yaml/worker-fanout
  r_b      == all checks passed
  real     failures=0
```

`r_a`, `r_d` and `r_k` move the LIVE line out of `parallel-when` into each delegation-deciding list
in turn — rounds 001, 003 and 004 each found one of these passing. `r_n` replaces `requires:` with
`- # commented out`, which parses as `[nil]`. `r_neg` and `r_neg2` are round 006's negations: both
keep the pinned token and reverse the meaning. `r_multi` adds a second YAML document inside one
fence; `r_dupkey` declares `worker-fanout:` twice inside one mapping. `r_b` is the false-positive
control — the pinned entry with trailing whitespace, which normalization must accept — and it stays
green.

(`r_n` and `r_neg` also trip a neighbouring `has` assertion, because replacing a whole list body
removes other lines from it. The fixtures are blunt, not a second defect.)

`bash claude/skills/full-cycle/tests/skill-schema.test.sh` -> `== all checks passed`.

## Gate status

- [x] Verification: document invariants confirmed by direct run
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
