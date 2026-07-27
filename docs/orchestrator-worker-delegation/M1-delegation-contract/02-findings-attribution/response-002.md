# Maintainer response — Round 002

Outside the reviewed corpus. Six findings, all accepted, nothing rebutted. The blocking count went
up, 4 to 5, and that is worth saying plainly rather than burying: fixing four things opened five
more, which is the shape the previous Goal's M1 showed at round 9 and the reason the
non-convergence rule exists.

**[medium] The state machine had no path to `verified` for an ordinary fix.** My own notation put
`expansion-requested` on the only route, so a worker that simply fixed the thing in scope stayed
`assigned` forever. Fixed: `assigned` → `verified` → `closed` is the ordinary path, expansion
rejoins it at `verified`.

**[medium] "A worker may READ anywhere" was an unqualified secret-read capability.** The finding I
should have caught myself, because the same change spends a paragraph on `.env` and service-account
keys living in gitignored paths. Reads are still wider than writes — a worker cannot diagnose
anything if it may only read its own declaration — but the secret deny list now outranks that
permission unconditionally. The reason is specific: a read is not harmless when the bytes land in a
tool-call transcript that persists for 30 days.

**[medium] Taint recovery could not undo what the rule itself named.** I wrote "discard the
worktree, re-create from the recorded base, re-run" one sentence after naming databases and paths
outside the repository. Recreating a checkout does nothing to a mutated database. Repository taint
and external side effects are now separate dispositions, and an uncleaned external effect blocks
sealing rather than being swept into "recovered".

**[medium] Resource isolation was still gating DELEGATION.** The sharpest finding of the round,
because it shows the decoupling was incomplete rather than merely under-described. `requires` is
all-or-nothing, and it held "shared resources isolated per worktree, else serial" — a statement
about CONTENTION. A single delegated task whose tests bind a fixed port has no competitor for that
port, and was nonetheless refused. That is exactly the coupling T01 set out to remove, surviving one
level down. Moved to `parallel-when`, where contention is the actual question.

**[medium] `WorktreeCreate` had no defined handoff.** Correct and concrete: the hook receives a
name and returns a path, and is handed no base commit, branch or fixture list — so the base-identity
requirement had nothing to bind to, and the orchestrator cannot inspect a worktree it did not create
before briefing. The orchestrator now writes a durable intent record under `.dstack/` keyed by that
same name, holding the recorded base SHA, the task branch and the named fixture list; the hook reads
it, verifies it, and emits NO PATH when it is missing or mismatched. Emitting nothing is the
fail-closed direction.

**[low] The record embedded reviewer-facing directives.** Accepted, and it is the second time this
class landed in this Goal. My Round-001 repair opened with an imperative aimed at the reviewer and
asserted that other units' reviews were closed — which, obeyed, would have excluded supplied hunks
from examination. Rewritten as neutral provenance. Provenance is data; scope is the caller's. Both
earlier drafts of that paragraph are recorded in the task document, because the failure mode is more
useful than the fix.
