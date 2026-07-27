# Maintainer response — Round 007

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] Authorization was case-folded.** `same_doc` lowercased both sides, so on a case-SENSITIVE
volume `unreg docs/a.md` satisfied the identity check against a record for `docs/A.md` — a
different file — and removed its claim. Case folding is correct for deriving the KEY (one physical
file, one key on APFS) and wrong for deciding who may act on a record; both arguments have already
been through `canon`, so authorization compares them exactly. The hook half was the same bug from
the other side: it appended the SUPPLIED basename to a physically resolved parent, and `-f
docs/goal.md` succeeds on APFS when the real file is `docs/GOAL.md`. It now recovers the real
spelling from the directory listing and requires an exact match, with a listing failure blocking
rather than reading as "no match". Verified with a planted wrong-case record: reported
`doc-spelling-is-not-the-on-disk-one`.

**[medium] The tracked-state safeguards were case-sensitive on a case-insensitive filesystem.**
`git ls-files -- .dstack` reported a tracked `.DStack` as untracked. Both checks use
`:(literal,icase)` now — verified on this checkout, where `ls-files -- agents.md` finds nothing
while `:(icase)agents.md` finds `AGENTS.md`.

**[medium] The status sweep missed two more producers.** A failing `sed` in migration emptied the
path, which then failed the `docs/*` test and was classified droppable — the authority file would
have been archived having silently discarded a valid claim; normalisation failure is a conflict
now. And `canon`'s `ls | grep | head` observed `head`, so a failed listing looked like "no
case-variant found" and published the caller's wrong-case spelling as the record's identity. One
listing, its own status checked, failure fatal.

**[medium] `section()` matched headings by prefix.** `index($0,h)==1` also matched
`## Gate status-old` and `## Goal gate-archived`, so a document with a typo'd or renamed heading
still had its checkbox rows read as if the required section were present — the byte-frozen surface
enforcing nothing. Exact match, trailing whitespace the only tolerated difference. Verified: a
`## Gate status-old` section now yields zero lines.

**Lows, all fixed.** The legacy lock is released through one checked helper on EVERY path out of
`migrate`, not only the archival one. The secret guard's `.gitignore` exemption requires `-f` (a
FIFO passes `! -L` and makes `wc`/`cat` block forever). `status` no longer returns before listing
run captures when `active/` is absent, and enumerates hidden entries at both levels. `AGENTS.md`
states the real retention threshold (`-mtime +7` truncates to whole days, so removal starts at 8
complete days). A record with an empty `ts` — what a failing `date` produces — is refused rather
than published. `--help` shows the absolute invocation the file itself insists on.

Verified by direct run (repo policy: no TDD): `bash -n` on both artifacts; the hook against the
plain, `stop_hook_active`, and wrong-case-record cases plus an isolated `section()` probe;
`dstack status`, `--help`; both pinned checks green. Noted honestly: the wrong-case probe
OVERWROTE the real `GOAL.md` record, because the key is derived from the lowercased path and both
spellings share it — the record was re-registered immediately, and the incident is itself evidence
that the key-folding stance is doing what it claims.
