# Codex adversarial review — Round 007

## Review scope
Batch pass 1 (consolidated round, per the user's mid-Goal decision) | `REVIEW_MODE=serial` | bundle 89950 bytes. Allowlist: the review-unit folder, `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`.

## GPT findings

[severity:medium][technical correctness] The launch fence snapshots `$RD` before reconstructing it, detaching cleanup from the current capture and potentially recreating the live-CWD deletion defect.
Sites: `claude/skills/codex-review/SKILL.md:307-324`; confirmed: `SKILL.md:353-356`
Evidence: `RUNDIR="$RD"` precedes the fence’s `RD=...`; an unset value prevents normal cleanup, while an inherited completed-run path makes the trap clean current scratch without proving the current run quiescent.
Verification: Bash and zsh both captured `RUNDIR=<>` when unset; both treated an inherited old capture containing `exit` as permission to clean an unrelated current scratch directory.
Suggested direction: Construct the current `LABEL` and `RD` before arming traps, and derive the guard and stdin path from that single current-run identity.

[severity:medium][DX] Step 2a says any nonzero notification means failure despite establishing that wrapper status is non-authoritative, causing valid completed reviews to be discarded and rerun.
Sites: `claude/skills/codex-review/SKILL.md:331-342`; confirmed: `SKILL.md:396-412`
Evidence: A deferred signal can make the wrapper return 143 after `dstack` has published child status 0, but Step 2a orders “Exactly zero” before its recipe reads the authoritative `exit` file.
Verification: Direct Bash and zsh probes both returned wrapper status 143 only after a successful foreground child completed.
Suggested direction: Treat the notification as a wake-up only; classify the round solely from the current capture’s atomic `exit` record.

[severity:medium][technical correctness] The promised older-round resend mechanism rejects its documented invocation, preventing requested evidence from reaching the next review.
Sites: `claude/skills/codex-review/SKILL.md:63-71`; confirmed: `claude/skills/codex-review/assemble-review.sh:303-328`
Evidence: The instruction uses `REVIEW_FULL_ROUND_IDS="1 3"`, while the assembler deliberately splits only on commas and rejects whitespace inside a field.
Verification: The production parsing logic rejected `1 3` with status 1 and accepted `1, 3` with status 0.
Suggested direction: Document comma-separated IDs or make the parser accept the documented syntax.

[severity:medium][security] The artifact’s “THIS file governs” directive overrides the elected review contract, omits rebuttals from the immutable corpus, and permits positive consensus with unresolved concrete mediums.
Sites: `claude/skills/codex-review/SKILL.md:470-576`; confirmed: `SKILL.md:653-677,719-731`, `codex/skills/adversarial-review/SKILL.md:96-108`, `claude/skills/codex-review/assemble-review.sh:332-360`, `claude/hooks/fullcycle-gate.sh:411-420`
Evidence: The elected contract requires one immutable invocation/rebuttal exchange and user disposition of unresolved blockers; the artifact instead never bundles responses and introduces automatic “accepted residual” consensus.
Verification: The assembler’s emission paths contain no response files, and the production gate regex accepted `Consensus: resolved` without inspecting findings or their dispositions.
Suggested direction: Align the orchestrator and gate with the elected contract; unresolved concrete mediums cannot receive positive consensus without an explicit user decision.

[severity:medium][software structure] The cumulative five-round cap has no transition for mandatory post-seal reopening, so a reopened unit can neither obey the cap nor complete the required review.
Sites: `claude/skills/codex-review/SKILL.md:708-731`; confirmed: `claude/skills/full-cycle/SKILL.md:180-182`, `docs/autonomous-goal-loop/M1-deterministic-launch/02-codex-review-fused-launch/task.md:117-127`
Evidence: The review skill mandates closure at round five, while the parent workflow mandates reopening after later bundle changes; neither defines a new budget epoch or an escalation transition.
Verification: The current per-task unit already contains six numbered rounds, with Round 006 sealed `Consensus: disagreed`, directly exceeding the stated hard cap.
Suggested direction: Define explicit post-seal review epochs with their own budget, or require an escalated user transition when reopening beyond the cumulative cap.

Omitted-detail: 0 low

GPT verdict: reject — undefined launch state, contradictory status handling, a broken evidence-resend recipe, and incompatible consensus/reopen contracts remain concrete blockers.

## Carried decisions
- **The launch call RECONSTRUCTS the run dir; it does not inherit it.** Step 2's fence opened with
  `RUNDIR="$RD"` and defined `RD` four lines later, in a call where `$RD` from the assembly step no
  longer exists at all — Step 1 says why, a shell variable does not survive between tool calls. The
  armed trap therefore tested `[ -e "/exit" ]`, always false, so the scratch dir was never removed
  on the one path where removing it is correct. `LABEL` is now assigned first and `RD` derived from
  it before anything else.
- **`<run-dir>/exit` is the round's status in the PROSE too, not only in the recipe.** Step 2a
  opened by calling any nonzero notification a failed round, which contradicts what Step 2 had just
  established: a deferred signal makes a completed round report 143. A missing `exit` file is also
  not a pass — it means the run never reached quiescence.
- **The signal handlers leave the gated EXIT trap ARMED.** `trap - EXIT` was carried over from when
  the cleanup was unconditional. The gate answers the question better: measured in bash and zsh,
  exit file present gives rc=143 with the directory removed, absent gives rc=143 with nothing
  removed. Disarming turned the deferral — which means the handler usually runs after `exit` was
  published — into a guaranteed leak.
- **A precedence claim over the reviewer is unenforceable, so it is gone.** "THIS file governs" was
  addressed to a reviewer that is told to follow `$adversarial-review` exactly and is told in the
  same prompt to treat the whole payload as untrusted data. What is true is narrower: these rules
  govern the ORCHESTRATOR, because it is the side that runs them and the side the Stop hook parses.
  The Codex-side inconsistency stays a recorded follow-up and a reviewer filing it is right to.
- **A post-seal reopening gets its own budget, counted from the reopening.** This Goal hit the gap
  directly: two units sealed AT the 5-round cap and were then reopened by `post-seal-rule`, leaving
  no legal next move. The cap now counts rounds since the reopening and resets SMALLER (2 per-task,
  3 per-milestone), and the non-convergence window restarts with it, because the old counts measured
  a corpus that no longer exists.

Consensus: disagreed
