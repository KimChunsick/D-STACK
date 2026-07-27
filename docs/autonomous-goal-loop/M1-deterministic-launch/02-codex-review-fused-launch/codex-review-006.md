# Codex adversarial review — Round 006

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 87085 bytes. REOPENED after sealing by the post-seal rule (T04 round 004 found the wrapper's EXIT trap deletes a live child's scratch dir).

## GPT findings

[severity:medium][technical correctness] The regression fix leaves the operative non-reopening rule syntactically broken and still names “a variant of an already-recorded class” as exempt, contradicting the next paragraph.
Sites: `claude/skills/codex-review/SKILL.md:680-695`
Evidence: The edit removed “code a fix just introduced” but retained the dangling phrase “a variant of an already-recorded class in / or an objection,” leaving ambiguous instructions immediately before “ALWAYS REOPENS.”
Verification: The scoped diff confirms the repair deleted only the continuation of the old exemption, not the exemption’s prefix.
Suggested direction: Replace the entire operative sentence so only restatements about unchanged code and non-concrete objections do not reopen.

[severity:medium][security] Substitute-able paths and the launch label are embedded as shell source instead of quoted data, allowing glob expansion, argument splitting, or command execution before validation.
Sites: `claude/skills/codex-review/SKILL.md:234-265`; confirmed: `SKILL.md:302-322`
Evidence: Array entries such as `path/to/changed1` are unquoted, while `dstack run <label>` repeats an unquoted textual placeholder; `dstack` cannot validate a label that the invoking zsh has already parsed as syntax.
Verification: Bash and zsh expanded the existing `*/task.md` probe into three allowlist entries; substituting `safe; printf INJECTED` executed `INJECTED` before the fake `dstack`, whereas `"$LABEL"` preserved one argument.
Suggested direction: Define every substituted path and `LABEL` as quoted data in each fence, then use `"${ALLOW[@]}"` and `"$LABEL"` exclusively.

[severity:medium][technical correctness] The payload’s “THIS file governs” directive still overrides the elected review contract by removing rebuttals from the immutable record and permitting open concrete mediums to receive positive consensus.
Sites: `claude/skills/codex-review/SKILL.md:464-520`; confirmed: `SKILL.md:547-570,647-671,708-720`, `codex/skills/adversarial-review/SKILL.md:96-108`, `claude/hooks/fullcycle-gate.sh:411-420`
Evidence: The elected contract requires one immutable invocation/rebuttal exchange and user disposition for unresolved concrete mediums; the current document explicitly replaces both requirements.
Verification: The gate’s production regex accepted `Consensus: resolved` with status 0, while `assemble-review.sh` emits no `response-<NNN>.md`, so neither later reviewers nor the gate can validate the rebuttal record.
Suggested direction: Align the orchestrator with the elected contract; termination may record unresolved work but cannot manufacture positive consensus.

[severity:low][DX] The conditional cleanup repair leaks scratch directories on pre-launch failures and after a deferred wrapper signal when `dstack` has already published a terminal record.
Sites: `claude/skills/codex-review/SKILL.md:303-316`; confirmed: `SKILL.md:325-336`
Evidence: No `exit` exists after a pre-fork failure, while each signal handler disarms EXIT without invoking the now-safe terminal-record predicate.
Verification: With `/dev/null` representing an existing terminal record, bash and zsh printed foreground completion and TERM but no `CLEAN`; normal exit printed `CLEAN`.
Suggested direction: Centralize guarded cleanup and invoke it from EXIT and deferred signal handlers, retaining scratch only while quiescence is genuinely unknown.

[severity:low][technical correctness] `SIGPROF` is repeatedly described as untrappable, although it is catchable and merely bypasses Bash’s implicit EXIT-trap behavior.
Sites: `claude/skills/codex-review/SKILL.md:172-176,305-310`; confirmed: `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/task.md:119-123`, `claude/bin/dstack:928-930`
Evidence: The document simultaneously calls `SIGPROF` untrappable and proposes adding it to `RUN_SIGNALS`.
Verification: Explicit PROF handlers printed `bash-prof-caught` and `zsh-prof-caught`, both exiting 155; only the EXIT-only probe remained silent.
Suggested direction: Describe PROF as currently unhandled, reserving “untrappable” for SIGKILL.

Omitted-detail: 0 low

GPT verdict: reject — the botched termination edit, contract override, and source-parsed substitutions remain concrete medium blockers.

## Carried decisions
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
