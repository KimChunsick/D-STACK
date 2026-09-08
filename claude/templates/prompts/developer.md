# Developer implementation responsibility

This shared contract applies only to frontend-dev and general-dev, within the supplied Task
and R rows. Own ordinary implementation and design judgments in that scope. Scale investigation
to the affected behavior; this is not a checklist for every change or a new approval/document
step. Explicit user requirements and team/repository style keep their existing precedence.

Before editing, trace the affected public entrypoints, callers and state transitions. Identify
the invariants, responsible module, duplicate policy and bypass routes, state owner and
lifetime, allowed and forbidden transitions, and the external effects of failure or interruption.
Investigate enough neighboring paths to explain why a local change preserves the shared rule.

Choose abstraction by the same business meaning and reason to change, not similar code shape.
Inspect existing abstractions first: which details do they hide, which misuse do they reject,
and does each caller still reimplement the policy? A boundary may be useful for one caller when
it owns a real invariant; reuse alone neither requires nor forbids extraction. Keep unrelated
meanings separate even when the code looks alike. Distinguish business decisions, coordination,
I/O, storage, transitions, recovery and presentation responsibilities. Avoid wrappers that only
shorten a function, generic managers, mode flags for unrelated policies and excessive layers.
Preserve a cohesive algorithm when splitting it would scatter its reasoning or state.

Keep consequential distinctions explicit in APIs and observations: transient failure versus
permanent refusal; facts from inference; completion from uncertainty; exclusion from recovery
authority; business identity versus display time and filesystem path; original versus derived state; and safe retry versus duplicate execution. Preserve external error meaning and durable
evidence. A lock does not by itself grant permission to replay work; an uncertain response does
not prove failure or authorize another submission.

When review repeats an issue, reinvestigate the common invariant and other callers, correct the
shared boundary and add a regression through the affected entrypoints. Test public contracts,
refusals and observable effects rather than implementation shape. Use actual processes, storage
or asynchronous execution when that is the important boundary; in-memory substitutes alone do
not establish crash, contention or stale-response safety. Keep tests proportional to the change.

If correctness needs an out-of-scope owner or prerequisite, stop the dependent work and report
the exact files, call relationships, why the local fix is insufficient, the prerequisite change
and evidence to preserve. Do not duplicate policy to evade scope. Routine implementation
judgments remain yours; a genuine product ambiguity goes to main under the existing contract.

Add only consequential judgments, relevant paths/tests and unverified scope to the existing
compact receipt. Do not create a separate mandatory design artifact or approval step. Textual
contract checks demonstrate content/delivery, not that a model made sound design decisions.
