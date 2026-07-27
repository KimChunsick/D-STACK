# Maintainer response — Round 007

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The assembler could label changed code "no change".** `diff="$(git diff HEAD -- … || true)"`
swallowed every failure, and the empty result then took the `tracked, no change vs HEAD` branch —
so a git exiting 128 (corrupt object, unreadable index, bad `GIT_DIR`) presented CHANGED code to
the reviewer as unchanged, and the mandatory gate could approve a task whose implementation it
never saw. It now captures the status and emits a `SKIPPED` marker on any diff error, which the
launch guard turns into a refused round. The second half of the finding was also right: under
worker fan-out the reviewed identity is the recorded `base..HEAD` committed range, and diffing
against `HEAD` there yields nothing at all — so `REVIEW_BASE` was added, validated as a real
commit and as an ancestor of `HEAD` before use. Verified: normal assembly unchanged; a forced
diff failure produces the SKIPPED marker; `REVIEW_BASE=deadbeef` is rejected by name.

**CROSS-DECLARATION, recorded rather than hidden:** `assemble-review.sh` is in neither
milestone's declared `files`. It is the enforcement point of the review gate, and the finding is
that the gate can approve unreviewed code, so leaving it for a later Goal would mean every round
between now and then runs through a tool known to fail open. Fixed here, named here, and included
in the next round's bundle.

**[high] The generated runner re-parsed the repository root as shell source.** `cat > run.sh <<EOF`
(unquoted) expanded `$RD` at generation time and wrote `RD="/the/path"` into the script — so a
checkout whose path contains `$(...)` had that substitution written out verbatim and EXECUTED when
the runner ran, and a `$HOME` inside a path silently resolved elsewhere. A directory name is data.
The heredoc is quoted now and the path arrives as `$1`, passed as `argv[2]` to the launcher.

**[medium] The review-unit conversion was still partial where it decides ORDER.** P9's prose said
"different tasks may overlap" while P9 is per review unit — at milestone granularity several tasks
share one unit, so that reading permits two concurrent rounds of the SAME unit, and the round-number
allocator is check-then-write: both would pick the same filename. And the worktree merge gate waited
on "that task's review consensus", which at milestone granularity does not exist, making fan-out
unsatisfiable. Both parameterized over the unit, with the failure each one produces spelled out.

**[medium] The convergence case made the triage recipe fail.** `grep -c` exits 1 when the count is
zero and `grep -n` exits 1 when nothing matches — so a clean, approving round, the one outcome the
loop exists to reach, reported command failure. Normalised: the count still prints, and "no
blocking findings" is success.

**[low] Closure cleanup was not fail-closed.** `rm-run` returns 0 for a label that was never there
(correct idempotence), so a typo read as "cleaned up", and the following `prune` masked a real
failure behind its own success while being unable to help — these captures are minutes old, not
eight days. The recipe checks the status and then verifies the directories are actually gone.

Verified by direct run (repo policy: no TDD): `bash -n` on the assembler; assembly normal, under a
forced diff failure, and with an invalid `REVIEW_BASE`; `skill-schema.test.sh` green (it caught a
line-wrap that split the pinned `Merge precedes P10` string, which is exactly what it is for);
`tests/secret-guard.sh` green.
