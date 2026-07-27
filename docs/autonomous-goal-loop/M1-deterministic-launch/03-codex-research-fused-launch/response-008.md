# Maintainer response — Round 008 (batch pass 2, §4 cap closure)

Not bundled. Two mediums and three lows, all fixed. This is the closure round.

## F031 [medium] the wrapper traps three signals; `dstack` traps eight — AGREED, fixed

The finding also falsifies a claim this file had been making since round 003: that the only gaps are
`SIGKILL` and `SIGPROF`. Measured, and the measurement is the point — under zsh a wrapper-only USR1
exits 158 WITHOUT running the EXIT trap, so the scratch directory leaks; under bash it cleans either
way, because bash 3.2's EXIT trap fires on fatal signals. The old three-signal set WAS the gap, on
the shell this pipeline actually launches from.

Fixed with a loop over the same set `dstack` traps. And the limit is now stated instead of implied:
this does not keep the run attached. A handler cannot cancel a foreground `dstack run` — that was
measured two rounds ago — so `codex exec` survives regardless. What covers that is `dstack`'s own
teardown plus the standing rule that a capture with no terminal record must be checked for a live
group before relaunching.

## F032 [medium, security] root anchoring is not write confinement — AGREED, fixed

Round 007 fixed the wrong-tree bug and I described it as fixing where writes land. It does not:
`mkdir -p` and every subsequent open follow ancestor symlinks, so `docs/<goal>` pointing at
/tmp/target sends both the brief and the `-o` artifact outside the repository while every path in
the recipe still reads as repo-relative. `dstack` does not catch it either — it checks only whether
the `--stdin` file itself is a symlink, not its ancestors.

Symlinked ancestors are refused before `mkdir`, and the physical directory is confirmed to be under
the physical repository `docs` before any write.

## F033 [low] the session-id check does not match the checker — AGREED, fixed

Testing for non-emptiness while `dstack` requires `[A-Za-z0-9_-]+` is a check that passes exactly
the inputs that will fail later: `../cross-session` satisfied the recipe and `dstack run` refused
the launch — after scratch had been allocated with no terminal record to authorise cleaning it. Same
grammar now. The run-dir test is labelled a pre-check, since `dstack`'s `.launch` mkdir is the
atomic claim and this only turns the common case into a clear refusal.

## F034 [low] the zero-source gate lets source-free output through — AGREED, fixed

Three ways, all reproduced. `sed '/^## Sources/,$p'` runs to end of file, so a Sources section with
no citation followed by an Appendix link counted 1. `https://-` counted as a source. And
`<https://example.com>` counted separately from its bare form. Bounded at the next `## `, a real
host required, Markdown delimiters neutralised — and the check that this is not over-tightening:
22 / 12 / 7 / 5 on the four real artifacts, identical to before, while the reviewer's fixtures went
4 → 1 and 1 → 0.

## F035 [low, security] evaluator-disposition language, fifth instance — AGREED, fixed

"Second and last round" and "Accepted as a stated limit" prescribe review termination and acceptance
inside data the reviewer is told to distrust. The reopening section records what changed and what
was measured; the round budget and the accepted residual belong in the round file and the ledger,
which is where dispositions live.

Consensus: resolved
