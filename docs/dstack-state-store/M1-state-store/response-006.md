# Maintainer response — Round 006

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The empty-file branch ran outside the lock.** Round 5 acquired the legacy lock after the
`! -s` test, so the sequence "observe empty → compliant writer appends under the lock → we delete
the now-nonempty file and report 'empty — removed'" silently destroyed a claim. The lock is now
the FIRST thing `migrate` does, and every decision about that file — existence, tracked status,
size, removal, snapshot, archive — is made while holding it, with the file re-validated after
acquisition.

**[high] The absence walk used the logical `$PWD`.** From `/tmp/repo-link/sub` it examined
`/tmp/repo-link`, `/tmp`, `/` and never saw `/real/repo/.git`, so a fatal git failure inside a
real checkout still exited 0 — the fallback failing open through a symlink instead of through a
missing binary. It starts from `pwd -P` now and blocks if that cannot be resolved or is not
absolute. Verified: from a symlinked path with a broken `GIT_DIR`, the hook now blocks and names
the real `.git` it found.

**[medium] A key helper was used as a file digest.** `sha1_try` lowercases, because it derives
document KEYS on a case-insensitive filesystem. Reused for the migration snapshot it made
`OwnerA<TAB>docs/X.md` → `ownera<TAB>docs/x.md` produce an identical digest, so a case-only
rewrite of the authority passed the unchanged check. A separate byte-preserving `digest_file`
handles snapshots, and it reads the file directly rather than through a command substitution.

**[medium] The dependency-status sweep missed siblings.** Round 5 fixed `sha1_try` and left the
hook's own key derivation on `printf | tr | $SHA1TOOL` — where a failing `tr` plus a successful
`shasum` returns 0 and `da39a3ee…`, the digest of an empty stream, which every record would
"match". Also `cat` status for records, and `wc`/`cat` for the version marker. This is the third
round in which a class fix landed in one artifact and not its sibling; it is now the first carried
decision for a reason.

**[medium] `migrate` could move a TRACKED file.** `ensure_store` proves `.dstack` is untracked but
nothing checked the legacy source, so in any repository with a committed `.fullcycle-active`
holding one syntactically valid line, this global CLI would remove it from its tracked path and
rename it to `.migrated`. It now refuses a tracked legacy file outright. Verified in a scratch
repository: tracked → refused; untracked empty → removed; untracked non-empty → migrated, lock
released.

**[medium] Bounded retention was not bounded by anything.** `prune` only ever ran when a human
typed it, while the documentation sells a 7-day window as the mitigation for plaintext code diffs
on disk. It now runs on capture creation — a lifecycle boundary that actually happens — and a
sweep failure warns rather than failing the allocation.

**Lows, all fixed.** Byte-exact `.gitignore` in both `ensure_store` and the secret guard (`$(cat)`
strips trailing newlines, so `*\n\n\n` passed an "exact byte" check — negative-controlled).
`run-dir` no longer claims a label was released when the `rmdir` failed; it says the label is
stuck and names it. `status` distinguishes a record that VANISHED mid-read (a concurrent `unreg`,
which POSIX permits and which is not corruption) from one that is corrupt. `canon` prefers the
caller's exact spelling when it exists on disk, so a case-sensitive volume holding both `A.md`
and `a.md` no longer rewrites one to the other. The retention message states the threshold that
actually applies (`-mtime +7` selects at 8 complete days) instead of the configured number.
Migration warns instead of silently leaving a stale lock.

Verified by direct run (repo policy: no TDD): `bash -n` on both artifacts; the hook against the
plain, `stop_hook_active`, and symlinked-path-with-broken-`GIT_DIR` cases; a scratch repository
exercising tracked/empty/non-empty migration plus lock release; `run-dir` + `rm-run`; the prune
message; a `reg` on a lowercase document; `tests/secret-guard.sh` green with a negative control on
the tampered `.dstack/.gitignore`; `skill-schema.test.sh` green.
