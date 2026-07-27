## Design verdict: rework

The per-record directory is a good foundation, but the proposed protocol is not concurrency-safe for same-key operations, cannot determine which records are reclaimable, and cannot migrate the legacy ownership model without losing information.

### Blocking structural failures

1. **Atomic rename does not provide atomic conflict resolution.**

   Two sessions can register the same document:

   - Both read or assume the key is available.
   - Both publish complete temporary files.
   - Both rename successfully, with the later rename replacing the earlier record.
   - Both commands may report success, although only one registration survives.

   The losing session can then pass its Stop hook because its ownership silently disappeared. Similarly, `unreg` can verify that it owns a record, race with another registration, and then delete the replacement record.

   One-file-per-key eliminates coordination between distinct keys only. Same-key `reg`, owner-checked `unreg`, and `reclaim` require create-if-absent or compare-and-swap semantics. Migration requires repository-wide serialization. “No lock directory at all” is incompatible with the stated ownership guarantees unless another atomic conflict primitive is specified.

2. **“Last writer wins” contradicts exclusive ownership.**

   If ownership is exclusive, a registration by the current owner should be idempotent and a registration by another owner should fail visibly. Silent replacement means successful registration is not a durable assertion.

   A legitimate two-tab workflow also regresses: under the legacy design, both owners were independently blocked by the document’s gates. Under this design, the overwritten owner becomes unconstrained without being told. An explicit takeover operation could support the accepted one-owner model; ordinary `reg` must not perform that takeover implicitly.

3. **`reclaim` has no way to identify an abandoned owner.**

   Without heartbeats, a session registry, or another liveness source, “not owned by this session” is not equivalent to “owned by no current session.” The proposed command therefore steals every other live tab’s records.

   If two tabs run it, they can repeatedly replace one another’s ownership and both report success. `reclaim` must either be narrowed to explicit documents or a named previous owner, guarded by an expected-owner comparison, or removed. Its current semantics cannot be made safe from the available information.

4. **Migration cannot faithfully map the legacy state.**

   The legacy file can contain:

   - Multiple owners for the same document.
   - Untagged or empty-owner records that are enforced by every session.
   - Malformed records.
   - New records appended by an older tab during or after migration.

   The new format represents none of the first three cases unambiguously. Selecting one owner silently would weaken enforcement. Migration must fail with an explicit conflict report unless each record has a lossless mapping.

   Having only the Stop hook reject a non-empty legacy file is insufficient. Every mutating command must respect the cutover state. Otherwise `reg` can create new state while `migrate` is taking its snapshot. A migration also cannot safely finalize while older helpers remain capable of recreating or appending to the legacy registry. The design needs a repository-wide authority/version marker and a quiescent cutover rule.

5. **Path identity is undefined.**

   Hashing the supplied argument allows the same document to acquire several keys:

   - `doc.md` versus `./doc.md`
   - Relative versus absolute paths
   - Symlinked versus physical paths
   - Case variants on case-insensitive APFS
   - Unicode normalization variants
   - A path supplied from a different working directory

   This defeats the one-document/one-owner invariant and makes `unreg` dependent on reproducing the original spelling. The design must define repository-root discovery, whether paths outside the repository are allowed, lexical versus physical identity, case and Unicode handling, and behavior after a document is renamed or deleted.

### State-format problems

- **SHA-1 filenames are acceptable as opaque keys**, and fixed lowercase hexadecimal names do not introduce an APFS case-folding problem. A reversible filename encoding is worse for long paths, filename-length limits, and escaping.
- Every lookup must nevertheless compare the stored canonical path with the requested canonical path. A hash collision or corrupt record must fail loudly rather than operate on the wrong document.
- Bash 3.2 cannot calculate SHA-1 itself. The design must name the digest utility as a dependency; “bash plus jq and no other runtime” does not currently account for it.
- The two-line record format cannot represent a legal filename containing a newline and offers no schema evolution. Since `jq` is already required, records should have an escaped, versioned structure with explicit fields.
- Session IDs are used as directory names but have no specified grammar. An unexpected slash, `..`, newline, or empty value can redirect or corrupt state.
- Same-directory temporary publication can still strand temporary files after interruption. Readers need an exact record-name grammar and a defined policy for incomplete, malformed, and stale temporary entries.
- Atomic rename guarantees visibility, not power-loss durability. If survival across a machine crash matters, the durability contract needs to say so; bash does not expose the required file-and-directory synchronization cleanly.

A schema/version marker is important before committing this layout. Without one, future changes to hashing, serialization, ownership, or run metadata will require another ambiguous migration.

### Run-state failures

`runs/<session-id>/` cannot represent multiple concurrent or sequential runs from one session, and captured input/output alone cannot distinguish:

- A live run
- A completed run
- A process that crashed
- A producer that created the directory but has not created its first file

Consequently, `status` cannot truthfully report “in-flight,” and empty-directory cleanup can remove a directory in the gap between its creation and first write. Each run needs a distinct identity and explicit lifecycle state. With no liveness signal, a non-terminal run surviving a crash must be reported as “unknown/abandoned,” not “in-flight.”

Raw bundles also need bounded retention. A reasonable default contract is:

- Delete successful-run input and output immediately after the result is consumed.
- Retain failed or interrupted bundles for a short fixed period, such as seven days.
- Enforce a per-repository size cap.
- Require explicit opt-in to preserve a bundle longer.

Gitignore status is not a confidentiality boundary. These files can enter backups, synced folders, filesystem snapshots, diagnostic archives, or manual repository copies. The directory and files need restrictive permissions, and repositories stored in externally synchronized locations may require the run bundles to live outside the repository entirely.

### Git-ignore assessment

A nested `.dstack/.gitignore` containing `*` generally works. If a parent ignores `.dstack`, Git need not descend because the directory is already ignored; if a parent re-includes it, the lower-level `.gitignore` normally overrides parent patterns for its descendants. An allowlist-style root ignore is therefore not inherently a problem.

It does not cover every relevant case:

- Already tracked `.dstack` paths remain tracked.
- `git add -f` can add the files.
- `git clean -fdx` can delete all runtime state.
- A cloned repository can already contain a `.dstack` file, directory, or symlink.
- Removing the nested ignore file can expose state in repositories without a parent exclusion.

The CLI must refuse an occupied or tracked namespace rather than merge with it. In particular, following a pre-existing `.dstack` symlink could cause writes or deletions outside the repository.

### Bash suitability

Bash is adequate for a small filesystem-backed CLI if the protocol uses operations that POSIX shell utilities can express. It becomes the wrong tool if the design insists on lock-free conditional replacement, conditional deletion, multi-record migration transactions, or strict crash durability. Bash 3.2 provides no direct compare-and-swap, rename-without-replacement, or directory-fsync interface.

The choice is structural: admit narrowly scoped serialization or redesign state transitions around immutable atomic claims. Retaining both “no locks” and owner-checked mutable records leaves unavoidable races.

### Untrusted-input finding

The supplied design payload contains evaluator-control directives, including exclusions on what may be challenged and a required output ending. Mixing such directives into design data can suppress valid findings in automated consultation; they must be separated from the artifact or treated solely as untrusted content.

Design verdict: rework — the directory layout is viable, but ownership, reclamation, migration, path identity, and run lifecycle lack enforceable state-transition semantics.