# Codex adversarial review — Round 004

## Review scope

Re-review. Verifies Round 003's ordered-block validator and the companion suffix binding.
Bundle 84,397 bytes; Round 001 compacted, 002-003 full.

## GPT findings

Round 003 verification: the state machine closes the detached, duplicate, and out-of-order label counterexamples; the terminal newline and task-record corrections are also present. The suffix binding and complete-grammar claims remain unresolved.

[severity:medium][technical correctness] The companion check accepts a proper suffix containing no carried decisions, allowing live decisions to disappear once a round leaves the two-round full-history window.
Sites: primary: `claude/skills/codex-review/assemble-review.sh` `carried_ok`; confirmed: `claude/skills/codex-review/SKILL.md` "Sealing also writes the companion", `task.md` "Files changed"
Evidence: `body` comes from the companion's own length, so a header followed only by `Consensus: disagreed` compares equal to the final one-line suffix of any round with that consensus.
Verification: Replaying the equality with a round containing two live decisions returned status 0 for the decision-free companion; its header, consensus grammar, and round-consensus equality also satisfy the preceding gates.
Suggested direction: Bind acceptance to an independently recorded carried-section boundary or retain the full round when no trustworthy boundary exists.

[severity:medium][technical correctness] Step 2b still accepts malformed finding-header grammar and findings placed after the supposedly closing disclosure, contradicting its fail-closed contract claim.
Sites: primary: `claude/skills/codex-review/SKILL.md` `contract_ok`; confirmed: `task.md` "What was done" item 6
Evidence: The tag regex validates only the prefix, while the disclosure branch resets state without preventing later finding tags.
Verification: `Omitted-detail: 0 low`, then a truncated `[severity:high][` header, Evidence and Verification lines, and a valid final verdict passed every predicate, including the AWK block check with status 0.
Suggested direction: Validate the complete finding-header form and make the sole disclosure an irreversible end to finding blocks.

Omitted-detail: 0 low

GPT verdict: reject — two concrete medium failure paths still let truncated review history and malformed contract output pass their respective safeguards.

## Maintainer response

Both accepted, neither rebutted. Both are the same mistake in different files: I validated a
*relationship* whose anchor the untrusted side got to choose.

**M1 — the companion chose its own comparison length. Agreed, and the counterexample is
exact.** Suffix equality is meaningless when the suffix's length comes from the file being
checked: a companion of header plus `Consensus: disagreed` matches the round's final line and
passes. The boundary now comes from the round instead. `carried_ok` computes
`boundary = round_lines - body` and requires the round's line at that computed position to be
its `## Carried decisions` heading — so the companion's body must start exactly where the
round's carried section starts, and a short companion lands the boundary on a decision bullet
or a blank line and is refused. The line is read at a computed offset, never searched for, so
a heading quoted elsewhere in the round cannot move it; searching is the derivation six
earlier rounds were spent failing at, and this does not reintroduce it.
Verification: all eight companions of the previous Goal still compact; the round's exact attack
(header plus consensus only) is refused with the full round emitted; a companion missing one
decision line is still refused; restoring either file compacts again.

**M2 — prefix-shaped header and a non-terminal disclosure. Agreed on both.** The tag pattern
now requires the complete form — `[severity:<level>][<axis>]` with a non-empty axis and
non-empty content after it — and any line starting `[severity:` that fails that form is an
explicit error rather than invisible text. The disclosure now sets a `closed` flag, so a
finding appearing after it is rejected instead of silently reopening the block sequence.
Verification, six probes: the round's after-disclosure counterexample, a truncated
`[severity:high][` header, an empty axis, and an axis with no content all exit 1; a
zero-finding review and a full Codex-shaped finding carrying `Sites:` and
`Suggested direction:` both exit 0.

## Carried decisions

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar and block structure**: one verdict line from the three
  allowed values as the final nonblank line; exactly one `^Omitted-detail: [0-9]+ low$`, which
  terminates the finding sequence; and each finding as a complete
  `[severity:<level>][<axis>] <content>` header followed by its own `Evidence:` then its own
  `Verification:`. Zero findings is valid. It checks shape, never substance.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- **Never validate a relationship whose anchor the checked side chooses.** The companion check
  failed twice on exactly that: first accepting any suffix, then accepting a
  companion-chosen length. The boundary is now computed from the round and confirmed against
  the round's own heading line at that offset.
- A companion's body must equal its round's trailing lines exactly, starting at the round's
  carried-decisions heading. Read at a computed offset, never searched for.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
