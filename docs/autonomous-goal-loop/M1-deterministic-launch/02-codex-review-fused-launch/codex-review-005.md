# Codex adversarial review — Round 005

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 79105 bytes (r1 41621, r2 50262, r3 65104,
r4 75879); rounds 001 and 002 compacted. **This is the round cap for a per-task unit (§4).**
Allowlist unchanged: the review-unit folder, `claude/skills/codex-review/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] The point-by-point rebuttal remains outside the immutable round and is never bundled, allowing the decision record to change or disappear without invalidating sealed consensus.
Sites: `claude/skills/codex-review/SKILL.md:446-525`; confirmed: `SKILL.md:629-653`, `assemble-review.sh:337-359`, `codex/skills/adversarial-review/SKILL.md:96-108`.
Evidence: No rule seals or validates `response-<NNN>.md`, while the elected contract requires each invocation/rebuttal exchange in one immutable numbered file.
Verification: The assembler emits task, rounds, and explicit allowlisted changes but no response files; later reviewers and the gate therefore cannot verify the preserved rebuttal record.
Suggested direction: Preserve and validate the complete exchange in the immutable round, or provide an equivalently sealed and automatically bundled rebuttal record.

[severity:medium][technical correctness] The closure override still manufactures positive consensus with concrete blockers and explicitly suppresses fix-introduced regressions merely because their defect class was previously recorded.
Sites: `claude/skills/codex-review/SKILL.md:529-552`; confirmed: `SKILL.md:560-579,662-695,713-717`, `codex/skills/adversarial-review/SKILL.md:86-108`, `claude/hooks/fullcycle-gate.sh:411-420`.
Evidence: Disposition 4 and the hard cap allow unresolved concrete mediums to become `Consensus: resolved`; §3 says a variant introduced by a fix does not reopen, contradicting both the elected contract and the document’s own discovery-time rule.
Verification: The gate regex accepted `Consensus: resolved` with status 0, and the backslash-path failure below is a concrete regression in the already-recorded skip-gate class that §3 would suppress.
Suggested direction: Loop termination may record unresolved work, but positive consensus must still require every concrete high/medium to be fixed, disproved, or user-disposed.

[severity:medium][technical correctness] The repaired per-path skip gate is not byte-literal: `awk -v` interprets backslash escapes, allowing skipped allowlisted paths containing backslashes to pass validation.
Sites: `claude/skills/codex-review/SKILL.md:269-280`; confirmed: `claude/skills/codex-review/assemble-review.sh:214-258`.
Evidence: POSIX filenames may contain backslashes, and the Bash assembler preserves them in marker lines while awk transforms or removes them when parsing `-v p=...`.
Verification: Synthetic markers for `path\to`, `path\new`, `path\qz`, and `path\123` all returned awk status 1—treated as “not skipped”—while `plain/path` returned 0.
Suggested direction: Transfer the prefix through a byte-preserving channel such as `ENVIRON` or compare input fields without awk’s `-v` escape decoding.

[severity:medium][DX] The new zsh signal handlers exit with the requested status but do not forward the signal to foreground `dstack`, so signaling only the wrapper leaves the paid review running.
Sites: `claude/skills/codex-review/SKILL.md:295-313`; confirmed: `SKILL.md:329-335`.
Evidence: zsh defers these traps while waiting for a foreground external command; cleanup and `exit 143` occur only after that child finishes.
Verification: Sending TERM to the documented wrapper around a five-second foreground child produced `rc=143` only after the full five seconds elapsed.
Suggested direction: Ensure each wrapper handler signals and waits for the active `dstack` process before cleaning scratch and exiting.

[severity:low][technical correctness] The displayed Bash signal probe is misquoted and does not reliably signal the Bash process when invoked by the declared zsh consumer.
Sites: `claude/skills/codex-review/SKILL.md:164-168`.
Evidence: Its double-quoted Bash program lets zsh expand `$$` before `/bin/bash -c` starts.
Verification: A nested probe showed `expanded-target` equal to the zsh parent PID while Bash’s own PID differed; the claimed table reproduced only after protecting `$$` from outer-shell expansion.

Omitted-detail: 0 low

GPT verdict: reject — unresolved contract, skip-validation, and cancellation defects can certify known blockers, omit reviewed evidence, or leave cancelled paid rounds running.

## Carried decisions
- **A REGRESSION INTRODUCED BY A FIX ALWAYS REOPENS, whatever class it belongs to.** §3 used to
  exempt "a variant of an already-recorded class in code a fix just introduced". That was a licence
  to ship the defect your last repair created, and it contradicted the discovery-time rule directly
  above it. This round proved it with a live example — the `awk -v` backslash bug is a variant of a
  recorded class, in code a fix had just introduced, and it silently passed a skipped file. The
  exemption now covers only a restatement about code that has NOT moved.
- **Pass the skip marker through the ENVIRONMENT, not `awk -v`.** `-v` decodes backslash escapes, so
  a path containing `\t`, `\n` or `\1` is transformed before the comparison and its marker never
  matches. Measured: `path\to` and `path\new` were MISSED by the `-v` form and are caught via
  `ENVIRON`, while `plain/path` behaves identically in both. Third defect in this one check, and the
  first two were also "the matcher is not what I thought it was".
- **`<run-dir>/exit` is the round's status; the wrapper's exit code is not, and the signal handlers
  cancel nothing.** Both shells defer a pending trap while a foreground command runs, so a TERM to
  the launching shell lands only after `dstack run` returns — measured against a five-second child,
  `rc=143` after the full five seconds. A completed round can therefore be reported 143, and the
  handlers must not clean up, or they delete the scratch directory a live `codex exec` is using. To
  stop a round in flight, stop the recorded process group.
- A printed measurement command is code. `"… kill -<sig> $$ …"` in double quotes is expanded by the
  INVOKING shell and signals that shell, not the bash under test — single-quote the program and pass
  the signal name as an argument. The table reproduces with the corrected form.
- The response-file and consensus-disposition disagreement with `codex/skills/adversarial-review/SKILL.md`
  is RAISED for the third time and its disposition is unchanged: this file governs the pipeline's
  closure semantics, the Codex-side contract needs the same two edits, and that file is outside this
  unit's declaration. Recorded as a follow-up, not reopened.

Consensus: resolved
