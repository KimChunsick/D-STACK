# Codex adversarial review — Round 003

## Review scope

Re-review. Verifies Round 002's grammar-checking validator and redirect-based capture. Bundle
67,486 bytes; Rounds 001-002 emitted in full.

## GPT findings

Round 002 verification: the malformed verdict/disclosure counterexample is now rejected, the direct redirect preserves the producer's nonzero status, and the explanation correctly references Step 2b. The complete-block portion of the first fix remains unresolved.

[severity:medium][technical correctness] Aggregate label counts still accept malformed finding blocks, so the elected-skill gate remains bypassable.
Sites: primary: `claude/skills/codex-review/SKILL.md:151-166`; confirmed: `codex/skills/adversarial-review/SKILL.md` "Output discipline", `codex-review-002.md` M1 response
Evidence: The validator only equates total severity, Evidence, and Verification counts; it neither associates labels with findings nor validates their order, and it permits multiple disclosure lines.
Verification: Replaying its predicates accepted `Evidence: detached` before a tagged finding followed by `Verification: detached`, a valid disclosure, and a valid final verdict.
Suggested direction: Validate each finding as an ordered block and require exactly one well-formed disclosure rather than comparing aggregate counts.

[severity:medium][technical correctness] The new history compaction can silently discard unresolved decisions while accepting the companion as trustworthy.
Sites: primary: `claude/skills/codex-review/SKILL.md:42-88`; confirmed: `claude/skills/codex-review/SKILL.md:235-263`, `task.md` invariant
Evidence: Older full rounds are replaced by manually authored companions, while validation checks only the round-identifying first line and final consensus line; the text itself acknowledges that wrong or truncated content misleads later reviews.
Verification: A companion with the correct header and consensus footer but an omitted live blocker passes the stated checks; once that round ages beyond the two-round floor, the reviewer cannot know the missing decision exists or request its source round.
Suggested direction: Require machine-checkable correspondence between carried decisions and sealed-round decision identifiers, otherwise retain the full round.

[severity:low][DX] The Round 001 EOF repair removed the terminating newline rather than only the extra blank line.
Evidence: The supplied `codex/AGENTS.md` diff ends with `\ No newline at end of file`.
Verification: That marker confirms the blank-line defect disappeared by leaving the file without a conventional final line terminator.

[severity:low][DX] The task record inaccurately describes both the extraction and its verification.
Sites: primary: `task.md` "What was done" item 3; confirmed: `task.md` "E2E verification" item 7, both new skill files
Evidence: It calls the move "verbatim" although the review skill adds right-sized-technology, scale-fit, blast-radius, sketch, and output-budget rules; it also reports five research headings while the caller requires six.
Verification: Direct comparison of the removed global contract with the new skill and counting the caller's listed headings confirms both discrepancies.

Omitted-detail: 0 low

GPT verdict: reject — two concrete medium failures remain in the structural safeguard and the newly introduced review-history compaction.

## Maintainer response

All four accepted, none rebutted.

**M1 — aggregate counts are not block validation. Agreed; the counterexample is exact.** A
stray `Evidence:` before the first tag and a stray `Verification:` after the last one balance
the totals while leaving every finding unsupported. Replaced the three `grep -c` comparisons
with an `awk` pass that walks the output and requires each finding to be an ordered block: its
severity tag, then its own `Evidence:`, then its own `Verification:`, before the next tag. It
also now requires *exactly one* disclosure line rather than at least one.
Verification, seven probes: this round's detached-label counterexample, two disclosure lines,
`Verification:` before `Evidence:`, two tags sharing one Evidence, and Round 002's
`nonsense`/`banana` case all exit 1; a zero-finding review and a two-finding review carrying
optional `Sites:`/`Suggested direction:` lines both exit 0.

**M2 — a companion could quietly drop a live blocker. Agreed, and it is closable without the
derivation this design abandoned.** The suggested direction — machine-checkable correspondence
between the companion and the sealed round — is right, but it does not need decision
identifiers or any structural read. `carried_ok` now requires the companion's body (everything
after its identifying first line) to be **exactly the round's own last lines**: a plain suffix
comparison. Omit a decision and the suffix stops matching, so the round is sent whole. That
also makes the authored-not-extracted rule enforceable rather than merely stated.
Verification: all ten existing companions across both Goals match their rounds' suffixes and
still compact; deleting one line from a companion flips that round to
`carried-001.md is not a complete carried-state companion` with the full round emitted;
restoring it compacts again.

**L1 — the EOF repair overshot. Agreed**, the terminating newline is back and
`git diff --check HEAD -- codex/AGENTS.md` is clean.

**L2 — the task record overclaimed. Agreed on both counts.** "Verbatim" was true only relative
to `AGENTS.md` as the *previous* Goal had just left it, which is not what the word conveys;
reworded to say byte-for-byte as they stood, and to name the rules that had recently arrived
there. The research-headings claim conflated two different things: the skill defines five
evidence categories to gather, while the caller's prompt fixes the six output headings the
artifact must carry. Both are now stated separately.

## Carried decisions

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar and block structure, not presence**: one verdict line from
  the three allowed values as the final nonblank line, exactly one
  `^Omitted-detail: [0-9]+ low$`, and each finding as an ordered tag → Evidence → Verification
  block. Zero findings is valid. It checks shape, never substance.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- A companion's body must equal its round's trailing lines exactly. This is a suffix
  comparison, never a structural read of the round — that derivation is the mistake six earlier
  rounds were spent on. It is what makes "author the companion, do not extract it" checkable.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
