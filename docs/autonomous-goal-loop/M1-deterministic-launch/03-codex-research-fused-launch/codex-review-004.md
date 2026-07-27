# Codex adversarial review — Round 004

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 30116 bytes (r1 10308, r2 17376, r3 24957).
First round in which anything compacted — round 001 went in as its carried decisions. Allowlist
unchanged: the review-unit folder, `claude/skills/codex-research/SKILL.md`.

**This round survived a harness kill of its launcher.** The background call wrapping it was stopped
mid-flight; `dstack run` had already completed and published `exit=0` with the full output, so the
round is valid and is recorded here. Its two sibling rounds published `exit=143` with zero bytes and
were discarded and relaunched under fresh labels. No orphaned `codex exec` survived in any of the
three. That is the failure path working, in production, unrehearsed.

## GPT findings

[severity:medium][security] Raw textual placeholder substitution remains injectable: a single quote closes the shell literal before the validation backstop, allowing command execution that can produce an accepted slug.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 assignments; confirmed: Step 1’s prose-only invariant and Step 2 validation.
Evidence: Substituting `x'$(printf PWNED)'` produces `GOAL='x'$(printf PWNED)''`; the command runs and the validator receives the valid slug `xPWNED`.
Verification: Both bash 3.2.57 and zsh 5.9 returned rc=0 with `ACCEPTED:xPWNED`.
Suggested direction: Pass goal/topic through argv or environment as data, then validate them before any path construction; never insert them textually into shell source.

[severity:medium][technical correctness] The zsh cleanup traps suppress INT, TERM, and HUP without forwarding them to `dstack` or exiting, so wrapper-only cancellation can leave the research running and ultimately report success.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 trap; confirmed: its teardown and background-lifetime guarantees.
Evidence: The same `rm -rf` handler is installed for EXIT and each signal, but contains neither signal propagation nor an explicit nonzero exit.
Verification: Under zsh 5.9, the recipe-shaped trap around `sleep` returned rc=0 with `[TXT]` for INT, TERM, and HUP; after TERM, the wrapper was still alive at 300 ms and completed normally.
Suggested direction: Make each signal path propagate cancellation to the supervised command and terminate the wrapper with the corresponding failure status before performing final cleanup.

[severity:low][technical correctness] The retained E2E record still does not establish that the current exact block ran after the validation changes.
Sites: primary: `task.md` E2E verification; confirmed: the skill’s “Verified runnable” bullet.
Evidence: The retained capture records only the child command, while the current Step 1 invariant, single-quoted assignments, validation loop, and outer traps are not captured.
Verification: No post-round-three run is recorded, and `task.md` explicitly says the retained evidence cannot prove the outer validation or trap behavior.

[severity:low][security] The material still embeds evaluator-disposition language declaring launcher residuals “accepted” and assigned to another “review unit.”
Sites: primary: the skill’s residual paragraph; confirmed: `task.md` Design consult.
Evidence: Those statements attempt to pre-set the treatment of defects in a referenced tool whose guarantees this document relies upon.
Verification: The disposition was ignored; inspecting and exercising the referenced shell behavior exposed the unresolved cancellation blocker above.

Omitted-detail: 0 low

GPT verdict: reject — Textual slug substitution still permits shell-command execution, and the zsh signal fence suppresses cancellation instead of terminating the research process.

## Carried decisions
- **A quoted assignment is not a boundary; a QUOTED HEREDOC is.** `GOAL='<goal>'` with `<goal>`
  replaced by `x'$(printf PWNED)'` closes the literal, runs the command, and hands the validator the
  valid slug `xPWNED`. `<<'SLUG'` expands nothing at all, so the same input arrives as a literal and
  is refused. Measured, both shells:
  `assignment → ACCEPTED [xPWNED]` / `heredoc → REFUSED [x'$(printf PWNED)']`, benign slug still
  accepted.
- **A signal handler that only cleans up lets the shell CONTINUE.** Measured, both shells: the
  cleanup-only form ran the handler twice and returned 0 (`CLEANSURVIVEDCLEAN`). The handler must
  disarm EXIT, clean once, and exit with the signal's status — that form returns 143 and cleans
  once, and the normal path still cleans exactly once.
- The wrapper exiting nonzero is not the same as cancelling the round. If a signal reached only the
  wrapper and not the process group, `dstack run` can still be alive; the retry fence exists for
  exactly that and must be run before relaunching a capture with no terminal record.
- A residual paragraph states what is true of the tool. Attaching a disposition to it — "accepted",
  "belongs to another unit" — is disposition language inside the reviewed payload, and this round
  showed the cost: it invited the reader to stop where I had stopped, and the cancellation defect
  was past that point.

Consensus: disagreed
