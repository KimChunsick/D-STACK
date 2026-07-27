# Maintainer response — Round 002

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Fifteen findings, all accepted. The shape of this round matters more than the count: three of
the blockers exist because I applied a Round-1 fix to ONE of two sibling sites. That is the
class-wide sweep this skill's Step 0 demands, and I did not do it — twice, since M2's review
caught the same discipline failure independently.

**[high] Nested symlinks: the repair landed in the hook, not the CLI.** Confirmed. `ensure_store`
checked `.dstack` and then followed `active`, `runs`, `.gitignore`, and `version`; `migrate`
followed the legacy file; `status` stat'd it with `-s`, which also follows. Replaced with one
`require_plain <path> dir|file` helper applied to every component, and it runs BEFORE `mkdir -p`
— through a symlink whose target does not exist, `mkdir -p` would have created that target
outside the repository and only then been noticed. Verified: `active -> outside dir`,
`version -> /etc/hosts`, `runs -> /tmp`, and a symlinked legacy file are each refused by `reg`,
`status`, and `run-dir`.

**[high] The hook silently ignored malformed registry state.** Confirmed on every count. A
regular-file `active` exited 0 (a malformed namespace opened the gate); hidden entries were
never enumerated; `-f` was tested before `-L`, so a dangling record symlink was skipped instead
of reported; and a nonempty-but-malformed owner passed the schema check and then took the
foreign-owner branch, making it unattributable state nobody enforced. All fixed: non-directory
`active` blocks, the glob now covers dot-prefixed entries, `-L` is tested first, the filename
must be a 40-hex key, and the owner must satisfy the same grammar `dstack` enforces. Verified
one case at a time — regular-file `active`, hidden junk, dangling symlink, `session:"bad/slash"`,
and a non-key filename each produce `block`, and removing them returns `ALLOW`.

**[high] Dependency failures were fail-open or destructive.** Confirmed. `block()` called `jq`
without checking its status, so a present-but-failing `jq` emitted nothing and the gate opened;
it now verifies the emission produced output and otherwise prints a static block string. The
hook classified every `git rev-parse` failure as "outside a repository"; it now distinguishes
git's exit 128 (genuinely no repository, nothing to gate) from any other status, which blocks.
The CLI left the physical root and the SHA output unchecked — an empty root would place state at
`/.dstack`, and an empty digest would collapse a record path onto `active/` itself, which
migration then reads as "already present" before archiving the legacy source. Both are validated
now (absolute root; 40-hex digest or hard failure).

**[medium] Migration could still archive away an owner.** Confirmed — the Round-1 counterexample
survived because the existing-key check sat in the publish loop and treated any existing record
as "already present". Moved into the preflight, where an existing key must match this record
exactly (same document, same owner) or it is a conflict. Verified: legacy owner B against an
existing owner-A record now exits 4 and leaves the legacy file in place.

**[medium] Lowercasing the key did not canonicalise the stored spelling.** Confirmed, and the
consequence was concrete: the gate classifies Goals by matching `GOAL.md`, so `goal.md` on this
case-insensitive volume registered as a task and the one-Goal rule silently stopped applying.
`canon` now resolves the real on-disk spelling of the final component, and the hook's
classification is case-insensitive as a backstop for stores written by an older build. Verified:
registering `docs/g/goal.md` stores `docs/g/GOAL.md`.

**[medium] `reg`'s failed-`ln` branch read ownership outside the lock.** Confirmed; the Round-1
fix covered only the existing-record branch. That branch now takes the key lock and revalidates
before reporting anything.

**[medium] PATH.** Confirmed and swept: the Stop hook's remediation messages and `AGENTS.md` both
still named a bare `dstack`, which resolves to nothing in the setup those very documents
describe. All corrected to the absolute path.

**[medium] The `<doc>` sentinel dedupe.** Confirmed. An accepted path containing the delimiter
sequence could mask a distinct registered Goal, letting a completed first Goal hide an incomplete
second. Dedupe now keys on the record filename — which IS the key — and `canon` rejects angle
brackets outright.

**[medium] `run-dir` check-then-create.** Confirmed. Replaced `-e` plus `mkdir -p` with
`mkdir -p` on the parent and a plain `mkdir` on the leaf as the atomic claim. Verified: a taken
label now fails loudly.

**Lows, all fixed.** `status` now applies the same schema predicate the hook does (a `v:999`
record was listed as healthy while the gate called it unreadable); `prune` verifies traversal
instead of pipelining into `wc` and reporting a clean sweep it never made; the archive
non-clobber loop also tests `-L`, since `-e` is false for a dangling symlink and `mv` would have
followed it; `--help` rejects extra arguments; a dot-prefixed run label is refused because
`status` would never list it; and the retention message now says "strictly older than N full
days" rather than implying the boundary is exact.

**A regression I introduced and repaired mid-round.** Rewriting the record parser, I wrote a
`$'\0'` into the hook, which landed as a literal NUL and made the highest-blast-radius file in
this repository binary. Repaired, and every tracked shell and skill artifact swept for NUL bytes
(clean). The delimiter idea was abandoned rather than patched: there is no byte a document path
cannot contain that command substitution also preserves.
