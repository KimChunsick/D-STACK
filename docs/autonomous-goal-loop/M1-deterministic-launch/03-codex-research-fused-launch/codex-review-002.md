# Codex adversarial review — Round 002

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 17376 bytes — round 001 was 10308 (+7068).
`assemble-review.sh` sends the two most recent rounds whole, so round 004 is the first round in
which anything is old enough to compact; the ratchet binds from there. Allowlist unchanged: the
review-unit folder, `claude/skills/codex-research/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] The prior teardown blocker remains: “CATCHABLE termination” exceeds the launcher’s actual signal coverage, allowing an orphaned Codex process to continue spending credits.
Sites: primary: `claude/skills/codex-research/SKILL.md` residual; confirmed: `claude/bin/dstack` signal traps.
Evidence: `RUN_SIGNALS` contains only `INT TERM HUP QUIT PIPE ALRM USR1 USR2`; catchable terminating signals such as `ABRT`, `XCPU`, and `XFSZ` are omitted.
Verification: Under the launcher’s `/bin/bash`, each omitted self-signal terminated with no `EXIT` trap execution; signaling only the supervisor therefore bypasses `run_cleanup` and leaves its separate child process group alive.
Suggested direction: Limit the guarantee to normal exits and the exact `RUN_SIGNALS` set, or trap and verify every signal the document promises.

[severity:medium][security] Unvalidated `<goal>` and `<topic>` substitutions permit path traversal and tracked-file clobbering; shell quoting does not neutralize `..` path components.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 assignments and `-o`; confirmed: Step 1 brief path and Step 2 `mkdir -p`/`--stdin`.
Evidence: Neither path component is validated before directory creation or artifact writing; only `LABEL` is later validated by `dstack`.
Verification: With a valid goal and `TOPIC="../../../AGENTS"`, the output path resolves to `$PWD/AGENTS.md`; `dstack` accepts the label and cannot protect that path.
Suggested direction: Reject goal/topic components outside a strict slug grammar, including `.` and `..`, before any filesystem operation.

[severity:low][technical correctness] The E2E record overstates its retained evidence: it calls the block “unedited” and reports 33 cited sources, neither of which matches the capture and artifact.
Sites: primary: `claude/skills/codex-research/SKILL.md` verification bullet; confirmed: the task document’s E2E record.
Evidence: The template’s `<goal>-research` label became `autonomous-goal-loop-lifetime`, while the artifact contains 12 unique URLs plus one local source entry.
Verification: The retained command record confirms the child flags and exit 0, but not the outer `set -u`, cleanup trap, or `run_in_background` call; the source list has 13 entries.

[severity:low][technical correctness] The claim that `-o` is the invocation’s “one deliberate repository write” remains false because `dstack run` also creates its capture beneath the repository.
Evidence: The launcher writes `.dstack/runs/<sid>/<label>/{out.txt,err.txt,cmd,exit,.launch/...}` in addition to the artifact.
Verification: The retained E2E capture contains all of those files under the repository’s `.dstack` directory.

[severity:low][security] The prior evaluator-directive fix is incomplete: Deployment context still tells the reviewer how to interpret a scope-related statement.
Evidence: It describes unedited areas and says this is “not as a scope instruction to a reviewer.”
Verification: That direction was ignored and the complete instruction document was reviewed from the trusted prompt’s scope.

Omitted-detail: 0 low

GPT verdict: reject — Unlisted catchable signals still bypass teardown, and unconstrained path substitutions provide a concrete repository-file clobbering path.

## Carried decisions
- The teardown guarantee names the ACTUAL trap set — normal exit plus
  `INT TERM HUP QUIT PIPE ALRM USR1 USR2`. "Any catchable termination" was still too wide after
  round 001 narrowed it once: `ABRT`, `XCPU` and `XFSZ` are catchable and are not trapped.
  Widening `RUN_SIGNALS` is a change to `claude/bin/dstack`, recorded as a follow-up for its own
  review unit rather than bolted onto this allowlist.
- Placeholders in a recipe are UNVALIDATED INPUT. Quoting stops word-splitting and does nothing
  about `..`; `TOPIC=../../AGENTS` sends `-o` onto a tracked file. Validate against a plain-slug
  grammar before the first filesystem operation, not after `dstack` validates the label.
- A capture proves the CHILD invocation. It does not record the wrapper's `set -u`, its trap, or
  whether the Bash call was backgrounded — those are observations and are labelled as such.
- Count sources from the `## Sources` section, not with `grep -c 'https\?://'` over the document.
  The latter counts inline citations and inflated 13 into 33.
- `-o` is not the only repository write: `dstack run` writes its capture under `.dstack/` too. Two
  deliberate writers; the read-only sandbox constrains the model, not the harness around it.
- A Deployment context states facts about the change. Any sentence explaining to the reviewer how
  to read it is itself the evaluator directive being removed.

Consensus: disagreed
