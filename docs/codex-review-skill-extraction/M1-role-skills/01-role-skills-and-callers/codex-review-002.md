# Codex adversarial review — Round 002

## Review scope

Re-review. Verifies Round 001's fail-closed rewrite of Step 2b and the EOF fix. Bundle 61,018
bytes; Round 001 emitted in full.

## GPT findings

Round 001 verification: `$OUT` is now captured and gated, valid zero-finding output is permitted, and `git diff --check` confirms the EOF-whitespace defect is closed. The prior medium finding is only partially resolved.

[severity:medium][technical correctness] `contract_ok` still accepts structurally invalid reviews, so the elected-skill safeguard remains bypassable.
Sites: primary: `claude/skills/codex-review/SKILL.md:141-166`; confirmed: `task.md:11-14,66-70,121-126`, `codex-review-001.md:31-43,59-63`
Evidence: It never requires `Verification:`, only requires any tag when `Evidence:` exists, and validates `Omitted-detail:` and `GPT verdict:` by prefix rather than their contract grammar.
Verification: `[severity:high][security] x` followed by `Omitted-detail: nonsense` and `GPT verdict: banana` passes every check; the final false `if` condition returns status 0.
Suggested direction: Validate exact disclosure/verdict forms and complete tagged finding blocks while retaining an explicit zero-finding branch.

[severity:medium][technical correctness] The new capture pipeline masks a failing `codex exec` because it uses `tee` without `pipefail` or an explicit producer-status check.
Evidence: `claude/skills/codex-review/SKILL.md:98` evaluates the pipeline by `tee`'s status before Step 2b examines the captured text.
Verification: Under Bash, `false | tee /dev/null` produced pipeline status 0 while `PIPESTATUS` reported producer 1 and `tee` 0; a failing Codex process that emitted contract-shaped text would therefore be recordable.
Suggested direction: Preserve and require the `codex exec` status before running `contract_ok`.

[severity:low][DX] The safeguard explanation incorrectly says Step 3 checks output, although the check is Step 2b.
Evidence: `claude/skills/codex-review/SKILL.md:112-113` points to Step 3, which begins at line 168 and handles round recording.
Verification: The only `contract_ok "$OUT"` invocation is at line 162 under Step 2b.

Omitted-detail: 0 low

GPT verdict: reject — the core fail-closed safeguard still accepts malformed reviews, and its new capture pipeline masks producer failure.

## Maintainer response

All three accepted, none rebutted. Both mediums are the same underlying mistake in different
places: I checked that a marker was *present* rather than that it was *well formed*, and I
checked the wrong process's exit status.

**M1 — prefix matching is not grammar checking. Agreed.** `contract_ok` now validates form,
not presence. The verdict must be one of the three allowed values, not merely start with
`GPT verdict:`. The disclosure must match `^Omitted-detail: [0-9]+ low$`, so `nonsense` no
longer passes. And instead of the conditional "tag required only if Evidence exists", the
validator counts severity tags, `Evidence:` lines, and `Verification:` lines and requires all
three equal — which catches an untagged finding, a tagged finding missing either label, and a
stray label, while zero of each remains a valid empty review.
Verification, seven probes: the round's exact counterexample, a bad verdict value alone, a bad
disclosure form alone, a tagged finding with no `Verification:`, and an `Evidence:` line with
no tag all exit 1; a zero-finding review and a two-finding review both exit 0.

**M2 — the pipeline hid the producer's failure. Agreed.** Replaced the `| tee "$OUT"` pipeline
with a plain redirect, `rc=$?` captured immediately, `cat "$OUT"` to show the result, and an
explicit non-zero guard that refuses to continue. Dropping the pipeline is better than adding
`pipefail` here: there is no pipeline left whose status could be misread.
Verification: `false > "$OUT"; rc=$?` yields `rc=1`, where `false | tee /dev/null` yields 0.

**L1 — wrong step named. Agreed**, the explanation now points at Step 2b.

## Carried decisions

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar, not presence**: one verdict line from the three allowed
  values as the final nonblank line, `^Omitted-detail: [0-9]+ low$`, and equal counts of
  severity tags, `Evidence:`, and `Verification:`. Zero of each is a valid empty review. It
  checks shape, never substance — do not describe it as more.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
