# Maintainer response — Round 010

Deliberately OUTSIDE the reviewed corpus, like every other response file: prose about what was
fixed is not evidence, the diff is.

Six findings fixed, one carried. This is the closing round for the unit — see
`codex-review-010.md` for the non-convergence measurement.

**[high] The containment repair only landed on one of two read paths.** Round 9 gave `emit_file`
a physical-parent check and left `validate_snapshot` — the path that reads the unit `task.md` and
the round history — testing only the leaf. Exactly the sibling miss this file keeps warning about.
Both now call one `contained` helper. Verified: with `docs/sym2 -> /tmp/outside2`, assembly stops
with `FATAL: snapshot resolves outside the repository: docs/sym2/task.md`; a normal serial
assembly still produces its bundle.

The check-then-read residual is NOT fixed and is stated in the code: resolution and the read are
two steps, so a parent swapped in between is not caught. Closing that needs `openat`/`O_NOFOLLOW`,
which a shell does not have. This is a mistake tripwire, not a boundary against an attacker who
already has write access to the repository — the same scope the Stop hook's self-attestation has.

**[high] The committed invocation was a commented ellipsis, so the only runnable line said
`serial`.** Reported twice for the same reason: a contract nobody can run is not a contract. It is
now written out in full alongside the serial block, with the base/head sourcing and the
"delete whichever you are not running" instruction, plus the document-supply rule that stops
someone committing the orchestrator-owned `task.md` onto an integration branch to make the
assembler find it.

**[medium] `unit-scope` was a checker mode that did not exist.** `check-parallel.sh` accepts
`plan|scope` and answers `INVALID: unknown mode 'unit-scope'`. The choice was to build a union
mode or delete the claim. Nothing in this Goal fans out, so building it would be speculative
work behind a false promise. Deleted, and replaced with an explicit fail-closed precondition:
worker fan-out requires the review unit to be exactly one task; a wider unit runs serial.

**[low] `run.sh` could be launched half-written.** `-s` proves nonempty, not complete. Now written
to `run.sh.tmp` and renamed, same as `pid` and `exit`.

**[low] The schema check printed `ok` on an empty set.** If `mktemp` or `awk` failed, every
per-fence loop iterated nothing and reported success — a check that verifies nothing while
reading as verified. Both now abort loudly, and a zero-fence extraction is itself a failure. The
verb list is defined once, so the destructive `rm-run` can no longer be missing from the bare-call
scan while present in the positive loop. Negative-controlled: a fence-free document produces
`FAIL: no bash fence was extracted`, an injected bare `dstack rm-run` produces
`FAIL no bare dstack call in a runnable block (fences: f1.sh)`, and the real file stays green
through the same patched harness.

**[low] Record drift.** Fixed in place: the assembler IS declared (T04), `claude/bin/dstack` is
M1's T02, and the claim that the assembler's allowlist and budget logic were unchanged stopped
being true at Round 7.

**[low, CARRIED] Capture cleanup cannot enumerate a unit's captures.** Real, and a real feature to
fix — `dstack` would have to persist capture-to-unit ownership. It is retention hygiene on a
mode-700 directory that prunes after 7 days, not an exposure. Recorded in `task.md` under
«Recorded follow-ups» with its evidence rather than silently dropped.
