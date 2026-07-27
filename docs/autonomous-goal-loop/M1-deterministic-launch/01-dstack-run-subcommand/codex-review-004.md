# Codex adversarial review — Round 004

## Review scope
Re-review (wind-down round) | `REVIEW_MODE=serial` | bundle 49029 bytes (round 003: 40247 —
**ratchet exceeded by 8782 bytes**).
Allowlist unchanged from round 003: the review-unit folder, `claude/bin/dstack`.

**Ratchet, stated rather than quietly broken.** Composition: `claude/bin/dstack` diff 26764 (the
code under review, grown by the fixes rounds 001–003 demanded), three sealed rounds 10736, task.md
10836. The prose was cut twice already; cutting further removes the direct-run evidence the
reviewer is asked to judge, which defeats the purpose the rule serves. The structural cause is
recorded in `carried-003.md`: the assembler compacts an older round only when that round file
contains a `## Carried decisions` section matching its companion, while the termination rules say
the round file carries findings, size and consensus and nothing else — so this unit's sealed rounds
never compact, and the bundle line for round 001 says so outright
(`carried-001.md is not a complete carried-state companion`). Sealed rounds were not rewritten to
dodge it. What the rule exists to prevent — a corpus growing faster than fixes close it — is not
what is happening here: the finding stream is 4 → 1 → 2 → 1, all in one subsystem.

## GPT findings

[severity:high][technical correctness] The `EXIT` cleanup loses its required state on every `die`: Bash unwinds `cmd_run`'s locals before running the trap, so `set -u` aborts `run_cleanup` at `$run_published`.
Sites: primary: `claude/bin/dstack:967-988`; confirmed: `claude/bin/dstack:39`, `claude/bin/dstack:991-992`, `claude/bin/dstack:1038`, `claude/bin/dstack:1041-1065`, `claude/bin/dstack:1086-1087`, `claude/bin/dstack:1107-1108`, `task.md:E2E verification / post-claim die`
Evidence: Before-fork failures strand `.launch`; a child-record write failure after the fork exits without signalling the launched group, recreating the orphan path.
Verification: An exact scope reproduction on the declared macOS Bash 3.2.57 produced `run_published: unbound variable`; therefore the recorded claim-release output cannot describe the current code.
Suggested direction: Keep cleanup state in process-lifetime storage that remains defined during `EXIT`, then directly exercise both pre-fork and post-fork `die` paths.

[severity:high][technical correctness] Group quiescence is not actually a publication gate: both callers continue when `run_group_settle` reports a group still alive after SIGKILL, publish `exit`, and make the capture deletable.
Sites: primary: `claude/bin/dstack:1100-1108`; confirmed: `claude/bin/dstack:978-984`, `claude/bin/dstack:1158-1187`, `task.md:What was done / F007`, `task.md:E2E verification / Teardown completeness`
Evidence: A false return means the final negative-PGID liveness probe still succeeded, but `|| printf WARNING` is followed unconditionally by `run_publish`; `rm-run` performs no liveness check once `exit` exists.
Verification: Tracing the implemented false branch yields live group → warning → terminal publication → deletion allowed; the supplied busy-loop run covers only successful SIGKILL teardown.
Suggested direction: Publish only after confirmed group quiescence; otherwise retain a nonterminal claim so deletion continues to fail closed.

[severity:low][technical correctness] After `wait` releases the group leader, settlement treats its numeric PGID as durable and can TERM/KILL an unrelated group if that identifier is recycled.
Sites: primary: `claude/bin/dstack:1094-1101`; confirmed: `claude/bin/dstack:run_group_settle`, `claude/bin/dstack:run_cleanup`
Evidence: Negative-PGID probes and signals carry no ownership token; the abort fallback can likewise signal a recycled positive PID after the original group disappears.
Verification: The supplied runs do not exercise PID/PGID reuse in the wait-to-probe interval; this is the signalling form of the recycled-identity hazard already documented for liveness probes.
Suggested direction: Pin an owned process-group identity through settlement or explicitly record this narrow race as an accepted residual.

Omitted-detail: 0 low

GPT verdict: reject — The advertised EXIT owner fails on concrete `die` paths, and teardown failure can still publish a terminal, deletable capture while launched work remains alive.

Consensus: disagreed
