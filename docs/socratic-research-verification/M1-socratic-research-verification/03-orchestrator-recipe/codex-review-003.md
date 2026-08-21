# Codex adversarial review — Round 003

## Review scope
Re-review | serial | bundle 56391 bytes (round 002: 44768) | label socratic-research-verification-t03-r003

## GPT findings
[severity:high][security] Deferred-check specifications remain a confused-deputy path: untrusted data selects local inputs, while the orchestrator records command output verbatim and sends it to Codex.
Sites: Primary: `SKILL.md:286-301`; confirmed: `SKILL.md:255,365-382`.
Evidence: A scratch working directory is explicitly not filesystem confinement, and "no secrets" does not protect confidential files whose names are not recognized as secret-bearing.
Verification: A specification naming a readable private dataset can cause an orchestrator-authored comparison to emit records into `<topic>.data-checks.md`, after which Step 2b transmits them in its stdin bundle.
Suggested direction: Independently authorize and confine every input, reject repo-external/private sources, and record bounded derived results instead of unrestricted verbatim output.

[severity:medium][technical correctness] F8 remains open: the research gate rejects only the all-three-`none` case, while audit acceptance permits partial F coverage and mere unresolved-column mentions for ledger rows.
Sites: Primary: `SKILL.md:390-399`; confirmed: `SKILL.md:474-487`, `adversarial-research/SKILL.md:24-49`, `socratic-audit/SKILL.md:22-27,100-107`.
Evidence: The contracts require every checkable H-item to have a ledger row and every H/F/data-check target to be audited, but the acceptance predicates enforce neither complete mapping.
Verification: A measurable H1 with `ledger: none` bypasses the all-`none` trigger; a seven-heading audit with one token F-item, an H-only summary, and ledger IDs parked under unresolved checks then passes despite omitted probes and findings.
Suggested direction: Derive the expected H, ledger, deferred-check, and F target sets independently and require substantive coverage of each before acceptance.

[severity:medium][technical correctness] F9 still lets the orchestrator waive the auditor by deciding that a completed check "CONFIRMS" the existing verdict and therefore needs nothing more.
Sites: Primary: `SKILL.md:402-409`; confirmed: `SKILL.md:426-429`, `socratic-audit/SKILL.md:49-51,100-104`.
Evidence: Determining whether a result confirms or changes a verdict is itself the dataset/unit/denominator/transformation judgment that the audit contract assigns to the auditor.
Verification: A wrong-denominator calculation can numerically match the prior claim, be classified as confirming, and reach the reconciled summary without the data-reading probes that would expose the error.
Suggested direction: Return every completed new check to the auditor and let the auditor decide whether it confirms or changes the verdict.

[severity:medium][technical correctness] Delta audits cannot preserve the original artifact as promised because every Step 2b invocation writes the same `<topic>.audit.md` path.
Sites: Primary: `SKILL.md:381`; confirmed: `SKILL.md:405-413,423-429`.
Evidence: The recipe says the original audit remains untouched and names the delta audit as provenance, but neither the output path nor the durable artifact name includes the retry label.
Verification: Running Step 2b under `research-audit-2` invokes `codex exec -o` on the first audit's path, overwriting the only durable original before reconciliation.
Suggested direction: Preserve attempt-specific audit files and publish a canonical latest artifact only after validation without deleting predecessors.

[severity:medium][real Why] An empty research brief passes both the new Step 2 leaf guard and `dstack`'s stdin checks, allowing structurally valid but goal-free research to complete Phase 3.
Sites: Primary: `SKILL.md:127-131`; confirmed: `SKILL.md:178-186,434-478`, `claude/bin/dstack:1035-1037`.
Evidence: The brief guard checks aliasing but not readability or non-emptiness, and `dstack` requires only a readable regular file.
Verification: A zero-byte unaliased brief reaches Codex with only the static prompt; a generic nine-section artifact with one source satisfies every stated research fallback condition.
Suggested direction: Require the brief to be readable and non-empty before allocating or launching the research round.

[severity:low][security] Audit-input assembly failures leak the scratch directory because its cleanup trap is installed only after assembly succeeds.
Sites: Primary: `SKILL.md:361-370`.
Evidence: Any failed `printf`, `cat`, or redirection exits at line 369 before line 370 arms cleanup.
Verification: Removing or invalidating the second input after its guard leaves a partial `audit-input.txt` and its scratch directory behind.
Suggested direction: Arm unconditional pre-launch cleanup immediately after `mktemp`, then replace it with the exit-record-gated trap when launch begins.

Omitted-detail: 0 low

GPT verdict: reject — Untrusted deferred checks can expose local data, while incomplete coverage, unaudited reconciliation, overwritten audit provenance, and empty-brief acceptance remain concrete blockers.

## Carried decisions
- F1–F9: fixes stand as sharpened here — F8's repair is deepened by F11, F9's is replaced
  by F12's always-delta rule, F1/F6's leaf class gains F14's brief-content test and
  F15's trap ordering.
- F10 (high, confused-deputy deferred checks): ACCEPTED — Step 2a gains input
  AUTHORIZATION (legitimate inputs are public internet-addressable sources and
  scratch-derived files from them; a spec naming a local path outside scratch, a private
  or internal service, or anything credentialed is `not-run (unauthorized input)`), and
  recording is BOUNDED (derived value/comparison plus the few justifying lines, never
  wholesale contents — everything recorded rides into Step 2b's stdin).
- F11 (medium, coverage predicates incomplete): ACCEPTED — per-item research reading (a
  checkable H beside `ledger: none` is the defect at finer grain), audit-side substantive
  per-target coverage with expected sets derived from the artifact, unresolved-column
  parking counts only for checks that genuinely could not run, token-F over a
  finding-rich artifact is breakage.
- F12 (medium, orchestrator classifies "confirms"): ACCEPTED — every executed new-check
  result returns to the auditor via a delta audit; the orchestrator never classifies
  confirm/change; bounded termination (a third round of new checks marks affected claims
  `unverifiable (unstable check set)` and stops, recorded).
- F13 (medium, delta overwrites the original audit): ACCEPTED — attempt-suffixed audit
  artifacts (`ATTEMPT` parameter drives both label and `-o` path); predecessors never
  overwritten; Step 3 links the accepted attempt. GOAL.md artifact-name assumption
  amended.
- F14 (medium, empty brief): ACCEPTED — the brief now requires the full input test
  (regular, non-symlink, readable, non-empty, unaliased) before allocation.
- F15 (low, scratch leak on assembly failure): ACCEPTED — unconditional cleanup armed at
  mktemp, swapped for the exit-record-gated trap at launch.
- Standing context: no-new-tests repo policy; install.sh untouched by this unit.

Consensus: disagreed
