# 01-dstack-run-subcommand

## Intent / Why

One command that launches a long external run AND blocks until it finishes, so a single
harness-tracked background call is the whole mechanism. Its completion notification becomes the
resume signal, and the separate "arm a watcher" step — the one that kept being skipped or armed
with its 5-minute default against a 25-minute job — disappears from the contract.

```
"$HOME/.claude/bin/dstack" run <label> [--stdin <file>] -- <cmd> [args...]
```

## Deployment context

Single maintainer, one macOS machine, one checkout at a time (occasionally two tabs on different
Goals). `dstack` is invoked by absolute path from skill recipes and from Claude Code's Bash tool;
nothing puts `~/.claude/bin` on `PATH`. The command it launches is `codex exec` — network-bound,
3–25 minutes, no repository writes (`-s read-only`). `.dstack/runs/` is gitignored and mode 700; it
holds plaintext code diffs, so it is damage-limited, not confidential. Repo policy is no TDD and no
new test files: verification is a direct run whose real output is recorded below.

Out of scope by construction: multi-machine coordination; surviving `--resume`/`--continue`
(Claude Code documents that background tasks are never restored, so automatic pickup is lost on a
session crash); commands assembled from untrusted input — every caller is a recipe in this repo.

## Design consult

Ran GPT-5.6 Sol at xhigh before any code was written, because this adds a CLI contract and gives a
state-store CLI a process-execution boundary. Verdict **proceed-with-changes**. Capture:
`.dstack/runs/<sid>/t01-design-consult/`; full dispositions as D001–D010 in `findings.md`.

Its most valuable point was not on the fix list: *without a recovery path, direct foreground
ownership inside the harness-tracked background command is simpler and fully addresses ordinary
end-of-turn survival.* The design under review both detached the child and blocked on a
pid/sentinel protocol. Detaching buys nothing here — a detached process is invisible to the
harness, and Claude Code does not restore background tasks on `--resume` — while costing a protocol
whose races the consult took apart. Dropping it made three of the five high findings unreachable
rather than patched. The rest were fixed as advised.

## What was done (what / why)

`cmd_run` validates the label, then `--stdin`, then requires a literal `--` (so a mistyped flag
cannot be absorbed into the command line). It allocates the capture directory or **adopts** one
`run-dir` already made for this attempt — which is what lets the review recipe assemble its bundle
in before launch. It claims the launch atomically with `mkdir "$d/.launch"`, runs the command as a
direct child with argv (never shell source), captures `out.txt`/`err.txt`, publishes the status to
`exit`, prints one `DONE <label> exit=<n> dir=<path>`, and exits 0 or 6.
`require_label` is the label grammar extracted from `cmd_run_dir` unchanged, because `run` must
validate before building a path from the label and a second hand-written copy of those rules is the
drift this file has paid for elsewhere.

Rounds 001–003 rewrote the process-lifetime half of this command. Findings in
`codex-review-00N.md`, responses in `response-00N.md`, ledger in `findings.md`. All four blocking
findings turned out to share one root cause — **process lifetime was reasoned about rather than
measured** — and each repair had been "verified" against a child that cooperated, so each passed
while the uncooperative case stayed broken:

- **F001** — no teardown at all. This document and the code both asserted a killed supervisor takes
  its child with it. It does not. The child now runs under `set -m` in its own process group.
- **F006** — the teardown did not span what it protected: trap up *after* the fork, down *before*
  publication, and a failed pid write called `die` while the child ran.
- **F007** — it signalled the wrong entity. The group *leader*'s death is not the group's: a
  TERM-resistant descendant survived while the capture was published terminal and `rm-run` would
  have deleted it. `run_group_gone` (`kill -0 -<pgid>`) and `run_group_settle` now gate publication
  on group quiescence with bounded TERM→KILL escalation, on the normal path as well as the abort
  path, and `rm-run` refuses while the pid *or* the group has a member.
- **F008** — ownership started one statement too late and covered too few signals. One
  `run_cleanup` owner is now armed on `EXIT` plus `INT TERM HUP QUIT PIPE ALRM USR1 USR2`, one
  statement after the claim succeeds, disarmed only after publication. Because EXIT is covered,
  every `die` past the claim leaves through it, which let a second release path be deleted rather
  than maintained.
- **F009** — the post-wait liveness loop rested on a wrong belief (bash caches a reaped status, so
  a second `wait` never returns 127) and was unnecessary, since the handler exits. Removed.
- **F011** — quiescence was a *warning*, not a gate: `run_group_settle || printf WARNING` then
  published anyway, which made the capture terminal and therefore deletable while a group that
  outlived SIGKILL was still writing into it. Publication now happens only inside the
  `run_group_gone` branch at both call sites; otherwise a loud ERROR and no terminal record, so
  `rm-run` keeps refusing. `run_done` is the single finalisation state, so the EXIT handler cannot
  re-enter and undo that refusal.
- **F010 was rebutted, and improved the code anyway.** It claimed bash unwinds `cmd_run`'s locals
  before the EXIT trap so `run_cleanup` dies on `set -u`. On the deployed /bin/bash 3.2.57 — both
  the shebang target and the shell on PATH here — it does not, verified from a nested `die` and by
  fault-injecting the exact post-fork failure into the real script (group torn down, exit 143
  published, no stray). Its suggestion was taken regardless: every read in `run_cleanup` is
  defaulted, and that change surfaced a real bug — an unguarded `rm -rf "${d-}/.launch"` would
  resolve to `/.launch` if `d` were ever unset, so the claim release is now guarded on `$d`.
- **F012 (low)** — a pgid carries no ownership token, so settlement could signal an unrelated group
  if the id were recycled between probe and signal. Recorded as an accepted residual in the code;
  the window needs the group fully gone first, so it is the probe-to-signal instant only.
- **F013** — F011's fix set `run_done=1` *before* settlement, so a signal arriving during that
  15-second escalation window found the run already "done", skipped teardown, and exited with
  descendants alive. Reentrancy protection (the handler disarming its own traps on entry) and
  finalisation (`run_done`) are now separate; `run_done` is set only after publication or an
  explicit refusal to publish.
- **F014 was disproved**, restating F010 with a sharper claim. `run_cleanup` was instrumented in
  the real script: both `die` paths print populated `d` and `label`, the pre-fork path releases the
  claim, the post-fork path publishes exit 143 with no stray. Two instrumented runs now contradict
  the mechanism, so nothing is built on it. **F015 (low)** — a failed publish printed both "could
  not publish" and "recorded exit"; the success line now runs only when publication succeeded.

**Closed at the round cap.** Five rounds, each finding at least one genuine reproducible defect,
all in one subsystem. Nothing concrete is open at closure. The residual worth naming: F013's fix is
verified by direct run but has not itself been through an adversarial round, and the loop stopped
on its cap rather than on exhaustion. What bounds that: every finding has been in one place, the
behaviour is now measured rather than reasoned about, and the code fails closed wherever it cannot
prove the launched work is gone — no terminal record without confirmed group quiescence, no
deletion of a capture with a live pid or group, no claim released once anything has been launched.

Also: `$!` is the tiebreaker for the instruction-level window between `&` and `child=$!`, which
separates "nothing was launched" (release the claim) from "a child exists, unrecorded" (never
release). Every failure before the fork releases the claim; every path after it keeps it — that
asymmetry is what lets `rm-run` refuse on unknown state without stranding routine failures.
**F002**: `require_label` rejects `''` first; it had been putting a claim and capture files in the
session root. **F003**: every reserved name must be *absent*, not merely non-symlink.

**Accepted residual:** `SIGKILL` cannot be trapped, so it can still leave the launched group
running with no terminal record. Its capture is then protected from deletion and the refusal names
what to stop. If session-crash recovery ever matters, the option is a `resume <label>` path, not
re-detaching.

## Files changed (where / why)

- `claude/bin/dstack` — the `run` subcommand and `run_abort`, arity and dispatch entries, the
  extracted `require_label`, the `rm-run` active-capture guard, and the usage text (exit code 6).
- `AGENTS.md` — documents `run` where the `.dstack` store is described: that it deliberately does
  not detach, and what exit 6 means.

## E2E verification

Repo policy: no TDD. Everything below is real output from a direct run.

**The harness assumption the design rests on** — a background command survives the turn boundary
and its exit re-invokes the session with no human input:

| probe | mechanism | survived turn end | woke the session |
|---|---|---|---|
| B | `run_in_background` bash | yes, t+75s | yes, exit notification |
| M | persistent `Monitor` | yes | yes, event notification |
| C | detached `start_new_session` | yes, t+120s | **no** — invisible to the harness |
| L | `run_in_background`, 25 min | yes, 25 markers over four turn boundaries | yes, exit notification |

**The end-to-end use is review round 001 itself** — one `dstack run` under one background call,
**no watcher armed anywhere**:

```
command started 23:59:10 → exit published 00:09:26      = 10.3 min
out.txt 4185 B, err.txt 179599 B   — inside the capture, invisible to the harness
harness saw     : DONE t01-r1 exit=0 dir=…   (the command's entire output)
notification    : "completed (exit code 0)" → re-invoked this session, no human input
```

Long *and* output-silent, the combination round 001 said had never been exercised. The foreground
Bash tool caps at a 10-minute timeout, so this round could not have completed as a foreground call
at all; probe L bounds the duration question at 25 minutes.

**Teardown completeness (F007)** — the test that matters, because a `sleep`-based probe only ever
demonstrates teardown for children that cooperate. Leader exits 7 immediately, leaving a
TERM-IGNORING busy loop in its process group:

```
start 00:49:19  →  DONE f7 exit=7, dstack status 6  →  end 00:49:30
   11 seconds = 5s quiesce, TERM ignored, 5s, KILL
exit file [7]                  ← the leader's real status, not the signal's
stray TERM-ignoring loops: 0
```

**Publication gate (F011)** — `run_group_gone` fault-injected to report the group alive forever:

```
dstack: the command exited 0 but its process group 97561 survived SIGKILL — refusing to publish a
        terminal record while something may still be writing into …/q1
exit file present: no          ← capture stays nonterminal, so rm-run keeps guarding it
```

**Signal during settlement (F013)** — TERM delivered while the supervisor is inside the escalation
window, which is the interval the round-004 fix had left unguarded:

```
leader 7713 exited -> supervisor is now inside settlement
supervisor status=6
TERM-ignoring descendant still alive: 0   (want 0)
exit file=[7]
```

**Instrumented `run_cleanup` (F010/F014 rebuttal)** — the real handler, printing its own state:

```
pre-fork die : [PROBE] d=[…/k-pre]  label=[k-pre]  child=[]      run_done=[0]
               → claim released, .launch gone
post-fork die: [PROBE] d=[…/k-post] label=[k-post] child=[7506]  run_done=[0]
               → group torn down, exit=[143], stray 0
```

**Interpreter.** Both `/bin/bash` and the `bash` on PATH here are GNU bash
3.2.57(1)-release — the shebang target, so evidence and production run under one shell. An EXIT
trap fired by `exit` from inside a function reads that function's locals, including from a nested
`die`; and the post-fork child-record failure, injected into the real script, tore the group down
and published exit 143 with no stray process.

**Signal coverage and claim ownership (F006, F008).**

```
USR1 (untrapped before round 003) → child gone, exit [143], stray 0
TERM straddling the fork, 20 samples stepped across the post-claim interval
                                  → stray 0, published 20, STUCK 0
post-claim die (adopted dir holds `exit`)
                                  → "claim released, label is free to retry"; .launch gone
```

**Unplanned, and better than any of the tests:** the harness itself stopped review round 003
mid-run — a real `codex exec` under a real `dstack run`, signalled by the real caller. Cleanup tore
the group down, published `exit=143`, and said so on stderr. Launched pid gone,
`pgrep -f 'codex exec'` → 0, capture intact with its terminal record. That round was discarded per
the nonzero-exit rule and re-run under the next label.

```
normal completion              → DONE n-ok exit=0, dstack status 0
failing child (exit 7)         → dstack status 6, exit file [7]  (never 7 — no collision with
                                  dstack's own 1/3/4)
TERM after the pid record      → child alive after: no,  exit file [143]
SIGKILL the supervisor         → orphan survives (untrappable); rm-run REFUSES its capture,
                                  naming the pid; after stopping it, removal succeeds
pre-fork failure               → refusal, and NO .launch left behind (label reusable)
rm-run vs a live claim         → refuses, naming the process
relaunch of a used label       → refused (claim present)
empty label                    → refused                                              (F002)
adopted dir holding `exit`     → refused, whether `exit` is a file or a directory      (F003)
--stdin symlink / missing file → refused, before any directory was claimed
adopt + assembled bundle       → child reads it; the bundle survives the run
cmd record, newline-bearing arg→ stored `%q`-quoted, so the record stays unambiguous
usage errors                   → no args / label only / `--` with nothing after / missing `--` /
                                  `bad/label` / `.hidden`, each refused before anything allocated
```

`bash -n claude/bin/dstack` → OK and `bash tests/secret-guard.sh` → ✓ PASS after every change. All
test captures removed; `dstack status` shows only this Goal's real captures.

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus — closed at the round cap (5 rounds);
      `codex-review-005.md` sealed `resolved` with F013 fixed, F014 disproved, F015 fixed, and the
      residual named above and in `findings.md`
- [x] E2E capture verified — five real `codex exec` rounds, each launched as one background
      `dstack run` with no watcher armed, each resuming this session by itself on completion
