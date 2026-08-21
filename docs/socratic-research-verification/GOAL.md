# GOAL — Socratic and data-grounded verification in the research pipeline

## Goal (the one Why)
Research findings currently enter decomposition on the strength of citations alone. The
user wants every research round to earn its conclusions: hypotheses and findings must
survive Socratic interrogation (definitions, assumptions, evidence, counterexamples,
implications), claims that can be checked with data must be checked with data, and the
data readings themselves must survive the same interrogation — so false or overconfident
premises die in P3 instead of shaping the Goal.

## Interview record (Phase 4)
- Q: Execution structure for the Socratic verification? → A: **a separate
  cross-examination invocation** — a second codex call in a fresh context, acting as an
  evidence auditor (the measured-effective design), accepting ~2x research cost.
- Q: How far does data verification go? → A: **ledger + deferred execution** — the
  researcher writes a data-check ledger per checkable claim (source, version, unit,
  denominator, transformation, recomputed/quoted value) and flags checks that need
  execution; the orchestrator runs those locally; the executed results are themselves
  interrogated.
- Q: When does this layer run? → A: **every research round**, as the user's request
  states.
- Recorded assumptions (careful-colleague readings, stated for review rather than asked):
  - Ordering keeps two invocations total: research (with enumerated hypotheses + ledger +
    deferred-check list) → orchestrator executes the feasible deferred checks and records
    outputs → audit invocation interrogates claims AND the recorded data results in one
    fresh-context pass. Checks the audit itself surfaces are run and recorded; a delta
    audit happens only when their outcome would change a decision-critical verdict.
    [Amended by T03 review round 002: a delta audit runs for ANY verdict-changing check
    outcome — an orchestrator-assigned verdict would be an unaudited data reading —
    while decision-criticality decides only Phase 4 re-entry.]
  - The interrogation record is compact and structured — open-form probes (definitions,
    assumptions, evidence, counterexamples, implications, data reading), grounded
    answers with fresh citations, and a per-claim verdict (upheld / weakened / refuted /
    unverifiable). Yes/no probe forms are banned (CoVe measured models agreeing with
    yes/no premises right or wrong). No theatrical dialogue transcripts.
  - The auditor is a new Codex skill (`socratic-audit`), following this repo's
    role-per-skill pattern; it runs GPT-5.5 xhigh like research (the mechanism is the
    fresh context and grounding, not a bigger model).
  - Audit artifacts live beside the research artifact: `<topic>.audit.md` and
    `<topic>.data-checks.md` under `docs/<goal>/research/`; the audit run label is
    `<goal>-research-audit`. [Amended by T03 review round 003: audit attempts are
    suffixed — label `<goal>-research-audit-2`, artifact `<topic>.audit-2.md` — so a
    retry or delta audit never overwrites its predecessor's output.]
  - A refuted decision-critical premise routes back through the existing rule ("if
    research contradicts captured intent, return to P4"); the GOAL.md research summary
    now carries per-claim verdicts.
  - Audit fallback mirrors the research fallback: nonzero after one retry, or output
    missing its required sections → the orchestrator performs the Socratic examination
    itself and records that the degraded path ran; never silently skip.

## Research summary (Phase 3)
Artifact: [research/socratic-and-data-verification.md](research/socratic-and-data-verification.md) — 17 unique sources.

Key findings:
- The evidence-backed shape is NOT a self-written Socratic dialogue: it is claim
  decomposition → open-form (never yes/no) verification questions answered in a context
  that does NOT see the original answer (CoVe's factored variant) → evidence/data/tool
  grounding → synthesis with an explicit unverified residue.
- Cross-examination with role separation works with the same weights: LM vs LM detected
  >70% of incorrect claims at >80% precision using examiner/examinee prompts.
- Intrinsic self-correction without external feedback is unreliable (ICLR 2024; TACL 2024
  survey: no reliable general-task self-correction from prompted feedback alone); external
  grounding — retrieved sources, primary data, executed code — is what makes critique work.
- Data verification should be contracted as a data-check ledger per checkable claim:
  dataset/API/table, date/version, unit, denominator, transformation, recomputed or
  source-quoted value, and a deferred-executable flag when only the orchestrator can run
  it. Execution-based checks beat prose-only judging across benchmarks; LLMs misread data
  often enough (BLADE best F1 44.8%, CORE best 45.93%) that the data reading itself needs
  interrogation — which directly supports the user's third requirement.
- Cost signal: a second invocation roughly doubles research cost/latency (SocREval stayed
  under 2.1x); Socratic decomposition methods ran ~9 calls per instance.

Strongest against-the-goal point: a Socratic layer written by the model that authored the
claims, in the same context, is verification theater — the self-correction literature's
strongest general result is negative for exactly that design. Multi-agent debate can even
be harmful under persuasive adversaries; a second agent must be an evidence auditor, not a
debater. The mitigation is structural: separate context, open-form questions, mandatory
grounding, and a ledger rather than a transcript.

Unverified: no controlled result compares a separate Codex invocation against
same-invocation self-checks for delegated research artifacts specifically; published cost
multipliers are proxies, not measurements of this pipeline; whether GPT-5.5 mirrors the
studied models' self-correction behavior is unmeasured.

## Milestones & tasks (Phase 5)

Review granularity: per task

### M1 — socratic-research-verification
- [x] **T01** research-contract-hypotheses-ledger — extend the Codex research contract so
  every research artifact enumerates its hypotheses/claims as stable targets and carries a
  data-check ledger (source, date/version, unit, denominator, transformation,
  recomputed/quoted value) plus a deferred-executable-checks list for the orchestrator.
  deps: []; files: [codex/skills/adversarial-research/SKILL.md]
- [x] **T02** socratic-audit-skill — add the new `socratic-audit` Codex skill (fresh-context
  evidence auditor: open-form Socratic probes over claims and data readings, fresh-source
  grounding, per-claim verdicts, no yes/no probes, no debate persona) with its plumbing:
  install.sh map entry, .gitignore allow lines, secret-guard pinned negation list and hash
  pin in the same change. deps: []; files: [codex/skills/socratic-audit/SKILL.md,
  install.sh, .gitignore, tests/secret-guard.sh]
- [ ] **T03** orchestrator-recipe — rewire the codex-research skill: pass-1 prompt demands
  the new sections, a new step runs the feasible deferred checks locally under
  untrusted-data discipline and records them, a new step launches the audit invocation
  (hardened dstack shape, label `<goal>-research-audit`), fallback rules for a failed
  audit, and the GOAL.md summary step now records per-claim verdicts. deps: [T01, T02];
  files: [claude/skills/codex-research/SKILL.md]

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [ ] M1 E2E: one real research round runs the full new pipeline — research with
  hypotheses + ledger, orchestrator-run deferred checks, fresh-context audit with
  per-claim verdicts — captured end to end
- [ ] GOAL E2E: the captured round's artifacts demonstrate all three user requirements
  (Socratic verification of hypotheses/findings, data verification where feasible,
  Socratic verification of the data results), and the contracts that produced them are
  the installed, symlinked ones

## Goal closure record (2026-08-22)
HALTED BY USER before the E2E gates. T01/T02/T03 are implemented, adversarially reviewed
to consensus (5 rounds each for T02/T03), committed (eff69c1, 59331bb, bc69880), and
live through the installed symlinks. The M1/GOAL E2E round was launched (label
`socratic-research-verification-research-2`) and stopped mid-run at the user's direction;
both E2E boxes above are honestly UNCHECKED — no research round has exercised Steps
2a–2c end to end yet. The next real Goal's P3 runs the new pipeline unconditionally and
will provide the behavioral confirmation. Recorded follow-ups riding out of the review
loops: T02 F10 and T03 F20 (both fixed at their round-005 cap seals, neither
reviewer-re-verified), T03 F19 (regenerate this Goal's own research artifact through the
new pipeline). The docs were deregistered from the Stop-hook gate at closure.
