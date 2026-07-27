# Codex adversarial review — Round 003

## Review scope
Re-review | `REVIEW_MODE=serial` | bundle 40247 bytes (round 002: 39716 — **ratchet exceeded by 531
bytes, 1.3%**; see the note below)
Allowlist SHRANK again: `AGENTS.md` dropped (in scope for rounds 001–002, no findings, unchanged
since). Remaining: the review-unit folder, `claude/bin/dstack`.

A first attempt at this round (`t01-r3`) was terminated mid-run by the harness; per the
nonzero-exit rule it was discarded rather than sealed, and the round was re-run under `t01-r3b`.
Its teardown is recorded as evidence in `task.md`.

**Ratchet note, stated rather than quietly broken.** The bundle's PROSE shrank (18.7KB → 15.6KB);
the overage is the `claude/bin/dstack` diff, which grew because rounds 001–002 demanded fixes, plus
~460 bytes of new evidence. There is also a structural cause: `assemble-review.sh` compacts an
older round only when that round file contains a `## Carried decisions` section matching its
companion, while the skill's termination rules say the round file holds findings, bundle size and
the consensus line *and nothing else*. Following the latter means this unit's sealed rounds can
never compact, so every subsequent bundle carries all of them whole. Recorded as a defect for the
task that owns the `codex-review` skill; not worked around by rewriting a sealed round.

## GPT findings

[severity:high][technical correctness] `run_abort` mistakes process-group-leader death for complete teardown: after one TERM it waits only for `$child`, publishes `exit`, and `rm-run` later checks only that positive PID. A TERM-resistant descendant can remain invisible while its capture is considered terminal or deletable.
Sites: primary: `claude/bin/dstack:run_abort`; confirmed: `claude/bin/dstack:cmd_rm_run` launched-process guard, `task.md:Accepted residual`, `task.md:E2E verification / Signal handling`
Evidence: Neither abort publication nor deletion verifies that the recorded process group has become empty; the recorded zero-stray probe therefore covers only children that all honor TERM.
Verification: Reproducing the exact `set -m` → group TERM → leader wait sequence with a leader exiting 143 and a descendant ignoring TERM produced `direct_status=143 group_survived=yes`.
Suggested direction: Treat `$child` as both PID and PGID: require negative-PGID quiescence, with bounded TERM-to-KILL escalation, before publication and make `rm-run` refuse while that PGID has any member.

[severity:medium][technical correctness] Cleanup still does not own every catchable exit from claim through publication: the trap is installed after claim and capture writes, and it covers only INT/TERM/HUP. A pre-trap TERM strands the claim; an untrapped terminating signal after fork orphans the child.
Sites: primary: `claude/bin/dstack:cmd_run` claim/trap ordering; confirmed: `claude/bin/dstack:run_abort`, `claude/bin/dstack:cmd_rm_run` unknown-child refusal, `task.md:E2E verification / Signal handling`
Evidence: `.launch` is created before reserved-file and command-record writes, but cleanup starts only immediately before `set -m`; the 20-sample evidence says every sample had an existing child, so it does not cover this pre-fork interval.
Verification: A Bash reproduction exited 143 after a claim marker when TERM arrived before trap installation; with the launched child in its own group, untrapped USR1 produced `supervisor_alive=no child_alive=yes`.
Suggested direction: Establish an EXIT cleanup owner immediately after the claim succeeds, retain it through publication, and explicitly disarm it on normal completion.

[severity:low][technical correctness] The recycled-PID rationale in the post-wait loop is incorrect for the deployed Bash behavior: repeated `wait "$child"` returns the cached status, so a recycled live PID can make the loop spin until that unrelated process exits.
Sites: primary: `claude/bin/dstack:cmd_run` wait/`kill -0` loop
Evidence: The comment expects an extra wait to return 127, but Bash retains the reaped job's status while `kill -0` examines the kernel's current PID occupant.
Verification: Direct probes with child statuses 0, 7, and 143 returned the same cached status on the second wait, never 127.
Suggested direction: Remove the positive-PID liveness probe after a successful wait; the signal handler exits instead of returning to this loop.

Omitted-detail: 0 low

GPT verdict: reject — Process-group teardown remains incomplete and cleanup still has concrete abnormal-exit gaps that can leave invisible work or stranded claims.

Consensus: disagreed
