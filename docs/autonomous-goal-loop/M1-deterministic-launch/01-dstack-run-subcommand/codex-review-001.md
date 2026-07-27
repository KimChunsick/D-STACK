# Codex adversarial review — Round 001

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 40922 bytes
Allowlist: the review-unit folder, `claude/bin/dstack`, `AGENTS.md`,
`docs/autonomous-goal-loop/research/autonomous-resume.md`.
Round wall clock 10.3 min (`codex exec` started 23:59:10, `exit` published 00:09:26).

## GPT findings

[severity:high][technical correctness] Terminating the `dstack` supervisor can leave `codex exec` alive and unobserved; `rm-run` can then delete the capture while the orphan continues writing and consuming credits.
Sites: primary: `claude/bin/dstack:971`; confirmed: `claude/bin/dstack:896-898`, `claude/bin/dstack:1023-1039`, `task.md:52`, `task.md:88-92`
Evidence: No trap, child PID, or process-group teardown exists; cleanup checks only the recorded supervisor PID.
Verification: A foreground Bash-child reproduction followed by supervisor-only `SIGTERM` printed `foreground_child_alive_after_supervisor_term=yes`; the child required separate termination.
Suggested direction: Own a child process group, terminate and reap it on every catchable supervisor exit, and make cleanup refuse while either the supervisor or launched process remains alive.

[severity:medium][technical correctness] An empty label is accepted and collapses the capture path to `runs/<sid>/`, placing `.launch` and output files in the session root where `rm-run` cannot address them.
Sites: primary: `claude/bin/dstack:820-830`; confirmed: `claude/bin/dstack:932`, `claude/bin/dstack:1013`
Evidence: `require_label` lacks an empty-string case, while `cmd_run` uses the label directly and `rm-run` separately rejects `''`.
Verification: Evaluating the exact label patterns reported `empty_label_accepted_by_require_label_patterns=yes` and resolved the path to `/repo/.dstack/runs/session/`.

[severity:medium][technical correctness] Adoption does not establish reserved-path invariants: a pre-existing `exit` bypasses active-run protection, while an `exit` directory receives `exit.tmp` and lets `dstack` report `DONE` without publishing the promised file.
Sites: primary: `claude/bin/dstack:954-956`; confirmed: `claude/bin/dstack:976-978`, `claude/bin/dstack:1023`, `claude/bin/dstack:1102-1104`
Evidence: Reserved paths are checked only for symlinks; any existing `exit` suppresses the liveness guard, and `mv source existing-directory` succeeds by nesting the source.
Verification: An adopted capture containing a regular `exit` is deletable during execution; with an `exit` directory, publication lands at `exit/exit.tmp` rather than `exit`.
Suggested direction: Require `exit` and `exit.tmp` to be absent when claiming an adopted directory and validate every reserved path's exact type before launch.

[severity:medium][the real Why] The recorded evidence never exercises a long, output-silent `dstack run` through one harness background call, so the completion-wake behavior that constitutes the task's intent remains unverified.
Sites: primary: `task.md:105-117`; confirmed: `task.md:162-168`, `claude/bin/dstack:971-978`, `autonomous-resume.md:Unverified`
Evidence: Probe L emitted minute-by-minute markers, whereas `cmd_run` redirects all child output and emits only final `DONE`; the actual 15–25-minute E2E is deferred and both verification gates remain unchecked.
Verification: The official Bash tool reference documents background execution but supplies no completion-reinvocation or output-silence lifetime guarantee, leaving this as an unverified local inference.

[severity:low][technical correctness] The task disposition says the `rm-run`/`prune` active-deletion finding is fixed, but `prune` ignores launch state and liveness entirely.
Sites: primary: `task.md:56`; confirmed: `claude/bin/dstack:1029-1030`, `claude/bin/dstack:1060-1069`
Evidence: `prune` selects solely by directory mtime and deletes matches with `rm -rf`; no `.launch`, supervisor, or terminal-record check occurs.
Verification: The stated 3–25-minute run and eight-day threshold make ordinary exposure unlikely, but the documented claim still exceeds the implemented invariant.
Suggested direction: Narrow the disposition to `rm-run`, or make `prune` honor active-run liveness before claiming full lifecycle protection.

Omitted-detail: 0 low

GPT verdict: reject — The supervisor termination path recreates the invisible orphan and capture-loss failure the change claims to eliminate, with additional unresolved correctness and core-verification blockers.

Consensus: disagreed
