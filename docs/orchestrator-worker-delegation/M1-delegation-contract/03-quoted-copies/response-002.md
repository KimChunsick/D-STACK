# Maintainer response — Round 002

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium][real Why] Exploratory frontend work matches two rules with no precedence.** Accepted,
fixed. Round 001's repair stated both facts and stopped there, which left the collision the reviewer
names: an exploratory frontend task satisfies "exploratory work stays with the orchestrator" and
"frontend is delegated unconditionally" at the same time. Section 0 now says outright that 0.2
outranks the whole retention list, so exploratory frontend still goes to `frontend-dev`. This is not
a new decision — `frontend-takes-precedence` in SKILL.md already said it, and it is P4's Q4 answer.
The summary had lost it, which is exactly the failure mode this task exists to fix.

**[medium][technical correctness] The guard still treats raw text as schema data.** Accepted, fixed,
and the reproduction is correct: a comment-only body is non-empty to `-z`, matches `grep -qi
PARALLEL`, and loads as `nil`. The suggested direction was to use the existing Ruby YAML parse and
inspect actual entries; adopted as written. The harness already extracts and parses every fenced
YAML block for validity, so the extraction was hoisted above both checks rather than duplicated, and
the placement check now reads `scheduling.modes.worker-fanout` from the parsed document: all four
lists must be non-empty Arrays, PARALLEL must appear among `parallel-when`'s entries and must not
appear among `requires`'. A missing `worker-fanout` node fails rather than passing vacuously.

Re-controlled with three cases, including the reviewer's own: live line moved into `requires` fails
twice; the older phrasing under `parallel-when` passes; `requires:` gutted to comments fails with
`parsed as NilClass`. Recorded in `task.md`.

No ruby means the placement check is skipped with a printed `skip` line, matching the file's
existing policy for its YAML-parse checks. Worth naming as a residual rather than leaving implicit.
