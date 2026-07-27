## Carried decisions — Round 003
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
