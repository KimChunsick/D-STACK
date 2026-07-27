# Codex adversarial review — Round 004

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 75879 bytes (r1 41621, r2 50262, r3 65104).
First round in which anything compacted — round 001 went in as its carried decisions, and the
total still rose, which is the measurement §1 now records rather than treats as a verdict.
Allowlist unchanged: the review-unit folder, `claude/skills/codex-review/SKILL.md`.

**Relaunched after a harness kill.** The first attempt (`t02-r4a`) published `exit=143` with
zero bytes; the retry fence confirmed nothing alive in the capture, and the same bundle was
re-sent under `t02-r4b`.

## GPT findings

[severity:medium][technical correctness] The document still overrides the elected review contract: rebuttals leave the immutable round, and `Consensus: resolved` may certify an open concrete medium without user disposition.
Sites: `claude/skills/codex-review/SKILL.md:429-444`; confirmed: `SKILL.md:478-535,612-679`, `codex/skills/adversarial-review/SKILL.md:100-107`, `claude/hooks/fullcycle-gate.sh:411-420`.
Evidence: The authoritative contract requires one immutable invocation/rebuttal exchange and every concrete high/medium to be fixed, disproved, or user-disposed; the current document explicitly replaces both rules.
Verification: The gate regex returned success for `Consensus: resolved`, so the prescribed non-convergence path can pass while a known concrete medium remains open.
Suggested direction: Align the orchestrator and reviewer contracts; loop termination must not become positive consensus without disposition of every concrete high/medium.

[severity:medium][technical correctness] The zsh cleanup-only traps swallow TERM, HUP, and INT instead of terminating the wrapper, allowing cancellation to delete the scratch directory while `dstack` and Codex continue.
Sites: `claude/skills/codex-review/SKILL.md:286-301`; confirmed: `claude/bin/dstack:1097-1124`.
Evidence: Each signal handler only runs `rm -rf "$SCRATCH"`; it neither exits nor re-raises the signal.
Verification: With a foreground child, direct zsh probes for TERM, HUP, and INT all returned 0 and printed `CLEAN-SURVIVEDCLEAN`, proving the child completed after the signal.
Suggested direction: Disarm the EXIT trap, clean once, and exit with the signal-specific status from each terminating-signal handler.

[severity:medium][technical correctness] The per-path skip gate still matches substrings rather than complete metadata lines, so ordinary reviewed prose can falsely prevent a valid review from launching.
Sites: `claude/skills/codex-review/SKILL.md:209-227`; confirmed: `SKILL.md:269-282`, `assemble-review.sh:219-258`.
Evidence: The first grep accepts any occurrence of `--- <path> (` and the second accepts `SKIPPED:` anywhere later on that line, wider than the documented exact-marker residual.
Verification: A prose sentence containing the actual allowlisted path and discussing `SKIPPED:` produced `recipe=REFUSE`, while the assembler’s normal header produced `PASS`.
Suggested direction: Match a complete per-path marker line; retain the exact-line impersonation as the documented residual until the assembler provides a separate status channel.

[severity:medium][DX] “Nothing else in that call” contradicts the runnable fence’s mandatory scratch creation and path reconstruction, giving the orchestrator mutually exclusive instructions.
Sites: `claude/skills/codex-review/SKILL.md:286-301`; confirmed: `SKILL.md:365-369`, `claude/skills/full-cycle/SKILL.md:399-408`.
Evidence: Variables do not survive the foreground assembly call, while the background call requires locally defined `SCRATCH` and `RD`; the full-cycle contract explicitly permits setup before the blocking terminal step.
Verification: Removing that setup left zsh with `RD: parameter not set`; without nounset, the recipe instead supplies empty or incorrect `-C` and stdin paths.
Suggested direction: Permit setup before `dstack run` and prohibit only post-run work whose result is needed.

Omitted-detail: 0 low

GPT verdict: reject — four concrete instruction failures can manufacture consensus, ignore cancellation, refuse valid bundles, or make the launch recipe self-contradictory.

## Carried decisions
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
