# Maintainer response — Round 009

Outside the reviewed corpus by design. Six blocking findings fixed, two lows fixed, the rest
carried. This is the closing round — see `codex-review-009.md` for the measurement.

**[high] A lock-release trap could delete a SUCCESSOR's lock.** `kunlock` and
`release_legacy_lock` both did `rmdir` first and `trap - EXIT INT TERM` second. Between those two
statements the lock pathname is free while the traps are still armed, so a signal arriving in that
window ran a trap that removed whatever now sat at that path — by then possibly another process's
freshly acquired lock, letting a third enter mid-`reclaim` or mid-`migrate`. Traps carry no
ownership, so the fix is to not have one armed once the name can belong to someone else: disarm
first, then rmdir. Accepted residual, written into the code: a signal landing between the disarm
and the rmdir leaves a stale lock behind. That direction is the safe one — a stale lock blocks
loudly and says how to clear it; a stolen lock corrupts silently.

**[medium] `ls -1` hid dot-prefixed documents, and an empty spelling match was ignored.** Two
defects in one place. `reg docs/.task.md` fell through `canon`'s spelling resolution with the
caller's spelling intact, while the Stop hook — which requires an exact match from the SAME
listing — could not find it and blocked the record as wrongly spelled. Both listings gained `-a`
in the same change, and the `[ -n "$real" ] && b="$real"` fallthrough became a hard failure plus
an ambiguity check. Verified: `reg docs/.hidden.md` → `status` lists it → `unreg` releases it.

**[medium] The heading match accepted any suffix, and terminated on too little.** These are the
same function and two opposite bugs. An exact-only match once broke the live gate (the real
heading is `## Goal gate (Stop-hook enforced — …)`); the whole-token repair swung too far and let
`## Gate status archived` satisfy the required section. And termination fired only on `/^## /`, so
`##<TAB>Archive` did not end the section and its `- [x] GOAL E2E` leaked in as gate content.
Now: exact heading or the documented `(` parenthetical, terminating on ANY ATX heading with
CommonMark's permitted indentation. Terminating on any heading can only make the section smaller,
never leak later rows in. Verified both directions — the live `GOAL.md` still yields its three
gate rows, and the tab fixture now yields only the real one.

**[medium] Recovery commands rendered as one single-quoted word.** `'/Users/…/dstack migrate'`
copy-pasted is a search for one filename containing a space. Every site now renders
`"$HOME/.claude/bin/dstack" migrate` — quoted executable, arguments outside.

**[medium] A deleted or renamed document could not be released by any command.** `read_record`
requires the document to be an existing regular file, deliberately, so a record for a deleted
document is invalid — and `unreg` died on that same invariant. The gate then blocked forever on a
file nobody could tick and no command could release. Two additions: `stale_record_ok` accepts
EXACTLY that case (every other invariant still holds, including "the filename is the key of its
own doc", so it can only match a record whose document is genuinely gone), and `unreg` now accepts
a 40-hex record key as well as a path, because a removed parent directory or a case-only rename
makes `canon` derive a different key or fail outright. `status` already prints the key of every
record it calls invalid, so a handle always exists. Verified: register → delete → `unreg <path>`
releases with a stale-record note; register → `rm -rf` the parent → `unreg <key>` releases;
another session's stale record is still refused with exit 3.

**[medium] Fatal git discovery could still open the gate.** With `GIT_DIR` pointing at external
metadata there is no in-tree `.git` for the walk to find, so a broken `GIT_DIR` gave status 128,
the walk found nothing, and the gate opened inside a live worktree. Explicit git environment is
now itself evidence a repository was intended, and an ancestor `.dstack` store counts too — this
pipeline's own record that work is registered there. Verified: `GIT_DIR=/definitely/missing`
blocks; a bare directory containing only `.dstack/` blocks.

**Also fixed here, from Round 008's still-open blockers.** Two of that round's three were believed
closed and were not — `codex-review-008.md` now records that correction rather than the version I
believed at the time. (a) The active-record scan globbed a directory proved only by `-d`; over an
UNREADABLE directory the glob stays literal, every `-e` fails, the loop body never runs, and zero
entries reads as "nothing is registered". The traversal is now proved before an empty scan is
believed. Verified: `chmod 000 .dstack/active` blocks; readable-and-empty still opens. (b) The
milestone sweep ran as one `$(grep | grep | grep | tr | sort)`, whose status is `sort`'s, so a
failed READ of the Goal document produced zero milestones and enforced no milestone gate at all.
Each producer's status is checked now, with grep status 1 kept as a legitimate "no milestones".

**[low] A FIFO at the legacy registry path read as absent.** `-s` is false for a FIFO, a device
node, or a directory, so a non-regular thing occupying `.fullcycle-active` passed the cutover
check here while `dstack status` refused the same state through `require_plain`. Verified: a FIFO
there now blocks.

**[low] The seven-day pruning claim.** `find -mtime +7` removes a capture once it is eight
complete days old. `AGENTS.md` was corrected earlier; `02-dstack-cli/task.md` still said seven and
now says the same thing as the code.

**Carried, with evidence, in `task.md` and `findings.md`:** terminal-control bytes in diagnostics,
inconsistent timestamp validation, `status` returning 0 despite invalid records, migration
refusing losslessly-collapsible duplicate legacy lines, an ignored temp-link removal failure, the
`cat`-race record reported as corruption rather than a deregistration, and the legacy-lock
cleanup that stays silent on `die` paths. All low, none carrying a demonstrated failure that
costs correctness — and all recorded rather than dropped.
