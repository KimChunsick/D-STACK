# Codex adversarial review — Round 010

## Review scope
Re-review of Round 009's repairs. Final round for this unit: closed by the non-convergence rule
recorded in `claude/skills/codex-review/SKILL.md` Step 4, not by reviewer approval.

## GPT findings
[severity:high][security] The Round-9 containment repair covers only allowlisted change files and remains check-then-read; automatic task and history snapshots have no parent containment.
Sites: primary: `assemble-review.sh:111-128`; confirmed: `emit_file` at lines 197-217 and automatic snapshot/history reads at lines 133-180 and 326.
Evidence: `validate_snapshot` rejects only a leaf symlink before `-f`, `wc`, `grep`, and `cat` follow parent components; the physical-root check exists only in `emit_file`, which subsequently reopens the original pathname.
Verification: Make `docs/unit` a symlink to an external directory containing a regular `task.md`; validation passes and `emit_snapshot` sends the external file. A leaf swap after `emit_file` validation similarly redirects an untracked-file read.
Blocking: Yes — genuinely blocking; repository-external data can still be sent to Codex.
Suggested direction: Centralize all reads behind one physical-root-contained, no-follow boundary that binds validation and reading to the same filesystem object.

[severity:high][technical correctness] The committed-worker review remains optional in the executable recipe: its only live assembler call selects `REVIEW_MODE=serial`, while the committed invocation is still a commented ellipsis.
Sites: primary: `codex-review/SKILL.md:198-213`; confirmed: `assemble-review.sh:58-93` and `full-cycle/SKILL.md:174-197`.
Evidence: Serial mode leaves `DIFF_ARGS` empty and executes `git diff HEAD`; nothing in the assembler distinguishes an integration checkout that requires committed mode.
Verification: Follow the runnable fence literally in a clean integration checkout: committed tracked changes produce empty diffs and “no change vs HEAD” records without a `SKIPPED` marker.
Blocking: Yes — genuinely blocking; P9 can approve a worker integration without reviewing its implementation.
Suggested direction: Provide a separate fully executable worker-fanout invocation populated from the recorded integration base and head.

[severity:medium][technical correctness] The unit-level downstream repair is prose-only: `unit-scope` has no checker mode, and the main-checkout document snapshot has no safe assembler input contract.
Sites: primary: `full-cycle/SKILL.md:124,184-202`; confirmed: `check-parallel.sh:52,311`, `assemble-review.sh:50-93,326`, and the committed recipe.
Evidence: A direct `check-parallel.sh unit-scope …` invocation returns `INVALID: unknown mode 'unit-scope'`; committed assembly requires a clean integration checkout while its relative `TASK_DIR` resolves there, not in the main checkout holding the current document.
Verification: M2’s integration head cannot pass the documented landing gate; copying the document makes the tree dirty, while reading it by an external main-checkout path relies on the high-severity containment defect above.
Blocking: Yes — genuinely blocking; milestone-granularity worker fanout still has no satisfiable review-and-land sequence.
Suggested direction: Add an executable union-scope checker interface and a separately validated main-document snapshot input, then document one runnable worker flow.

[severity:low][security] Closure cleanup still cannot derive all captures belonging to a review unit.
Sites: primary: `codex-review/SKILL.md` Step 4; confirmed: `dstack` status and `rm-run`.
Evidence: `status` records only `session/label`, with no unit ownership; the recipe manually selects labels and repeats that same list for verification.
Verification: Omit one attempt label and its plaintext bundle survives both verification and the age-zero `prune`.
Blocking: No — retention/privacy follow-up only.
Suggested direction: Persist capture-to-unit ownership and delete from that authoritative inventory.

[severity:low][technical correctness] Runner publication still accepts a nonempty partial `run.sh`.
Evidence: The heredoc writes directly to `run.sh`, and launch checks only `[ -s "$RD/run.sh" ]`.
Verification: Interruption or disk exhaustion after the first bytes leaves a nonempty truncated script; `Popen` starts Bash and reports a launch before the watcher later reports `VANISHED`.
Blocking: No — it wastes an attempt but cannot seal a completed round.

[severity:low][software structure] The durable records still contradict their declarations and design history.
Sites: primary: M2 `task.md`; confirmed: `04-review-io/task.md` and `GOAL.md` T04.
Evidence: M2 says the assembler appears in no milestone declaration and denies API/sanitization work; GOAL T04 declares the assembler, while `rm-run` is a validated destructive API. The subordinate record still says neither executable file is declared.
Verification: Direct comparison of the three records produces mutually exclusive ownership claims.
Blocking: No — documentation drift only.

[severity:low][technical correctness] The maintained schema check still reports per-fence invariants as passing after setup failure and omits the destructive `rm-run` verb from its bare-call scan.
Evidence: `mktemp` and `awk` are unguarded under `set -u`, and the regex lists `run-dir|prune` but not `rm-run`.
Verification: In the enforced read-only sandbox, both commands failed and the check still printed `ok` for both fence invariants; a runnable bare `dstack rm-run` also evades the scan.
Blocking: No — this weakens a policy check but is not itself a runtime failure.
Suggested direction: Guard extraction, require at least one extracted fence, and define the complete CLI verb set once.

Check execution: Shell syntax checks passed. The pinned checks were attempted but their temporary-file/probe operations were denied by the read-only sandbox; those environmental failures are not repository findings.

Omitted-detail: 0 low

GPT verdict: reject — External snapshot reads remain possible, committed worker changes can still be omitted from review, and the documented unit-level landing path has no executable implementation.

## Bundle size (the ratchet, recorded)

R5 110,913 · R6 115,229 · R7 136,496 · R8 155,283 · R9 170,782 · **R10 184,373** bytes.

It grew every single round, which is the measurement that produced the ratchet rule in
`codex-review/SKILL.md` («The bundle ratchets DOWN»). Stated plainly: this unit did NOT satisfy
that rule — the rule was authored at this round, out of this data, and binds from the next review
unit onward. Recording a number that fails the rule is the point; a rule whose own origin round
is quietly reported as passing is worth nothing.

## Round outcome

Six of seven findings fixed; one carried. The maintainer's reasoning is in `response-010.md`,
which is deliberately outside the reviewed corpus.

Blocking findings per round across this unit: **4 (R7), 2 (R8), 3 (R9), 3 (R10)**. Not strictly
decreasing over three consecutive rounds, so by the rule this unit's own T04 wrote into
`claude/skills/codex-review/SKILL.md`, the loop is non-convergent **by measurement** and closes
here. The `GPT verdict` line is advisory under that rule and does not by itself keep the loop
open. Every finding not fixed is recorded as an evidence-backed follow-up in this unit's
`task.md`; the running ledger is `findings.md`.

Nothing was downgraded to make this close. Both highs were fixed before sealing, and the medium
was resolved by deleting a false capability claim rather than by re-rating it.

Consensus: resolved
