# Maintainer response — Round 005 (round cap)

Not bundled. This round's HIGH is what the maintainer's decision answered.

## F019 [high] the P6 fence still did not derive its expected set — AGREED, and the fence LEFT

Five rounds, five repairs, five new defects, all in the same thirty lines. This round's version
still hand-set `GRAN`, compared cardinality rather than identities, and never cross-checked the
milestone case. Rather than attempt a sixth, I asked the maintainer whether to keep patching prose
or move the check into code. They chose code.

`check-registration.sh` (T06) reads the granularity from GOAL.md, parses the declared task
identities from the same section `check-parallel.sh` parses, compares them BOTH WAYS against the
scaffolded folders, checks registration WITH ownership, refuses a closed unit that is still
registered, and separates exit 2 (could not run) from exit 1 (blocked). Nine scenarios recorded in
that unit's `task.md`, including the same-count-wrong-identity case this finding names.

## F020 [medium] the OTHER loop returned 1 on its SUCCESS path — AGREED, fixed

`grep && { exit 1; }` as the last statement in a `while` body leaves status 1 when grep does not
match, so the pipeline is 1 and the trailing `|| exit 1` aborted the whole fence silently.
Reproduced under bash 3.2 and zsh 5.9. The replacement collects offenders into a variable and tests
that. The script does the same.

## F021 [medium] `find | sort` masks a failing find — AGREED, fixed

`U="$( (echo a; exit 1) | sort )"` assigns rc=0 with the partial output. find runs alone, its
status is checked, then sort.

## F022 [medium] wrappers still clean unconditionally — DISPROVED for the current tree

Both wrappers were changed to the terminal-record-gated trap before this round was assembled. The
bundle carried only `full-cycle/SKILL.md`, so the reviewer could not see it and inferred from round
004's carried evidence — a reasonable inference from what it was shown. Verified: terminal record
present → removed; absent → KEPT.

## F023 [medium] P9 vs autonomy for concrete-MEDIUM cap closure — AGREED, fixed

`waits.external-residuals` now states that the precedence table overrides any per-skill retry text,
and `autonomy` is the single source. P9's broader "escalate blockers" wording defers to it.
