# Codex adversarial review — Round 005 (closing round)

## Review scope
Re-review | `REVIEW_MODE=serial` | bundle 58287 bytes (round 004: 49029 — ratchet exceeded; cause
unchanged and recorded in `carried-003.md`/`carried-004.md`: this unit's sealed rounds cannot
compact, so each one is carried whole, and the code diff grows with each demanded fix).
Allowlist unchanged: the review-unit folder, `claude/bin/dstack`.

**This is the closing round.** The round cap for a per-task unit is 5. Closure is applied here per
the skill's cap rule: findings are resolved or recorded, the round is sealed, and the residual is
named in `task.md` and in the final report rather than carried into a sixth round.

## GPT findings

[severity:high][technical correctness] `run_done` is set before group settlement, so a catchable signal during settlement makes cleanup return immediately and exits while descendants remain alive.
Sites: primary: `claude/bin/dstack:1129`; confirmed: `claude/bin/dstack:983-987`, `claude/bin/dstack:1001-1013`, `claude/bin/dstack:1063-1064`, `claude/bin/dstack:1130-1138`
Evidence: The normal path marks the run done before the bounded settlement window; the signal handler then observes `run_done=1`, skips teardown, and exits 6.
Verification: On macOS GNU Bash 3.2.57, a leader exiting 7 with a TERM-ignoring descendant followed by TERM during settlement produced `supervisor_status=6` and `group_alive_after_supervisor_exit=yes`.
Suggested direction: Separate reentrancy protection from finalization and keep one idempotent cleanup owner active until settlement and publication—or explicit refusal—finish.

[severity:medium][technical correctness] Every `die`-driven EXIT still loses `cmd_run`'s locals; default expansions avoid nounset at reads but cause pre-fork failures to skip claim release and post-fork failures to lack the capture path needed for publication.
Sites: primary: `claude/bin/dstack:1017`; confirmed: `claude/bin/dstack:39`, `claude/bin/dstack:983-1009`, `claude/bin/dstack:1063`, `claude/bin/dstack:1075-1090`, `claude/bin/dstack:1111-1112`, `claude/bin/dstack:1130-1138`
Evidence: With `d` unwound, `[ -n "${d-}" ]` suppresses `.launch` removal; after a fork, `$!` recovers the process identifier but `run_publish` still dereferences unset `$d`.
Verification: Direct, nested-`die`, and trap-function probes on macOS GNU Bash 3.2.57 all found the caller's local unavailable; the current defaulted shape reported `d=UNSET` and `claim-release=skipped-because-d-is-unset`.
Suggested direction: Keep cleanup state in process-lifetime storage until traps are disarmed, then directly re-run both pre-fork and post-fork `die` paths.

[severity:low][UI & UX / DX] Non-blocking follow-up: an abort whose terminal publication fails reports both "could not publish" and "recorded exit."
Sites: primary: `claude/bin/dstack:1007-1010`
Evidence: Failure of `run_publish` executes its warning through `||`, after which the success-sounding `printf` runs unconditionally.
Verification: Direct control-flow inspection reaches both messages on the same false return.

Omitted-detail: 0 low

GPT verdict: reject — A verified Bash 3.2 signal window can still orphan a live process group, and `die` paths still lose the state required to release or finalize their capture.

## Closure

The `GPT verdict` line is recorded as data and is advisory. At closure every finding of this round
is resolved: F013 fixed and verified, F014 disproved by instrumenting the real handler, F015 fixed.
No concrete high or medium remains open, so no escalation is required. The residual carried into
the final report is that F013's fix is verified by direct run but has not itself been through an
adversarial round — the cap is what stops the loop, not an absence of things left to look at.

Consensus: resolved
