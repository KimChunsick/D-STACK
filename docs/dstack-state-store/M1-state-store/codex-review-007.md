# Codex adversarial review — Round 007

## Review scope
Re-review (Rounds 1-6 rejected; all findings accepted and fixed). Budget extended by the user — see GOAL.md Q5.

## GPT findings
[severity:high][technical correctness] The Round-6 exact-spelling repair still authorizes case-folded equality, while the hook cannot verify the final component’s actual spelling. Genuinely blocking: yes.

Sites: Primary: `dstack.same_doc` and `assert_record`; confirmed: `cmd_reg`, `cmd_unreg`, `cmd_reclaim`, `cmd_migrate`, and the `fullcycle-gate.sh` record scan.
Evidence: `canon` preserves exact canonical spellings, but `same_doc` lowercases both paths; the hook merely appends the supplied basename to a physical parent, accepting wrong-case names on case-insensitive APFS.
Verification: `same_doc docs/A.md docs/a.md` returned 0; on a case-sensitive volume, `unreg docs/a.md` can therefore remove `docs/A.md`’s same-key claim, while the live APFS probe showed the hook accepting `goal.md` where the CLI resolves `GOAL.md`.
Suggested direction: Use exact canonical equality for authorization at every site; case-fold only key derivation, and independently recover the exact final-component spelling in the hook.

[severity:medium][security] The Round-6 tracked-state safeguard remains case-sensitive on a case-insensitive filesystem, so the global CLI can still mutate or move tracked state. Genuinely blocking: yes.

Sites: Primary: `dstack.cmd_migrate`; confirmed: `ensure_store`.
Evidence: Both checks use exact `git ls-files -- .fullcycle-active` or `.dstack`, although APFS resolves ASCII-case variants to the same filesystem object.
Verification: This checkout has `core.ignorecase=true`; `git ls-files -- AGENTS.md` found the tracked file while `git ls-files -- agents.md` returned nothing, so a tracked `.FullCycle-Active` can be opened through `$LEGACY` yet declared untracked and moved.
Suggested direction: Prove untrackedness against every filesystem-equivalent ASCII-case spelling before modifying either namespace.

[severity:medium][technical correctness] The dependency-status sweep still missed migration normalization and final-name canonicalization pipelines. Genuinely blocking: yes.

Sites: Primary: `dstack.cmd_migrate`; confirmed: `canon`.
Evidence: The `sed` assignment’s status is ignored, while `canon`’s `ls | grep | head` fallback observes `head` rather than its producers.
Verification: An isolated failing `sed` changed valid legacy `docs//x.md` to an empty string, classified it as droppable, and would allow the authority file to be archived; a failed listing can similarly preserve a wrong-case APFS spelling and publish an unaddressable record.
Suggested direction: Capture and validate every normalization/listing producer’s status, treating failure as a conflict or fatal error rather than as absent output.

[severity:medium][technical correctness] Gate-section parsing accepts prefixed headings instead of the exact frozen headings, allowing schema typos to satisfy enforcement. Genuinely blocking: yes.

Sites: Primary: `fullcycle-gate.sh.section`; confirmed: task `Gate status` and Goal `Goal gate` consumers.
Evidence: `index($0,h)==1` matches any line beginning with `## Gate status` or `## Goal gate`.
Verification: Probes using only `## Gate status-old` and `## Goal gate-archived` returned checked rows, so documents lacking the required headings can pass their checkbox checks.
Suggested direction: Match the complete required heading exactly, with only explicitly permitted trailing whitespace.

[severity:low][technical correctness] The Round-6 legacy-lock cleanup repair covers only the non-empty success path. Genuinely blocking: no.

Evidence: The no-file and empty-file branches return with only a silent `EXIT` trap; the checked `rmdir` and warning occur solely after archival.
Verification: If removal fails on either early branch, migration reports success and leaves a lock that makes later migrations time out.

[severity:low][security] The accepted FIFO repair remains absent from the secret-guard exemption. Genuinely blocking: no.

Evidence: The exemption checks only `! -L` before redirecting both `wc` and `cat` from `.dstack/.gitignore`.
Verification: A FIFO is not a symlink, so the first read blocks indefinitely awaiting a writer instead of rejecting the non-regular file.

[severity:low][UI & UX / DX] `status` still incompletely reports stored runs. Genuinely blocking: no.

Evidence: It returns immediately when `active/` is absent and enumerates run sessions and labels using only non-hidden `*` globs.
Verification: Existing captures disappear from output when `active/` is missing, and dot-prefixed corrupt namespace entries remain invisible.

[severity:low][technical correctness] Retention remains one day later than the documentation promises. Genuinely blocking: no.

Evidence: `AGENTS.md` says captures are pruned after seven days, while `find -mtime +7` and the revised message establish an eight-complete-day threshold.
Verification: A capture aged seven days and 23 hours is retained despite the documented seven-day window.

[severity:low][security] Invalid-state diagnostics emit attacker-controlled path and legacy-record bytes directly to terminals. Genuinely blocking: no.

Sites: Primary: `cmd_status`; confirmed: `cmd_migrate` conflict diagnostics and hook malformed-record reasons.
Evidence: Raw record basenames, document values, and legacy lines reach `note`/`printf` without `%q` or control-byte filtering.
Verification: An ESC/OSC sequence in a malformed entry is emitted unchanged, permitting terminal-title, hyperlink, clipboard, or display spoofing.

[severity:low][technical correctness] Record writers ignore `date` failure and accept an empty timestamp because validation checks only that `ts` is a string. Genuinely blocking: no.

[severity:low][UI & UX / DX] `--help` mandates the absolute executable path but presents every command synopsis using bare, normally unresolved `dstack`. Genuinely blocking: no.

Omitted-detail: 2 low

GPT verdict: reject — distinct case-variant documents can still lose ownership, tracked case-variant state can be moved, migration can discard claims on normalization failure, and prefixed headings can bypass required gates.

## Carried decisions — Round 007
Rounds 1-6 decisions stand. Added in Round 7:

- **Fold case for identity derivation, never for authorization.** One physical file gets one key;
  who may act on a record is an exact-string question.
- **Ask the filesystem for the real spelling.** Appending a supplied name to a resolved parent
  proves nothing on a case-insensitive volume.
- **`git ls-files` is case-sensitive; the filesystem is not.** Use `:(icase)` for any
  "is this tracked" question that guards a destructive action.
- **A frozen heading is matched whole.** Prefix matching turns a byte-frozen surface into a
  suggestion.

Consensus: disagreed
