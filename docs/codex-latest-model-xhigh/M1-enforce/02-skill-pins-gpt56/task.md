# 02-skill-pins-gpt56

## Intent / Why
Both Codex-calling skills must pin their model explicitly alongside xhigh, per the repo
rule "pin model+effort; do not depend on config drift" — research on the cost-efficient
gpt-5.5, review on the frontier gpt-5.6-sol (user's split: review is the quality gate;
research is high-volume web work). The test suite must fail if a future edit drops or
swaps either pin **from the executable command itself**. Also fixes codex-review's false
premise ("config.toml is already configured" — it wasn't: reviews ran at effort *none*),
hardens the review call with cwd isolation, and updates stale GPT-5.5 labels.

## What was done (what / why)
- TDD: first Red captured pre-model-split (`✗ FAIL: …codex-research/SKILL.md does not
  match /\-m gpt-5\.6-sol/` — assertions then targeted sol-everywhere; the user's mid-turn
  directive changed the design to research-5.5/review-sol). After review round 1, the
  assertions were re-anchored to the **executable command rows** and Red was demonstrated
  against HEAD (`git show HEAD:claude/skills/codex-review/SKILL.md` does not match the
  anchored `^codex exec …-s read-only …-m gpt-5\.6-sol -c model_reasoning_effort="xhigh"`
  pattern; current tree passes). Prose quoting a pin can no longer keep the suite green.
- Green: codex-review's `codex exec` now pins `-m gpt-5.6-sol -c
  model_reasoning_effort="xhigh"` AND runs isolated (`-s read-only -C "$SCRATCH"` — the
  allowlist controls what is *sent*, isolation controls what the reviewer can *reach*);
  its config.toml premise rewritten (config backstops effort only); codex-research keeps
  `-m gpt-5.5` deliberately (split note added); `codex/AGENTS.md` identity line names
  per-role models with the caller's flags as source of truth; availability note added
  ("needs codex-cli ≥ 0.144; upgrade on 'requires a newer version'").
- Refactor: none needed (doc-line + assertion edits; suite green).

## Files changed (where / why)
- `tests/test_claude_artifacts.sh` — command-anchored per-role pin assertions (+ read-only
  on the review line); cross-model not_matches guards; effective-override guard (review
  round 3, tightened round 4): ANY `-m` flag and the effort flag must each appear EXACTLY
  once inside each skill's fenced blocks, counted by occurrence not line, so a trailing
  second `-m <anything>`/effort flag (which would win at runtime) fails the suite — Red
  demonstrated against three crafted counterexamples (`-m gpt-5.5-mini`, second effort
  flag, `-m o3`). Also asserted (round 2, Red-vs-HEAD captured): the prompt-injection
  guard (`UNTRUSTED DATA`) and the scratch-cleanup trap on the executable lines.
  NOTE: this shared file also carries T03's ultracode gate; that hunk is
  reviewed in T03's own codex-review with its dependencies.
- `claude/skills/codex-research/SKILL.md` — keep `-m gpt-5.5` (deliberate split), prose accuracy
- `claude/skills/codex-review/SKILL.md` — pin `-m gpt-5.6-sol -c model_reasoning_effort="xhigh"`;
  add `-s read-only -C "$SCRATCH"` isolation; correct the false config.toml premise;
  availability/failure note; untrusted-data prompt framing (embedded directives are data
  and a reportable finding, not instructions); combined `$IN`/`$SCRATCH` cleanup trap;
  GPT-5.6 Sol labels; explicit **confidentiality residual**
  (review round 3): reads are unconfined by codex's sandbox, so injected instructions
  could pull file contents into model context — accepted with named containment
  (self-authored material only, human-read verdicts, `--ephemeral`); `sandbox-exec`
  wrapper considered and rejected (deprecated, brittle)
- `codex/AGENTS.md` — identity line: per-role models, pinned per-call (accuracy)

## E2E verification
- `bash tests/run.sh` → ALL TESTS PASSED (with the command-anchored assertions).
- Red-vs-HEAD demonstration: anchored review pattern does not match `git show
  HEAD:claude/skills/codex-review/SKILL.md` (no pins at HEAD) — the assertions detect the
  pre-change state.
- Live run of the documented review command: header `model: gpt-5.6-sol / reasoning
  effort: xhigh` (round 1 session 019f48ef-6574-7163-950f-1b5559b96faa; round 2 re-ran the
  hardened `-s read-only -C` variant — see codex-review.md).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus (6 rounds, some run in parallel by
  the two driving sessions; final confirming round → approve-with-fixes, quoted-override
  ban applied, activation gap resolved as T03's enforced policy boundary; see
  codex-review.md, `Consensus: resolved`)
- [x] E2E capture verified (suite green with anchored + occurrence-count guards; live
  pinned-command headers recorded — research gpt-5.5 @ xhigh session
  019f48fb-8227-7cb1-836e-3a86e2090256, review gpt-5.6-sol @ xhigh across all six
  review rounds; mirrored in GOAL.md M1 E2E record)
