# Findings ledger — 01-research-contract-hypotheses-ledger

| id | round raised | severity | finding | status |
|----|-------------|----------|---------|--------|
| F1 | 001 | medium | Deferred checkable H-item contradicts the mandatory-ledger-row clause | fixed (round 001 response: `status` field recomputed/quoted/deferred; deferred is a row status) |
| F2 | 001 | medium | No canonical placement/empty-state for new outputs; fixed-shape callers bypass them | fixed (round 001 response: always-present named blocks with explicit `none`, appended when the format omits them; caller update is T03) |
| F3 | 001 | medium | Conjunctive field eligibility silently exempts valid claims (no denominator/unit) | fixed (round 001 response: reproducibility-based checkability, justified `N/A` fields) |
| F4 | 001 | medium | Deferred "exact command" is an executable prompt-injection handoff | fixed (round 001 response: declarative non-mutating specs, consumer authors/sandboxes execution; class-swept into T02's skill) |
| F5 | 001 | low | Gate row overstated what the recorded runs prove | fixed (round 001 response: honest wording; behavior → M1 E2E) |
| F6 | 001 | low | task.md carried evaluator-scope/settled-claim language | fixed (round 001 response: factual ownership statements only) |
| F7 | 002 | medium | Canonical Markdown headings unsatisfiable under closed structured formats | fixed (round 002 response: semantic blocks encoded per shape; closed-schema case flagged as caller defect) |
| F8 | 002 | medium | "Reachable" eligibility excluded data only the orchestrator can reach | fixed (round 002 response: identified-primary-input eligibility; access selects row status only) |
| F9 | 002 | low | P3 research record retains the superseded conjunctive definition | accepted-residual (immutable evidence; contract supersedes; recorded in carried decisions) |
| F10 | 002 | low | Residual Out-of-scope directive in task.md (F6 fix incomplete) | fixed (round 002 response: factual ownership wording; sibling instances fixed at their seals) |
| F11 | 003 | medium | Partial closed schemas (some but not all block fields) could silently drop mandatory blocks | fixed (round 003 response: any-missing-block = incomplete; encode carried, flag missing) |
| F12 | 003 | low | Task record stale versus the round-002/003 contract shape | fixed (round 003 response: task doc updated) |
| F13 | 004 | medium | No defined behavior when a closed schema carries neither blocks nor a flag channel | fixed (round 004 response: first-line refusal before generation) |
| F14 | 004 | low | Residual stale task-record passages (universal appending, two-section count) | fixed (round 004 response: passages updated) |
| F15 | 005 | low | Task-record bullet omitted the no-channel refusal branch | fixed (round 005, in-round under the low-only closure rule) |
