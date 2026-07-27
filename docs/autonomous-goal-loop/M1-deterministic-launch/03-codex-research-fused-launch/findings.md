# Finding ledger — 03-codex-research-fused-launch

The loop closes when a round raises nothing both NEW to this ledger and CONCRETE.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| F001 | 001 | medium | a guarantee stated wider than it holds | the teardown residual promised the round dies with the supervisor, but `SIGKILL` cannot be trapped and orphans `codex exec` | fixed — guarantee scoped to catchable termination, with the recorded pid and `rm-run`'s refusal named as how an orphan is noticed |
| F002 | 001 | medium | a verification claim wider than its evidence | the skill called the fused block verified by this Goal's P3 round, which predates `dstack run` and used the launcher this task removes | fixed — claim split by what was verified when, then made true by running the exact block once under `run_in_background` (evidence in `task.md`) |
| F003 | 001 | medium | one instruction contradicting another | `-s read-only` was glossed as "never mutate the tree" two lines above `-o` writing into `docs/` | fixed — read-only blocks MODEL-initiated mutation; `-o` is named as the one deliberate repository write |
| F004 | 001 | low | resource leak | every attempt leaked its `mktemp -d` scratch directory | fixed — `trap 'rm -rf "$SCRATCH"' EXIT`, same fix applied in `codex-review` |
| F005 | 001 | low (security) | an evaluator directive inside the reviewed payload | the task document's Deployment context told the reviewer what was "Out of scope" | fixed — reworded as filing information; the reviewer correctly ignored it and reported it instead |

| F006 | 002 | medium | a claim narrowed once and still wider than its evidence | "any CATCHABLE termination" exceeds `RUN_SIGNALS` — `ABRT`, `XCPU`, `XFSZ` are catchable and untrapped | fixed in wording — the residual now names the actual trap set; widening `RUN_SIGNALS` is a follow-up on `claude/bin/dstack` |
| F007 | 002 | medium (security) | unvalidated interpolation | `<goal>`/`<topic>` reach `mkdir -p`, `--stdin` and `-o` unchecked; `TOPIC=../../AGENTS` clobbers a tracked file | fixed — plain-slug validation before the first filesystem operation; 9-case direct run recorded, including the reviewer's counterexample |
| F008 | 002 | low | a claim wider than its evidence | the E2E record said "unedited" (placeholders were substituted) and "33 cited sources" (a whole-document URL-line count; the Sources section has 13) | fixed in `task.md`, `GOAL.md` and `response-001.md`; counts now taken from the `## Sources` section |
| F009 | 002 | low | a claim wider than its evidence | `-o` called "the one deliberate repository write" while `dstack run` also writes its capture under `.dstack/` | fixed — two deliberate writers named |
| F010 | 002 | low (security) | an evaluator directive inside the reviewed payload | the round-001 repair itself ended by telling the reviewer how to read the paragraph | fixed — Deployment context now states facts only |

| F011 | 003 | medium (security) | a guard placed after the thing it guards | slug validation ran in Step 2, after Step 1 had already built a path from both values and after a substituted `$(…)` would have executed at assignment | fixed — invariant moved to Step 1, placeholders single-quoted; measured that single quotes stop `$()` in bash and zsh and the `case` then refuses the literal |
| F012 | 003 | low | a guarantee wider than the launcher | the pid is recorded just after the fork, so a kill in that window leaves a live group with no record, and the fence's trap removes `$SCRATCH` from under a surviving orphan | fixed in wording — both caveats stated; narrowing the fork window is a `dstack` follow-up |
| F013 | 003 | low | a claim wider than its evidence | the traversal counterexample used `../../AGENTS`, which reaches `docs/AGENTS.md`, not the tracked root file | fixed — `../../../AGENTS`, measured. The "no post-fix end-to-end re-run" half is answered with what was actually verified, in `response-003.md` |
| F014 | 003 | low (security) | an evaluator directive inside the reviewed payload | `task.md` told a reader where a residual should be filed | fixed in `task.md`; HELD on the skill's own process rule — see `response-003.md` for the disagreement |

| F015 | 004 | medium (security) | a boundary that is not one | a quoted assignment is escapable — `x'$(printf PWNED)'` closes the literal, runs the command, and yields the valid slug `xPWNED` | fixed — values arrive through a quoted heredoc, which expands nothing; measured refused in bash and zsh, benign slugs still accepted |
| F016 | 004 | medium | a handler that suppresses instead of terminating | `trap 'rm -rf …' EXIT INT TERM HUP` let the shell CONTINUE after a signal and return 0, running cleanup twice | fixed — each signal handler disarms EXIT, cleans once, exits with the signal's status; measured rc=143 / one CLEAN in both shells |
| F017 | 004 | low | a claim wider than its evidence | no end-to-end run of the block existed after the round-003 validation changes | fixed — the corrected block run in full, recorded in `task.md` |
| F018 | 004 | low (security) | disposition language inside the reviewed payload | the residual paragraph marked launcher defects "accepted" and assigned to another review unit | fixed — the residual states what is true of the tool; the follow-up bookkeeping lives in this ledger, not in the payload |

| F019 | 005 | medium (security) | a claim no mechanism can support | the quoted heredoc is closed by a payload line equal to its delimiter, and `SLUG` is itself a valid slug so it also broke on legitimate input | RESOLVED by withdrawing the claim — no quoting form is a boundary when the orchestrator writes the whole command; the check is stated as defence-in-depth against a mistake, with the condition under which the recipe is the wrong shape |
| F020 | 005 | medium | a fix that was wrong in two directions | both shells defer a pending trap while a foreground command runs, so a wrapper-only signal neither cancels the child NOR preserves its status — and the round-004 handler deleted `$SCRATCH` while codex was still using it | fixed — `<run-dir>/exit` is the round's status, the notification a hint; signal handlers terminate without cleaning; both measured |
| F021 | 005 | low | a printed command that does not reproduce its own measurement | the signal fence's `$$` is expanded by the invoking shell, signalling that shell instead of the bash under test | fixed — single-quoted program, signal name as an argument. Confirmed by reproducing the bug accidentally while re-measuring |
| F022 | 005 | low | a counter that counts the wrong thing | `[^ )]*` accepts a bare `https://` and double-counts a URL followed by a comma | fixed — host class required, trailing punctuation stripped; verified 22/12/5 on the real artifacts and 0 for a bare `https://` |

| F023 | 006 | low | a repair that leaks on the path it was built for | the cleanup leaks `$SCRATCH` on a trapped wrapper signal and when the session id is absent | fixed in round 007 — handlers leave the gated EXIT trap ARMED (measured, four cases, both shells) and an empty `CLAUDE_CODE_SESSION_ID` is refused before `mktemp` |
| F024 | 006 | low | a counter still counting the wrong thing | the fallback regex accepts malformed URL-shaped strings and mis-deduplicates Markdown-delimited URLs | partially fixed at 005; the remaining half is that the counter was not runnable at all — see F027 |
| F025 | 006 | low | a "verified" claim outliving what it verified | the skill still called the current block end-to-end verified after the status-gated trap was reopened | fixed in round 007 — the bullet now names what the recorded run does NOT cover and what backs the rest |
| F026 | 006 | low (security) | evaluator-disposition language in the reviewed payload | prose pre-assigning scope to another file rather than describing behaviour | fixed in round 007 — restated as a fact about where the code lives, with no disposition |

| F027 | 007 | medium | a promise the recipe does not keep | the recipe was cwd-relative while promising root-level `docs/`, so running from a subdirectory silently builds a second docs tree nothing else can find | fixed — `ROOT` resolved once via `git rev-parse --show-toplevel`; artifact path and run dir both derived from it |
| F028 | 007 | medium | stale state answering for an attempt that never ran | a reused label makes a REJECTED invocation look successful, because the previous attempt's `exit=0` and `-o` artifact are still there for Step 2a's rule to read | fixed — the run dir is refused before anything is allocated, so the capture answers only for this attempt |
| F029 | 007 | medium | a pinned command nobody can run | the source counter was published as prose with a literal ellipsis where the file argument belongs | fixed — a runnable fence; returns 22 / 12 / 7 / 5 against this Goal's four artifacts, every one nonzero |
| F030 | 007 | low | F023, F025 and F026, still open | the three round-006 lows | fixed with F023/F025/F026 |

| F031 | 008 | medium | a claim narrower than the hole | the wrapper trapped only TERM/INT/HUP while `dstack` traps eight; under zsh an untrapped USR1 exits 158 WITHOUT running the EXIT trap, leaking the scratch dir — which also falsified the "exactly two gaps" claim | fixed — the full `RUN_SIGNALS` set is trapped; measured old vs new in both shells, and what it does NOT buy (a handler cannot cancel a foreground `dstack run`) is now stated |
| F032 | 008 | medium (security) | anchoring is not confinement | `mkdir -p` and every later open follow ancestor symlinks, so `docs/<goal>` pointing outside the repo redirects both the brief and the `-o` artifact while every path still reads as repo-relative; `dstack` checks only the `--stdin` file itself | fixed — symlinked ancestors refused, then the physical directory confirmed under the physical repo `docs` before any write |
| F033 | 008 | low | a check that does not match the checker | the session id was tested only for non-emptiness while `dstack` requires `[A-Za-z0-9_-]+`, so `../cross-session` passed here and was refused after scratch had been allocated | fixed — same grammar; and the run-dir test is labelled a pre-check, since `dstack`'s `.launch` mkdir is the atomic claim |
| F034 | 008 | low | a gate that lets source-free output through | `sed '/^## Sources/,$p'` runs to end of file so an Appendix link counts; `https://-` counted as a source; `<url>` and its bare form counted twice | fixed — bounded at the next `## `, a real host required, Markdown delimiters neutralised. 22/12/7/5 unchanged on the real artifacts; the reviewer's fixtures went 4→1 and 1→0 |
| F035 | 008 | low (security) | evaluator-disposition language, fifth instance | "second and last round" and "Accepted as a stated limit" prescribe review termination and acceptance | fixed — the reopening section records what changed and what was measured; the round budget and the accepted residual live in the round file and the ledger, where dispositions belong |

## Non-blocking follow-ups (recorded, not carried into another round)

- **From F006/F012 — `RUN_SIGNALS` in `claude/bin/dstack` does not name `PROF`.** Corrected at round
  003 after measuring instead of reasoning: `dstack` runs under `/bin/bash` 3.2.57, whose EXIT trap
  DOES fire on a fatal signal, so `run_cleanup` runs for `ABRT`, `XCPU`, `XFSZ` and `VTALRM` despite
  their absence from the trap list. `SIGPROF` is the one catchable signal that reproducibly skips
  it, and `SIGKILL` is untrappable. Covering `PROF`, and narrowing the fork-to-pid-record window
  from F012, are changes to `dstack` — T01's declaration, not this unit's — and the allowlist may
  not grow to absorb a finding. Follow-up for its own review unit.
- **From F011/F012 — the same corrections apply to `claude/skills/codex-review/SKILL.md`.** Applied
  there in its own round 003, not carried across mid-round.
- **From F016 — the same trap defect is in `claude/skills/codex-review/SKILL.md`'s Step 2 fence.**
  That file sits inside an open round-4b bundle and is frozen; queued for its next round rather than
  edited mid-round. Its fence has no interpolated placeholders, so F015 does not apply there.

## Blocking count per round

| round | new concrete blocking findings |
|---|---|
| 001 | 3 (F001, F002, F003 — all concrete medium) |
| 002 | 2 (F006, F007 — both concrete medium) |
| 003 | 1 (F011 — concrete medium) |
| 004 | 2 (F015, F016 — both concrete medium) |
| 005 | 2 (F019, F020 — both concrete medium) — **round cap; 0 open at close** |
| 006 | 0 concrete blocking (4 lows) — post-seal reopening, round 1 of its reset budget |
| 007 | 3 (F027, F028, F029 — all concrete medium) + F030 = the round-006 lows | 0 open |
| 008 | 2 (F031, F032) + 3 low | 0 — **§4 cap closure** |

**Rounds 006 and 007 are the post-seal reopening**, running under §4's reset budget of 2 rounds for
a per-task unit, counted from the reopening. Round 007 is the second and last it is entitled to.
Round 006 raised no concrete blocking finding at all, which under §3 would have closed the loop —
the reopening continued to 007 because 006's lows were real and unfixed, and closing on "nothing
blocking" while four known defects sat open is the box-protecting move this Goal exists to remove.

## Closure

Sealed at round 005, the §4 round cap for a per-task unit. **Open concrete findings at close: 0.**
Raised per round: 3, 2, 1, 2, 2 — flat rather than decaying, because rounds 3, 4 and 5 each found a
defect in the previous round's fix, all three in the same two lines of shell. That pattern is why
F019 closed by withdrawing an unsupportable claim instead of attempting a fourth quoting form.
