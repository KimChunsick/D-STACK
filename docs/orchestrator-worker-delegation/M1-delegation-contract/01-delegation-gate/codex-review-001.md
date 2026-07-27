# Codex adversarial review — Round 001

## Review scope
First round for this unit. Bundle: the reworked `full-cycle/SKILL.md`, this unit's `task.md`, and
the Goal's research artifact.

## GPT findings
[severity:high][security] The `.worktreeinclude` policy calls the file an allowlist but never requires literal, anchored paths or rejects pattern syntax, so one entry can silently copy multiple ignored files—including credentials—into every worker checkout.
Evidence: `bootstrap` prohibits directories but `.worktreeinclude` uses gitignore syntax; basename patterns match at every depth and non-trailing-slash patterns can match files or directories. [Claude Code worktree documentation](https://code.claude.com/docs/en/worktrees), [Git pattern-format documentation](https://git-scm.com/docs/gitignore)
Verification: An entry `config.json` intended for one fixture also matches an ignored `local-auth/config.json`; both are copied despite the “individual non-secret fixture” rule.
Suggested direction: Permit only exact repository-relative regular-file paths, reject gitignore metacharacters and directory matches, and verify each resolved source against the secret deny list before spawning.

[severity:medium][technical correctness] The contract validates a manually created worktree without binding the spawned worker to that checkout; isolation creates another worktree, while a non-isolated subagent starts in the parent working directory.
Sites: `worker-fanout.requires`; confirmed: `worker-fanout.per-task`, `worktree-lifecycle.create`, `worktree-lifecycle.bootstrap`, `worktree-lifecycle.cleanup`.
Evidence: `create` mandates `git worktree add`, but `per-task` only places its path in the brief; Claude documents that `isolation: worktree` creates a temporary checkout and otherwise starts the subagent in the parent cwd. [Claude Code subagent documentation](https://code.claude.com/docs/en/sub-agents)
Verification: Create and confirm worktree A, then spawn with isolation: the worker edits platform-created B, so A’s verified base is irrelevant; without isolation, nothing requires the worker’s actual cwd or HEAD to equal A.
Suggested direction: Before any worker write, require the worker’s actual cwd, Git common directory, branch, and HEAD to match the recorded checkout, using one creation/bootstrap/retention mechanism throughout.

[severity:medium][right-sized technology] The gate delegates every determined implementation, including trivial edits whose negligible context savings cannot justify agent startup, worktree bootstrap, commit, fan-in, and retained-checkout costs.
Evidence: The deployment envelope is one maintainer with a few terminals, `delegate-when` has no benefit threshold, and Claude recommends the main conversation for quick targeted changes. [Claude Code subagent documentation](https://code.claude.com/docs/en/sub-agents)
Verification: “Correct one typo in `src/x.ts`,” declared only for that file, satisfies every new eligibility predicate yet incurs the complete serial delegation lifecycle.
Suggested direction: Require a positive context-isolation benefit—such as predictably verbose or materially multi-step execution—and keep quick targeted changes in the orchestrator.

[severity:low][DX] The checked “behavior confirmed” gate is unsupported because the recorded commands parse YAML and exercise unchanged checker verdicts, not the new routing behavior.
Sites: task document “Direct verification”; confirmed: “Gate status”.
Evidence: The schema test proves syntax, while both `check-parallel.sh` examples explicitly exercise a checker this task did not modify.
Verification: No recorded scenario evaluates `delegate-when`, binds a worker to the verified checkout, or exercises bootstrap and review-closure retention.

[severity:low][DX] The task record remains internally unfinished, with duplicated “Files changed” sections, one containing `<pending>`, and E2E verification still pending.
Evidence: Both incomplete sections are present in the supplied full task snapshot.
Verification: The document cannot serve as the claimed completed-change record without choosing one authoritative file inventory and recording the pending verification state consistently.

Omitted-detail: 0 low

GPT verdict: reject — The change leaves concrete credential-copy, worker-checkout identity, and deployment-scale blockers unresolved.

## Bundle size (the ratchet, recorded)

Round 001 bundle: 40,728 bytes. Round 002 must be at or below this.

## Round outcome

Three blocking findings and two lows, all accepted; one accepted with its mechanism corrected.
Reasoning is in `response-001.md`, deliberately outside the reviewed corpus. Running ledger:
`findings.md`.

Consensus: disagreed
