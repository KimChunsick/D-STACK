# Response — Round 003 (never bundled)

All six findings accepted; fixed in `claude/skills/codex-research/SKILL.md`. GOAL.md's
artifact-name assumption carries the F13 amendment.

- F10 (high, confused deputy): verified — the spec chose the input and the record shipped
  verbatim into the audit stdin. Fixed with an authorization step (legitimate inputs:
  public internet-addressable sources and scratch-derived files from them; local paths
  outside scratch, private/internal services, credentialed anything → `not-run
  (unauthorized input)`) and bounded recording (derived value/comparison + justifying
  lines only).
- F11 (medium): fixed — per-item research reading (checkable H beside `ledger: none`, or
  a `deferred` row naming no list entry, is the finer-grain defect); audit acceptance
  demands substantive per-target coverage with expected sets derived from the research
  artifact, unresolved-column parking only for checks that could not run, token-F over a
  finding-rich artifact treated as breakage.
- F12 (medium): fixed — the confirm/change classification IS the audited judgment, so
  every executed new-check result returns to the auditor via delta audit; termination
  bounded (third round of new checks → `unverifiable (unstable check set)`, recorded).
- F13 (medium): fixed — `ATTEMPT` suffixes both the label and the `-o` artifact; the
  fence now refuses an EXISTING audit artifact outright (stronger than the link-count
  test for an output that need not exist); the fallback writes the next attempt's name;
  predecessors preserved.
- F14 (medium): fixed — the brief takes the full input test (regular, non-symlink,
  readable, non-empty, unaliased) before allocation; probe recorded (empty brief refused,
  real brief passes).
- F15 (low): fixed — unconditional scratch cleanup armed at `mktemp`, swapped for the
  exit-record-gated trap at launch; probe recorded (assembly failure removes scratch,
  rc=1).

Verification after fixes: three fences pass `bash -n`; ATTEMPT guard probe (accepts
''/-2/-99, refuses x/-abc/--2); secret guard green. Round 004 requested on the same
allowlist.
