# Codex adversarial review — Round 005

## Review scope
Re-review (per-task 5-round cap) | serial | bundle 76255 bytes (round 004: 65132) | label socratic-research-verification-t03-r005

## GPT findings
Re-review verification: F16, F17, and F18 are closed. The inert-data prohibition is explicit, H/F summary coverage is enforced in both acceptance and fallback, and the cleanup condition matches `dstack`'s directory-based `.launch` claim. All three Bash fences pass `bash -n`. F19 remains the accepted low-priority follow-up.

[severity:low][technical correctness] The advertised `--output-schema` option remains incompatible with the Markdown-only downstream flow. At this final-round cap, record this as F20 rather than opening another fix round.
Sites: Primary: `claude/skills/codex-research/SKILL.md:289`; confirmed: `claude/skills/codex-research/SKILL.md:195,295,480`.
Evidence: The option permits semantic blocks as JSON fields, but Step 2a searches for a literal `## Deferred executable checks` heading and fallback requires the pinned Markdown sections.
Verification: A valid schema object containing all three required fields has no Markdown headings, so Step 2a cannot locate the check list and the missing-section gate rejects the artifact.
Suggested direction: Remove the option or define schema-aware extraction and acceptance throughout Steps 2a–2c and fallback.

Omitted-detail: 0 low

GPT verdict: approve-with-fixes — F16–F18 are closed, with only the recorded low-severity F19 and F20 follow-ups remaining.

## Carried decisions
- F16, F17, F18: verified CLOSED by this round (inert-data rule explicit, H/F summary
  coverage in acceptance and fallback, cleanup condition matches dstack's
  directory-based `.launch` claim; fences `bash -n` clean).
- F19 (low): stands as the recorded follow-up — regenerate and audit this Goal's own
  research artifact through the new pipeline; non-blocking.
- F20 (low, `--output-schema` contradicts the Markdown-only flow): FIXED at this seal —
  the bullet now forbids the option for this flow and explains why (Steps 2a–2c and the
  fallback gate on pinned Markdown headings; schema fields remain available to OTHER
  callers of the research contract). This round is the per-task 5-round cap, so the loop
  closes under codex-review §4: the fix is recorded here and in task.md, it has NOT been
  independently re-verified by a further reviewer round, and it is named in the final
  report.
- Standing context: no-new-tests repo policy; install.sh untouched by this unit.

Consensus: resolved
