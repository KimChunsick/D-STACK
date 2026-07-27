# 02-dstack-cli

## Intent / Why
Registry mutation currently lives as ~25 lines of bash inside `full-cycle/SKILL.md` that the
model must reproduce correctly on every run; when it fails it prints a warning and the work
proceeds ungated. A stranded zero-byte `.fullcycle-active.tmp` at the repo root is the visible
residue of that design. Move the state into one gitignored `.dstack/` directory and put the
mutations behind a real CLI, so a deterministic transform is done by code rather than by the
model.

## Design consult
Covered by this milestone's consult (see `../design-consult.md` and the disposition table in
`../milestone.md`). Verdict was **rework**; every blocking finding is reflected below.

## What was done (what / why)
`claude/bin/dstack` (bash 3.2, deps `jq` + `git` + `shasum`/`sha1sum`) owns `.dstack/`.

**Ownership is a claim, not a write.** `reg` publishes with `ln`, which fails when the name
already exists, so two sessions racing on one document cannot both believe they won. Had this
used temp+`rename` as originally designed, the later rename would have silently replaced the
earlier record and **released the losing session from the gate without telling it** — a
straight enforcement regression against the old line format, where both owners stayed blocked.
Same owner re-registering is idempotent; a different owner is exit 3 naming who holds it.

**The global lock is gone; a per-key lock is not.** One record per document means distinct
documents never contend, which is what stranded the old `.tmp` and lock directory. `unreg` and
`reclaim` genuinely read-then-write, so they take a lock on that key alone.

**`reclaim` never sweeps.** With heartbeats deliberately out of scope there is no liveness
signal, so "not owned by this session" does not mean "abandoned". It takes explicit document
paths and prints whose record it took.

**Path identity is canonical.** Directory components resolve physically (`pwd -P`), so
`doc.md`, `./doc.md`, an absolute path, a path from another working directory, and a path
through a symlinked parent all collapse to one key. The final component may not be a symlink,
matching the Stop hook. Non-`docs/` paths are refused outright rather than registered into a
record the gate would silently ignore.

**`migrate` refuses rather than guesses.** Untagged lines (which every session enforces),
multi-owner documents, empty or malformed owners have no lossless representation in a
one-owner format, so migration stops and reports them. Records the gate already ignored
(non-`docs/` paths, missing documents) are dropped with a printed reason. On success the legacy
file moves to `.fullcycle-active.migrated`. Until that runs, every mutating command refuses
(exit 4) so the two stores can never both look authoritative.

**Namespace and content hygiene.** Refuses a symlinked, non-directory, or git-tracked
`.dstack`. Session ids are validated against `[A-Za-z0-9_-]+` before becoming directory names.
Records are one-line JSON with a `version` marker, since a two-line format cannot hold a path
containing a newline and offers no schema evolution. Run captures are mode 700 and pruned by
`find -mtime +7`, which removes a capture once it is eight complete days old — not seven, which
an earlier draft of this line claimed. `AGENTS.md` carries the same corrected wording. The point
stands either way: gitignored is not private.

Not provided, deliberately: fsync durability (bash cannot express it; this state is
reconstructible from the work documents) and Unicode-normalisation folding on APFS.

## Files changed (where / why)
- `claude/bin/dstack` — new; the whole CLI.
- `.gitignore` — allowlist `!/claude/bin/` + `/claude/bin/*` + `!/claude/bin/dstack`, matching
  the pinned-file pattern already used for `hooks/`.
- `tests/secret-guard.sh` — pinned negation list gains the two new lines and
  `GITIGNORE_SHA_PIN` is updated, in the same change as the allowlist edit as `AGENTS.md`
  requires.
- `install.sh` — one MAP row; the installer already creates missing parent directories.
- `AGENTS.md` — records the config/runtime-state split and the `.dstack/` layout.

## Verification (direct run — repo policy: no TDD, no tests)
`bash -n` clean. All exercises ran in throwaway git repos, never against the live registry.

*Registration and identity* — `reg` registers; re-registering is idempotent; `./docs/…`, an
absolute path, and a path from a subdirectory all collapsed to a single record (2 documents, 2
record files).

*Ownership* — session B registering A's document exits 3; B's `unreg` of A's document exits 3;
`reclaim` transfers it and names the previous owner; B can then `unreg`; a second `unreg` is a
clean no-op; `reclaim` with no arguments exits 1.

*Refusals* — non-`docs/` path, missing document, symlinked document, path outside the
repository, session id containing `/`, empty session id: all exit 1 with a specific message.

*Namespace* — symlinked `.dstack`, `.dstack` as a regular file, and a git-tracked `.dstack` are
each refused.

*Migration* — happy path moved 2 records and renamed the legacy file; an untagged line, a
two-owner document, and an empty owner each exit 4 with the offending line quoted; droppable
records (non-`docs/`, missing) printed a reason and the good record still migrated; an empty
legacy file is simply removed. Cutover guard confirmed: `reg` exits 4 while a non-empty legacy
file exists, and `status` still runs read-only.

*Runs* — `run-dir` creates `drwx------`; a bad label exits 1; `prune` removed a bundle
backdated past the window.

*Store* — `.dstack/.gitignore` contains `*`, `version` contains `1`, and `git status` sees
nothing after `git add -A`.

**One defect found and fixed during verification.** `status` exited 1 whenever it found
records: its last statement was `[ "$n" -eq 0 ] && note …`, so a false test became the
command's exit status. A caller branching on the exit code would have read "found work" as
"failed". Replaced with an `if` plus an explicit `return 0`, and the rest of the file was
grepped for the same trailing-`&&` shape (nine other sites, none of them a function's last
statement).

*Wiring* — `./install.sh --dry-run` showed the new entry creating `~/.claude/bin`; the real run
linked it; `ls -l` confirms the symlink; `~/.claude/bin/dstack --help` runs. `bash
tests/secret-guard.sh` passes with the change staged.

## Known follow-up (carried into T03)
The live `.fullcycle-active` in this repository is deliberately **not** migrated yet. The gate
hook still reads the legacy path, so migrating before T03 would leave this Goal registered in a
store nothing enforces — silently ungated. Migration happens with T03, in one step.
