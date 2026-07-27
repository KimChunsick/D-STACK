# 06-registration-check-script

## Intent / Why

P6's registration proof was ~30 lines of bash inside `full-cycle/SKILL.md`. It took five adversarial
review rounds and **every repair introduced the next defect**:

1. `|| echo "WARN: UNGATED"` — a failure that let the pipeline continue, survivable only while a
   human read the transcript.
2. `set -e` added above a REFERENCE LIST of subcommands, so the success path ran `unreg` and
   deregistered the document it had just registered.
3. A hand-listed `DOCS` array — simultaneously the assertion and its own proof. `DOCS=(GOAL u1 u1)`
   printed "3 documents" with a required unit missing.
4. A `find` derivation that compared how MANY units existed, never WHICH; `GRAN` hand-set so a
   milestone-granularity Goal went unnoticed; `find | sort` masking a failing `find`.
5. A `while` loop whose success path left status 1, so the trailing `|| exit 1` aborted the whole
   fence silently.

Asked whether to keep patching prose or move the check into code, the maintainer chose code — which
is also what the standing rules say: if code can answer, code answers. A deterministic transform
does not belong in prose the model must re-execute correctly every round.

## Deployment context

A bash script under `claude/skills/full-cycle/`, invoked by the pipeline at P6. Symlinked into
`~/.claude/skills/` with the rest of the skill by `install.sh`. Its consumers are the orchestrating
model (which runs it) and the maintainer (who reads its output). Repo policy: no TDD, no new test
files; verification is a direct run whose output is recorded here.

## Design consult

Skipped — no trigger. The behaviour is fully specified by five rounds of findings against the fence
it replaces; there is no new architecture, API, persistence or sanitization boundary.

## What was done (what / why)

- **`check-registration.sh`**, taking one argument (the Goal directory) and proving IDENTITIES
  rather than counts:
  - **granularity comes from `GOAL.md`'s own `Review granularity:` line**, never from a flag. The
    fence had it hand-set, so a milestone-granularity Goal checked at task depth and passed. Exactly
    one unfenced line is required; naming both or neither is exit 2.
  - **the declared set comes from the `## Milestones & tasks` section**, parsed **exactly** as
    `check-parallel.sh` parses it — fences tracked GLOBALLY from line one, task rows accepted only
    at column zero with the `-` marker, a repeated section heading keeping the section open. Round
    001 showed why "same idea, different code" is not enough: with fences tracked only from the
    section, a fenced example ANYWHERE above it inverts the file.
  - **identities are compared both ways, at both granularities** — `T<NN>` rows against
    `<NN>-<slug>/task.md`, `### M<n>` headings against `M<n>-<slug>/task.md` — so "declared but not
    scaffolded" and "scaffolded but not declared" are separate, named failures. Ids are zero-padded
    and sorted lexically under `LC_ALL=C`, because `sort -n` inputs make `comm` report present
    entries as missing.
  - **nothing is dropped and nothing is collapsed.** A folder with no readable id is reported, not
    discarded. Duplicate ids are reported per id with the colliding paths, not deduped — `uniq` hid
    a duplicate AND the missing unit it was masking.
  - **registration is checked with ownership**, matching the whole `status` line including
    `(this session)`, and a foreign-owned record is its own message. It is worse than an absent one:
    it looks registered and the Stop hook skips it, so the fix is different.
  - **a CLOSED unit must NOT be registered**, and unreadable or gate-less is neither open nor
    closed but an error. A unit whose `## Gate status` still holds an unchecked box is active and
    must be registered; one with every box ticked is done, and leaving it registered holds the Goal
    gate open forever. The gate section is read with `fullcycle-gate.sh`'s own heading and checkbox
    rules, so the two cannot disagree about which boxes count.
  - **nothing may be registered at the other depth** for the declared granularity. Scope stated in
    the file: one alternate depth, files named `task.md` only.
  - **exit 2 is separate from exit 1** — a check that could not run is never mistaken for a check
    that passed — and that separation is backed by checking every `find`, `sort`, `comm` and
    extraction status, plus a count-in/count-out guard so a silently shrunken id set cannot produce
    empty deltas and a false pass.
  - **`--depth` prints the review-unit depth and exits**, so P6's registration loop reads the level
    from the same GOAL.md parse instead of hard-coding a task-depth glob.
- **`full-cycle/SKILL.md`'s fence shrank to a few lines** that read the depth, register `GOAL.md`
  and every unit at that depth, then invoke the script, with the five-round history recorded as the
  reason.

## Files changed (where / why)

- `claude/skills/full-cycle/check-registration.sh` — new.
- `claude/skills/full-cycle/SKILL.md` — the P6 fence, replaced by the invocation. (Declared under
  T04, which owns that file; this task owns the script.)

## E2E verification

Repo policy: no TDD. Everything below is a direct run. The fixture is a **real throwaway git
repository** in the scratch area with a real `docs/` tree and real `dstack` records — not a stub —
so ownership is exercised through the actual registry, with `CLAUDE_CODE_SESSION_ID` varied to
create a foreign owner. It was removed afterwards.

**Round 001 rebuilt this battery**, because the version recorded here previously tested a script
with four highs in it. Each case below names the behaviour it pins:

```
1  fenced decomposition example in an EARLIER section    confirmed: task granularity, 3 units
     same file through the OLD parser                    granularity "per milestone" (from the
                                                         fence), 2 task rows — both FAKE, and all
                                                         3 real rows discarded
2  dup id 03 scaffolded twice, 02 absent                 BLOCKED: two or more units share id 003
                                                         BLOCKED: declared but not scaffolded: 002
                                                         (old: uniq collapsed both -> PASS)
3  milestone gran: M1,M2 declared / M1,M3 scaffolded     BLOCKED both ways (old: 2==2 -> PASS)
4  T02 + T10 declared and scaffolded                     confirmed  (old sort -n broke comm)
5  unreadable unit doc + a doc with no gate section      BLOCKED, one message each
                                                         (old: both fell through to "closed")
6  01 owned by ANOTHER session, 02 unregistered          BLOCKED with DIFFERENT messages
                                                         (old: identical "not registered to THIS")
7  unnumbered folder + a ticked unit left registered     BLOCKED: no readable id (old: dropped)
                                                         BLOCKED: ticked but still registered
8  milestone gran, a task-depth doc registered           BLOCKED: not a review unit at this gran

the comm collation, measured directly:
  declared {2,10,20} vs scaffolded {10}
    sort -n  + comm -23  ->  2 10 20      (reports a PRESENT id as missing)
    padded+C + comm -23  ->  002 020      (correct)

exit 2 — could not run — is separate from exit 1 throughout:
  no granularity line 2 | two granularity lines 2 | names both 2 | names neither 2
  milestone gran with no ### M rows 2 | no GOAL.md 2 | no argument 2 | two arguments 2

--depth, the mode P6's fence reads:
  --depth on this Goal -> 3, rc=0        --depth with no dir -> usage, rc=2

live, against this Goal:
  P6 registration confirmed: task granularity, 6 units + GOAL.md, all owned by this session
  and before T05/T06 were scaffolded it correctly said: declared but not scaffolded: T05 T06
  and it did NOT demand registration for T01, which is closed and deregistered
```

The live run is the strongest single piece of evidence here: pointed at a real Goal mid-flight it
reported the true state twice, without being told what that state was. Case 1's second line is the
second strongest — the old parser did not merely miss the real rows, it substituted the fenced
example's granularity and task ids for them and reported that with no sign anything was wrong.

```
/bin/bash -n check-registration.sh    syntax OK
P6 fence, placeholders substituted    /bin/bash -n OK   /bin/zsh -n OK
bash tests/secret-guard.sh            ✓ PASS
claude/skills/full-cycle/tests/*.test.sh   PASS (both)
./install.sh --dry-run                = up to date: .claude/skills/full-cycle
```

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified

### Round 002 (batch pass 2) — the closing round

Four highs, two mediums, two lows; all fixed and re-verified against a fixture repository.
`findings.md` F008-F015, `response-002.md`.

- **Fence tracking now matches fence length and character.** A ```` block containing ``` lines broke
  the naive toggle so badly it read neither the fenced fake row nor the real one. Residual named
  rather than hidden: `check-parallel.sh` still has the naive toggle, so the two can disagree on
  such a file — in the fail-closed direction, because this checker blocks on the mismatch. Follow-up
  recorded for that file's own unit.
- `Review granularity: not task` selected task mode. Only the documented values are accepted.
- **Producer failures could be erased before comparison** — a pipeline reports only its last
  command's status and a process substitution's is unobservable at all (measured, rc 0 after
  `exit 7`). An erased producer yields empty deltas, which read as "no differences". Every stage is
  materialised and checked on its own.
- Ownership is classified in every branch, not just the open-unit one.
- "Nothing else is registered" is enumerated FROM THE REGISTRY. Round 001 answered this by narrowing
  the claim; the reviewer was right that a real check beats an accurate disclaimer.
- Milestone identity requires a heading boundary and a real `M<n>-<slug>` folder; the Goal directory
  is canonicalised; the success line stopped claiming more than it checked.

Sealed `Consensus: resolved` with the batch authorisation spent.

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
