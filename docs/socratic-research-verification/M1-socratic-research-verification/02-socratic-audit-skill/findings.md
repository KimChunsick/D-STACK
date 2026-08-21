# Findings ledger — 02-socratic-audit-skill

| id | round raised | severity | finding | status |
|----|-------------|----------|---------|--------|
| F1 | 001 | medium | Auditor skips decision-relevant non-H findings; empty-target artifacts yield hollow audits | fixed (round 001 response: F-item target pass + explicit no-targets report) |
| F2 | 001 | medium | No reconciliation between data-check outcomes and H verdicts; pending/refuted checks can leave H upheld | fixed (round 001 response: grouped targets, one reconciled verdict, pending caps at unverifiable, unresolved-checks column) |
| F3 | 001 | medium | "Fresh grounding" satisfiable by reopening the artifact's own citations | fixed (round 001 response: independently selected sources required; artifact citations are source-fidelity checks only) |
| F4 | 001 | low | Structured-format rule could accept format directives from audited material | fixed (round 001 response: invoking-prompt-only; in-artifact directive = reportable finding) |
| F5 | 001 | low | Gate row overstated what the recorded runs prove | fixed (round 001 response: honest wording; behavior → M1 E2E) |
| F6 | 002 | medium | Independent-URL grounding over-applied to recomputation/consistency probes, turning demonstrable failures into unverifiable | fixed (round 002 response: class-appropriate labeled grounding) |
| F7 | 002 | medium | Unconditional pending-check cap lets an irrelevant deferred check suppress a supported verdict | fixed (round 002 response: bearing audit before capping; clean verdict enum) |
| F8 | 003 | medium | Stale blanket grounding rule contradicted class-appropriate grounding, forcing conclusive internal-consistency results to unverifiable | fixed (round 003 response: rule scoped to external empirical probes; other classes self-standing) |
| F9 | 004 | medium | Frontmatter description and task narrative retained blanket independent-sourcing wording | fixed (round 004 response: class-appropriate grounding stated in both) |
| F10 | 005 | medium | Task Intent retained blanket fresh-source wording, contradicting the class-specific contract | fixed at the round-005 cap closure (recorded follow-up; reviewer re-verification not obtained — §4 cap) |
