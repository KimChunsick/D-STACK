# Codex adversarial review — Round 008

## Review scope
Re-review of Round 007. Relaunch of a voided attempt: the first Round-8 bundle was
assembled while a file inside it was still being edited (freeze violation), so that
attempt was discarded unread and the round re-run as `m1-r8b`.

## GPT findings
[severity:medium][security] Enumeration failures are still interpreted as empty state, allowing an unreadable active registry to open the Stop gate and a failed guard traversal to pass. Genuinely blocking: yes.
Sites: Primary: `fullcycle-gate.sh` active-record scan; confirmed: `dstack.cmd_status`, `tests/secret-guard.sh` nested-ignore scan.
Evidence: The hook and `status` validate `-d` before iterating unchecked globs; the guard consumes `find` through process substitution, whose status is never propagated.
Verification: An unreadable directory passed `-d` while Bash retained its literal glob, and a loop fed by `< <(false)` returned status 0 with zero entries.
Suggested direction: Prove each namespace traversal succeeded before interpreting zero discovered entries as empty state.

[severity:medium][technical correctness] The Round-7 heading repair fixes target matching but not section termination, so valid tab-separated or indented H2 headings leak later checkboxes into the gate section. Genuinely blocking: yes.
Evidence: `section()` accepts space/tab after the target but terminates only on `/^## /`, excluding `##<TAB>Next` and CommonMark’s permitted leading indentation.
Verification: A fixture containing `## Goal gate`, then `##<TAB>Archive`, then `- [x] GOAL E2E` returned the archive heading and checkbox as Goal-gate content.
Suggested direction: Use one whitespace-aware H2 boundary rule for termination, including tabs and permitted leading indentation.

[severity:medium][technical correctness] The dependency-status sweep still masks producer failures in canonical spelling and milestone enforcement, creating corrupt records or silently omitting required milestone gates. Genuinely blocking: yes.
Sites: Primary: `dstack.canon`; confirmed: `fullcycle-gate.sh` section consumers, consensus parsers, and milestone extraction pipeline.
Evidence: `grep -ixF | head -1` observes only `head`, while the milestone `grep|grep|grep|tr|sort` substitution ignores the pipeline result entirely.
Verification: A failing `grep` yielded status 0 and an empty canonical name through `head`; the live APFS wrong-case path still resolved, while a failing `sort` produced zero milestone iterations.
Suggested direction: Check every producer independently and fail closed before retaining caller spelling or treating an empty milestone set as authoritative.

[severity:low][technical correctness] The hook still reports a record removed between its initial existence check and `cat` as corruption instead of a tolerated deregistration race. Genuinely blocking: no.
Evidence: `fullcycle-gate.sh:214` appends `(unreadable)` unconditionally on `cat` failure, unlike `read_record()`’s vanished-record status.
Verification: Removal after line 193 but before line 214 reaches `bad`, causing one spurious block.

[severity:low][technical correctness] The Round-7 legacy-lock cleanup repair remains partial on failure exits. Genuinely blocking: no.
Evidence: Conflict and `die` paths rely on a silent `EXIT` trap; only successful no-file, empty-file, and archive paths call `release_legacy_lock()`.
Verification: If trap-time `rmdir` fails after a migration error, the stale lock receives no warning and later migrations time out.

[severity:low][security] Invalid-state diagnostics still emit attacker-controlled control bytes directly to terminals or rendered hook reasons. Genuinely blocking: no.
Sites: Primary: `dstack.cmd_status`; confirmed: `cmd_migrate`, `canon` rejection, and the Stop-hook malformed-record report.
Evidence: Raw basenames, legacy lines, and rejected path values reach `note`, `printf`, or `bad` without control-byte escaping.
Verification: No sanitization occurs between a malformed ESC/OSC-bearing entry name and the output calls.

[severity:low][technical correctness] Timestamp validation remains inconsistent: writers ignore a nonzero `date` status when it emitted text, while both record readers accept an empty timestamp. Genuinely blocking: no.
Sites: Primary: `dstack.cmd_reg`; confirmed: `cmd_reclaim`, `cmd_migrate`, `read_record`, and the hook record predicate.
Evidence: `date` runs inside a `jq` argument substitution, so only `jq` status is observed; read predicates require only `ts:string`.
Verification: Nonempty output from a failing `date` satisfies `written_record_ok`, and `{"ts":""}` satisfies both read schemas.

[severity:low][UI & UX / DX] `status` and retention still hide malformed regular-file entries in the run namespace. Genuinely blocking: no.
Evidence: Both session and label scans silently continue unless entries are directories, and pruning selects only depth-two directories.
Verification: A regular file at `runs/<sid>/<label>` is neither listed nor removed by the retention sweep.

Omitted-detail: 0 low

GPT verdict: reject — three concrete medium blockers still permit fail-open registry scans, gate-section leakage, and dependency-failure corruption or enforcement bypass.

## Bundle size (the ratchet, recorded)

Round 008 bundle: 214387 bytes.

## Round outcome

Three blocking mediums. This file is written LATE — the round ran, its findings were
triaged, but the record was never committed to disk before the next round started, so
the series briefly ran 007 → 009 with a hole in it. Recorded here rather than quietly
renumbering, because a review series with a missing round is exactly what the
contiguity rule exists to catch.

Honest correction to what was believed at the time: only ONE of the three blockers was
actually closed before Round 009 was launched (the `grep -ixF | head -1` producer
masking in `canon`). Section termination and the enumeration-failure-reads-as-empty
defect were still live and were fixed at Round 010 closure, with direct-run evidence in
`response-009.md`. Round 009 did not re-raise them; absence from one round is not proof
of a fix, which is why they were re-checked by hand rather than assumed.

Consensus: disagreed
