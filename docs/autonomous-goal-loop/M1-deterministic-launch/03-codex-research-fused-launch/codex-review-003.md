# Codex adversarial review — Round 003

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 24957 bytes (r1 10308, r2 17376). The ratchet
binds from round 004, which is also the first round in which anything compacts. Allowlist
unchanged: the review-unit folder, `claude/skills/codex-research/SKILL.md`.

## GPT findings

[severity:medium][security] The prior placeholder-validation blocker remains: the check runs after Step 1 uses the values as a path and after zsh parses and expands them as shell source.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 1; confirmed: Step 2 assignments and validation comment.
Evidence: Step 1 writes `docs/<goal>/research/<topic>.brief.txt`; textual substitution into `GOAL="<goal>"` permits `$()` execution before the `case` validation.
Verification: `TOPIC=../../codex-latest-model-xhigh/research/latest-codex-model` targets an existing tracked brief before Step 2 rejects it; both zsh and Bash executed a substituted command before reporting the slug invalid.
Suggested direction: Establish the slug invariant before Step 1 and pass validated values into Step 2 through a non-code channel rather than textual shell-source substitution.

[severity:low][technical correctness] The untrapped-signal recovery guarantee exceeds the launcher: an orphan can exist before its PID is recorded, while the outer EXIT trap removes its scratch directory.
Sites: primary: the skill’s residual paragraph; confirmed: the recipe’s scratch trap and `dstack`’s fork-to-PID-record sequence.
Evidence: `dstack` forks the child before writing `.launch/child`; the surrounding shell unconditionally removes `$SCRATCH` when `dstack` terminates.
Verification: `SIGKILL` or another untrapped signal between the fork and PID write leaves a live group without the promised PID record, after which the parent shell executes scratch cleanup.
Suggested direction: Describe PID recording as best-effort in this window and make scratch cleanup conditional on terminal completion or explicit orphan cleanup.

[severity:low][technical correctness] The current exact-block verification claim exceeds the retained evidence, and its path-clobber counterexample uses the wrong traversal depth.
Sites: primary: `task.md` E2E verification; confirmed: the skill’s “Verified runnable” bullet and Step 2 comment.
Evidence: Round 002 assessed the retained run before requesting the new validation loop; no post-fix exact-block run is recorded, while the comment changed the demonstrated `../../../AGENTS` case to `../../AGENTS`.
Verification: From `docs/<goal>/research`, `../../AGENTS.md` resolves to `docs/AGENTS.md`; reaching the tracked root `AGENTS.md` requires `../../../AGENTS.md`.

[severity:low][security] The current material still embeds evaluator-scope directives that could bias a later LLM review.
Sites: primary: the skill’s “an allowlist does not grow to absorb a finding”; confirmed: `task.md` says residuals “belong” elsewhere.
Evidence: Both statements attempt to pre-dispose where findings may be reviewed rather than merely describing implementation state.
Verification: The directives were ignored, and the referenced launcher was inspected because the trusted prompt requires its guarantees to be checked.

Omitted-detail: 0 low

GPT verdict: reject — validation still occurs after concrete tracked-file clobbering and shell-command execution paths have already become reachable.

## Carried decisions
- The slug invariant is established in **Step 1**, not Step 2. Step 2's `case` is a backstop, and a
  backstop is all it can be: Step 1 already builds a path from both values, and a placeholder
  substituted into double quotes runs its `$(…)` at assignment time, before any later check exists.
- Placeholders that get substituted TEXTUALLY are single-quoted. Measured: `GOAL="$(printf PWNED)"`
  yields `PWNED` in both bash and zsh; `GOAL='$(printf PWNED)'` yields the literal, which the `case`
  then refuses.
- Traversal depth, corrected: from `docs/<goal>/research`, `../../AGENTS.md` is `docs/AGENTS.md` and
  the tracked root file needs `../../../AGENTS.md`. The reviewer was right and the comment was
  wrong.
- The recovery path has two real caveats and they are stated rather than smoothed over: the launched
  pid is recorded just after the fork, so a kill inside that window leaves a live group with no
  record (`rm-run` treats a missing record as live, which is the mitigation), and the fence's trap
  removes `$SCRATCH` on exit, so a surviving orphan can lose its cwd.
- Signal coverage is the measured table, not the trap list: bash 3.2.57 fires the EXIT trap on
  fatal signals, so the gaps are `SIGKILL` and `SIGPROF` only. zsh 5.9 never fires it, which is why
  the fence names its signals.

Consensus: disagreed
