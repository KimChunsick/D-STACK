## Carried decisions — Round 002
- **Fence tracking is length- and character-aware.** A naive `^```` toggle flips on the ``` lines a
  ```` block legitimately contains: measured on such a block, it read NEITHER the fenced fake row
  NOR the real one below. A closer must now be the same character, at least as long, and carry
  nothing else. HONEST RESIDUAL, and it is the F024 tension: `check-parallel.sh` still uses the
  naive toggle, so on a four-backtick block the two disagree — but in the FAIL-CLOSED direction,
  because this checker blocks on the mismatch instead of confirming the scheduler's wrong reading.
  One parser blocking loudly beats two being confidently wrong together. The identical fix is a
  recorded follow-up for the unit that owns `check-parallel.sh`.
- **Granularity must be the DOCUMENTED value.** A substring test read `Review granularity: not task`
  as task granularity — the worst possible reading of the line whose whole job is to fix the depth
  everything else is checked at.
- **Ownership is classified in EVERY branch, not just the open-unit one.** The closed-unit and
  wrong-depth branches tested `! owned`, so a foreign-owned record passed silently. Absent, ours and
  another session's are three different states with three different fixes, everywhere.
- **Every transformation is materialised and checked separately.** A pipeline reports only its last
  command's status and a process substitution's status is not observable at all — measured,
  `while read …; done < <(exit 7)` leaves the loop at rc 0. An erased producer yields an empty set,
  an empty set yields empty deltas, and empty deltas read as "no differences". That is a false PASS
  produced by a crash.
- **"Nothing else is registered" is ENUMERATED FROM THE REGISTRY.** Looking only at the alternate
  depth and only at `task.md` meant `<goal>/<Mn>/note.md` could be registered to this session and
  never be seen. Reading `status` and subtracting the allowed set has no blind spot, and it replaces
  a narrowed claim with an actual check.
- **The milestone bijection is enforced at both ends** — a heading boundary after `M<n>`, and
  `M[0-9]+-<non-empty-slug>/task.md` for the folder. `###M1oops` and `M1oops/task.md` both yielded
  id 1 before.
- **The Goal directory is canonicalised** to the repo-relative spelling the registry stores, so
  `docs/g`, `./docs/g`, `docs/g/` and an absolute path all check the same thing rather than
  reporting every document unregistered.
- **The success line stopped overstating.** It said "all units owned by this session" after a branch
  that requires closed units to be UNregistered; it now reports scaffolded, open-and-registered, and
  what was checked.

Consensus: resolved
