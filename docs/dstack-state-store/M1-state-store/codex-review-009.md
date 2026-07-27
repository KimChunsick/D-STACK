# Codex adversarial review — Round 009

## Review scope
Re-review of Round 008. Final round for this unit: closed by the non-convergence rule
in `claude/skills/codex-review/SKILL.md` Step 4, not by reviewer approval.

## GPT findings
[severity:high][technical correctness] CONCRETE; genuinely blocking: yes. Lock-release traps can delete a successor’s lock, restoring overlapping same-key mutations and silent ownership loss.
Sites: Primary: `dstack.kunlock`; confirmed: `release_legacy_lock` and their `INT`/`TERM`/`EXIT` traps.
Evidence: Both functions remove the lock directory before clearing traps, while each active trap removes that pathname without verifying ownership.
Verification: After A’s `rmdir`, B can acquire the empty lock; a pending signal then runs A’s old trap and removes B’s lock, allowing C to enter while B is reclaiming or migrating.
Suggested direction: Disarm cleanup traps before publishing the lock pathname as free, accepting a stale lock if a signal lands during release, or attach verifiable ownership to each lock instance.

[severity:medium][technical correctness] CONCRETE; genuinely blocking: yes. The final-component spelling repair still makes the CLI and hook disagree and still masks a failed spelling lookup.
Sites: Primary: `dstack.canon`; confirmed: `dstack.read_record` and the `fullcycle-gate.sh` record scan.
Evidence: `ls -1` excludes dot-prefixed names; `canon` accepts an empty case-match and preserves the caller’s spelling, while the hook requires an exact match from the same incomplete listing; `grep | head` also masks `grep` failure.
Verification: `reg docs/.task.md` can publish a record that `status` considers valid, but the hook cannot find `.task.md` in `ls -1` output and blocks it as wrongly spelled.
Suggested direction: Enumerate dot entries and require one checked, exact physical-name result under the same invariant in both implementations.

[severity:medium][technical correctness] CONCRETE; genuinely blocking: yes. The heading repair accepts every whitespace-delimited suffix, not merely the documented Goal suffix.
Evidence: `section()` recognizes any line beginning `## Gate status ` or `## Goal gate `.
Verification: The implemented predicate extracted checked rows from `## Gate status archived`, so a stale or mistyped section can satisfy the required gate while the frozen heading is absent.
Suggested direction: Give each heading an explicit grammar—exact `Gate status`, and only the documented parenthetical form for `Goal gate`.

[severity:medium][UI & UX / DX] CONCRETE; genuinely blocking: yes. Multiple failure messages still render recovery commands as one single-quoted executable word.
Sites: Primary: the hook’s legacy-cutover message; confirmed: its `status`/`unreg` messages, `dstack.require_cutover`, and `dstack.cmd_status`.
Evidence: Messages emit forms such as `'/absolute/path/dstack migrate'` and `'/absolute/path/dstack unreg <doc>'`.
Verification: Copying either form makes the shell search for one filename containing spaces instead of invoking `dstack` with arguments.
Suggested direction: Render `"$HOME/.claude/bin/dstack"` as the quoted executable word and place each argument outside those quotes at every confirmed site.

[severity:medium][UI & UX / DX] CONCRETE; genuinely blocking: yes. A deleted or renamed registered document cannot be released through any supported CLI command.
Sites: Primary: `dstack.cmd_unreg`; confirmed: `assert_record`, `read_record`, `cmd_reclaim`, and the hook’s invalid-record remediation.
Evidence: `read_record` rejects missing or noncanonical documents before `unreg` can validate ownership and remove their records.
Verification: After registering and then deleting `docs/task.md`, the hook blocks on the missing document, while `unreg docs/task.md` dies on the same invalid record; a case-only APFS rename produces the equivalent trap.
Suggested direction: Add a guarded stale-record removal path that validates schema, stored owner, document bytes, and filename hash without requiring the document to still exist.

[severity:medium][technical correctness] CONCRETE; genuinely blocking: yes. Fatal Git discovery can still open the global gate for environment-defined worktrees without an in-tree `.git` marker.
Evidence: The status-128 fallback checks only ancestor `.git` entries and exits successfully without considering explicit Git environment state or an existing ancestor `.dstack`.
Verification: Create state while `GIT_DIR` and `GIT_WORK_TREE` identify an external-metadata worktree, then make `GIT_DIR` unavailable; Git returns 128, no ancestor `.git` exists, and the hook exits 0 despite the active store.
Suggested direction: On fatal Git discovery, treat explicit Git environment state or any physically located ancestor `.dstack` as evidence that absence was not proved.

[severity:low][security] CONCRETE; genuinely blocking: no. Invalid-state diagnostics still emit terminal-control bytes without escaping.
Sites: Primary: `dstack.cmd_status`; confirmed: migration conflict diagnostics.
Evidence: Malformed record basenames, document fields, and legacy lines flow directly through `note` or `printf`.
Verification: An ESC/OSC sequence in an invalid record name is printed unchanged.

[severity:low][technical correctness] CONCRETE; genuinely blocking: no. Timestamp validation remains inconsistent and still ignores the timestamp producer’s exit status.
Sites: Primary: record writers; confirmed: `written_record_ok`, `read_record`, and the hook schema check.
Evidence: Writers accept any nonempty timestamp without checking `date`, while readers accept even an empty timestamp.
Verification: A `date` command that prints one byte and exits nonzero still permits publication; a handcrafted empty timestamp is accepted by both readers.

[severity:low][technical correctness] CONCRETE; genuinely blocking: no. The hook still treats a nonregular legacy namespace as absent.
Sites: Primary: `fullcycle-gate.sh`; confirmed: `dstack.cmd_status`, which rejects the same state through `require_plain`.
Evidence: The hook checks the legacy path only for symlink status and nonzero size.
Verification: A FIFO at `.fullcycle-active` is neither a symlink nor size-positive, so a repository without `.dstack` exits successfully.

[severity:low][UI & UX / DX] CONCRETE; genuinely blocking: no. `status` reports invalid active records but still returns success.
Evidence: Invalid records increment the displayed count, after which `cmd_status` unconditionally returns 0.
Verification: Automation using the exit status cannot distinguish a healthy registry from one the Stop hook refuses to trust.

[severity:low][technical correctness] CONCRETE; genuinely blocking: no. The milestone documentation still states seven-day pruning while the implementation and `AGENTS.md` specify eight complete days.
Evidence: `02-dstack-cli/task.md` says captures are pruned after seven days; `find -mtime +7` begins deletion after eight complete days.
Verification: A capture aged seven days and 23 hours remains present.

[severity:low][UI & UX / DX] CONCRETE; genuinely blocking: no. Migration rejects duplicate legacy lines with the same owner and document even though collapsing them is lossless.

[severity:low][technical correctness] CONCRETE; genuinely blocking: no. Successful registration ignores failure to remove its published temporary-link name.

Omitted-detail: 2 low

GPT verdict: reject — lock ownership can still be broken by signal timing, and concrete gate-bypass, invariant-divergence, and unrecoverable-state paths remain.

## Bundle size (the ratchet, recorded)

R7 198591 · R8 214387 · **R9 193178** bytes.

Same story as M2: the reviewed surface grew while the blocking count did not fall. The
ratchet rule was authored out of this data and binds from the next review unit onward;
this unit did not satisfy it, and saying so is the point.

## Round outcome

All six blocking findings fixed — one high and five mediums — plus two lows. Reasoning
and direct-run evidence are in `response-009.md`, deliberately outside the corpus.

Blocking findings per round across this unit: **8, 9, 6, 7, 6, 6, 4 (R7), 3 (R8), 6 (R9)**.
R7-R9 is 4, 3, 6: not strictly decreasing across three consecutive rounds, so the loop is
non-convergent by measurement and closes here. The `GPT verdict` line is advisory under
that rule. Round 9 rising above Round 8 is the clearest single data point for why
"loop until the reviewer approves" does not terminate: the fixes were real and the count
went up anyway, because each repair opened new surface to examine.

Nothing was downgraded to make this close. Every finding the reviewer marked
"genuinely blocking: yes" was fixed and verified by direct run before sealing; the lows
are recorded as evidence-backed follow-ups in `task.md` and `findings.md`.

Consensus: resolved
