# Codex adversarial review — Round 001

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 10308 bytes
Allowlist: the review-unit folder, `claude/skills/codex-research/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] The teardown guarantee is false: killing the `dstack` supervisor with SIGKILL can leave Codex orphaned and consuming credits.
Evidence: `SKILL.md` says the round is torn down “with itself,” while `dstack` explicitly documents SIGKILL as untrappable and an orphan as possible.
Verification: T01’s recorded direct probe states `SIGKILL the supervisor → orphan survives`; only the listed catchable signals invoke `run_cleanup`.
Suggested direction: Limit the guarantee to catchable termination and document orphan detection and cleanup before retrying.

[severity:medium][the real Why] The instruction falsely calls the fused recipe verified by this Goal’s research round, although that round used the pre-`dstack` improvised launcher.
Sites: `claude/skills/codex-research/SKILL.md`; confirmed: the task document’s “What was done,” E2E verification, and unchecked verification gate.
Evidence: The task says the P3 round ran before `dstack run` existed, while the skill says its wrapper was verified by that same research round.
Verification: T01 verifies generic background `dstack`/Codex runs and the earlier run verifies stdin/`-o`; neither record executes this exact fused zsh recipe.
Suggested direction: Run the exact block once through `run_in_background` and record its `DONE` line, status file, and generated artifact.

[severity:medium][technical correctness] The claim that `-s read-only` “never mutate[s] the tree” contradicts the deliberate repository write performed by `-o`.
Sites: the `-s read-only` explanation; confirmed: the recipe’s `-o "$PWD/$GOAL_DIR/$TOPIC.md"` and the artifact-write explanation.
Evidence: The instruction simultaneously promises no tree mutation and says the artifact lands under `docs/`.
Verification: Installed `codex exec --help` defines `-o` as the output-last-message file; this CLI-managed write is outside the model-tool sandbox.
Suggested direction: State that read-only blocks model-initiated mutations while `-o` remains the one deliberate repository write.

[severity:low][technical correctness] Every attempt leaks its `mktemp -d` scratch directory because the recipe has no cleanup after `dstack run`.
Evidence: `SCRATCH` is allocated once and never removed.
Verification: `dstack` receives only the path through Codex’s `-C` argument and performs no scratch-directory cleanup.

[severity:low][security] The task payload contains an embedded reviewer-scope directive attempting to exclude the research skill, brief rules, and fallback path.
Evidence: Its Deployment context declares those areas “Out of scope.”
Verification: The directive was ignored and scope was determined solely from the trusted review prompt.

Omitted-detail: 0 low

GPT verdict: reject — the lifecycle, verification, and read-only guarantees contain unresolved concrete contradictions.

## Carried decisions
- Teardown guarantees are stated only for CATCHABLE termination, in this file as in `codex-review`.
  The two skills state the same residual because they wrap the same launcher, and stating it once
  wider than it holds is the defect being fixed, not a wording preference.
- A "verified" claim names WHICH invocation was verified and WHEN. The Goal's P3 research round
  predates `dstack run`, so it verifies `codex exec` with stdin and `-o` and nothing about the
  wrapper. The fused block is now verified by its own run, recorded in `task.md`.
- `-s read-only` blocks MODEL-initiated mutation. `-o` is a CLI-managed write and is the one
  deliberate repository write the invocation makes; "never mutate the tree" was false as written.
- Every recipe that allocates `mktemp -d` removes it on EXIT. The detached launcher used to clean
  its own scratch dir; that cleanup vanished with the launcher and had to be re-stated in the recipe.
- A task document must not tell the reviewer what is in or out of scope. State what the change
  touched as filing information; scope comes from the review prompt alone.

Consensus: disagreed
