# Maintainer response — Round 008

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The bundle still did not bind to the committed identity.** Two mistakes in the Round-7
repair. `REVIEW_BASE` existed only inside the assembler and the launch recipe never set it, so a
worker review defaulted to `git diff HEAD` and contributed zero implementation bytes. And
`git diff <commit> -- <path>` compares that commit against the WORKING TREE, not another commit,
so even set by hand it did not produce the range it claimed — the reviewer measured 14,200 bytes
base-to-worktree against 12,024 base-to-HEAD on this checkout, which is a post-assembly dirty edit
silently replacing what the merge later carries. `REVIEW_BASE` and `REVIEW_HEAD` are now required
together, both validated as commits with base an ancestor of head, the recorded head must equal
the checked-out HEAD, the tree must be clean, and the diff is two-tree. Verified: base alone
refused by name, a dirty tree refused with both commit ids printed, serial case unchanged.

**[medium] The merge/seal cycle was mine.** Round 7's repair said merge is gated on unit consensus
AND every owned branch merges before that unit's round seals — sealing waiting on merges waiting
on sealing. Split into two steps that cannot circle: `integrate` merges every worker branch the
unit owns into one integration branch off the recorded base, gated on checker scope and NOT on
review, and what that produces IS the reviewed identity; `merge` is landing that integration head
on the mainline, and THAT is what consensus gates. Serialization in Step 3 now says review UNIT
rather than task, with the round-file race spelled out.

**Lows, all fixed.** Cleanup reads its labels from `dstack status` instead of retyping them —
`rm-run` is idempotent by design, so a typo in a hand-written list reported "cleaned up" while the
capture stayed. The round-file write is checked before the launcher claims success. `grep`'s exit
2 (read error) is no longer collapsed into "no blocking findings". The schema check has one trap
armed before the first temp dir, since a normal-path `rm -rf` does nothing when an assertion exits
early. And the size accounting was corrected AGAIN, this time with its measurement round attached
and the command to re-take it — the number kept going stale because later rounds keep editing that
file, which is exactly why a bare figure was the wrong thing to record.

**Declarations.** The assembler is now declared in T04's `files`, and both undeclared executable
surfaces this task's own findings created — `dstack rm-run` and the `REVIEW_BASE`/`REVIEW_HEAD`
contract — are named in the task record instead of being denied by a stale "no new API" line.

Verified by direct run (repo policy: no TDD): `bash -n` on the assembler; base-without-head
refused, dirty-tree refused, serial assembly unchanged at 70,966 bytes; `skill-schema.test.sh`
green — it caught a YAML break in my own new scheduling text, where an unquoted `identity: exactly`
made ruby read a mapping; `tests/secret-guard.sh` green.
