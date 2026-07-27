# Codex adversarial review — Round 007

## Review scope
Re-review (Rounds 1-6 rejected; all findings accepted and fixed). Budget extended by the user at Phase 9 — see GOAL.md Q5.

## GPT findings
[severity:high][technical correctness] The assembler can label changed tracked code as “no change,” allowing the mandatory review gate to approve without the implementation.
Sites: Primary: `claude/skills/codex-review/assemble-review.sh:158`; confirmed: `claude/skills/full-cycle/SKILL.md:171`.
Evidence: It suppresses every `git diff HEAD` failure and cannot assemble the lifecycle’s recorded `base..HEAD` committed worker diff.
Verification: A committed range produced 7,218 bytes while the assembler’s `HEAD` comparison produced zero; a forced diff failure exited 128 while `ls-files` succeeded, selecting “tracked, no change.”
Blocking: Yes — genuinely blocking; unreviewed code can satisfy P9.
Suggested direction: Make the assembler accept and validate the recorded base/head identity, and fail closed on any diff error.

[severity:high][security] The generated runner reparses the repository root as shell code, enabling command execution from a valid specially named checkout path.
Evidence: `claude/skills/codex-review/SKILL.md:190-207` expands `RD` into an unquoted heredoc that emits `RD="$RD"` into `run.sh`.
Verification: Substituting a root containing `$(printf PATH_INJECTION >&2)` executed that substitution when the generated runner was interpreted; `$HOME` was likewise expanded into a different path.
Blocking: Yes — genuinely blocking; path content reaches automatic shell execution.
Suggested direction: Pass the run directory as a positional argument or environment value instead of embedding it into generated shell source.

[severity:medium][technical correctness] The Round 6 review-unit conversion remains partial, leaving milestone review serialization and worker merging internally contradictory.
Sites: Primary: `claude/skills/full-cycle/SKILL.md:387`; confirmed: its worktree lifecycle at line 180, `claude/skills/codex-review/SKILL.md:338`, and `skill-schema.test.sh`.
Evidence: P9 is per review unit, yet the procedures still allow different-task reviews concurrently and require each task’s nonexistent review consensus before merge.
Verification: M2 has three tasks in one unit; following the prose permits concurrent allocation of the same check-then-write round filename, while milestone fan-out cannot reach unit review because merging waits on per-task consensus.
Blocking: Yes — genuinely blocking; it permits round clobbering and makes milestone fan-out unsatisfiable.
Suggested direction: Parameterize serialization, review identity, merging, reopening, and gate placement consistently over the selected review unit.

[severity:medium][technical correctness] A contract-valid closure round makes the self-contained triage recipe exit nonzero.
Evidence: `claude/skills/codex-review/SKILL.md:259-261` ends with uncaught `grep` commands, whose no-match status is 1.
Verification: An output containing `Omitted-detail: 0 low` and a valid approving verdict made both the finding-count and blocker queries exit 1; an all-low closure likewise fails on the final blocker query.
Blocking: Yes — genuinely blocking; the normal successful convergence path reports command failure.
Suggested direction: Preserve zero as the displayed count and explicitly normalize an expected no-match blocker query to success.

[severity:low][security] Final capture cleanup is not fail-closed.
Sites: Primary: `claude/skills/codex-review/SKILL.md:110`; confirmed: `claude/bin/dstack:726`.
Evidence: Missing or mistyped labels return success, and a genuine `rm-run` failure is followed by `prune`, whose success masks the fence status.
Verification: The API reports “nothing removed” with status zero; fresh captures survive because `prune` only removes captures older than seven full days.
Blocking: No — non-blocking follow-up, but plaintext bundles can outlive the claimed closure cleanup.

[severity:low][DX] The Round 6 size-accounting correction is still false.
Sites: Primary: `docs/dstack-state-store/GOAL.md:134`; confirmed: M2 `task.md` and `06-inject-slim/task.md`.
Evidence: The records claim +495 bytes, +7 lines, and even “net length stays flat.”
Verification: `claude/CLAUDE.md` actually changed from 8,670 to 9,304 bytes, 8,019 to 8,647 characters, and 163 to 171 lines.
Blocking: No — documentation-only accounting drift.

[severity:low][software structure] The M2 design-consult rationale still denies the API and sanitization work added by its own review loop.
Evidence: M2 `task.md:30-34` says there is no new API contract or sanitization path, while lines 98-103 record the new destructive `rm-run` API.
Verification: `claude/bin/dstack:715-755` implements label validation, session-path identity checks, and recursive deletion.
Blocking: No — review-history and design-record drift only.

[severity:low][technical correctness] The schema check’s temp cleanup is no longer signal-safe despite claiming a consolidated trap.
Evidence: `skill-schema.test.sh:119-121` installs no trap; it only performs final-line cleanup.
Verification: Interrupting after either `mktemp -d` and before line 121 leaves the directory behind.
Blocking: No — bounded temporary-directory leakage.

[severity:low][technical correctness] Runner-file creation remains unchecked before the repaired launcher-status check.
Evidence: `codex-review/SKILL.md:190` does not test the heredoc write’s status, and execution continues to `Popen`.
Verification: A failed or partial `run.sh` write can still yield successful `Popen` creation and the unconditional “launched” message before the monitor later reports VANISHED.
Blocking: No — it wastes an attempt but cannot seal a completed round.

Omitted-detail: 0 low

GPT verdict: reject — The review gate can omit committed changes, execute repository-path content, corrupt same-unit round ownership, and falsely fail its successful closure path.

## Carried decisions — Round 007
Rounds 1-6 decisions stand. Added in Round 7:

- **A path is data, never source.** Quoted heredocs, arguments instead of interpolation. Anything
  that writes a script must assume the values it embeds are hostile.
- **The gate's own tool must fail closed.** `|| true` on the diff that IS the review material is
  the review approving what it never saw.
- **Parameterize what decides ORDER, not just what decides naming.** Serialization and merge
  gating are where a half-converted scope produces clobbering and deadlock.
- **The success path must exit zero.** A recipe that fails on convergence teaches the loop that
  converging is an error.
- **Idempotent success is not proof of effect.** Verify the state, not the return code.

Consensus: disagreed
