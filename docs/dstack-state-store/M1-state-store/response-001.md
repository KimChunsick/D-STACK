# Maintainer response — Round 001

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted. Two were bypasses of the exact fail-closed property
this milestone claims, and one of the lows was a defect I had already spotted myself but had
correctly declined to fix while the round's bundle was open.

**[high] CWD-vs-git-root state anchoring.** Confirmed and fixed. `dstack` resolved state at the
git root while the hook resolved it relative to its invocation CWD, so the hook found nothing
from any subdirectory and opened the gate on live work. The hook now resolves the root exactly
the way `dstack` does and `cd`s there first. Missing `git`, or a root it cannot enter, blocks.
Outside a repository it exits clean, which is consistent: `dstack` refuses to write a store
there, so there is nothing to gate. Verified: running the hook from `docs/dstack-state-store/`
returned `block` where it previously returned nothing.

**[high] Symlink checks only on the top `.dstack` path.** Confirmed and fixed. `.dstack` being a
real directory said nothing about `.dstack/active`, and neither said anything about the legacy
file. All three are now checked per-component, each record file is rejected if it is a symlink,
and a `.dstack` that exists but is not a directory blocks.

**[medium] Case folding on APFS.** Confirmed and fixed. The key is now the SHA-1 of the
LOWERCASED path and `assert_record` compares case-insensitively, so `docs/g/GOAL.md` and
`docs/g/goal.md` are one key with one owner. This is collision-conservative on a case-sensitive
volume — two genuinely distinct files would share a key and the second registration is refused,
not silently allowed — which is the same stance `check-parallel.sh` already takes on declared
file overlap. Verified: session B registering the other spelling now exits 3 naming session A.

**[medium] `reg` ignored the per-key lock on its existing-record path.** Confirmed and fixed.
That branch reads then reports, so a concurrent `unreg`/`reclaim` between the two made it claim
an ownership that no longer existed. It now takes the key lock, re-checks under it, and falls
through to a normal claim if the record was released while waiting. The creation path stays
lock-free because `ln` is the atomic decision there.

**[medium] Records neither read as one snapshot nor schema-validated.** Confirmed and fixed. The
record is read once into memory and every field comes from those same bytes, so a concurrent
reclaim can no longer pair one session's owner with another's document. `v`, `session`, and
`doc` are type- and version-checked; anything off-schema is reported as a bad record rather than
partially believed.

**[medium] Migration could discard ownership or create unaddressable records.** Confirmed, and
the fix exposed that my first attempt was in the wrong order. Canonicalising after the `docs/*`
test meant a legacy `./docs/...` line failed the prefix test and was **dropped as "not a docs/
path"** — silently discarding a live registration, exactly the failure named. Leading `./` and
repeated slashes are now normalised first, then the drop tests run, then the path is canonicalised
with the same function `reg`/`unreg` use so the key is reproducible. A path that survives the
drop tests but will not canonicalise is a conflict, not a drop. Verified: `./docs/g/GOAL.md`
plus `docs//g/GOAL.md` now correctly collide as one document (exit 4) where both were previously
dropped; a single such line migrates and is addressable by its canonical spelling.

**[medium] Missing/failing `jq` opened the fail-closed gate.** Confirmed and fixed. The verdict
was emitted with `jq`, so a missing `jq` meant no output and an open gate. Blocking no longer
depends on it: a `block()` helper falls back to a hand-written JSON string (deliberately free of
quotes and backslashes), and absent `jq` blocks immediately.

**[medium][DX] `~/.claude/bin` is not on `PATH`.** Confirmed and fixed in the documentation
rather than by editing the user's shell configuration: every documented invocation now uses the
absolute path, which also works in the non-interactive contexts where no rc file runs.

**Lows, all fixed.** `run-dir` rejected `.`/`..`/separators and now allocates a unique directory
per call (a repeated label silently mixed two rounds' bundles and reset the retention mtime);
the gate's closing message named the retired `.fullcycle-active` escape hatch and now names
`dstack unreg` (**self-found in the previous turn and deliberately not fixed then, because the
file was inside this round's open bundle — recording it here rather than voiding the round**);
`status` no longer prints "(none)" beneath a reported corrupt record; `--help` works outside a
git repository and every command validates its arity; `migrate` never clobbers an existing
`.migrated` archive; `unreg` and `prune` verify their deletions instead of reporting success
after a suppressed failure; `prune` now respects the cutover guard it was exempt from; a
document path ending in a newline is refused rather than silently rewritten by command
substitution, and the JSON-format rationale is narrowed accordingly — JSON was chosen for schema
evolution and safe escaping of the paths this accepts, never as a claim to accept every byte
string a filesystem allows.

**Class-wide sweep triggered by my own repair.** Fixing the record parser I wrote a `$'\0'` into
the hook, which landed as a literal NUL and turned the highest-blast-radius file in this
repository into a binary. Repaired, and swept every tracked shell and skill artifact for NUL
bytes: clean. The delimiter idea was abandoned rather than patched — there is no byte a document
path cannot contain that command substitution also preserves.

Round 1 defect count is high enough to be worth stating plainly: the design consult caught the
structural problems, and this round caught the implementation ones. Both were needed.
