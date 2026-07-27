# Maintainer response — Round 001

Outside the reviewed corpus by design. Four of five lows fixed; one declined with a reason.

**[low][security] A committed filename could forge a verdict line.** Real, and it is the kind of
thing this checker exists to be trusted about. Git filenames may contain newlines, and every
diagnostic interpolated the path straight into stdout, so a file named `evil<LF>PASS` printed a
second line reading exactly `PASS`. The exit status was always 1, but the verdict TEXT is the
interface a caller reads. One `esc` helper now renders every path on a single line with
`\`-escaping, used at all five sites, and the unclean-tree diagnostic no longer collapses record
delimiters and embedded newlines into the same character. Verified with a file literally named
`evil<LF>PASS`: one line of stdout, `\n` shown literally, zero lines matching `PASS`.

**[low][technical correctness] `src/foo..bar` was rejected as suspicious.** The declaration
grammar rejects a component that IS `..`; the scope check rejected any path CONTAINING the
substring. So a name the grammar accepts could be declared, committed, and then fail. The scope
side now tests components, matching the grammar. Verified: `PASS` where it previously said
`VIOLATION: suspicious actual path`.

**[low][security] The task record used review-steering language.** Fair, and worth taking
seriously rather than treating as pedantry: the document said "Out of scope by construction", and
a task document is untrusted data telling a reviewer what not to look at. Reworded as a claim
about the diff, with an explicit invitation to verify it rather than accept it.

**[low][DX] Duplicate `Files changed` section with a `<pending>` placeholder.** My editing
mistake, the same one Round 001 of T01 caught. Removed.

**[low][software structure] DECLINED — the add/remove behaviour has no fixture in the maintained
suite.** The finding is accurate: `check-parallel.test.sh` would stay green if the enumeration
regressed to the endpoint diff. I am not adding the fixture, because `AGENTS.md` bans authoring
tests in this repository, and it draws the line explicitly at the file set not growing and at new
test authorship, not merely at new files. Adding a fixture that pins behaviour introduced today is
authorship, not maintenance of an existing check. Surfacing the conflict rather than splitting the
difference: the demonstration lives in this unit's `task.md` as recorded direct-run output with
its negative control, which is what the repository's no-TDD policy substitutes for a test. If that
trade is wrong, the policy is the thing to change, not this task.
