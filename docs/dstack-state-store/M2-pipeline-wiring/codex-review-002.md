# Codex adversarial review — Round 002

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] Bundle construction is not a checked precondition, so a failed assembler can be followed by a successful review of an empty or partial bundle.
Evidence: The assembly command has no guard, the same-shell recipe checks only `codex exec`’s status, and the document acknowledges that invalid file arguments may be silently skipped.
Verification: Bash continues after an unsuccessful command without `set -e` or an explicit check; if Codex then exits zero, the shown status check accepts the round.
Suggested direction: Require successful assembly and validation of every allowlisted file before launching Codex.

[severity:medium][technical correctness] Backgrounding loses the only `RD`/`OUT` bindings, while the allocator’s no-reuse rule also makes a failed deterministic round label unretryable.
Sites: primary: `claude/skills/codex-review/SKILL.md` Step 2 handoff; confirmed: run-directory allocation and post-completion triage commands.
Evidence: The document states that shell variables do not survive tool calls, emits no resolved run path, later triages `"$OUT"`, and says `run-dir` rejects an already-used label.
Verification: After the background shell exits, a new shell has no `OUT`; after any post-allocation failure, retrying the prescribed label fails before assembly.
Suggested direction: Provide durable non-allocating lookup by round label, with separate unique attempt identifiers for retries.

[severity:medium][technical correctness] Required pause and handoff paths still use bare `dstack` commands despite declaring that its installation directory is not on `PATH`.
Sites: primary: `claude/skills/full-cycle/SKILL.md` `waits.user-input`; confirmed: concurrent-stream status guidance, milestone-boundary handoff, and `skill-schema.test.sh`.
Evidence: The document warns that bare `dstack` fails in the declared setup, yet prescribes `dstack unreg`, `dstack status`, and `dstack reclaim`.
Verification: Without `~/.claude/bin` on `PATH`, those commands resolve to “command not found,” preventing the documented pause or ownership handoff.
Suggested direction: Use the absolute CLI path consistently and update schema assertions accordingly.

[severity:medium][technical correctness] The migration repair requires refusing an existing path but not atomic no-overwrite creation, leaving a check-then-write race that can clobber concurrent migrations.
Sites: primary: `claude/skills/full-cycle/SKILL.md` concurrent-stream guidance; confirmed: `docs/dstack-state-store/research/dstack-state-store.md` same-key atomicity analysis.
Evidence: The research requires `O_EXCL`, hard-link publication, or equivalent atomic conflict handling; the resulting instruction specifies only an existence condition.
Verification: Two same-worktree streams can both observe an absent timestamped path and then write it, allowing the later write to replace the earlier migration.
Suggested direction: Require atomic exclusive creation and handle `EEXIST` by generating a collision-resistant replacement name.

[severity:low][technical correctness] The file-list warning reverses normal Bash expansion behavior.
Evidence: It says `FILES="a b c"` passed as `$FILES` can become one argument; normally unquoted expansion produces three arguments, while `"$FILES"` produces one.
Verification: Executing those exact expansions under Bash yielded argument counts of three and one respectively.

[severity:low][DX] The milestone declares a “no tests” policy while changing `skill-schema.test.sh` and recording multiple test scripts as rerun.
Sites: primary: milestone Gate status; confirmed: Files changed and Pre-review defect-class self-sweep.
Evidence: The gate calls for direct runs because the repository has “no tests,” while the same document describes updating and executing named test scripts.
Verification: An instruction-following model cannot determine whether these scripts are prohibited tests or required direct-run verification.
Suggested direction: State whether these scripts are policy-exempt verification checks and name the required commands consistently.

Omitted-detail: 0 low

GPT verdict: reject — The repairs leave concrete medium failure paths in review-bundle integrity, background result recovery, CLI invocation, and concurrent migration creation.

## Carried decisions
Round-1 decisions still standing (discovery time never changes blocking status; the six-round
budget escalates rather than downgrades; triage matches the contract's `[severity:…][axis]`
format with no cap on the high/medium query; `<review-unit>` is a single abstraction). Added in
Round 2:

- **Assembly is a precondition, not a step.** Guard `run-dir` and the assembler with `|| exit 1`
  and check the bundle's entry count before launching. A review of an empty bundle exits 0.
- **This harness runs zsh, not bash.** Unquoted parameter expansion does NOT word-split here.
  Pass file lists as literal arguments, never through a variable.
- **Run labels are per-attempt.** The allocator refuses a used label by design; retry with a new
  suffix. The durable path is `.dstack/runs/$CLAUDE_CODE_SESSION_ID/<label>/out.txt` — never
  call `run-dir` again to recover it.
- **Atomic exclusive creation, never test-then-write**, wherever two streams can generate the
  same path. Same lesson as `dstack reg`'s `ln` publish.
- **The CLI is always invoked by absolute path**; nothing puts `~/.claude/bin` on `PATH`.
- Open follow-up: sharpen the no-tests-versus-pinned-checks distinction in `AGENTS.md`, deferred
  while that file is inside M1's open review bundle.

Consensus: disagreed
