# Maintainer response — Round 005

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The migration cutover was still detection, not exclusion.** Round 4 digested the legacy
file before and after, which narrowed the window but left the final digest and the `mv` as two
separate operations. The answer was in the protocol being migrated FROM: the old skill serialized
every append on a `mkdir` mutex at `.fullcycle-active.lock`, and it appended INSIDE that lock. So
`migrate` now takes that same lock and holds it across the whole cutover — a writer following the
old protocol cannot append at all while we hold it. Residual, stated rather than papered over: a
writer that ignores the lock (a hand edit, or one holding an fd opened before we acquired it) is
outside that guarantee, which is why the digest comparison stays as the net for exactly that case.

**[medium] The marker check sat behind the "no active/" shortcut.** A store with `version: 2` and
no `active/` made the hook exit 0 while every `dstack` mutation refused the same store — the
writer saying "unsupported schema" and the gate saying "all clear". Order fixed: no `.dstack` at
all exits 0; a store that EXISTS has its marker validated before anything else.

**[medium] The record invariant still diverged from the CLI.** The hook filtered foreign
ownership BEFORE checking the document, so a foreign-owned record for a deleted document was
skipped here and reported invalid by `status`. Document existence, symlink status and the
printable-ASCII rule now run for every record, before ownership is consulted. An empty owner is
`bad` rather than "unattributable but otherwise fine" — `dstack` never writes one, so it is a
corrupt record, and both tools now say so. `bad` still blocks, so nothing was weakened.

**[medium] The "prove repository absence" fallback failed open.** `nd="$(dirname …)" || break`
fell through to `exit 0`, so a traversal that could not continue opened the gate — the same
defect the fallback was added to fix, one level down. Every non-progress condition other than
reaching `/` blocks now.

**[medium] Dependency output still not validated.** Two sites. `printf | tr | digest` reported
only the digest's status, so a failing `tr` produced the valid SHA-1 of an EMPTY stream — one
constant key every document would collide on; each stage has its own status now. And all three
record writers linked jq's output into the namespace without reading it back; `written_record_ok`
re-parses the temp file and checks `doc`/`session`/`v`/`ts` before publication.

**[medium] Recovery commands were injectable.** `canon` accepts quotes, semicolons, `$` and
spaces (legal in a filename), so `reclaim '$doc'` let `docs/x'; touch PWN; echo '.md` close the
quote and append a command to whatever the reader pasted. Every dynamic value in a copyable
command goes through a `printf %q` helper now, and the lock-timeout message recommends a quoted
`rmdir --` rather than a raw `rm -rf`.

**[low] `status` hid dot-prefixed entries** the hook blocks on — so the tool you run to find out
why the gate is blocking printed "(none)". It enumerates both hidden globs now.

Verified by direct run (repo policy: no TDD): `bash -n` on both artifacts; the hook against a
store with `version: 2` and no `active/` (now blocks, previously exited 0), a fresh repo with no
`.dstack` (exits 0), a foreign-owned record for a missing document (now reported
`doc-missing:…`, previously skipped), plus the plain and `stop_hook_active` cases; `dstack status`,
`migrate` with no legacy file, and both pinned checks green.
