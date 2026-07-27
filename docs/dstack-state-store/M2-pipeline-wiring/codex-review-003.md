# Codex adversarial review — Round 003

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The Round-2 same-shell repair is not executable: Step 1 still assembles separately, while Step 2 calls undefined `DS_run_dir`/`$AS`, never assigns `IN`, and later triages a lost `OUT`.
Sites: primary: `claude/skills/codex-review/SKILL.md` Steps 1–2; confirmed: durable-path and triage blocks.
Evidence: Repository search finds no definition of `DS_run_dir` or `AS`; `IN` is assigned only in the contradictory Step-1 block, although the real invocation consumes it.
Verification: A fresh shell following Step 2 fails at `DS_run_dir`; inferring past that leaves `"$IN"` empty, and the completion turn has no `OUT` for `tail` or `grep`.
Suggested direction: Provide one complete background-launch command and a separate fresh-turn recovery command that reconstructs the repo-rooted output path and binds `OUT`.

[severity:medium][technical correctness] The bundle entry-count guard accepts omitted review material because skipped files still produce countable `--- ` headers.
Sites: primary: `claude/skills/codex-review/SKILL.md` Step 2; confirmed: `claude/skills/codex-review/assemble-review.sh` `emit_file`.
Evidence: Every missing, symlinked, binary, oversized, secret-denied, or unchanged argument emits a header matching `grep -c '^--- '`, so cardinality does not establish successful inclusion.
Verification: Replacing one valid allowlisted file with one nonexistent file preserves the header count while removing that file’s diff from the review.
Suggested direction: Make any disallowed skip fail assembly or validate an exact filename/status manifest rather than header count.

[severity:medium][technical correctness] The claimed no-reuse allocator remains a check-then-create race, so concurrent duplicate attempts can receive the same directory and mix evidence.
Sites: primary: `claude/bin/dstack` `cmd_run_dir`; confirmed: `claude/skills/codex-review/SKILL.md` per-attempt-label guarantee.
Evidence: `cmd_run_dir` tests `[ -e "$d" ]` and then uses `mkdir -p "$d"`; `mkdir -p` succeeds when another process created the directory between those operations.
Verification: Two invocations can both observe absence, both return success from `mkdir -p`, and then truncate the same `bundle.txt`, `out.txt`, and `err.txt`.
Suggested direction: Create only the parent first, then use one non-`-p` `mkdir "$d"` as the atomic ownership decision.

[severity:medium][technical correctness] The absolute-CLI repair still relies on `"$DS"` in later tool calls where that shell variable is unset.
Sites: primary: `claude/skills/full-cycle/SKILL.md` `waits.user-input`; confirmed: concurrent-stream status, orphan reclaim, milestone handoff, and `skill-schema.test.sh`.
Evidence: `DS` is assigned only inside the later P6 shell block, while earlier and future-turn procedures use it; the review skill itself correctly states that shell variables do not survive tool calls.
Verification: With `DS` unset, the prescribed `"$DS" status` exits 126; the schema check passes because it merely finds one absolute-path substring elsewhere.
Suggested direction: Use the literal absolute CLI path in every independently executable procedure, or rebind `DS` inside each such block.

[severity:medium][technical correctness] The always-loaded `claude/CLAUDE.md` still directs models to the removed registry and contradicts the new background-handoff gate behavior.
Sites: primary: `claude/CLAUDE.md` mandatory-gate section; confirmed: `claude/hooks/fullcycle-inject.sh`, `claude/skills/full-cycle/SKILL.md`, and `claude/hooks/fullcycle-gate.sh`.
Evidence: It says active state is `.fullcycle-active`, pausing removes a line there, and unchecked work prevents turn end; the new implementation uses `.dstack/active/`, rejects a non-empty legacy file, and deliberately permits the continuation stop.
Verification: Following the always-loaded instructions either creates a legacy cutover failure or removes state that no longer controls pausing, while its turn-end claim encourages the forbidden wait loop.
Suggested direction: Update the minimal standing contract in `claude/CLAUDE.md` and include that dependency in the review bundle.

[severity:medium][technical correctness] The accepted convergence fix was not propagated class-wide: durable task records still instruct that late concrete medium defects on unchanged code become follow-ups.
Sites: primary: `docs/dstack-state-store/M2-pipeline-wiring/task.md`; confirmed: `04-review-io/task.md` and `docs/dstack-state-store/GOAL.md` T04.
Evidence: These records retain the Round-4 rule with only high/newly-reachable exceptions, while the repaired skill and carried decision say discovery time never changes a concrete medium’s blocking status.
Verification: A concrete medium path-injection defect first found in Round 4 is simultaneously blocking under the skill and shippable under the durable task records.
Suggested direction: Replace every stale representation with the accepted rule and ensure referenced subordinate records are included in milestone review material.

[severity:low][security] Running retention pruning when a loop closes does not prune that loop’s fresh plaintext captures, and no later invocation is guaranteed.
Sites: primary: `claude/skills/codex-review/SKILL.md`; confirmed: `claude/bin/dstack` `cmd_prune` and the milestone’s “pruned” claim.
Evidence: `prune` deletes only directories older than seven days, while the skill says nothing else invokes it.
Verification: A newly closed loop has age zero, so closure removes nothing; without another future loop, its bundles persist indefinitely.
Suggested direction: Explicitly remove the closed unit’s captures while retaining age-based pruning for abandoned runs.

[severity:low][technical correctness] The ultracode documentation falsely says subagents never inherit the parent session’s reasoning effort, and the subordinate task record still misclassifies interactive `claude -p`.
Sites: primary: `claude/ultracode.zsh`; confirmed: `docs/dstack-state-store/M2-pipeline-wiring/06-inject-slim/task.md`.
Evidence: Claude Code documents subagent `effort` as inheriting the session by default, though subagent frontmatter may override it. [Claude Code subagent documentation](https://code.claude.com/docs/en/sub-agents)
Verification: An unoverridden subagent inherits session effort; separately, interactive zsh expands the alias for `claude -p`, contradicting the subordinate record.
Suggested direction: Distinguish inherited reasoning effort from ultracode workflow orchestration and update every stale `claude -p` statement.

Omitted-detail: 0 low

GPT verdict: reject — Round-2’s assembly, recovery, absolute-path, and allocator repairs remain concretely broken, while stale model-facing documents still prescribe the superseded registry and blocker-downgrade behavior.

## Carried decisions
Rounds 1-2 decisions stand. Added in Round 3:

- **A recipe in a skill must be runnable as written.** Define every variable it uses, in the
  block that uses it, and never reference one across a tool-call boundary.
- **Never validate a bundle by counting headers.** Skipped files emit headers too; grep for
  `(SKIPPED` and treat any skip as disqualifying.
- **Always-loaded instruction files are part of the change surface.** `claude/CLAUDE.md` drifted
  because it was in no `files` declaration. When behaviour changes, declare the documents that
  describe it, not just the code.
- **Propagate an accepted rule to every durable record that restates it**, or the docs and the
  skill disagree about what blocks.
- Age-based pruning never covers the run that just closed — delete it explicitly.
- Subagents inherit session reasoning effort by default; ultracode is a session mode, which is a
  different claim.

Consensus: disagreed
