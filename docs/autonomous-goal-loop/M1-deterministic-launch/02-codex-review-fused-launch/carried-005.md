## Carried decisions — Round 005
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
