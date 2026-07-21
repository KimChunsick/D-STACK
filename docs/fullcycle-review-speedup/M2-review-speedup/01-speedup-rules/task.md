# 01-speedup-rules

## Intent / Why
Serial Codex review rounds dominate full-cycle wall-clock (15–25 min each, up to 21
rounds observed). Encode round-count reduction into the pipeline itself, keeping the
reviewer at full strength (GPT-5.6 Sol, xhigh — untouched):
1. **Pre-review defect-class self-sweep** (research: strongest intervention) — before
   every Codex invocation, the builder adversarially sweeps the task scope against an
   evidence-derived recurring defect-class checklist; fixes sweep the whole class, not
   the cited instance, killing the "one fix exposes the adjacent edge case" cascade.
2. **Slimmed finding format** (interview Q1) — drop mandatory `Illustrative example:` +
   `Reviewer caveat:`; `Suggested direction:` optional one-liner only when non-obvious;
   `Evidence:`/`Verification:` stay mandatory.
3. **Conditional design consult** (interview Q2) — trigger-listed tasks get one
   timeboxed Codex design review before implementation; others skip it.
4. **Closure + rebuttal discipline** — medium=0 closes as approve-with-fixes; reworded
   answered concerns are rebutted by citing the prior round, not re-fixed.
5. **Wait-time parallelization** — while a review round runs, prepare independent next
   steps; never mutate files under review mid-round.

## Design consult (Phase 7 pre-step)
Skipped — trigger list does not apply: this task edits instruction prose in existing
skill/agent docs and one hook string; no new architecture, API contract, persistence,
idempotency, partitioning, or sanitization surface.

## What was done (what / why)
_(filled as the work happens)_

## Files changed (where / why)
_(filled as the work happens)_

## E2E verification
_(evidence recorded on completion)_

## Gate status
- [ ] TDD: Red→Green→Refactor complete
- [ ] Codex (GPT-5.6 Sol) adversarial review consensus
- [ ] E2E capture verified
