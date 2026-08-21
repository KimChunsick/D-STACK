# 01-research-contract-hypotheses-ledger

## Intent / Why
Give the audit pass stable targets: the research contract must make every artifact
enumerate its hypotheses/claims explicitly and carry a data-check ledger with a
deferred-executable-checks list. Without enumerated claims the auditor interrogates prose;
without the ledger, "verify with data" has no contract; without the deferred list, the
orchestrator does not know what to execute.

## Deployment context
Markdown contract text in `codex/skills/adversarial-research/SKILL.md`, symlinked into
`~/.codex/skills/` and loaded only when a caller invokes the skill. Public repo; no
secrets. Consumers: every future full-cycle P3 research round. File ownership: the audit
skill file sits in T02's declaration and the orchestrator recipe file in T03's.

## Design consult
Skipped — no trigger (instruction-document contract edit; the structural decisions were
made from cited research at P3/P4 and are recorded in GOAL.md).

## What was done (what / why)
Extended `codex/skills/adversarial-research/SKILL.md` with three new contract sections
plus an amended format rule, placed between "What to gather" and the rules (the bare
`Rules:` label became `## Rules` so the list stays attached to a heading after the
insertion):
- **Hypotheses and claims (audit targets)** — every artifact enumerates decision-relevant
  hypotheses as `H1..Hn`, one falsifiable sentence each, naming supporting
  sections/sources. Non-falsifiable findings (taste, intent, tradeoffs) stay in the
  evidence sections where the audit probes assumptions instead — this is the boundary the
  research called "checkable with data vs interrogated through assumptions".
- **Data-check ledger** — checkability is defined by reproducibility from primary
  evidence (schema fields are not eligibility gates; unit/denominator take justified
  `N/A`); every checkable H-item gets a row whose `status` is `recomputed`, `quoted`, or
  `deferred`, and a deferred row names its entry in the deferred-checks list — deferral
  is a row status, never a substitute for the row.
- **Deferred executable checks** — declarative specifications (input, computation or
  comparison, confirm/refute criterion), explicitly never ready-to-run commands: a shell
  line copied or derived from a fetched page is an injection handoff, mutating operations
  are banned, and the consumer is told to author/validate/sandbox its own execution
  treating the spec as untrusted data.
- **Output blocks (research mode)** — hypotheses, data-check ledger, and deferred
  executable checks are SEMANTIC requirements with explicit `none` when empty, encoded
  per requested shape: literal `## …` headings in Markdown (filled in place, appended
  when omitted), the schema's corresponding fields in structured shapes. A format that
  cannot encode ANY one of the three yields an incomplete artifact: where a flag channel
  exists, the carried blocks are encoded and each missing one is flagged as the caller's
  defect; where the shape can carry neither the blocks nor a flag, the artifact is
  refused on the first line — so no shape can silently drop a block.
The frontmatter description now names the new outputs so skill selection stays truthful.
Why: the audit pass (T02) needs stable targets and the orchestrator (T03) needs an
executable worklist; both come from this contract. The second, third, and fourth items
above are the round-001 review fixes (F1 deferral contradiction, F3 conjunctive
eligibility, F4 injection handoff, F2 canonical blocks).

## Pre-review defect-class self-sweep (codex-review Step 0)
- Contract-conflict class: read the new sections against each existing rule. The format
  rule ("follow it exactly") conflicted with always-present blocks and was amended in the
  same edit to carry the three blocks per shape (appended headings in Markdown, schema
  fields where provided, refusal when neither the blocks nor a flag can be carried). The
  caller in `claude/skills/codex-research/SKILL.md` still pins the six legacy sections;
  that file is owned by declared task T03 (deps: T01, T02), which updates the pinned
  list.
- Public-repo/secret class: text-only change to an already-allowlisted file; guard green.
- Injection-handoff class (from round 001 F4): swept the deferred-checks path in this
  contract (declarative-only, non-mutating, untrusted-spec consumer rules added). The
  sibling `socratic-audit` skill's `## New deferred checks` section has the same class of
  text ("exact command or calculation") — that file belongs to T02's declaration and its
  round was open when this was found; recorded in carried decisions for T02's next
  round.
- Hook-parsed surfaces: none touched.

## Files changed (where / why)
- `codex/skills/adversarial-research/SKILL.md` — three new contract sections (hypotheses,
  data-check ledger, output blocks) + amended format rule + description update; loads
  only when a caller invokes the skill.

## Direct verification (repo policy: no TDD)
Recorded from actual runs (2026-08-21):
- `readlink ~/.codex/skills/adversarial-research` → resolves into this repo (symlinked
  dir; content changes propagate with no install step).
- `grep -c "Data-check ledger"` and `grep -c "Hypotheses and claims"` on
  `~/.codex/skills/adversarial-research/SKILL.md` → `1` / `1` (both new sections present
  through the live path).
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard`.

## E2E verification
Post-merge (commit eff69c1), 2026-08-21. Behavioral probe: one live research-mode
invocation (GPT-5.5, low effort) requesting ONLY the six legacy sections, on a small real
question (latest stable Git release). Capture: [e2e-research-probe.md](e2e-research-probe.md);
run label `socratic-research-verification-t01-e2e`, exit 0.
- All three semantic blocks are present and correct in the artifact: `H1.` enumerated
  with its sources; a data-check ledger row carrying an `N/A` denominator (justified — a
  version fact) and `quoted` status; `Deferred executable checks: none` stated
  explicitly. The N/A field and the status field exercise exactly the review's
  round-001/002 refinements.
- Recorded variance: the low-effort model carried the blocks INLINE inside `## Needed
  info` rather than appending them as literal `## …` headings, which the contract's
  Markdown branch specifies for a format that omits them. The semantic property the
  review named as the real target — no artifact silently missing the blocks — held; the
  literal-placement adherence at low effort did not. The production caller (T03)
  requests the three headings explicitly, so the appending branch is the legacy-caller
  fallback; this variance is recorded rather than silently passed.

## Gate status
- [x] Verification: direct-run checks recorded above (text present through live symlink
  paths, guard green); behavioral confirmation captured in the E2E probe below and in
  the M1 E2E round (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
