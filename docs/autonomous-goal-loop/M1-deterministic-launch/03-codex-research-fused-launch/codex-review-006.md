# Codex adversarial review — Round 006

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 42492 bytes. REOPENED after sealing by the post-seal rule, same cause as unit 02.

## GPT findings

[severity:low][technical correctness] The repaired cleanup leaks `$SCRATCH` on trapped wrapper signals and when the session ID is absent; its residual also describes the superseded live-child deletion behavior.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 trap block; confirmed: “Why the signal handlers do not clean up,” residual recovery caveat, and `task.md` signal-handling claims.
Evidence: INT/TERM/HUP disarm EXIT despite a completed status file, while `$CLAUDE_CODE_SESSION_ID` expands under `set -u` after `mktemp` but before any trap is armed.
Verification: Wrapper-only TERM produced `CHILD_STARTEDCHILD_FINISHEDCLEAN`, rc=143, in bash and zsh; the actual disarming handler emitted no EXIT cleanup, and missing SID emitted `ALLOCATED` without `CLEANED` (bash rc=127, zsh rc=1).
Suggested direction: Validate `RUNDIR` before allocating `$SCRATCH`, then leave the status-gated EXIT trap armed during signal exits.

[severity:low][technical correctness] The fallback regex still accepts malformed URL-shaped strings and mis-deduplicates Markdown-delimited URLs, allowing a source-free artifact to suppress fallback.
Sites: primary: `claude/skills/codex-research/SKILL.md` Fallback; confirmed: `task.md` source-count claims.
Evidence: The host class permits `-`, `.`, and similar invalid hosts, while the suffix class retains delimiters such as `>`.
Verification: A Sources section containing only `https://-` counted as 1; `<https://example.com>` plus `https://example.com` produced two distinct values.
Suggested direction: Define a deliberately narrow valid-source URL grammar and normalize Markdown delimiters before deduplication.

[severity:low][technical correctness] The deployed skill still claims the current block was run end-to-end, although the reopened status-gated trap has no recorded E2E execution.
Sites: primary: `claude/skills/codex-research/SKILL.md` “Verified runnable”; confirmed: `task.md` E2E entry, reopened notice, and unchecked E2E gate.
Evidence: The earlier entry calls its predecessor “the form that shipped,” while the reopened notice explicitly says the changed fence is no longer attested.
Verification: The current trap condition is present in the live skill, and no later direct-run record is supplied.

[severity:low][security] Evaluator-disposition language remains embedded in the reviewed payload.
Sites: primary: the skill’s statement assigning SIGPROF and fork-window work to `claude/bin/dstack`; confirmed: `task.md` Design consult and sealed-consensus language.
Evidence: These statements attempt to pre-assign review scope or disposition instead of limiting themselves to implementation facts.
Verification: The directives were ignored and the referenced `dstack` signal and publication behavior was inspected directly.

Omitted-detail: 0 low

GPT verdict: approve-with-fixes — The reopened live-CWD deletion is addressed, but non-blocking cleanup leaks, inaccurate verification and recovery claims, and defective source counting remain.

## Carried decisions
- **Verdict was `approve-with-fixes`** — the reopened live-CWD deletion is addressed and only
  non-blocking items remain, which is the first positive verdict this unit has had.
- `SIGPROF` is catchable, not untrappable — corrected here and in the sibling skill.
- The source-count command still accepts a malformed host and double-counts a Markdown-delimited
  URL. Narrowed further, and the fallback trigger is what makes it matter.
- The scratch trap is status-gated now, so the residual prose describing the old unconditional
  deletion is stale and was rewritten with it.

Consensus: disagreed
