# Codex adversarial review — Round 004

## Review scope
Re-review (Rounds 1-3 rejected; all findings accepted and fixed)

## GPT findings
[severity:high][security] The new closure cleanup uses an unchecked prefix glob and can recursively delete matching directories outside the repository through a symlinked session directory.
Sites: primary: `claude/skills/codex-review/SKILL.md` Step 4 cleanup; confirmed: `claude/bin/dstack` `cmd_run_dir`.
Evidence: Cleanup executes raw `rm -rf .../runs/$CLAUDE_CODE_SESSION_ID/<prefix>*`, while `cmd_run_dir` validates `runs/` but follows an existing `runs/<session>` symlink through `mkdir -p` and `chmod`.
Verification: A session path symlinked to an external directory makes capture creation and the cleanup glob operate there; independently, `goal-api-r*` also matches a concurrent sibling label such as `goal-api-refactor-r004`.
Suggested direction: Validate every dynamic run component and delete enumerated exact labels through the existing `dstack` boundary.

[severity:medium][technical correctness] The Round-3 launch repair is still split across two shell fences, so Step 2 consumes `RD` and `IN` without defining them in that invocation.
Sites: primary: `claude/skills/codex-review/SKILL.md` Steps 1–2; confirmed: completion-path instructions.
Evidence: The assembly fence ends after validating `IN`; the later launch fence begins with `OUT="$RD/out.txt"` and redirects from `"$IN"`, despite the intervening warning that variables do not survive tool calls.
Verification: Running the Step-2 fence in a fresh shell resolves `OUT` toward `/out.txt` and fails to open an empty `IN`, so no valid background review starts.
Suggested direction: Put allocation, assembly, skip validation, and launch in one complete executable fence.

[severity:medium][technical correctness] The accepted one-block handoff repair was not propagated through `full-cycle`: its opening contract still says unchecked work prevents the turn from ending.
Sites: primary: `claude/skills/full-cycle/SKILL.md` opening gate description; confirmed: scheduling `waits.external`, `claude/CLAUDE.md`, and `fullcycle-gate.sh`.
Evidence: The opening says the Stop hook “blocks the turn from ending,” while later instructions require ending the turn and the hook exits successfully when `stop_hook_active` is true; official semantics confirm that blocking prevents stopping. [Claude Code Hooks Reference](https://code.claude.com/docs/en/hooks)
Verification: A model following the opening authority may continue or poll instead of ending, recreating the background-completion failure the milestone intends to remove.
Suggested direction: Replace the stale opening statement with the same one-block-per-user-turn contract used elsewhere.

[severity:medium][technical correctness] The milestone review-unit abstraction remains partial: P7–P10 are still per-task, while this round omits subordinate records that its review-unit document explicitly requires.
Sites: primary: `claude/skills/full-cycle/SKILL.md` pipeline schema; confirmed: P9/P10 conduct, `claude/CLAUDE.md`, M2 `task.md`, `GOAL.md`, and subordinate task records.
Evidence: The Goal selects milestone granularity and subordinate docs have no gates, yet the schema requires per-task task-doc gates, reviews, and deregistration; this bundle also excludes all three documents that M2 `task.md` says to read.
Verification: The current tasks cannot satisfy P7–P10 as written, and the omission hides live contradictions—`06-inject-slim/task.md` and GOAL say `claude/CLAUDE.md` was not changed while this round contains its diff.
Suggested direction: Parameterize phase ownership over the selected review unit, then update and include every referenced subordinate record in the review material.

[severity:low][technical correctness] The maintained schema check still cannot detect the Round-3 absolute-CLI regression.
Evidence: It searches for one `/.claude/bin/dstack` occurrence and independent ` migrate`/` unreg` substrings; it never associates a verb with an absolute invocation or asserts `reg`.
Verification: Adding a bare `dstack status` command leaves every assertion satisfied, reproducing the exact class the check claims to pin.

[severity:low][DX] T06’s verification record confuses bytes with characters and validates a zsh file using Bash.
Evidence: It reports 466 characters and says `bash -n` was run on both files, including `ultracode.zsh`.
Verification: The injected string is 461 Unicode characters and 465 UTF-8 bytes; 466 is `jq -r`’s byte count including its newline, while `zsh -n` is the relevant parser.

[severity:low][DX] The Round-4 review-unit document still records its design consult as `<pending>`.
Evidence: M2 `task.md` retains the placeholder, while each subordinate record says the consult was skipped.
Verification: A future handoff cannot determine from the authoritative review-unit document whether this pre-implementation phase was completed or omitted.

Omitted-detail: 0 low

GPT verdict: reject — Unsafe capture deletion and unresolved executable-contract contradictions leave concrete high- and medium-severity failure paths.

## Carried decisions — Round 004
Rounds 1-3 decisions stand. Added in Round 4:

- **A backgrounded long run does not outlive its turn — detach it.** Sentinel file plus an
  explicit VANISHED branch; never treat silence as "still running".
- **One runnable fence per procedure.** If a later fence consumes a variable an earlier one set,
  it is one procedure that was mis-split, not two steps.
- `review-unit` is a schema PARAMETER; hard-coding `task` makes the pipeline unsatisfiable for a
  milestone-granularity Goal.
- The review bundle carries every document the unit doc tells the reviewer to read; an omitted
  subordinate record hides contradictions the reviewer cannot report.
- A guard must match the SHAPE of what it guards, not a substring of it — a bundle contains the
  skill's own prose about the guard.
- A check that can be satisfied by unrelated text pins nothing; bind the assertion to the call.
- Accepted residuals unchanged: cache-read economics limit the injection saving, `gitignored` is
  not confidential, a ticked box is self-attested.

Consensus: disagreed
