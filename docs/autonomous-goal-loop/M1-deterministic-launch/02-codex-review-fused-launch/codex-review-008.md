# Codex adversarial review — Round 008

## Review scope
Batch pass 2 (consolidated round) | `REVIEW_MODE=serial` | bundle 100877 bytes (r7 89950). Allowlist unchanged: the review-unit folder, `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`. **§4 cap closure** — every finding fixed, residuals and follow-ups recorded below and in `findings.md`, no concrete HIGH open.

## GPT findings

[severity:medium][technical correctness] The resend parser no longer fails closed: trailing-comma-plus-whitespace and whitespace-only requests silently reduce or discard requested evidence.
Sites: `claude/skills/codex-review/assemble-review.sh:303-338`; confirmed: `claude/skills/codex-review/SKILL.md:63-70`, `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/task.md:137-148`
Evidence: The precheck catches only a literal final comma, while Bash removes trailing IFS fields and the implementation explicitly treats whitespace-only input as no request.
Verification: Running the exact splitting logic produced one field `<1>` for `REVIEW_FULL_ROUND_IDS='1, '` and zero fields for `' '`, while `1,,3` remained fatal.
Suggested direction: Validate the complete delimiter grammar before splitting, rejecting whitespace-only values and empty fields followed by whitespace.

[severity:medium][technical correctness] The post-seal reset cannot close this unit: immutable Round 007 is `Consensus: disagreed`, yet it is declared the second and final reset round while cap closure requires `Consensus: resolved`.
Sites: `claude/skills/codex-review/SKILL.md:536-543,752-778`; confirmed: `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/codex-review-007.md:63-69`, `task.md:141-150`
Evidence: The new rule was introduced by Round 007’s response after that round required sealing as disagreed because its fixes were not independently re-reviewed.
Verification: The latest canonical round remains disagreed; another round exceeds the two-round budget, while rewriting Round 007 violates immutability.
Suggested direction: Define an activation boundary or one explicit verification-transition round for epochs whose final round was sealed before the reset rule existed.

[severity:medium][software structure] Removing the precedence sentence did not resolve the contract split: the orchestrator still separates never-bundled rebuttals and auto-disposes concrete mediums, while the elected reviewer requires one immutable exchange and explicit user disposition.
Sites: `claude/skills/codex-review/SKILL.md:487-589,670-702,773-778`; confirmed: `codex/skills/adversarial-review/SKILL.md:96-108`, `claude/hooks/fullcycle-gate.sh:411-420`
Evidence: The orchestrator adds a fourth cap-based disposition and permits `Consensus: resolved`; the elected contract permits only fixed, disproved, or user-disposed blockers.
Verification: The gate validates only the positive consensus token and never verifies findings, rebuttals, or user disposition, so an unresolved medium can pass.
Suggested direction: Align the filing and consensus contracts, then make the gate enforce the shared blocker-disposition invariant.

[severity:medium][technical correctness] The reopened task record was not reconciled with the final scope: it still lists only `SKILL.md`, although the fix modifies `assemble-review.sh` despite the unit’s no-growth/separate-follow-up rule.
Sites: `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/task.md:12-18,51-53,137-150`; confirmed: `claude/skills/codex-review/SKILL.md:640-646`, `claude/skills/codex-review/assemble-review.sh:303-338`
Evidence: The task’s changed-file declarations omit the parser change, while its own reopened narrative and current diff show that change landed here.
Verification: `git diff --name-only -- claude/skills/codex-review` returns both files, but both task inventories name only `SKILL.md`.
Suggested direction: Update the task inventory and either move the assembler fix to its declared review unit or define an explicit reopened-epoch scope-expansion rule.

[severity:low][DX] F030 is recorded as fixed although its pre-launch scratch leak was explicitly accepted and remains unchanged.
Sites: `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/findings.md:40`; confirmed: `response-006.md:25-29`, `claude/skills/codex-review/SKILL.md:314-333`
Evidence: The response accepts retaining scratch whenever quiescence is unknown, but the ledger later marks the entire finding fixed.
Verification: If `dstack` fails before publishing `exit`, the EXIT predicate is false and `$SCRATCH` remains.
Suggested direction: Record the pre-launch leak as an accepted residual rather than fixed.

Omitted-detail: 0 low

GPT verdict: reject — the resend parser can silently omit requested evidence, the reset epoch has no legal closure transition, and the unresolved review-contract split still permits unsupported positive consensus.

## Carried decisions
- **The resend grammar is validated BEFORE splitting, and that ordering is the fix.** Accepting
  whitespace as a separator let IFS absorb things the split could no longer see: `1, ` lost its
  empty field and came back as a quiet request for round 1 alone, ` ` came back as no request. Both
  silently REDUCE what the reviewer asked for, which is the one failure this validation exists to
  prevent — an unmet request is indistinguishable from one never made. A `case` over the trimmed
  string sees them; the split cannot. Verified end to end against this unit: `1 3`, `1,3`, `1, 3`
  all return rc=0 and "rounds 1 3 by request", while `1,`, `1, `, ` `, `1,,3`, `1, ,3`, `[1]`,
  `1 x` and an out-of-range round are all FATAL.
- **The reset budget starts when the rule does, and cannot expire on a `disagreed` round.** Both
  halves were learned by getting them wrong here. A budget cannot retroactively govern rounds that
  ran before it existed, and declaring one "spent" for an epoch whose rounds predated it left the
  unit with no legal move. And a `disagreed` round is not a closure — it is a round that found
  things. §4's closure is an ACTION (record open findings with severity and evidence, seal
  `resolved`, name them in the report), and reaching the cap obliges you to take it; the round that
  performs it is allowed whatever the count says. A budget that strands a unit between "no more
  rounds" and "no positive seal" is a trap, not a termination rule.
- **The wrapper traps every signal `dstack` traps.** Three was not enough: under zsh an untrapped
  fatal signal skips the EXIT trap entirely, so a wrapper-only USR1 exited 158 and LEAKED the
  scratch directory. Measured old vs new — bash cleaned either way, zsh only with the full set. What
  it does not buy is stated: a handler cannot cancel a foreground `dstack run`.
- **The scope expansion into `assemble-review.sh` is declared, not smoothed over.** The skill
  publishes an invocation the assembler rejected; fixing one without the other means publishing a
  corrected recipe for a command that still refuses it. The allowlist did not grow — the assembler
  was already in the bundle — but the task's file inventory had not been updated, and that was the
  finding.
- **F030 is split into a fix and an accepted residual.** The signalled path is fixed; the PRE-LAUNCH
  leak is deliberate, because if `dstack` dies before publishing `exit` quiescence is unknown and
  keeping the directory is the safe choice. The ledger said "fixed" for both halves, which was wrong.

Consensus: resolved
