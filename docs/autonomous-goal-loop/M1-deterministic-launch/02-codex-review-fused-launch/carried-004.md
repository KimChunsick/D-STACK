## Carried decisions — Round 004
- **A signal handler that only cleans up lets the shell CONTINUE**, in bash and zsh both. Measured:
  the cleanup-only form returned 0 and printed `CLEAN-SURVIVEDCLEAN` even with a foreground child,
  so the wrapper could report success while `rm -rf "$SCRATCH"` deleted a live codex's cwd. Each
  terminating-signal handler now disarms EXIT, cleans once, and exits with the signal's status.
- **The per-path skip gate matches a COMPLETE marker line**, not two substrings. `grep -F -- "--- $f ("`
  piped into `grep -q 'SKIPPED:'` still refused a bundle whose prose contained an allowlisted path
  and the word `SKIPPED:` in the same sentence — reproduced, `recipe=REFUSE` on ordinary content.
  The form is now `awk` with `index($0,p)==1`, a literal `) ---` suffix test, and `SKIPPED:` on that
  same line: every comparison literal, so no path needs regex escaping. Verified: prose PASSes, a
  real `--- <allowlisted path> (SKIPPED: symlink) ---` REFUSEs, a real bundle PASSes.
- **The launch invariant is "`dstack run` is the LAST thing in that call".** "Nothing else in that
  call" is unsatisfiable — `SCRATCH` and `RD` must be defined in the same fence because variables do
  not survive the foreground assembly call, and removing them leaves zsh with `RD: parameter not
  set`. What is forbidden is work AFTER the launch whose result you need. `full-cycle`'s
  `waits.external` now states the identical invariant, and this file points at it.
- The consensus/contract disagreement with `codex/skills/adversarial-review/SKILL.md` is RAISED
  AGAIN and its status is unchanged: this file governs the pipeline's closure semantics, the
  Codex-side contract needs the same two edits, and that file is outside this unit's declaration.
  This is the finding stream repeating a recorded item, not new information.

Consensus: disagreed
