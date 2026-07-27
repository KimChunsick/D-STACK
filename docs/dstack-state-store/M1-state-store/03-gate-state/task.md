# 03-gate-state

## Intent / Why
Point the gate hook at `.dstack/active/` and refuse to run against a stale layout. A silent
dual-read of both the legacy file and the new directory would make it impossible to tell which
one is authoritative, so a non-empty legacy `.fullcycle-active` fails loud with the migration
command instead. Directory scans tolerate entries vanishing mid-read, because POSIX does not
guarantee directory iteration is a snapshot.

## Design consult
Covered by this milestone's consult (`../design-consult.md`); the cutover and vanishing-entry
rules implemented here come straight from its findings 4 and 8.

## What was done (what / why)
The gate now reads `.dstack/active/` instead of `.fullcycle-active`.

**Cutover is a refusal, not a preference.** A repository still holding a non-empty legacy file
is blocked outright with the `dstack migrate` instruction. Reading both stores would leave no
answer to which is authoritative; reading only the new one would silently un-gate every document
still recorded in the old file. Both alternatives fail quietly, which is the failure mode this
whole pipeline is built against.

**Vanishing entries are tolerated, unreadable ones are not.** POSIX does not promise directory
iteration is a snapshot, so an entry may disappear between listing and opening — that is a
deregistration racing the scan, and it is skipped. A record that *exists* but will not parse is
reported as a gate problem instead. Silently skipping it would turn an unreadable registry into
an empty one, which reads as "no work is registered". Partial reads are not a concern: `dstack`
publishes with `ln`, so a record is either absent or complete.

Per-session scoping, fail-closed attribution, the docs/-only rule, the symlink refusal, and the
one-Goal rule all carry over unchanged; only the source of records moved. The dedupe pass is
kept as belt-and-braces even though `dstack` keys by document and cannot produce two records for
one path.

**Migrated this repository's live registry** in the same step, deliberately not earlier: with
the hook still reading the legacy path, an earlier migration would have left this Goal
registered in a store nothing enforced.

**Narrowed the secret guard** (discovered here, not planned). The guard refused *any* nested
`.gitignore` on the grounds that one can reopen a protected path — and `.dstack/.gitignore` is a
nested `.gitignore`. Reopening requires a negation, and a file whose entire content is `*` can
only close, so the refusal is now narrowed to exempt exactly that one path with exactly that
content, unstaged and not a symlink. The index-side check is left strict, so `git add -f
.dstack/.gitignore` is still caught. GOAL.md's T03 declaration was corrected to include the file
before the edit was made.

## Files changed (where / why)
- `claude/hooks/fullcycle-gate.sh` — reads `.dstack/active/`; fail-loud cutover refusal;
  unreadable records reported; header rewritten to describe the new registry and escape hatch
  (`dstack unreg`).
- `tests/secret-guard.sh` — one narrow, content-checked exemption to the nested-`.gitignore`
  refusal, plus the section comment that stated the old absolute rule.

## Verification (direct run — repo policy: no TDD, no tests)
Throwaway git repos for the behavioural matrix, then the live repository.

*Registry source* — no store at all: ALLOW. Registered Goal with unticked gates: block naming
them. All Goal gates ticked: ALLOW. Corrupt record dropped into `active/`: block naming the
record file.

*Cutover* — a non-empty `.fullcycle-active` beside the store: refused with the migrate
instruction, before any gate parsing.

*Review series* — registered task with no `codex-review-*.md`: block. With a sealed
`Consensus: agreed`: ALLOW. With `Consensus: disagreed`: block.

*Session scoping* — session B does not enforce session A's records: ALLOW. A forged
empty-owner record with an unticked gate blocks session B anyway, confirming unattributable
records stay enforced by every session.

*Guard exemption is narrow* — `.dstack/.gitignore` containing `*` plus a negation line: guard
FAILS. Restored to `*`: green. A different nested `.gitignore` (`docs/probe-nested/.gitignore`)
with the identical `*` content: guard FAILS. Removed: green.

*Live migration* — `dstack migrate` carried 3 records and renamed the legacy file to
`.fullcycle-active.migrated`. `dstack status` lists all three under this session. The hook then
produced the same incomplete-gate report as before the move, and returned ALLOW for a
`stop_hook_active: true` continuation. `bash tests/secret-guard.sh` passes with the real
`.dstack/` present and the change staged.

Also removed the zero-byte `.fullcycle-active.tmp` at the repository root — residue of an
interrupted deregistration by the old helper, and the concrete piece of litter cited when this
Goal was proposed.
