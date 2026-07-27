# Codex adversarial review — Round 003

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 65104 bytes (r1 41621, r2 50262). The ratchet
binds from round 004, which is also the first round in which anything compacts — see §1. Allowlist
unchanged: the review-unit folder, `claude/skills/codex-review/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] Evidence-backed rebuttals are placed in permanently unbundled files, so later reviewers cannot verify “disproved” findings or engage with the maintainer’s response as required.
Sites: `SKILL.md:410-412`; confirmed: `SKILL.md:459-460,493-502,581-586`, `assemble-review.sh:337-359`, `adversarial-review/SKILL.md:97-108`.
Evidence: The assembler sends task, rounds, and allowlisted material but never `response-<NNN>.md`; carried decisions are defined around unresolved blockers, risks, and user decisions—not the evidence disproving closed findings.
Verification: A runtime claim disproved only by measurements in `response-001.md` reaches Round 2 as the original finding without its rebuttal, while no code diff can convey that measurement.
Suggested direction: Provide each next round a bounded evidence-bearing rebuttal record, or revise the authoritative review contract in the same unit.

[severity:medium][technical correctness] The fourth disposition manufactures positive consensus with a concrete medium still unresolved and without user disposition, contrary to the authoritative review contract.
Sites: `SKILL.md:493-516`; confirmed: `SKILL.md:605-619,633-639`, `adversarial-review/SKILL.md:82-84,104-108`.
Evidence: The document orders `Consensus: resolved` after non-convergence and the gate accepts it, while the contract permits consensus only after every concrete medium is fixed, disproved, or user-disposed.
Verification: Three non-decreasing rounds containing the same open concrete medium trigger mandatory closure without asking the user; the gate then passes a state the reviewer contract still requires to reject.
Suggested direction: Separate loop termination from consensus, or require user disposition for every remaining concrete high or medium before writing a positive consensus.

[severity:medium][technical correctness] The teardown guarantee covers every “CATCHABLE termination,” but `dstack` traps only a subset of terminating signals, so an omitted signal can orphan the separately grouped Codex process.
Sites: `SKILL.md:156-160`; confirmed: `dstack:916-930,1068-1069,1108-1117`.
Evidence: `RUN_SIGNALS` omits catchable terminating signals including `SIGXCPU`, `SIGXFSZ`, `SIGVTALRM`, and `SIGPROF`; the child runs in its own process group.
Verification: `/bin/bash -c 'trap "printf EXIT-TRAP" EXIT; kill -XCPU $$'` exited 152 without running the EXIT trap, so that signal cannot invoke `run_cleanup` and the child group can survive.
Suggested direction: Narrow the guarantee to the explicitly trapped signals or expand and verify the supervisor’s signal coverage.

[severity:medium][technical correctness] Rounds 002–003 are simultaneously exempted from the size ratchet and unconditionally required to rebuild no larger than the preceding bundle.
Sites: `SKILL.md:563-579`; confirmed: `SKILL.md:626-629`.
Evidence: The first section calls early growth unavoidable arithmetic, while the final procedure applies the no-growth command after every rejecting round without a round-number qualification.
Verification: After Round 001 rejected, this task’s Round 002 grew from 41,621 to 50,262 bytes; one rule accepts that result while the other forbids invoking Round 002 with it.
Suggested direction: Apply the final no-growth instruction only from Round 004 onward, matching the declared ratchet.

[severity:medium][security] The task payload still embeds evaluator-scope direction by declaring the assembler outside the task declaration and prescribing that this be treated only as filing information.
Sites: `task.md:16-21`; confirmed: `SKILL.md:207-212,318-323,563-570`.
Evidence: The artifact names a dependency and tells the evaluator how to classify its inclusion, while the reviewed skill itself says payload scope claims can suppress findings.
Verification: Following that classification excludes the assembler whose behavior determines both skip detection and rebuttal bundling, hiding failures that are directly material to this instruction change.
Suggested direction: Remove the evaluator-facing classification and retain only the ordinary changed-files record.

[severity:low][technical correctness] The scratch-directory comment promises cleanup for every round, but an EXIT-only trap does not run when the invoking Bash or zsh process is terminated by a signal.
Sites: `SKILL.md:274-276`; confirmed: `task.md:71-72`.
Evidence: The recipe installs no TERM/HUP/INT handler around `rm -rf "$SCRATCH"`.
Verification: Both `/bin/bash` and `/bin/zsh` exited 143 on self-TERM without executing an EXIT trap, so a harness-killed round can leak its `mktemp` directory.

Omitted-detail: 0 low

GPT verdict: reject — unresolved contract violations can hide rebuttal evidence, certify open concrete defects, orphan paid review processes, and impose contradictory round procedures.

## Carried decisions
- A DISPROOF goes into `## Carried decisions` with its measurement, not only into
  `response-<NNN>.md`. A FIXED finding needs no prose because the diff shows it; a disproved one
  needs the number, and a number in a file the reviewer is never sent gets the finding re-raised
  and re-argued. The carried block is bounded and compacts; the response is neither.
- Where `claude/skills/codex-review/SKILL.md` and `codex/skills/adversarial-review/SKILL.md`
  disagree about round-file shape and consensus dispositions, THIS pipeline's file governs; the
  Codex-side contract needs the same two edits and is a follow-up for its own review unit. A
  reviewer filing the disagreement is reporting a real disagreement, not making an error.
- **Signal coverage, measured, because two rounds argued it from opposite wrong premises.**
  `dstack` runs under `/bin/bash` 3.2.57, whose EXIT trap DOES fire on a fatal signal — so
  `run_cleanup` runs for TERM, ABRT, XCPU, XFSZ and VTALRM even though the last four are not in
  `RUN_SIGNALS`. The gaps are exactly `SIGKILL` (untrappable) and `SIGPROF` (reproducibly skips the
  EXIT trap). The round-003 claim that XCPU/XFSZ/VTALRM bypass cleanup does NOT reproduce; the
  round-002-era claim that "any catchable termination" is covered was also wrong. Both replaced by
  the table.
- **zsh and bash differ here and the recipes run under zsh.** zsh 5.9 never fires an EXIT trap on a
  fatal signal (self-TERM: rc=143, trap silent), bash 3.2.57 always does. So a fence that must clean
  up names its signals — `trap … EXIT INT TERM HUP` — instead of relying on EXIT.
- The no-growth instruction in the closing procedure carries the same round-004 qualification as
  §1. One rule exempting early rounds while another forbids them is worse than either alone.
- §4's blocking count is the number of concrete findings still OPEN at the end of a round, not the
  number the round raised, and the test applies from round 004. Counting what was raised closes
  loops that are going well: a document under repair can raise more each round while the open set
  falls to zero.
- A Deployment context is a changed-files record. Naming a dependency and then telling the
  evaluator how to classify it is the evaluator directive, in its third costume in this unit.

Consensus: disagreed
