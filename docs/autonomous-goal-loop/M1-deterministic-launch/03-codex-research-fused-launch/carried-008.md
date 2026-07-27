## Carried decisions — Round 008
- **The wrapper traps every signal `dstack` traps, and the "exactly two gaps" claim was wrong.**
  Measured: under zsh a wrapper-only USR1 exits 158 WITHOUT running the EXIT trap, leaking the
  scratch directory; bash runs it either way. The old three-signal set was the gap. What the fix
  does not buy is now stated rather than implied — a handler cannot cancel a foreground
  `dstack run`, so `codex exec` survives regardless; that residual is `dstack`'s, plus the standing
  rule that a capture with no terminal record must be checked for a live group before relaunching.
- **Root anchoring is not write confinement.** `mkdir -p` and every later open follow ancestor
  symlinks, so `docs/<goal>` pointing at /tmp/target sends both the brief and the `-o` artifact
  outside the repository while every path in the recipe still reads as repo-relative. `dstack` does
  not cover it — it checks only whether the `--stdin` file itself is a symlink. Symlinked ancestors
  are refused and the physical directory is confirmed under the physical repo `docs` before any
  write.
- **The session id is checked against `dstack`'s OWN grammar**, `[A-Za-z0-9_-]+`, not merely for
  non-emptiness. `../cross-session` passed the old predicate and `dstack run` then refused the
  launch — after scratch had been allocated with no terminal record to authorise cleaning it. The
  run-dir pre-check is labelled a pre-check: `dstack`'s `.launch` mkdir stays the atomic claim.
- **The zero-source gate is bounded, host-validated and delimiter-normalised.** Three ways a
  source-free artifact suppressed its own fallback: `sed '/^## Sources/,$p'` ran to end of file so an
  Appendix link counted; `https://-` counted as a source; `<https://example.com>` and its bare form
  counted twice. Fixed and measured — 22/12/7/5 unchanged on the four real artifacts, so no true
  positive was lost, while the reviewer's fixtures went 4 -> 1 and 1 -> 0.

Consensus: resolved
