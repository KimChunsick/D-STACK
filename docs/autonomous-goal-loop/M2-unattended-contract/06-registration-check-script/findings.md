# Finding ledger — 06-registration-check-script

The loop closes when a round raises nothing both NEW to this ledger and CONCRETE.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| F001 | 001 | **high** | two parsers disagreeing about what a declaration is | Markdown fence context was tracked only from the section heading, so a fence opened in an earlier section inverts the file: real rows read as fenced, fenced examples read as real | fixed — fences tracked GLOBALLY from line one, mirroring `check-parallel.sh`. Reproduced on a fixture: the old parser read the fenced `per milestone` granularity and both fake task rows, and none of the three real ones |
| F002 | 001 | **high** | identity checks that are not identity checks | milestone mode compared counts only; malformed folder names were discarded; duplicate numeric prefixes were collapsed by `uniq` | fixed — both granularities compare identities both ways, an unreadable id is reported rather than dropped, duplicates are reported per id. Fixture: 03 scaffolded twice with 02 absent used to pass, now blocks with both reasons |
| F003 | 001 | **high** | a false result wearing an error's clothes | an unreadable unit doc read as "closed" and a foreign-owned registration as "absent" — the two states most likely to be broken became the two nothing checked | fixed — three separate outcomes with three messages; the foreign case says what it costs (the Stop hook SKIPS another session's records, so it looks registered and enforces nothing) |
| F004 | 001 | **high** | a fail-loud claim with unchecked dependencies | a failed `comm`/`sed`/`uniq` produces empty deltas, which read as "no differences" and pass | fixed — every `find`, `sort`, `comm` and extraction status checked, plus a count-in/count-out guard that dies when the extractor reads fewer ids than the parser found rows |
| F005 | 001 | medium | a comparator fed the wrong collation | `sort -n` violates `comm`'s lexical-ordering requirement | fixed — zero-padded ids sorted lexically under `LC_ALL=C`. Measured: declared {2,10,20} vs scaffolded {10}, `comm -23` over `sort -n` inputs returned `2 10 20`, reporting a present id as missing; the padded form returns `002 020` |
| F006 | 001 | medium | a guarantee wider than its implementation | "nothing else under the tree is registered" scans one alternate depth and only files named `task.md` | resolved by narrowing the CLAIM to what the code does, in the file — a registered document at some third depth, or under another name, is outside what this proves |
| F007 | 001 | **high** (found while fixing F001) | a caller implementing the opposite of the table above it | P6's registration loop iterated a literal `<Mn>/<NN-task>/task.md`, registering task-depth documents even for a milestone-granularity Goal | fixed — `--depth` returns 3 or 2 from the same GOAL.md parse the check uses, so the fence reads the level instead of writing it. Also filed against unit 04 as F023, where the fence lives |

| F008 | 002 | **high** | a fence toggle that ignores fence length | a ```` block legitimately contains ``` lines; the naive toggle flipped on each, and on such a block read NEITHER the fenced fake row NOR the real one | fixed — length- and character-aware fences. **Residual, named:** `check-parallel.sh` still uses the naive toggle, so the two disagree on a four-backtick block — in the FAIL-CLOSED direction, because this checker blocks on the mismatch. Identical fix is a follow-up for that file's unit |
| F009 | 002 | **high** | a substring where a value belongs | `Review granularity: not task` selected task mode | fixed — only the documented `per task` / `per milestone` values are accepted |
| F010 | 002 | **high** | producer failures erased before comparison | a pipeline reports only its last command's status and a process substitution's status is unobservable — measured, `while read …; done < <(exit 7)` leaves the loop at rc 0. An erased producer yields empty deltas, which read as "no differences" | fixed — every stage materialised into its own file with its own status check |
| F011 | 002 | **high** (same class as F003) | ownership classified in only one branch | the closed-unit and wrong-depth branches tested `! owned`, so a foreign-owned record passed silently | fixed — absent, ours and another session's are three outcomes in every branch |
| F012 | 002 | medium | a check that only looks where it expects trouble | "nothing else is registered" scanned one alternate depth and only `task.md`, so `<goal>/<Mn>/note.md` registered here was invisible | fixed — enumerated FROM THE REGISTRY and subtracted from the allowed set; the narrowed claim is replaced by an actual check |
| F013 | 002 | medium | a bijection enforced at neither end | `###M1oops` and `M1oops/task.md` both yielded id 1 | fixed — heading boundary required, and `M[0-9]+-<non-empty-slug>/task.md` for the folder |
| F014 | 002 | low | equivalent spellings, unequal results | `G="${1%/}"` did not canonicalise, so `./docs/g` reported every document unregistered | fixed — canonicalised to the repo-relative spelling the registry stores; four spellings verified identical |
| F015 | 002 | low | a success line that overstates | it claimed all units owned by this session after a branch that requires closed units to be UNregistered | fixed — it reports scaffolded, open-and-registered, and what was checked |

## Non-blocking follow-ups (recorded, not carried into another round)

- **From F008 — `claude/skills/full-cycle/check-parallel.sh` needs the identical fence fix.** Its
  toggle is the naive `^[[:space:]]*` + three backticks, so a four-backtick block containing a
  three-backtick fence inverts what it reads as a declaration. This script now handles that
  correctly, which means the two parsers can disagree on such a file — in the fail-closed direction,
  because the identity comparison here BLOCKS on the mismatch instead of confirming the scheduler's
  reading. `check-parallel.sh` is outside this unit's declaration and the allowlist may not grow to
  absorb a finding, so it is a follow-up for its own review unit. Its pinned test
  (`tests/check-parallel.test.sh`) will need a case for it in the same change.

## Blocking count per round

§4's counter is the number of concrete blocking findings still OPEN at the END of the round.

| round | raised (new, concrete, blocking) | OPEN at end of round |
|---|---|---|
| 001 | 6 (F001–F005, F007; four of them high) | 0 |
| 002 | 6 (F008–F013; four of them high) + 2 low | 0 — closed with the batch authorisation spent |

This unit's first round is the consolidated batch pass the maintainer authorised mid-Goal. Four
highs in one round on ~180 lines of new code is a lot, and the reason is worth recording: the script
exists because the same check failed five prose repairs in a row, and moving it into code moved the
defects rather than removing them. What changed is that they are now reproducible — every finding
above was demonstrated against a fixture repository and fixed against the same fixture, which is
what the prose version could never do.
