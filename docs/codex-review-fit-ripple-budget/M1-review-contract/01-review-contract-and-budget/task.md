# 01-review-contract-and-budget

## Intent / Why

Three defects were costing rounds without buying safety: findings that demand machinery the
project will never run, findings that stop at the primary site while the same invariant is
broken at siblings, and a review bundle that grows without bound until `codex exec` dies.
This task fixes the reviewer's contract and the material it is fed, and keeps every surface
that states the contract in agreement.

Hard invariant for the whole task: **no change here may reduce the reviewer's ability to
report a real defect.** Every affordance added for brevity or right-sizing ships with an
explicit counterweight — context-is-not-a-waiver, counterfactual-required, a budget that
trims elaboration but never the existence of a finding, and compaction that emits the round
whole whenever its carried-state companion is missing.

## Deployment context

Runs locally on the maintainer's own machines only. Single user, no network service, no
multi-node or multi-tenant deployment, no CI runtime. The artifacts are markdown instruction
files and one bash assembler; the "runtime" is a human invoking `codex exec` from a terminal.
Data criticality is low for the review documents themselves and **high for secret
non-exposure**, because this repository is public. Out of scope by construction:
availability, replication, horizontal scale, concurrent multi-user access.

## Design consult

Skipped — no trigger. Instruction prose plus one shell script; no module boundary, API
contract, persistence or logging path, cursor/idempotency semantics, partitioning, rendering
boundary, or multi-path sanitization.

## Method note (deviation from the standard pipeline, at the user's instruction)

The user directed mid-task that this repository does not run Red-Green-Refactor TDD and that
the work should be the change itself. A four-surface drift probe had been written and had
gone RED; it was deleted on that instruction. The `10. **Goal-level round trip through the finished mechanism.** Assembling this task's own
   review bundle with all eight sealed rounds emits rounds 001-006 as their companions and
   007-008 in full: the round history drops from 52,506 bytes all-full to 28,288 as sent, a
   46% reduction that grows with every further round. Prompt, hook, assembler and secret guard
   all re-verified green in the same pass.

## Gate status
- [x] Change applied and directly verified (9 checks above)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus — 8 rounds, `Consensus: agreed` in `codex-review-008.md`
