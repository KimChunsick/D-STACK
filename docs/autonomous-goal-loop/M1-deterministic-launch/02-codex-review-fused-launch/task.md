# 02-codex-review-fused-launch

## Intent / Why

Put the review round onto `dstack run` so that launching it and being woken by it are one action.
Until now Step 2 wrote a `run.sh`, launched it **detached** via `python3 start_new_session`, and
Step 2a told the model to hand-arm a `Monitor` on a sentinel file with `persistent: true`. A
detached process is invisible to the harness, so that watcher was the only thing that could ever
resume the session — and it was a late step in a 600-line file whose default timeout (5 minutes)
was wrong for the job (15-25 minutes). Skipped or mistimed, nothing fired and the round sat unread.

## Deployment context

Instruction document read by the orchestrating model, symlinked into `~/.claude/skills/` by
`install.sh`. Its consumers are this model and, for the fenced recipes, zsh. Repo policy: no TDD,
no new test files; verification is a direct run whose output is recorded here.

Changed files: `claude/skills/codex-review/SKILL.md`.

## Design consult

Skipped — no trigger. This rewrites prose to match a contract that T01 already built and reviewed;
no new architecture, API, persistence or sanitization boundary.

## What was done (what / why)

- **Step 2 rewritten.** Assembly stays foreground (seconds, must fail loudly before anything runs);
  the launch is one Bash call with `run_in_background: true` invoking `dstack run <label> --stdin
  <bundle> -- codex exec …`. The `run.sh` heredoc, the `python3` detach, and the whole pid/sentinel
  protocol are gone — `dstack run` owns them.
- **The stale claim is corrected in place, not deleted.** The file asserted that a
  `run_in_background` command is killed at turn end. That was an honest observation once; it is
  false on client 2.1.220, and correcting it matters because it is what made detaching look
  necessary. The replacement states what was measured and names the residual (`--resume` restores
  no background task).
- **Step 2a is now "read the result"** rather than "arm a watcher and wait". The nonzero-exit rule,
  the don't-cat-the-output rule and the finding-count greps stay; the `alive()`/`kill -0` watch
  loop and the `persistent: true` warning are gone with the mechanism they guarded.
- **The carried-decisions contradiction is fixed**, and it was not cosmetic. `assemble-review.sh`
  compacts an older round only when that round file carries a `## Carried decisions` section
  matching its companion, while §2 said the round file holds findings, size and consensus *and
  nothing else* — the one shape that cannot compact. Under that contradiction T01's five rounds
  each carried every predecessor whole, inflating every bundle and making §1's size ratchet
  unsatisfiable for reasons unrelated to the change under review. The round template and §2 now
  agree: carried decisions in, maintainer response out.
- **Step 1's cross-reference corrected** — it claimed allocation, assembly, the skip check and the
  launch were one shell invocation. Assembly and the launch are now deliberately separate calls,
  and the reason the first three must stay together (a shell variable does not survive between tool
  calls, and `run-dir` allocates) is stated where it belongs.

## Files changed (where / why)

- `claude/skills/codex-review/SKILL.md` — Steps 1, 2, 2a, 3 and termination rules §2 and §4.
- `claude/skills/codex-review/assemble-review.sh` — the `REVIEW_FULL_ROUND_IDS` grammar.
  **This is a scope expansion, made deliberately during the reopening, and it is recorded here
  rather than smoothed over.** Round 007 found that `SKILL.md` publishes
  `REVIEW_FULL_ROUND_IDS="1 3"` as the way to honour a reviewer's request for an older round, while
  the assembler rejected exactly that form. Fixing only the document would mean publishing a
  corrected recipe for a command that still refuses it, so the fix had to land in both files or in
  neither. The alternative — a follow-up against the assembler's own unit — leaves the skill
  documenting something that does not work for however long that takes, on the one mechanism the
  review prompt promises the reviewer. The allowlist did not grow to accommodate this: the
  assembler was already in the bundle. Round 008 raised the inventory mismatch, which was real —
  the change had landed and this list still named one file.

## E2E verification

Repo policy: no TDD. The recipe this task installs is the one that ran every round of T01 — five
`codex exec` rounds plus a design consult, each launched as a single background `dstack run` with
no watcher armed anywhere, each waking this session by itself on completion:

```
t01-design-consult, t01-r1, t01-r2, t01-r3(killed by the harness), t01-r3b, t01-r4, t01-r5
round durations 5-11 min; harness saw zero command output throughout, then
  "DONE <label> exit=<n> dir=<path>"  →  notification  →  session resumed, no human input
t01-r1 ran 10.3 min, longer than the foreground Bash cap, so it could not have completed inline
```

The harness's own mid-run kill of `t01-r3` exercised the failure path: group torn down, `exit=143`
published, no orphan, round discarded per the nonzero-exit rule and re-run under the next label.

**This unit's own five rounds are the second body of evidence**, and they used the recipe as it
stood at each point — including one harness kill (`t02-r4a`, `exit=143`, zero bytes, retry fence run
against the capture, nothing alive, relaunched as `t02-r4b`). Every one of them woke this session on
completion with no watcher armed.

**Closing checks (P10).** The guards this task rewrote, run against hostile input rather than
described:

```
skip gate, per allowlisted path
  prose line containing an allowlisted path AND "SKIPPED:"   old REFUSE   new PASS
  --- <allowlisted path> (SKIPPED: symlink) ---              old REFUSE   new REFUSE
  path\to  /  path\new   (backslash in the path)             awk -v MISSED   ENVIRON REFUSE
  a real assembled bundle, both allowlist entries            PASS
skip gate, pathless marker
  the recipe's own quoted marker inside a diff line          substring 1 match   whole-line 0
retry fence, 7 cases
  supervisor alive / child alive / child missing / malformed → REFUSE
  both dead / terminal record present / no claim             → PERMIT
signal handling
  cleanup-only handler, self-TERM        bash+zsh rc=0   "CLEAN-SURVIVEDCLEAN"   (survives!)
  handler with disarm+exit               bash+zsh rc=143 one CLEAN
  wrapper TERM vs 5s foreground child    rc=143 only AFTER the child finished
teardown coverage (bash 3.2.57, 3 runs each, corrected probe quoting)
  TERM 143 [T]  ABRT 134 [T]  XCPU 152 [T]  XFSZ 153 [T]  VTALRM 154 [T]  PROF 155 []
all 6 fenced bash recipes in this file, placeholders substituted
  /bin/bash -n  OK      /bin/zsh -n  OK
bash tests/secret-guard.sh                              ✓ PASS
claude/skills/full-cycle/tests/*.test.sh                  PASS (both)
./install.sh --dry-run                                  = up to date: .claude/skills/codex-review
```

The change lives in the working tree of `agent/harden-codex-review-workflow`. No worktree fan-out
for this unit, so there is nothing to merge; committing is the maintainer's call.

Review loop: five rounds, sealed `Consensus: resolved` at the §4 round cap with zero open concrete
findings. `codex-review-001..005.md` with their `carried-*.md` companions, `response-001..005.md`,
and `findings.md` (F001-F026). Three follow-ups are recorded against files outside this
declaration; one of them — the `adversarial-review` contract disagreement — is a real outstanding
inconsistency and is named as such rather than closed.

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified

## REOPENED after sealing (post-seal-rule)

This unit was sealed and deregistered, then reopened by the pipeline's own `post-seal-rule`: T04's
round 004 found that the wrapper's EXIT trap removes `$SCRATCH` unconditionally, so if `dstack`
dies to something it cannot trap (`SIGKILL`, `SIGPROF`) the child survives, this shell exits
normally, and the trap deletes the directory that live `codex exec` is running in. Fixing it means
editing a file inside a sealed bundle before M1 closes, which reopens this unit's review.

Leaving a known defect in place to preserve a ticked box is exactly the failure this Goal exists to
remove, so the box came off instead. The E2E box came off with it: the fence changed, so the run
recorded above no longer attests what shipped.

### What the reopening changed (rounds 006-007)

Round 006 fixed the trap, the quoting and the §3 wording. Round 007 — the consolidated batch pass —
found five more, all recorded in `findings.md` as F032-F036 and answered in `response-007.md`:

- `RUNDIR="$RD"` ran before `RD` was defined, in a call where the assembly step's `$RD` could not
  exist at all. The trap therefore tested `[ -e "/exit" ]` and never cleaned up.
- Step 2a opened by calling any nonzero notification a failed round, contradicting Step 2.
- The published `REVIEW_FULL_ROUND_IDS="1 3"` invocation was rejected by its own validator, which
  split on commas only. Fixed in `assemble-review.sh`; the documented form now returns rc=0 and the
  bundle header confirms "rounds 1 3 by request", with every malformed value still FATAL.
- The "THIS file governs" override was withdrawn — it addressed a reviewer instructed to ignore it.
- **§4 had no transition for a post-seal reopening past the round cap.** This unit IS that case, so
  the two rounds above were running under no rule at all. The cap now counts rounds since the
  reopening with a reset, smaller budget (2 per-task, 3 per-milestone), which makes rounds 006 and
  007 accountable to something. 007 is the second, so the budget is spent.

Re-verified after those edits: all 6 bash fences in this file parse under `/bin/bash -n` and
`/bin/zsh -n`; the assembler accepts the documented resend form and still rejects `1,,3`, `1, ,3`,
`1,`, `[1]`, `1 x` and an out-of-range round; a status-gated EXIT trap left ARMED through a handled
TERM cleans when `<run-dir>/exit` exists and leaves the directory alone when it does not, rc=143 in
both shells; `bash tests/secret-guard.sh` PASS.

### Round 008 (batch pass 2) — the closing round

Four mediums and a low, all fixed or recorded; `findings.md` F037-F042, `response-008.md`.

- **The round-007 resend fix had stopped failing closed.** Accepting whitespace as a separator let
  IFS absorb an empty field, so `1, ` became a quiet request for round 1 alone and ` ` became no
  request — both silently reducing what the reviewer asked for. The whole grammar is validated
  BEFORE splitting now, and every documented form and every malformed one was re-run end to end.
- **The reset budget I wrote one round earlier could strand this very unit.** It cannot govern
  rounds that ran before it existed, and it cannot expire on a `disagreed` round, because §4 closure
  is an action the cap OBLIGES you to take rather than a permission that runs out.
- The file inventory did not name `assemble-review.sh`; it does now, with the scope expansion stated.
- F030 was marked fixed although half of it was an accepted residual; the ledger says so.
- Found while fixing: three trapped signals was not enough — under zsh an untrapped USR1 skips the
  EXIT trap and leaks the scratch dir. The wrapper traps the full `RUN_SIGNALS` set.

Sealed `Consensus: resolved` under §4 cap closure. **One follow-up is genuinely outstanding and is
not being dressed up as agreement:** `codex/skills/adversarial-review/SKILL.md` still contradicts
this file on round-file shape and consensus dispositions, raised six times now, outside this unit's
declaration.

## P10 closure evidence (batch pass 2)

Recorded at the close of the review loop, against the tree as it ships. Repo policy: no TDD, so
these are direct runs.

```
tests/secret-guard.sh                            PASS
full-cycle/tests/skill-schema.test.sh            PASS
full-cycle/tests/check-parallel.test.sh          PASS
9 fenced bash blocks across the three skills     /bin/bash -n and /bin/zsh -n, 0 failures
check-registration.sh --depth                    3
check-registration.sh --list                     6 documents
check-registration.sh (full)                     confirmed, rc=0
assemble-review.sh                               all 6 units assemble
./install.sh --dry-run                           19 entries up to date
```

**The whole-Goal behavioural evidence, which is what this Goal was actually about:** 38 run captures
under `.dstack/runs/<sid>/`, 33 with a terminal record — 31 `exit=0` and 2 `exit=143` (both harness
kills, both torn down with no orphan). Every one of those rounds was launched as ONE background Bash
call whose blocking terminal step was `dstack run`, with no watcher armed anywhere, and every one
woke this session on completion with no human input.

The 5 captures with no terminal record were exercised against the rule this Goal added for exactly
that state: none has a `.launch` claim, so none was ever launched, and `pgrep -f 'codex exec'`
returns nothing. Abandoned bundle allocations, not orphans — which is the distinction the rule
exists to force you to make instead of relaunching over a live run.

No worktree fan-out for this unit, so there is nothing to merge; the change is in the working tree
of `agent/harden-codex-review-workflow` and committing is the maintainer's call.
