# 05-standing-rules-alignment

## Intent / Why

`claude/CLAUDE.md` is the standing instruction file, read on every session in every repository.
`full-cycle/SKILL.md` is loaded only when the pipeline runs. So a rule that lives only in the skill
does not hold before the skill is invoked — and the two rules this Goal adds are both about what to
do at moments where the skill may not yet be in context: how to launch a long external run, and
whether to end a turn on a question.

The standing file already carries a paragraph about backgrounding long runs. It says "background it
and END THE TURN" without naming the mechanism, which is the same gap `waits.external` had.

## Deployment context

Standing instruction file, symlinked into `~/.claude/CLAUDE.md` by `install.sh`, loaded on every
session. Repo policy: no TDD, no new test files; verification is a direct run recorded here.

## Design consult

Skipped — no trigger. This carries two settled rules into a second file.

## What was done (what / why)

- **The background-run bullet names the mechanism.** It said "background it and END THE TURN"
  without saying what backgrounds it — the same gap `waits.external` had, and the same one that let
  each round improvise a launcher. It now names the one call, says the completion notification IS
  the wake-up, and says **never detach**, because a detached process is invisible to the harness and
  can never notify at all. That was the actual cause of the failure this Goal started from.
- **The honest limits go with it**, so they hold outside the skill too: `<run-dir>/exit` is the
  run's status rather than the wrapper's, `--resume`/`--continue` restore no background task,
  `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` removes the mechanism, and completion re-invocation is
  observed installed-client behaviour rather than a documented guarantee.
- **A new bullet states the unattended rule** — conversation through P4, then no human input, with
  the full stop list — because a rule that lives only in the skill does not hold before the skill is
  invoked, and "should I end the turn on this question?" is decided in exactly that window.

## Files changed (where / why)

- `claude/CLAUDE.md` — §0's background-run paragraph, plus the unattended rule.

## E2E verification

Repo policy: no TDD. This file is instructions, so the check is that it is installed, consistent
with the skill it summarises, and carries no secret:

```
./install.sh --dry-run                        = up to date: .claude/CLAUDE.md
diff claude/CLAUDE.md ~/.claude/CLAUDE.md      identical (symlinked)
bash tests/secret-guard.sh                     ✓ PASS
cross-check against full-cycle/SKILL.md, re-run after the round-001 fixes:
  the fused-launch call, `run_in_background`, never-detach, <run-dir>/exit as the status,
  the three background-task residuals, the unattended rule and every stop — all present in both,
  with the skill holding the full schema and this file holding the short form
  every stop matched by phrase; "concrete HIGH" matches only across a line wrap, so it was
    checked with newlines folded rather than reported as absent
  non-migratable `reg` failure           now in BOTH (was skill-only — F001)
  `reclaim` "provably orphaned" carve-out gone from the skill; the only remaining occurrence
    is the sentence recording that it was removed and why (F002)
```

**What the commands above establish.** That the file is installed and symlinked, that it carries no
secret, and that every rule in it matches the authority phrase for phrase. That is textual parity,
not behaviour: no diff shows whether the orchestrator follows a standing rule. The evidence for the
rules themselves is in T01-T04, and this Goal's own ~30 background rounds are the behavioural
evidence for the launch rule specifically. What this task adds is that the rules are loaded before
the pipeline is.

## Gate status
- [x] Verification: installed, secret-clean, and phrase-for-phrase consistent with the authority —
      confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified

### Round 002 (batch pass 2) — the closing round

One medium and three lows; `findings.md` F005-F008, `response-002.md`.

- The standing file's honest limits omitted OS reaping and the `SIGKILL`/`SIGPROF` orphan case —
  the two you need precisely when the skill is not loaded, because without the second a capture with
  no terminal record reads as a plain failure and the documented move is to relaunch over a live
  `codex exec`.
- A repair became the next instance of the class it repaired: round 001's fix wrote "behaviour
  itself is out of scope here" into the gate row, an evaluator directive inside data the reviewer is
  told to distrust. The reviewer proved it operationally — distrusting the disclaimer is what
  surfaced the missing residual.
- Two findings against `full-cycle/SKILL.md` were deferred under the freeze-rule while that bundle
  was open, and completed in this session once it closed.

Sealed `Consensus: resolved`.

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
