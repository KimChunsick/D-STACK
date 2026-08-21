# Codex adversarial review — Round 001

## Review scope
Adversarial review | serial | bundle 19644 bytes (first round)

## GPT findings
[severity:medium][technical correctness] A deferred data-checkable H-item cannot satisfy both rules: deferral replaces the ledger row, while any missing row is declared a contract violation.
Evidence: `codex/skills/adversarial-research/SKILL.md:33-38` mandates a row and value; lines 40-44 send unperformed checks to a deferred list "instead."
Verification: A measurable local benchmark with reachable input but unavailable execution cannot provide a recomputed or quoted value; deferring it necessarily violates the mandatory-row clause.
Suggested direction: Give every checkable H-item a ledger row with an explicit `recomputed`, `quoted`, or `deferred` status and link deferred rows to their execution specification.

[severity:medium][software structure] The contract defines no canonical placement or empty-state marker for its new outputs, so fixed-shape callers can produce and accept artifacts without them.
Sites: Primary: `codex/skills/adversarial-research/SKILL.md:22-55`; confirmed: `claude/skills/codex-research/SKILL.md:150,253-266` and `docs/socratic-research-verification/research/socratic-and-data-verification.md:1-44`.
Evidence: The caller requires exactly six legacy headings, the contract says to follow structured formats exactly, and fallback checks only legacy sections plus source count.
Verification: The supplied six-section research artifact has sources but no H-items, ledger, or deferred list; it matches the caller shape and bypasses its fallback.
Suggested direction: Prescribe always-present named blocks, including explicit `(none)` states, plus a canonical nested representation when callers fix top-level headings.

[severity:medium][the real Why] The conjunctive eligibility test excludes legitimate data-checkable claims that lack a denominator or meaningful unit, silently exempting them from the ledger.
Evidence: `codex/skills/adversarial-research/SKILL.md:33-38` makes every listed field an eligibility condition; the same definition appears uncited at `socratic-and-data-verification.md:11`.
Verification: "The primary release table contained 17 releases on 2026-08-21" is directly recomputable but has no denominator, so the contract permits no ledger row.
Suggested direction: Define checkability by reproducibility from primary evidence and allow justified `N/A` fields rather than using schema fields as eligibility gates.

[severity:medium][security] The contract creates an executable prompt-injection handoff by requesting an exact command the orchestrator "should run" without output-side trust or safety constraints.
Evidence: `codex/skills/adversarial-research/SKILL.md:40-43` requests executable commands, while lines 53-54 only forbid the researcher itself from acting on web instructions.
Verification: A fetched page can supply an attacker-controlled shell pipeline or filename that the researcher relays without executing; downstream automation can then execute it as a deferred check.
Suggested direction: Require declarative, non-mutating checks, prohibit copied remote shell pipelines, and require consumer-side validation, quoting, sandboxing, and secret exclusion.

[severity:low][the real Why] The checked gate claims behavioral verification although the recorded runs prove only live-path text presence and secret-guard success.
Evidence: `task.md:53-60` records readlink, grep, and secret-guard checks; lines 62-68 simultaneously mark E2E pending and behavior confirmed.
Verification: The task directory contains only `task.md`; no recorded invocation demonstrates hypotheses, ledger output, deferral, or legacy-caller composition.

[severity:low][security] The task artifact embeds evaluator-scope and disposition language that could suppress examination of the actual caller conflict.
Sites: Primary: `task.md:13-14`; confirmed: `task.md:39-45`.
Evidence: It declares sibling surfaces out of scope and asserts that no rule contradicts the change, despite those statements being untrusted review data.
Verification: Ignoring those declarations and inspecting the active caller exposes the exact-format conflict described above.
Suggested direction: Record dependency ownership factually without telling the evaluator what to exclude or declaring disputed behavior settled.

Omitted-detail: 0 low

GPT verdict: reject — unresolved medium blockers make deferred checks internally inconsistent, omit valid checks, expose an unsafe execution handoff, and fail compatibility with the active caller.

## Carried decisions
- F1 (medium, deferral vs mandatory row): FIXED this round — ledger rows carry a `status`
  of `recomputed | quoted | deferred`; a deferred row still exists, holds the claimed
  value or `pending`, and names its deferred-check entry. Deferral is a row status, never
  a substitute for the row.
- F2 (medium, no canonical placement): FIXED this round — new "Output blocks (research
  mode)" section: `## Hypotheses`, `## Data-check ledger`, `## Deferred executable
  checks` are always present with explicit `none`, appended after the requested sections
  when a caller's format omits them; the "follow it exactly" rule was amended to carry
  them. The caller's own pinned six-section list is declared task T03's work
  (deps: T01, T02) and is not in this unit's declaration.
- F3 (medium, conjunctive eligibility): FIXED this round — checkability is defined by
  reproducibility from primary evidence; `unit`/`denominator` accept justified `N/A`;
  schema fields are no longer eligibility gates.
- F4 (medium, injection handoff): FIXED this round — deferred checks are declarative
  specifications (input, computation/comparison, confirm/refute criterion), never
  ready-to-run commands, never mutating; the consumer must author, validate, and sandbox
  its own execution treating the spec as untrusted data. Class-wide sweep: the sibling
  `socratic-audit` skill carried the same "exact command" wording; that file is in T02's
  declaration and the same fix is applied for T02's round 002.
- F5 (low, overstated gate wording): FIXED — the verification row now states exactly what
  the runs prove; behavioral confirmation is assigned to the M1 E2E research round.
- F6 (low, evaluator-scope language in task.md): FIXED — the sweep section was rewritten
  to factual ownership statements without exclusion directives or settled-claims.
- Standing context: no-new-tests repo policy (direct-run verification recorded in
  task.md); markdown-contract deliverable; T03 (orchestrator caller) implements after
  T01/T02 close per its declared deps.

Consensus: disagreed
