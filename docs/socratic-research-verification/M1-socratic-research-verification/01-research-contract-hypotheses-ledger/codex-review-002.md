# Codex adversarial review — Round 002

## Review scope
Re-review | serial | bundle 29917 bytes (round 001: 19644)

## GPT findings
[severity:medium][technical correctness] The canonical-block fix cannot satisfy exact closed structured formats because it mandates literal Markdown headings outside every requested shape that omits them.
Sites: Primary: `codex/skills/adversarial-research/SKILL.md:60-67`; confirmed: `codex/skills/adversarial-research/SKILL.md:77-79`.
Evidence: The contract simultaneously says "Whatever output shape," requires three `##` headings, and requires exact structured-format compliance.
Verification: A JSON-only schema with `additionalProperties: false` rejects trailing Markdown, while omitting the headings makes the artifact contractually incomplete.
Suggested direction: Require three semantic fields encoded inside the caller's format, reserving Markdown headings and appending for Markdown callers.

[severity:medium][technical correctness] The eligibility rule excludes one of its own deferred cases: primary evidence must be "reachable" before an H-item is checkable, but deferral covers data the researcher could not reach read-only.
Evidence: `codex/skills/adversarial-research/SKILL.md:32-35` requires a reachable dataset/API/table; lines 42-44 permit deferral for unreachable data.
Verification: A primary dataset unavailable to the research environment but available to the orchestrator should produce a deferred row, yet the literal eligibility rule permits classifying it as non-checkable and omitting it.
Suggested direction: Define checkability by an identified primary input and distinguish researcher accessibility from downstream executability.

[severity:low][software structure] The supplied research record retains the rejected conjunctive eligibility definition, leaving the design evidence inconsistent with the corrected contract.
Evidence: `socratic-and-data-verification.md` requires unit, denominator, and an available primary dataset, while the contract says schema fields are not eligibility gates and permits justified `N/A`.
Verification: A primary-table count of 17 has no meaningful denominator; it qualifies under the contract but not under the research record.

[severity:low][security] The claimed F6 fix is absent because the task snapshot still embeds an `Out of scope:` evaluator directive instead of recording ownership only.
Evidence: `task.md:13-14` directs exclusion of the auditor and orchestrator surfaces, while the carried decision says evaluator-scope language was removed.
Verification: Ignoring the directive did not suppress this review because the authoritative prompt independently established the same boundary, but the injection-shaped wording remains.

Omitted-detail: 0 low

GPT verdict: reject — closed structured formats remain unsatisfiable and inaccessible-but-deferable evidence can still escape the mandatory ledger.

## Carried decisions
- F1-F4 (round 001 mediums): verified fixed by round 002; F5 verified; F6 partially — see F8.
- F7 (medium, closed-format unsatisfiability): FIXED this round — the three blocks are now
  SEMANTIC requirements: literal `## …` headings (filled or appended) in Markdown shapes,
  schema-provided fields in structured shapes; a closed schema with no such fields cannot
  yield a complete research-mode artifact, is flagged in whatever channel the format
  leaves, and is named the caller's defect — never silent compliance, never silent drop.
  The format rule was re-worded to the same semantic framing.
- F8 (medium, "reachable" excludes deferred-only data): FIXED this round — checkability is
  decided by an IDENTIFIED primary input (named dataset/API/table + measurable variable +
  scope + date/version); researcher access decides only the row `status` (unreachable
  input = deferred row, never non-checkable).
- F9 (low, research record retains the old conjunctive definition): accepted as a noted
  divergence, not edited — the P3 research artifact is an immutable record of what
  research reported; its line 11 is the origin of the defect F3 fixed, and the contract
  supersedes the record. Retro-editing evidence to match fixes would falsify the record.
- F10 (low, residual `Out of scope:` directive in task.md): FIXED this round — the
  Deployment-context sentence now records file OWNERSHIP factually ("the audit skill file
  sits in T02's declaration..."), no exclusion directive. The same phrasing exists in
  T02's task.md, which sits inside T02's currently OPEN round-002 bundle (freeze rule);
  it is fixed there the moment that round seals, and in T03's task.md when T03 opens.
- Standing context: no-new-tests repo policy; caller file (pinned section list) is
  declared task T03's work.

Consensus: disagreed
