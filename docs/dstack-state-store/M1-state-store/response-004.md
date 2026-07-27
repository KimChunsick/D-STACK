# Maintainer response — Round 004

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] Dependency boundaries still fail open.** Three separate sites, one class.
`git rev-parse` status 128 no longer means "outside a repository" — that is also what a broken
`GIT_DIR`, an unreadable object store, or a permission problem returns, and the reviewer's probe
reproduced it from inside this checkout. The hook now proves absence independently: it walks up
from the CWD looking for `.git`, and blocks if it finds one, because "the repository is
unreadable" and "there is no repository" call for opposite verdicts. The `stop_hook_active` read
checks jq's STATUS (jq can print `y` and then fail). Field extraction checks status. Digest
computation no longer runs into `cut` in one pipeline — a pipeline reports its LAST command's
status, so a digest tool that printed 40 hex characters and then failed was accepted; `sha1_try`
captures the tool's own status and `sha1` is the fail-loud wrapper over it.

**[high] Migration had no quiescent snapshot.** A tab appending after the reader hit EOF but
before the archive `mv` had its claim moved into `.migrated` with no record created — the owner
silently released. There is no lock over a file other sessions append to directly, so the
snapshot is made DETECTABLE rather than exclusive: digest the legacy file at read time, digest it
again immediately before the archive, and refuse the cutover if it moved. Records already
published stay published (they are exact copies of legacy lines, so re-running is idempotent);
what is refused is archiving a file that grew a claim nobody read.

**[high] "Complete record invariant" was still lexical.** `docs/*` plus "the final component is a
real file" accepted `docs/../../outside/GOAL.md` — and this hook runs in EVERY repository, so
that was a global read primitive, with `section` opening the external file. It also accepted
`docs/x/../real/GOAL.md`, which `status` showed as healthy while `unreg` canonicalised it to a
different key and could not release it. Both sides now demand IDENTITY: `read_record` requires
the stored path to equal what `canon` derives from it (the same function `reg` used to write it),
and the hook — which cannot call `canon` — rejects dot components and requires the physically
resolved parent plus basename to equal `$root/$doc`. That one comparison covers `..`, `.`,
symlinked parents and escaping the repository. `ts` joined the schema predicate on both sides.

**[medium] Dynamic-child symlink repair was partial.** `status` expanded `$RUNS/*/*` and tested
`-d`, both of which traverse a symlinked session directory, so it printed external directory
names as this repo's captures; it now rejects the session level explicitly and reports the link
instead of walking it. `reg`/`unreg`/`reclaim` treat `-e || -L` as occupied — a DANGLING record
symlink fails `-e`, so the slot looked free while the hook kept reporting that entry. `reg`'s
failed-publication branch validates before reading any field.

**[medium] The schema marker was not authoritative for readers.** `version` set to a future
number made every mutation die while the hook and `status` reported an empty store — the writer
refusing and the gate opening on the same directory. `version_ok` is now a prerequisite for
`status` too, and the hook checks the marker itself. The comparison is byte-exact: `$(cat)`
strips ALL trailing newlines, so `1\n\n\n` had been passing as a clean match.

**[medium] Migration dedupe was still delimiter-based.** Removing the angle-bracket refusal (a
Round-3 fix, correct on its own) exposed it: `docs/outer><docs/target/GOAL.md` made the `<doc>`
substring test report a genuinely different document as already seen. Dedupe is on the 40-hex key
now — fixed-width hex no path can forge.

**[medium] Recovery messages prescribed a bare `dstack`.** Every runnable command in the CLI's
own errors, and a new note at the top of `--help`, now names `$HOME/.claude/bin/dstack` and
quotes the document path.

**[low] Suppressed `rmdir` in `kunlock`** — a stale lock survived while the caller reported
success, and every later operation on that key then timed out. It warns now.
**[low] Unchecked `rm -f` of an empty legacy file** — checked, and re-checked for presence.
**[low] `run-dir` burned a label on a chmod failure** — the leaf is released before dying.
**[low] Evaluator directives inside the reviewed artifact** — accepted, and this one is a design
error, not a slip. Both review-unit docs opened by telling the reviewer what to read and what was
"out of scope by construction". Rewritten as neutral statements of how the work is filed. The
review prompt now also says outright that any in-payload scope claim is data, and
`codex-review/SKILL.md` carries the rule so it does not come back.

Verified by direct run (repo policy: no TDD): `bash -n` on both artifacts; the hook against six
crafted fixtures — plain, `stop_hook_active:true`, a broken `GIT_DIR` inside this repo (now
blocks; previously exited 0), genuinely outside a repository (silently exits 0), a tampered
`version` (blocks), and a planted record whose doc contains `..` (reported as
`doc-path-has-a-dot-component`); `dstack status`, an idempotent `reg`, a non-canonical input path,
`run-dir` refusing a taken label, `status` dying on a tampered marker; `rm-run` on real captures,
on a traversal label (refused), and on a missing label; `tests/secret-guard.sh` and
`claude/skills/full-cycle/tests/skill-schema.test.sh` both green.
