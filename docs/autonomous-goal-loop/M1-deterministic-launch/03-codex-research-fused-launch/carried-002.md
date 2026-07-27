## Carried decisions — Round 002
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
