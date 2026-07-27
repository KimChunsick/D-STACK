# Maintainer response — Round 006

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium] The semantic guard accepts direct negations.** Accepted, fixed, and both examples are
exactly right — `verdict of PARALLEL` survives inside "a verdict of PARALLEL must never permit
concurrent execution", and `OUTRANKS` inside "`frontend-dev` never OUTRANKS orchestrator retention".
A token cannot carry polarity. The pins are now whole normalized decisions held in a `pins.txt`
heredoc: `parallel-when` must contain an entry EQUAL to `a checker plan verdict of PARALLEL for the
exact candidate set`, and the other five keys must contain their full canonical sentence
contiguously. A sentence that long cannot be negated without editing the sentence, which is the
failure the check is for. Controlled with `r_neg` and `r_neg2`, the reviewer's own two examples.

**[medium] `safe_load` reads only the first document of a stream.** Accepted, fixed. `parse_stream`
walked every document for duplicate keys while the value used for validation came from the first
one, so a second document in the same fence carried a regressed node past everything. Each fence
must now hold exactly one AST document, and the count is reported when it does not. Controlled with
`r_multi`.

**[low] Review-control language in the artifacts.** Partly fixed, partly carried, and the split is
worth being exact about. `task.md` now records what `AGENTS.md` states as provenance rather than
asserting a constraint. The sentence in round 005's sealed file about how this loop would close is
not rewritten — sealed rounds are the audit trail, and editing one to look better in a later round
is worse than the sentence. It is recorded in round 006 instead. And the `AGENTS.md` text still is
not in the bundle, because adding a file mid-loop is what this pipeline's own ratchet forbids;
carried as a follow-up.
