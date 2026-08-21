# Response — Round 002 (never bundled)

All four findings accepted; fixed in `claude/skills/codex-research/SKILL.md`, with one
scope boundary stated on F8. GOAL.md's interview record carries the F9 amendment.

- F6 (high, hard links + unguarded orchestrator writes): verified — `[ ! -L ]` passes a
  hard link and a truncating open mutates the shared inode. Fixed: both fences now
  require an existing leaf to be a regular file with link count 1 (POSIX
  `find -prune -links 1`), inputs additionally readable/non-empty, `-o` targets
  same-or-absent; a prose rule extends the discipline to every orchestrator-written
  artifact (Step 1 brief, Step 2a record, fallback artifacts). Functional probe recorded
  in task.md: hard link, symlink, directory, and aliased twin all REFUSED; absent path
  and unaliased file pass. We note the recipe's stated threat model is mistake-tripwire,
  not adversary boundary — accepted because `cp -al` working-tree clones make hard-linked
  leaves a real mistake. Full scratch-staging with atomic replace was NOT adopted for the
  `-o` targets: codex-cli writes `-o` directly and nothing may run after `dstack run` in
  the background call, so the guard-before-launch is the honest fix available in this
  shape.
- F7 (medium, fail-open assembly): verified and fixed — `-r` added to input guards, the
  concatenation is an `&&` chain, and a nonzero group status refuses the launch. Probe
  recorded: unreadable first input now refuses; the old plain-group form demonstrably
  passed (defect reproduced before fixing).
- F8 (medium, producer-declared coverage): accepted with a boundary. The structural test
  now also requires every declared ledger row / deferred check reconciled somewhere in
  the audit, treats an empty `## Audit of findings` over a claim-bearing artifact as
  broken, and the research-side triggers treat all-`none` blocks over measurable claims
  as missing sections. Boundary, stated in the text itself: these are the orchestrating
  model's reading obligations — a structural backstop. Claim-level semantic coverage is
  the auditor's own contract (its F-item pass); a mechanical orchestrator re-derivation
  of the audit would duplicate the auditor badly rather than back it up.
- F9 (medium, unaudited verdict changes): accepted in full — any verdict-changing check
  outcome re-enters the auditor (Step 2b re-run under the next label; the stdin
  concatenation carries the appended results), the `superseded:` line records the delta
  audit's verdict, and decision-criticality decides only Phase 4 re-entry. The
  conflicting GOAL.md interview assumption is amended in place with the revision marked
  as review-driven.

Verification after fixes: three fences pass `bash -n`; guard probes above; secret guard
green. Round 003 requested on the same allowlist.
