# Codex adversarial review — Round 006

## Review scope
Re-review (Rounds 1-5 rejected; all findings accepted and fixed). Budget extended by the user at Phase 9 — see GOAL.md Q5.

## GPT findings
[severity:high][technical correctness] The Round-5 quiescent-cutover fix is bypassed for an empty legacy file. Genuinely blocking: yes.

Evidence: `cmd_migrate` checks `! -s`, removes `$LEGACY`, and returns before acquiring `$LEGACY_LOCK`.

Verification: After the empty check, a compliant writer can append while holding the legacy lock; migration then removes the now-nonempty file and reports “empty — removed,” silently losing the claim.

Suggested direction: Acquire the legacy lock before every existence, size, removal, snapshot, or archive decision, then revalidate the file inside the lock.

[severity:high][technical correctness] The repository-absence fallback still fails open when the working directory uses a logical symlink path. Genuinely blocking: yes.

Evidence: On Git status 128, the hook walks upward from `$PWD` without first resolving it physically.

Verification: From logical `/tmp/repo-link/sub` pointing into `/real/repo/sub`, the walk examines `/tmp/repo-link/sub`, `/tmp/repo-link`, `/tmp`, and `/`; it misses `/real/repo/.git` and exits 0 after a fatal Git failure.

Suggested direction: Start the fallback from a validated absolute `pwd -P` result and block if physical resolution fails.

[severity:medium][technical correctness] Migration uses the case-folding document-key helper as a file snapshot digest, so case-only mutations are invisible. Genuinely blocking: yes.

Evidence: Both `snap` and `now` call `sha1_try`, which lowercases all input through `tr` before hashing.

Verification: Changing `OwnerA<TAB>docs/X.md` to `ownera<TAB>docs/x.md` after planning produces the same snapshot digest; migration archives the changed authority while publishing the earlier owner/path.

Suggested direction: Use a separate byte-preserving file-digest helper for migration snapshots.

[severity:medium][technical correctness] The Round-5 dependency-status repair still did not reach all sibling sites. Genuinely blocking: yes.

Sites: Primary: `fullcycle-gate.sh` key derivation; confirmed: hook record reads, hook version-byte counting, and `dstack.cmd_migrate` snapshot reads.

Evidence: The hook still uses `printf | tr | $SHA1TOOL`, observes only the digest status, and ignores `cat` status for records; migration likewise embeds unchecked `cat` inside `sha1_try`.

Verification: A failing `tr` followed by successful `shasum` returns status 0 and key `da39a3ee5e6b4b0d3255bfef95601890afd80709`, reproducing the Round-5 empty-input defect.

Suggested direction: Capture and validate every producer’s status independently at each confirmed site.

[severity:medium][security] `migrate` can move an unrelated tracked `.fullcycle-active` file in any repository using the global installation. Genuinely blocking: yes.

Evidence: `ensure_store` checks only whether `.dstack` is tracked; `cmd_migrate` never checks the legacy source before `mv "$LEGACY" "$arch"`.

Verification: A tracked file containing a syntactically valid legacy line is accepted, removed from its tracked path, and reclassified as a migration archive.

Suggested direction: Prove the legacy source is untracked before treating it as runtime state, with an explicit override if tracked-state migration is intentional.

[severity:medium][security] The advertised bounded retention is not enforced automatically. Genuinely blocking: yes.

Sites: Primary: `dstack.cmd_prune`; confirmed: `cmd_run_dir`, dispatch, `install.sh`, and the supplied runtime-state documentation.

Evidence: `prune` runs only through an explicit CLI invocation; capture creation, status, and installation establish no trigger or schedule.

Verification: A capture can remain indefinitely when the operator never invokes `prune`, contradicting the stated short-lived mitigation for plaintext review bundles.

Suggested direction: Trigger pruning from a normal lifecycle boundary such as capture creation, or install an explicit scheduled mechanism.

[severity:low][technical correctness] The accepted byte-exact `.gitignore` repair is still absent. Genuinely blocking: no.

Sites: Primary: `dstack.ensure_store`; confirmed: `tests/secret-guard.sh`.

Evidence: Both compare `$(cat file)` with `*`, and command substitution strips every trailing newline.

Verification: `*`, `*\n`, and `*\n\n\n` all pass despite the claimed exact-byte invariant.

[severity:low][technical correctness] The accepted `run-dir` cleanup repair still falsely reports label release. Genuinely blocking: no.

Evidence: The `chmod` failure branch suppresses `rmdir "$d"` failure and always says the label was released.

Verification: If the directory becomes nonempty before cleanup, it remains occupied while the error reports release.

[severity:low][technical correctness] The accepted disappearing-record repair is still absent from `status`. Genuinely blocking: no.

Evidence: `read_record` can pass `-f`, lose the file before `cat`, and return “cannot be read”; `status` reports that as corruption.

Verification: A concurrent successful `unreg` in that interval appears as an invalid active record instead of a tolerated disappearance.

[severity:low][technical correctness] The accepted case-sensitive-volume repair is still absent from `canon`. Genuinely blocking: no.

Evidence: `ls | grep -ixF | head -1` selects the first case-insensitive match even when the caller’s exact spelling exists.

Verification: With both `A.md` and `a.md`, registering either spelling can select `A.md`, preventing the lowercase file from retaining its physical identity.

[severity:low][technical correctness] The retention threshold remains one day later than documented. Genuinely blocking: no.

Evidence: `find -mtime +7` requires the truncated whole-day age to exceed seven.

Verification: A directory aged 7 days 23 hours is not selected; deletion begins at eight complete days, not immediately after seven.

[severity:low][technical correctness] Migration suppresses legacy-lock removal failure, clears its traps, and reports success with a stale lock; genuinely blocking: no.

[severity:low][security] The secret-guard exemption reads a non-symlink without requiring a regular file, so a FIFO at `.dstack/.gitignore` can hang the pinned check; genuinely blocking: no.

[severity:low][UI & UX / DX] `status` returns immediately when `active/` is absent and therefore hides any existing stored run captures; genuinely blocking: no.

Omitted-detail: 3 low

GPT verdict: reject — the unlocked empty-file migration branch can silently erase a compliant legacy claim, the repository fallback can still open on fatal Git failure, and multiple concrete medium blockers remain.

## Carried decisions — Round 006
Rounds 1-5 decisions stand. Added in Round 6:

- **Sweep siblings, and then sweep them again.** Three rounds running, a class fix landed in one
  artifact and not the other. Before claiming a class closed, grep every artifact for the
  construct, not just the one the finding named.
- **A lock must cover every decision about the thing it protects,** including the ones that look
  like early exits.
- **Resolve paths physically before reasoning about them.** A logical path defeats an
  ancestor walk exactly like a missing binary defeats a command check.
- **A key helper is not a content digest.** Case folding is correct for identity and wrong for
  detecting change.
- **Runtime state must be proven untracked before it is moved.** A global tool runs in
  repositories that never asked for it.
- **Advertised retention needs a trigger.** A cleanup nobody runs is not a mitigation.

Consensus: disagreed
