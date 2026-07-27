# Codex adversarial review — Round 001

## Review scope
Batch pass 1 (consolidated round; this unit's first) | `REVIEW_MODE=serial` | bundle 6762 bytes. Allowlist: the review-unit folder, `claude/CLAUDE.md`.

## GPT findings

[severity:medium][technical correctness] The exhaustive unattended-stop summary is not equivalent to its authority: it omits non-migratable registration failures and forbids autonomous `reclaim` even for provably orphaned handoffs. This can cause ungated continuation or stall a planned `/clear` handoff.
Sites: `claude/CLAUDE.md:55`; confirmed: `docs/autonomous-goal-loop/M2-unattended-contract/05-standing-rules-alignment/task.md:34`, `task.md:51`.
Evidence: `full-cycle/SKILL.md:484-495` requires stopping on unusable session IDs, unwritable registries, and failed ownership verification while permitting provably orphaned reclaim; lines 699-722 prescribe reclaim after deliberate ID rotation.
Verification: Direct runs with empty and malformed `CLAUDE_CODE_SESSION_ID` both made `dstack reg` exit 1, reproducing the omitted failure path.
Suggested direction: Include every fail-closed registration failure and prohibit autonomous reclaim only when orphanhood has not been proven.

[severity:low][technical correctness] “The call does not return until the command finishes” assigns blocking to the wrong lifecycle: `run_in_background` returns immediately, while the background task containing `dstack run` remains active.
Sites: `claude/CLAUDE.md:38`; confirmed: `claude/skills/full-cycle/SKILL.md:400`.
Evidence: Claude Code documents that background Bash immediately returns a task ID, contradicting `claude/CLAUDE.md:41-43`. [Claude Code interactive-mode documentation](https://code.claude.com/docs/en/interactive-mode)
Verification: The installed client reports version 2.1.220; current official documentation confirms immediate return and separately confirms completion tracking.

[severity:low][the real Why] The checked gate claims behavior was confirmed by direct run, while the same section explicitly says behavior was not verified; the recorded commands check installation, secrets, and textual parity only.
Evidence: `task.md:44-59` describes structural checks and disclaims behavioral verification, but `task.md:62` checks “behavior confirmed by direct run.”
Verification: None of the four recorded checks launches a Claude session or observes the standing instruction affecting orchestrator behavior.

Omitted-detail: 0 low

GPT verdict: reject — The standing contract’s incomplete and overbroad registration routing creates concrete fail-open and unattended-stall paths.

## Carried decisions
- **The stop list is labelled a SUMMARY and names its authority.** It read as exhaustive while
  omitting one entry — a `dstack reg` that failed for a cause `migrate` cannot fix (unusable session
  id, unwritable registry, a `status` line that never says `(this session)`). A summary that quietly
  drops a stop is exactly how an unattended run continues past one, so the missing entry is in and
  the text now points at `scheduling.autonomy` before anything concludes something is not a stop.
- **The `reclaim` divergence was closed by narrowing the AUTHORITY, not by loosening the summary.**
  The full-cycle stop table used to carve out a "provably orphaned" handoff; that carve-out named a
  state `reg` never produces, so the strict wording here was the correct one and the two now agree.
- **Blocking is attributed to the right thing.** "The call does not return until the command
  finishes" was wrong about which lifecycle blocks: the Bash tool call returns immediately, which is
  what `run_in_background` means, and it is the background task that stays alive. The rule that
  matters is unchanged — a line placed after `dstack run` does not run until the round is over — but
  stating it of the wrong object invites exactly the hand-rolled watcher this Goal removed.

Consensus: disagreed
