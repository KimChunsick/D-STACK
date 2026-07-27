# Maintainer response — Round 001

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium][real Why] `claude/CLAUDE.md` section 0 flattens two exceptions.** Accepted, fixed. The
sentence subjected frontend work to the task-shape gate (section 0.2 delegates it unconditionally)
and claimed all review-fix rounds stay with the orchestrator (T02's P9 rule returns a finding
contained in one task's declaration to that task's worker). Section 0 now carries both carve-outs.
This is the finding that most justifies the task existing: section 0 loads in every session in
every repository, and the skill it summarizes does not.

**[medium][technical correctness] The regression guard is not scoped and misses the live wording.**
Accepted, fixed, and the diagnosis is right in both halves. Rewritten block-scoped: `fanout_block()`
extracts `worker-fanout`'s `requires:` and `parallel-when:` blocks from the YAML and asserts where
the PARALLEL condition sits, with an empty extraction failing rather than passing vacuously.
Re-controlled with the live string `- a checker plan verdict of PARALLEL for the exact candidate
set`: moving it into `requires` now fails twice, the older phrasing under `parallel-when` passes,
and the unmodified file reports zero failures.

The reviewer's suggested direction was to parse the YAML properly rather than text-match. Adopted in
substance — block extraction — but not by adding a Ruby YAML walk: the harness already parses the
document once for validity, and this repo's policy caps the check set rather than growing it.

**[low][DX] Duplicate `Files changed (where / why)` section.** Accepted, fixed. My replacement left
the scaffold's placeholder behind. Fourth time this Goal; the template edit is the actual fix and it
is carried as a follow-up rather than done mid-review.
