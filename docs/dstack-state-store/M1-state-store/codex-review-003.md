# Codex adversarial review — Round 003

## Review scope
Re-review (Rounds 1-2 rejected; all findings accepted and fixed)

## GPT findings
Verification note: the supplied workspace is empty and not a Git repository, so repository checks were unavailable; verification used the supplied snapshots and isolated shell probes.

[severity:high][security] The Round-2 nested-symlink repair still stops at fixed store components; dynamic legacy, record, and session paths remain unsafe.
Sites: Primary: `dstack.cmd_migrate`; confirmed: `cmd_run_dir`, `cmd_status`, `cmd_reg`, `cmd_unreg`, and `cmd_reclaim`.
Evidence: `cmd_migrate` never calls `require_plain "$LEGACY" file`; `cmd_run_dir` follows an unchecked `$RUNS/$SID`; active-record readers use `-f` or `jq`, which follow symlinks.
Verification: A legacy symlink is read and may be echoed as a conflict; `runs/<SID> -> outside-dir` redirects capture creation; an active-record symlink is read by CLI commands.
Suggested direction: Apply the no-symlink/type invariant to the legacy source and every dynamic namespace child before access.

[severity:high][technical correctness] The complete-record repair remains partial: plausible JSON with an invalid key or document identity is accepted or silently ignored instead of blocking.
Sites: Primary: `fullcycle-gate.sh` record scan; confirmed: `dstack.cmd_status`, `assert_record`, and `cmd_migrate` preflight.
Evidence: The hook never recomputes the filename hash or canonicalizes `doc`, and silently continues for missing, symlinked, or non-`docs/` documents; CLI readers validate only subsets of the record.
Verification: A current-owned, 40-hex-named record pointing to a missing document leaves `bad`, `goals`, and `tasks` empty, so the hook exits 0; an existing `v:999` record with matching owner/doc passes migration preflight and replaces the valid legacy authority.
Suggested direction: Enforce the complete filename/key/schema/session/canonical-document invariant independently at every hook and CLI read site.

[severity:high][technical correctness] The hook validates stored owners but not its current `CLAUDE_CODE_SESSION_ID`, so malformed local identity opens the gate.
Evidence: Any nonempty `sid` participates in the foreign-owner skip, even when it violates the CLI’s `[A-Za-z0-9_-]+` grammar.
Verification: With `sid=bad/slash` and `owner=valid_owner`, the implemented predicate skips the record; repeating this for each valid owner leaves no enforced documents.
Suggested direction: Treat an empty or malformed current session ID as unattributable and enforce all records, or block immediately.

[severity:medium][technical correctness] The `jq` failure repair checks only nonempty output, not valid blocking JSON, and later extraction failures are unchecked.
Sites: Primary: `fullcycle-gate.sh.block`; confirmed: record `owner` and `doc` extraction.
Evidence: A status-0 `jq` result of `not-json` satisfies `[ -n "$out" ]`, is printed, and exits 0; failed field assignments can produce an empty `doc` that is silently skipped.
Verification: An isolated probe of the exact `block()` predicate returned `block-output=not-json`; Claude requires valid JSON containing `decision:"block"` for a Stop decision. [Claude Code hooks reference](https://code.claude.com/docs/en/hooks)
Suggested direction: Validate status and semantic output for every `jq` call, falling back to static blocking JSON on any mismatch.

[severity:medium][technical correctness] ASCII-only case folding still permits two keys for one physical non-ASCII path on the declared case-insensitive APFS volume.
Sites: Primary: `dstack.canon`; confirmed: `sha1` and `same_doc`.
Evidence: Global `LC_ALL=C` makes `grep -i` and `tr '[:upper:]' '[:lower:]'` ASCII-only, while case-insensitive APFS implements Unicode case insensitivity. [Apple APFS FAQ](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html)
Verification: The shell probe left `É` unchanged and unequal to `é`; therefore caller spellings can survive canonicalization and hash differently. This is Unicode case, not the accepted normalization residual.
Suggested direction: Use filesystem-authoritative Unicode-aware identity, or explicitly refuse non-ASCII document paths.

[severity:medium][security] `.dstack` self-isolation is not enforced: an existing arbitrary `.gitignore` is accepted and metadata-creation failures do not stop callers.
Sites: Primary: `dstack.ensure_store`; confirmed: its tracked-namespace check and every mutating command calling it.
Evidence: `require_plain` checks only file type, `[ -f "$STORE/.gitignore" ]` skips content validation, writes are unchecked, and `git ls-files | head` masks Git failure without `pipefail`.
Verification: An empty regular `.dstack/.gitignore` plus a valid version lets `reg` create visible runtime records, contradicting “never committed”; a failed Git lookup is likewise interpreted as “untracked.”
Suggested direction: Require exact metadata content and fail hard on Git, write, and permission-setting failures before creating records.

[severity:low][technical correctness] The Round-2 `prune` traversal fix still does not test the traversal that deletion actually needs.
Evidence: The guard searches only depth one; both depth-two searches suppress errors and pipe into successful `wc`.
Verification: An unreadable session directory passes the first `find`, disappears from both later counts, and leaves captures while reporting zero leftovers.
Suggested direction: Capture and check the status of each depth-two traversal before reporting success.

[severity:low][UI & UX / DX] `migrate` unnecessarily requires a Claude session ID, so the prescribed recovery command fails in an ordinary terminal or Codex invocation.
Evidence: `cmd_migrate` calls `require_sid` but never uses `SID`; the hook instructs the user to run this command to clear cutover.

[severity:low][technical correctness] Goal classification matches any basename ending in `goal.md`, not the exact `GOAL.md` convention.
Evidence: The pattern is `*goal.md`.
Verification: The isolated predicate classified both `docs/x/GOAL.md` and `docs/x/notgoal.md` as Goals, allowing a task-like document to satisfy the one-Goal structural count.

[severity:low][technical correctness] Newlines remain mishandled when a directory component—not the entire path—ends in a newline.
Evidence: `dirname` output is captured through command substitution, which strips trailing newlines; only a newline at the end of the complete input is rejected.
Verification: A path under a directory named with a trailing newline is canonicalized through the different newline-free directory when that directory also exists.

[severity:low][UI & UX / DX] Angle brackets are still rejected for a rationale the Round-2 repair removed.
Evidence: `canon` says the hook uses `<path>` delimiters, but the hook now deduplicates on record keys.
Verification: A valid documentation filename containing `<` or `>` is refused despite no remaining delimiter collision.

[severity:low][technical correctness] The version marker accepts arbitrary trailing lines because only its first line is read.

[severity:low][UI & UX / DX] Non-help command usage errors are masked by dependency and repository discovery because arity validation occurs afterward.

Omitted-detail: 2 low

GPT verdict: reject — Round-2’s namespace and fail-closed repairs remain incomplete, leaving reproducible external-path access, malformed-state gate bypasses, and ownership-identity failures.

## Carried decisions — Round 003
Rounds 1-2 decisions stand. Added in Round 3:

- **One invariant, enforced whole, at every site.** Partial checks that disagree are worse than
  no check: `read_record` (CLI) and the hook's record loop apply the SAME predicate, including
  the filename-equals-its-own-key test.
- **Changing a key derivation orphans every stored record.** Round 2 did exactly that and nothing
  noticed for two rounds. A derivation change needs a migration or a schema bump, not just a
  correct new formula.
- Fail-closed covers OUR identity too, not only stored owners.
- Every dependency boundary validates output SEMANTICS, not just status and non-emptiness.
- Dynamic path components get the same type/symlink guard as fixed ones.
- Refuse inputs whose identity cannot be guaranteed (non-ASCII, control bytes) instead of
  claiming a guarantee that does not hold.
- A usage error is knowable from the argument vector alone; report it before the environment.
- Accepted residuals unchanged: no fsync durability, gitignored is not confidential, a ticked
  box is self-attested.

Consensus: disagreed
