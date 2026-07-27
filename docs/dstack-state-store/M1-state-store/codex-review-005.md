# Codex adversarial review — Round 005

## Review scope
Re-review (Rounds 1-4 rejected; all findings accepted and fixed)

## GPT findings
[severity:high][technical correctness] The Round-4 digest repair still lacks a quiescent cutover: the final digest and `mv` are separate operations, so a late legacy claim can still be archived without publication.

Evidence: `cmd_migrate` compares the digest and then executes `mv` with no exclusion or validation covering the interval after the comparison.

Verification: A legacy writer can open the file for append, migration can pass the second digest and rename it, and the writer can then append through its open descriptor; no active record or newly recreated legacy path exposes that claim.

Suggested direction: Require verified legacy-writer quiescence, or maintain an explicit in-progress authority while stabilizing and processing both the detached inode and any recreated legacy path.

[severity:medium][technical correctness] The schema-marker repair is bypassed when `.dstack/active` is absent, because the hook exits successfully before validating `.dstack/version`.

Sites: Primary: `fullcycle-gate.sh` store discovery; confirmed: `dstack.cmd_status` and `ensure_store`, which reject the same store.

Evidence: The no-`active` branch exits at lines 137–140; marker validation begins only afterward.

Verification: `.dstack/version` containing `2\n` with no `active/` makes the hook exit 0 while `dstack status` rejects the unsupported schema.

Suggested direction: Validate an existing store’s marker and required namespace before treating absent `active/` as “nothing registered.”

[severity:medium][technical correctness] The claimed complete-record invariant still diverges: the hook filters foreign ownership before checking the final document’s existence or symlink status, while the CLI validates those properties for every record.

Sites: Primary: `fullcycle-gate.sh` record scan; confirmed: `dstack.read_record`, `cmd_status`, `cmd_unreg`, `cmd_reclaim`, and migration preflight.

Evidence: The hook’s foreign-owner `continue` precedes its `-L`/`-f` checks; it also accepts empty owners and omits the CLI’s printable-ASCII canonicalization rule.

Verification: A self-keyed foreign record for missing `docs/missing.md` passes the hook’s parent-canonical check and is skipped, so the gate exits 0; `dstack status` reports the same record invalid and mutations refuse it.

Suggested direction: Apply the full document, owner, and representability invariant before ownership filtering.

[severity:medium][technical correctness] The new “prove repository absence” fallback itself fails open when upward traversal cannot continue.

Evidence: `nd="$(dirname -- "$d")" || break` is followed by unconditional `exit 0`; `dirname` is neither checked as a dependency nor converted into a block.

Verification: An isolated execution of the exact control flow produced `fallback-result=allow` when the traversal command failed before reaching an ancestor `.git`.

Suggested direction: Any traversal failure or non-progress condition other than reaching `/` must call `block()`.

[severity:medium][technical correctness] Dependency-output validation remains incomplete: record generators accept arbitrary status-zero `jq` output, and hash pipelines still observe only the digest program’s status while masking `tr` failure.

Sites: Primary: `dstack.cmd_reg`; confirmed: `cmd_reclaim`, `cmd_migrate`, `sha1_try`, and the hook’s key derivation.

Evidence: The three `jq -cn` writers publish without reparsing their output; both hash paths retain `printf | tr | digest`.

Verification: A status-zero `jq` emitting `not-json` satisfies the publication predicate; a missing or failing `tr` can still yield the valid SHA-1 of an empty stream from the successful digest command.

Suggested direction: Validate generated record semantics before publication and capture every fallible pipeline stage’s status independently.

[severity:medium][security] Recovery-command quoting is still injectable: valid document paths are inserted inside unescaped single quotes, and the lock timeout prints an unquoted `rm -rf` path.

Sites: Primary: `dstack.cmd_reg`; confirmed: `klock`.

Evidence: `canon` permits quotes, semicolons, dollar signs, and spaces, while the recovery message renders `reclaim '$doc'`; repository roots are likewise unrestricted in the lock path.

Verification: A valid path containing `docs/x'; touch REVIEW_PWN; echo '.md` renders as `reclaim 'docs/x'; touch REVIEW_PWN; echo '.md'`, executing the inserted command if copied.

Suggested direction: Render every dynamic command argument with Bash `%q`, and recommend quoted `rmdir --` rather than raw `rm -rf` for an expected-empty lock.

[severity:low][UI & UX / DX] `status` still hides arbitrary dot-prefixed active entries even though the hook blocks on them.

Evidence: `cmd_status` enumerates only `"$ACTIVE"/*`; the hook additionally enumerates both hidden-entry glob classes.

Verification: Shell globbing confirmed hidden entries are excluded, so `.bad` can make the hook block while `status` prints `(none)`.

[severity:low][technical correctness] The promised byte-exact `.dstack/.gitignore` validation is not byte-exact.

Sites: Primary: `dstack.ensure_store`; confirmed: `tests/secret-guard.sh` exemption.

Evidence: Both compare `$(cat file)` with `*`, and command substitution removes all trailing newlines.

Verification: A file containing `*\n\n\n` was accepted as equal to `*`.

[severity:low][technical correctness] The `run-dir` permission-failure repair can still falsely claim that its label was released.

Evidence: The failure branch suppresses `rmdir "$d"` errors and unconditionally emits `(label '$label' released)`.

Verification: If the new directory becomes nonempty or otherwise unremovable before cleanup, the command fails while leaving the label occupied.

[severity:low][technical correctness] `status` does not fully tolerate a record disappearing during inspection.

Evidence: It checks existence before `read_record`, but deletion after `read_record`’s `-f` test and before `cat` becomes “cannot be read.”

Verification: A concurrent `unreg` in that interval makes a successful deregistration appear as an invalid active record.

[severity:low][technical correctness] On a case-sensitive volume, `canon` aliases distinct case-variant files instead of merely making their keys collide.

Evidence: `ls -1 | grep -ixF | head -1` selects the first case-insensitive match even when the caller’s exact file also exists.

Verification: With `A.md` and `a.md`, C-locale ordering selects `A.md` for either spelling, so `a.md` cannot retain its physical identity.

[severity:low][technical correctness] Retention uses `find -mtime +7`, so deletion begins only after eight complete 24-hour periods rather than after the documented seven days.

Omitted-detail: 1 low

GPT verdict: reject — migration can still silently discard a concurrent legacy claim, and five additional fail-closed or command-safety defects remain genuinely blocking.

## Carried decisions — Round 005
Rounds 1-4 decisions stand. Added in Round 5:

- **Exclusion beats detection.** When migrating away from a protocol, take THAT protocol's lock;
  digesting before and after is a net, not a guarantee.
- **Validate the store before taking any shortcut through it.** "Nothing registered" is a
  conclusion, not an early exit.
- **Apply a shared invariant before any filter**, or the two tools sharing it will disagree
  about the same bytes.
- **A fallback that fails open is the original defect, one level down.**
- **Every stage of a pipeline gets its own status**, and every generated record is read back
  before it is published.
- **Anything a human may copy and run gets `%q`.** A filename is not a safe shell word.

Consensus: disagreed
