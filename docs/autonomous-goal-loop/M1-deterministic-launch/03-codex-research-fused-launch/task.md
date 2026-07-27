# 03-codex-research-fused-launch

## Intent / Why

Give the research round the same one-call launch the review round now has. This step had **no
backgrounding recipe at all** — it showed a bare foreground `codex exec`, which either blocks the
session until the Bash tool's 10-minute cap kills it, or gets hand-wrapped into a launcher written
differently every time. This Goal's own P3 round is the example: the recipe as written could not be
used, so a `run.sh` plus a `python3 start_new_session` detach plus a hand-armed `Monitor` were
improvised on the spot — and a detached round is invisible to the harness, so that watcher was the
only thing that could resume the session.

## Deployment context

Instruction document read by the orchestrating model, symlinked into `~/.claude/skills/` by
`install.sh`. Its consumers are that model and, for the fenced recipe, zsh. Repo policy: no TDD, no
new test files; verification is a direct run whose output is recorded here. The change edited
`claude/skills/codex-research/SKILL.md`; the `adversarial-research` Codex skill is a separate file.

## Design consult

Skipped — no trigger. This is prose brought onto a contract T01 already built and reviewed.

## What was done (what / why)

- **Step 2 rewritten onto `dstack run` under `run_in_background`,** with the reason stated: a
  research round takes 3-25 minutes and must outlive its turn, and the completion notification is
  what resumes the session.
- **`--stdin` is called out as non-optional.** `dstack run` gives the launched command `/dev/null`
  by default, and a research round fed no brief still produces confident, generic output — which is
  worse than a failure, because it reads like a result. The old recipe used a plain `<` redirect,
  which does not survive the move.
- **The nonzero-exit rule is stated here too.** It existed in `codex-review` but not here, so a
  research round that died partway could have had its half-written artifact summarised into
  `GOAL.md`. Discard and re-run under the next label.
- **`-o` is left doing the artifact write**, so the artifact still lands under `docs/` while the
  capture keeps the transcript — the two were never the same thing.
- **The "long runs are fine" bullet now says why** that implies the background path, and the
  "verified runnable" bullet names WHICH invocation was verified and when — the Goal's P3 round
  predates `dstack run` and verifies nothing about the wrapper, which round 001 caught me claiming.
- **Placeholder handling, after three rounds of getting it wrong.** `<goal>`/`<topic>` are validated
  against a plain-slug grammar before any filesystem operation, and Step 1 owns the invariant since
  it builds a path from both. The recipe now states plainly that no quoting form makes textual
  substitution a boundary, what the check therefore IS (defence in depth against a `..`), and the
  condition under which the recipe would be the wrong shape.
- **Signal handling.** The EXIT trap removes the scratch directory on normal completion only. A
  signal handler terminates with the signal's status and does NOT clean, because both shells defer
  a pending trap while a foreground command runs — so cleaning there deletes the cwd of a `codex
  exec` that is still alive.
- **`<run-dir>/exit` is named as the round's status**, with the notification as a hint, for the same
  reason: a signalled wrapper can report 143 for a round that completed.
- **The source count is pinned** in the Fallback section, since two different wrong patterns
  produced a 33 and a 0 on artifacts holding 13 and 5 sources.

## Files changed (where / why)

- `claude/skills/codex-research/SKILL.md` — Step 1's slug invariant, the whole Step 2 fence
  (validation, traps, launch), the status/notification rule below it, the residual paragraph,
  three explanatory bullets, and the Fallback section's source count.

## E2E verification

Repo policy: no TDD. The evidence is the recipe itself, run. It was run three times, because the
fence changed materially in rounds 003, 004 and 005 — placeholder handling three times, traps
twice — and each earlier capture attests a block that no longer exists. They are listed newest
first, with what each one actually proves.

**1. `autonomous-goal-loop-final` — the form that shipped.** Run after round 005, exactly as
written.

```
DONE autonomous-goal-loop-final exit=0
capture:  exit=0, out.txt 3756 bytes
artifact: docs/autonomous-goal-loop/research/recipe-final.md
          3755 bytes, all six required sections present
          ## Sources: 7 unique URLs, counted with the command the Fallback section documents
scratch:  the `-C` directory recorded in `cmd` is gone — removed by the EXIT trap on normal
          completion, which is now the only path that removes it
```

Exercised: the single-quoted assignments, the slug validation loop, `--stdin` carrying the brief,
`-o` writing the artifact, and EXIT-only cleanup. NOT exercised here: the signal handlers, which
were measured separately in `response-005.md` — provoking them in a real round means killing a paid
one.

**2. `autonomous-goal-loop-selfcheck` — superseded, and worth keeping for why.** It ran the round-004
form: values through a `<<'SLUG'` heredoc, and signal handlers that cleaned up before exiting. Both
were replaced at round 005 (the heredoc is closable by a payload line equal to its delimiter, and
the cleaning handler deletes a live child's cwd). Exit 0, 2760-byte artifact, six sections, 5 unique
URLs. It attests that the *shape* runs; it does not attest the current fence.

**3. `autonomous-goal-loop-lifetime` — the first end-to-end run,** demanded by round 001 after I
claimed the recipe was verified by this Goal's P3 round, which predates `dstack run` entirely. Exit
0, 10524-byte artifact, six sections, 13 source entries (12 unique URLs plus one local
installed-CLI artifact), 345,874 tokens. Its brief was independently useful — it is the Goal's
follow-up research on background-task lifetime.

**What a capture proves, and what it does not.** `cmd` records the launched child and its flags. It
does not record the wrapper's own `set -u`, its traps, or that the Bash call was backgrounded;
those were observed at run time and are stated as observation. And since round 005, the wrapper's
exit status is not the round's status either — `<run-dir>/exit` is, because a signal to the wrapper
while `dstack run` is in the foreground lets the child finish and then reports 143.

**No watcher was armed and no human typed** in any of the three. The background call's completion
notification is what resumed the session each time, which is the property this task exists to
install. One of them survived an auto-compaction of the session while in flight.

**Two counting errors, recorded because both nearly cost a round.** `grep -c 'https\?://'` over the
whole document counts inline citations and reported 33 for a 13-source artifact. `grep -c '^- '`
counts bullets and reported 0 for an artifact that numbers its sources `[S1]…[S5]` — and zero
sources is a documented fallback trigger, so that one would have thrown away a good round. The
Fallback section now pins the command.

**Residual carried out of this unit.** The lifetime artifact reports several documented ways a
background task can end early — `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1`, a memory-pressure reap
after 30 minutes idle, the headless `-p` exit grace — and that `--resume`/`--continue` restore no
background task. Completion re-invocation is installed-client behaviour (2.1.220), not a documented
public guarantee. Recorded in `GOAL.md`'s research summary and written into
`claude/skills/full-cycle/SKILL.md` under `scheduling.waits.external-residuals` by T04.

**Closing checks (P10).** The shipped fence extracted from the file and syntax-checked as a whole,
since a recipe an orchestrator pastes must parse in the shell that runs it:

```
/bin/bash -n   syntax OK          /bin/zsh -n   syntax OK
bash tests/secret-guard.sh        ✓ PASS
./install.sh --dry-run            = up to date: .claude/skills/codex-research
```

The change lives in the working tree of `agent/harden-codex-review-workflow`. There is no worktree
fan-out for this unit, so there is nothing to merge; committing is the maintainer's call and is not
done here.

Review loop: five rounds, sealed `Consensus: resolved` at the §4 round cap with zero open concrete
findings. `codex-review-001..005.md` with their `carried-*.md` companions, `response-001..005.md`,
and `findings.md` (F001-F022).

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

Round 006 raised four lows and no concrete blocker. Under §3 that closes the loop, and closing there
would have sealed with four known defects open — the box-protecting move this Goal exists to remove
— so the reopening continued to 007, the second and last round its reset budget allows. Recorded as
F023-F030 with `response-007.md`:

- **The recipe was cwd-relative** while promising root-level `docs/`. From a subdirectory it built a
  second docs tree that the gate, the assembler and the next round all fail to find. `ROOT` is now
  resolved once and both paths derive from it.
- **A reused label made a REJECTED invocation look successful.** `dstack run` refuses one, but
  nothing launched, so the previous attempt's `exit=0` and `-o` artifact answered for it. The run
  dir is now refused before anything is allocated, which is what makes "read `<run-dir>/exit`" sound.
- **An empty `CLAUDE_CODE_SESSION_ID`** built `runs//<label>`, a path `exit` is never published
  into, so the cleanup gate could never fire — measured, bash 127 and zsh 1, both after `mktemp`,
  neither cleaning up. Checked before allocation now.
- **The signal handlers no longer disarm the EXIT trap.** The disarm was carried over from the
  unconditional-cleanup era; since the trap is gated on `<run-dir>/exit`, and the deferral means the
  handler usually runs after that file exists, disarming guaranteed a leak on the one path where
  removal is right. Measured in both shells: present → rc=143 and removed, absent → rc=143 and kept.
- **The pinned source counter is runnable**, in a fence with a real path. Against this Goal's four
  artifacts it returns 22, 12, 7 and 5.
- The "verified runnable" bullet now names what the recorded `codex exec` run does NOT cover, since
  every construct above landed after it.

Accepted as a stated limit rather than fixed: the URL grammar counts some malformed URL-shaped
strings. The counter answers one question — did the artifact cite anything at all — and a stricter
grammar fails it in the costlier direction, by reading a real citation as zero and re-running
research that already worked.

### Round 008 (batch pass 2) — the closing round

Two mediums and three lows, all fixed; `findings.md` F031-F035, `response-008.md`.

- **The wrapper trapped three signals where `dstack` traps eight**, which also falsified the
  "exactly two gaps" claim this file had carried since round 003. Measured: under zsh a wrapper-only
  USR1 exits 158 WITHOUT running the EXIT trap and leaks the scratch dir; bash cleans either way.
  Full set trapped, and the limit stated — it does not keep the run attached, because no handler can
  cancel a foreground `dstack run`.
- **Root anchoring is not write confinement.** With `docs/<goal>` a symlink, `mkdir -p` and every
  later open follow it and both the brief and the artifact land outside the repository. Symlinked
  ancestors refused; the physical directory confirmed under the physical repo `docs` before any
  write.
- The session id is checked against `dstack`'s own `[A-Za-z0-9_-]+`, not merely for non-emptiness.
- The zero-source gate is bounded at the next `## `, requires a real host, and neutralises Markdown
  delimiters — 22/12/7/5 unchanged on the real artifacts, the reviewer's fixtures 4→1 and 1→0.
- Evaluator-disposition language removed for the fifth time; dispositions live in the round file and
  the ledger.

Sealed `Consensus: resolved` under §4 cap closure.

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
