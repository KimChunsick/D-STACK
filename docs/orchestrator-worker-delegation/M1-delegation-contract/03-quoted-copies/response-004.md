# Maintainer response — Round 004

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium] `keep-in-the-orchestrator` also decides delegation and was not in `gates`.** Accepted,
fixed. A PARALLEL condition in the retention list recouples delegation to parallelism just as
effectively as one in `delegate-when` — it says "do not delegate when the checker says PARALLEL",
which is the same coupling with the sign flipped. `gates` is now all three lists, and the failure
message names which one carried it. This is the second round finding the same class, and the reason
is that round 003 fixed the instance it was handed instead of enumerating the lists that decide the
question. Recorded in the round file rather than quietly corrected.

**[medium] `[nil]` entries pass, and a duplicate `worker-fanout` node silently wins.** Accepted,
both fixed. `- # commented out` parses as `[nil]`, which is non-empty and contains nothing, so
entries must now be non-blank Strings. And the loop took the last matching node, so a malformed
node could hide behind a later valid duplicate; exactly one `scheduling.modes.worker-fanout` is now
required and the count is reported when it is not.

**[medium][security] `YAML.load_file` deserializes before validation.** Accepted, fixed — the new
parser reads with `File.read` and loads with `YAML.safe_load`, so tagged object construction cannot
happen ahead of the structural checks. Two honest qualifications. The input is this repository's own
`SKILL.md`, so this is ordering discipline rather than a live threat. And the pre-existing
parse-validity loop below still calls `YAML.load_file`: its assertion is "this fenced block parses
at all", and narrowing it to safe types would change what it pins for a reason unrelated to this
change. Left alone deliberately and carried as a follow-up rather than swept in.

**[low][security] The task record carries uncited process directives.** Accepted, fixed. "no TDD, no
new tests" now cites `AGENTS.md` and its section by name instead of asserting a rule from nowhere,
and the E2E placeholder no longer describes review timing. The reviewer is right that an artifact
under review should carry facts and citations, not instructions about how it is to be treated.
