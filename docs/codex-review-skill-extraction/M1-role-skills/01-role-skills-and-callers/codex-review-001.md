# Codex adversarial review — Round 001

## Review scope

Adversarial review. First round run through the extracted `$adversarial-review` skill, so the
round is simultaneously the mechanism's own end-to-end test. Bundle 54,972 bytes across 16
emitted items; no prior rounds.

## GPT findings

[severity:medium][technical correctness] Step 2b is not a fail-closed contract gate: its input is undefined, complete failure returns success, and valid zero-finding reviews are rejected.
Sites: primary: `claude/skills/codex-review/SKILL.md:97,140-151`; confirmed: `task.md:11-14,66-70,121-126`
Evidence: Step 2 neither defines nor writes `$OUT`; all three misses use `|| echo`, while the first grep requires a severity finding that the review contract does not require when none exist.
Verification: Repository search found `$OUT` only at lines 146-148; a direct shell probe returned status 0 with every marker absent and status 1 for the severity check on `Omitted-detail: 0 low` plus a valid final verdict.
Suggested direction: Capture raw output explicitly and use one fail-closed validator that accepts either complete findings or a genuine zero-finding shape while enforcing required labels and one exact final verdict.

[severity:low][DX] The edited global instructions fail the repository's whitespace check because they add a blank line at EOF.
Evidence: `codex/AGENTS.md:44` is an additional empty line after the final content line.
Verification: `git diff --check HEAD -- codex/AGENTS.md` reports `new blank line at EOF`.

Omitted-detail: 0 low

GPT verdict: reject — the structural safeguard required to prevent elected-skill failures is currently unusable and does not enforce its stated invariant.

## Maintainer response

Both accepted, neither rebutted. The medium is the sharper of the two: the entire justification
for moving a mandatory contract into an elected skill was that a structural output check would
backstop the election, and the check I wrote could not have caught anything.

**M1 — Step 2b was decorative. Agreed on all three counts.** `$OUT` was never created or
written (Step 2 piped to stdout); `|| echo` writes a warning and returns 0, so nothing was
ever blocked; and requiring a `[severity:...]` line rejected a legitimate review that found
nothing. Rewritten as a single fail-closed `contract_ok` function: Step 2 now creates `OUT`,
`tee`s the invocation into it, and cleans it up in the same trap. The validator requires
exactly one `GPT verdict:` line *and* that it be the final nonblank line, requires the
unconditional `Omitted-detail:` disclosure, and makes the severity-tag requirement conditional
— it keys off `Evidence:`, so findings must be tagged while a genuinely empty review passes.
`contract_ok "$OUT" || exit 1` is the gate.
Verification, seven probes against the extracted function: empty output, generic prose with no
markers, a verdict with no disclosure, findings present but untagged, and a verdict that is not
the final line all exit 1; a zero-finding review (`Omitted-detail: 0 low` plus verdict) and a
full tagged review both exit 0.

**L1 — blank line at EOF. Agreed**, removed; `git diff --check HEAD -- codex/AGENTS.md` is
clean.

**On the round itself.** The review loaded `$adversarial-review` and produced fully
contract-shaped output — severity tags, `Sites:` with confirmed entries, `Evidence:`,
`Verification:`, `Suggested direction:`, `Omitted-detail: 0 low`, and a single final verdict —
which is the working evidence that the extraction did not weaken the contract.

## Carried decisions

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role at all: it keeps stack
  neutrality, the language boundary, and the operational constraints, plus an order to stop if
  asked to review or research without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Any change that weakens one of the three must strengthen another.
- Step 2b must stay **fail-closed** and must not require a severity tag from a review that
  legitimately found nothing. It checks shape, never substance — do not describe it as more.
- Skills are installed to `~/.codex/skills/`, not the publicly documented
  `$HOME/.agents/skills`. That path is shared across agents and would recreate the
  contamination this Goal removes. Accepted residual: the chosen path is verified locally
  against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in the same
  change.

Consensus: disagreed
