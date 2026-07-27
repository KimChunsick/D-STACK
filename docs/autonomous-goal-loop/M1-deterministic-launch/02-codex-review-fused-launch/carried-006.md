## Carried decisions — Round 006
- **§3 is now stated POSITIVELY**: exactly two things do not reopen — a restatement about code that
  has NOT MOVED, and an objection with no demonstrated failure. The first repair of that clause
  deleted the middle of a sentence and left a dangling fragment, which is what happens when a rule
  is written as a list of exemptions.
- **Every substituted path and the label are quoted data.** Unquoted literals in `ALLOW=( … )` glob
  — measured, `*/task.md` expanded into three entries — and an unquoted `<label>` is parsed as shell
  syntax before `dstack` can validate it. `"${ALLOW[@]}"` and `"$LABEL"` throughout.
- **`SIGPROF` is CATCHABLE.** Measured: an explicit `trap … PROF` handler runs in bash and zsh both.
  It is simply not in `RUN_SIGNALS` and does not get bash's implicit EXIT-trap firing, which is why
  adding it there fixes it. "Untrappable" applies to `SIGKILL` and nothing else — this file said
  otherwise in two places.
- The `adversarial-review` contract disagreement is raised for the FOURTH time with the same
  disposition. It is a real outstanding inconsistency in the repository and is named as one in
  `findings.md`; closing it means editing that file, which is a separate review unit.

Consensus: disagreed
