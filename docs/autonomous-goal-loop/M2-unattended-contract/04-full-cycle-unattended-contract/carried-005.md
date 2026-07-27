## Carried decisions — Round 005
- **The P6 proof left this file entirely.** Five rounds, five repairs, five new defects — a
  hand-listed array that was its own proof, a `find` derivation comparing counts not identities,
  `GRAN` hand-set so a milestone Goal checked at the wrong depth, `find | sort` masking a failing
  `find`, a loop returning 1 on its success path. The maintainer's decision was to move it into
  `check-registration.sh` (T06), and the fence here is now three lines that register and invoke it.
- **`find | sort` reports SORT's status.** A find that fails after emitting one path assigns rc=0
  and the partial list is accepted. Run find alone, check it, then sort.
- **`grep && { exit 1; }` inside a `while` leaves status 1 on the SUCCESS path**, so a trailing
  `|| exit 1` aborts the whole fence silently. Reproduced under bash 3.2 and zsh 5.9. Collect the
  offenders into a variable and test it, or use an explicit `if`.
- The terminal-record-gated cleanup was applied to BOTH invoked wrappers, not only stated here —
  this round could not see that because the bundle carried only this file.

Consensus: disagreed
