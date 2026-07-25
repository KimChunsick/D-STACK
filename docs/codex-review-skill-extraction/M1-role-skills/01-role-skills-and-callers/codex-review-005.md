# Codex adversarial review — Round 005

## Review scope
Re-review

## GPT findings

Round 004 verification: post-disclosure findings and truncated headers are rejected, and a decision-free companion falls back to full history. The claimed independent boundary and complete grammar remain unresolved below.

[severity:medium][technical correctness] A duplicate carried-decisions heading still lets a truncated companion discard live decisions.
Sites: primary: `claude/skills/codex-review/assemble-review.sh` `carried_ok`; confirmed: `claude/skills/codex-review/SKILL.md` companion contract, `task.md` item 6
Evidence: `body` still comes from the companion's length; `boundary = round_lines - body` therefore selects whichever matching heading makes that companion an exact suffix.
Verification: A round containing heading → decision A → duplicate heading → decision B accepted a companion containing only decision B and consensus; the decision-free companion was correctly rejected.
Suggested direction: Accept compaction only when the selected heading is independently unique or recorded; otherwise emit the full round.

[severity:medium][technical correctness] Step 2b still accepts malformed or hidden finding blocks, leaving the elected-skill safeguard bypassable.
Sites: primary: `claude/skills/codex-review/SKILL.md` `contract_ok`; confirmed: `task.md` item 6, Round 004 carried grammar claim
Evidence: The header regex permits no separator and whitespace-only axes, while indented severity/Evidence/Verification lines evade every state-machine branch and are treated as a valid zero-finding review.
Verification: Exact predicate replay rejected the Round 004 post-disclosure and truncated-header cases but accepted indented blocks, `[severity:high][security]content`, `[severity:high][   ] content`, and a verdict without rationale.
Suggested direction: Enforce the complete line grammar, reject label lookalikes after optional indentation, and require the mandated verdict rationale.

[severity:low][DX] `REVIEW_FULL_ROUND_IDS` tokens undergo pathname expansion before numeric validation.
Evidence: `for r in $(...)` performs word splitting and glob expansion, contradicting the documented fail-fast behavior for malformed values.
Verification: In `/dev/fd`, `REVIEW_FULL_ROUND_IDS='[1]'` expanded to `1` and passed the digit validator.

Omitted-detail: 0 low

GPT verdict: reject — two reproducible medium failures still allow carried decisions to disappear and malformed review output to pass the safeguards.

## Maintainer response

**M1 — duplicate heading (agreed, fixed).** The finding is exactly right, and it names the
root cause better than the four previous attempts did: anchoring the boundary in the round
does nothing while the *length* that positions it still comes from the companion. With two
matching headings the companion picks which one it lands on.

The fix stops selecting. `carried_ok` now requires the round to contain **exactly one**
`## Carried decisions` heading; any other count falls back to the full round:

```bash
[ "$(grep -ciE '^##[[:space:]]+carried decisions[[:space:]]*$' "$f")" -eq 1 ] || return 1
```

This is the same search six earlier rounds were defeated at, used in the one direction that
is sound. Searching a self-quoting document to *choose* a boundary is unsound — a fenced
block, an HTML comment or a nested fence can always forge a candidate. Searching it to
*refuse* is safe, because every forgery makes the count exceed one and the round is sent
whole. The failure mode is a larger bundle, never a lost decision. With the count pinned at
one, the companion's length can only land on that heading or fail the offset check.

Verification (`carried_ok` extracted with its real `DENY`/`MAX`, four cases):
duplicate heading + companion holding only decision B → **full round**; duplicate heading +
a faithful companion → **full round**; unique heading + faithful companion → **compacted**;
unique heading + companion dropping the live decision → **full round**. The first run of this
matrix passed for the wrong reason — the count used `grep -cE`, which never matched the
capitalised heading — so the case-insensitive flag was added and all four re-verified.

**M2 — hidden and malformed finding blocks (agreed, fixed).** All four bypasses reproduce.
The indented one is the serious one: an indented label matches none of the state-machine
branches, so an entire review can hide inside leading whitespace and read as a valid
zero-finding review. Three changes to `contract_ok`:

- An indent guard runs before every other branch — any leading whitespace in front of
  `[severity:`, `Evidence:`, `Verification:`, `Omitted-detail:` or `GPT verdict:` is fatal.
- The header now requires the full line grammar: a real axis (at least one character that is
  neither whitespace nor a bracket) and at least one space before the content, so
  `[severity:high][security]content`, `[severity:high][   ] content` and `[severity:high][] c`
  are all rejected.
- The verdict must carry its mandated rationale: `(approve|approve-with-fixes|reject)[ \t]+[^ \t]`.

Verification: nine cases replayed against the live predicate — valid review **accept**,
zero-finding review **accept**, and reject for no-separator, whitespace-only axis, empty axis,
indented block, verdict without rationale, missing `Verification:`, and a finding placed after
the disclosure line.

**L1 — pathname expansion (agreed, fixed).** `set -f` now brackets the id-splitting loop, so
word splitting still happens and globbing does not. Verified in `/dev/fd`:
`REVIEW_FULL_ROUND_IDS='[1]'` now yields the token `[1]`, which fails the digit validator as
documented, instead of expanding to `1` and passing.

Repo gates after the fixes: `bash -n` clean on the assembler and on the extracted validator;
`bash tests/secret-guard.sh` → `✓ PASS: secret guard`.

## Carried decisions
- Compaction is now gated on a *unique* `## Carried decisions` heading in the round. This is
  deliberately fail-open on size and fail-closed on content: an ambiguous round is sent whole.
  Do not "improve" this later into picking the last or the outermost heading — that reopens the
  self-quoting defect class this Goal spent six rounds failing at.
- `contract_ok` checks shape, not substance. It cannot prove the reviewer applied the scale-fit
  guards or the blast-radius discipline; it proves the output was produced against the contract.
  Recorded as accepted, not oversold.
- `~/.codex/skills` remains an undocumented user-skill path (documented one is
  `$HOME/.agents/skills`), chosen because the documented path is cross-vendor shared and would
  recreate the contamination one level up. Verified locally against codex-cli 0.145.0.

Consensus: disagreed
