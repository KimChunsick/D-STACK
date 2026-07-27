# Maintainer response — Round 003

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted. What changed, by finding:

**[high] Dynamic paths still followed symlinks.** `require_plain` now covers the parts of the
path that VARY, not just the ones spelled out in a constant: `$RUNS/$SID` and the run leaf in
`cmd_run_dir`, and `$LEGACY` in `cmd_migrate` (which reads that file and echoes its contents back
in conflict messages). Record reads go through the new `read_record`, which rejects a symlinked
record before opening it.

**[high] The record invariant was split across partial checks.** New `read_record` in `dstack`
enforces the WHOLE thing at once: not a symlink, a regular file, a 40-hex filename, the v1 schema,
an owner matching the session grammar, a `docs/` path, the filename equal to the SHA-1 key of its
OWN `doc`, and a document that exists and is not a symlink. `assert_record`, `cmd_status` and
`cmd_migrate`'s preflight all call it. The Stop hook carries the same predicate inline — separate
implementations on purpose, so the tripwire never depends on the CLI being installed; both copies
say so and name each other.

This one found a live defect in this very repository. All three active records were named with
the SHA-1 of the UN-lowercased path, because Round 2's "fold the key to lowercase" fix changed the
key derivation without migrating what was already stored. They were exactly the failure the
finding describes: `unreg` could not address them, so nobody could release them, while the gate
went on enforcing them. Repaired by deleting the orphans and re-registering. Recorded rather than
quietly fixed, because "a schema change orphans existing records" is the general lesson.

**[high] The hook trusted its own session id.** A malformed `CLAUDE_CODE_SESSION_ID` is now
treated exactly like an empty one — unknown identity, enforce every record. Previously any
nonempty value took part in the foreign-owner comparison, so `bad/slash` was "not equal to" every
valid owner and skipped everything.

**[medium] `block()` validated emission, not semantics.** It now re-parses the JSON it is about
to print and requires `.decision == "block"`. A jq that exits 0 while printing `not-json` used to
pass, printing a non-verdict that opens the gate.

**[medium] ASCII-only case folding on a Unicode-folding filesystem.** Refused rather than
papered over: `canon` rejects any byte outside printable ASCII. Portable Unicode case folding is
not available in bash 3.2, so claiming one identity per physical file for non-ASCII paths would
be a guarantee this cannot keep. The same check subsumes the separate low finding about a
newline in a DIRECTORY component, which the old trailing-only test could not see.

**[medium] `.dstack` self-isolation was assumed, not enforced.** `ensure_store` now requires
`.dstack/.gitignore` to contain exactly `*`, fails hard on read/write/chmod failures, and checks
`git ls-files`' STATUS separately (the old `$(… | head -1)` reported `head`'s status, so a failing
git was read as "untracked").

**[low] `prune` verified a traversal it does not use.** Both depth-two scans now run unpiped with
their own status checked; counting happens afterwards on the captured text.

**[low] `migrate` demanded a session id it never used.** Removed — the command carries the owners
written in the legacy file, and requiring a Claude session made the Stop hook's own recovery
instruction unrunnable from a terminal.

**[low] `*goal.md` matched too much.** The gate classifies on the exact BASENAME now, so
`notgoal.md` and `my-goal.md` are tasks again.

**[low] Angle brackets rejected for a dead reason.** The `<path>` delimiter set is gone (dedupe
is on record keys), so the check went with it.

**[low] Version marker read only its first line.** Whole-file comparison now.

**[low] Usage errors masked by environment errors.** Arity and unknown-command rejection moved
ahead of dependency and repository discovery, so `dstack status extra` outside a repo reports the
usage mistake the caller actually made.

Verified by direct run (repo policy: no TDD): `bash -n` on both files; `dstack --help` outside a
repo; `dstack status extra` and `dstack bogus` from `/tmp`; `dstack reg docs/É/task.md` refused;
`dstack status` before and after the orphan repair; the hook against four crafted stdin fixtures
(plain, `stop_hook_active:true`, malformed session id, and a planted record whose key does not
match its own doc — reported as `key-does-not-match-its-own-doc`).
