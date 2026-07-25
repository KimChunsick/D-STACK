## Needed info

- Codex `AGENTS.md` is unconditional startup guidance: Codex reads global guidance from `CODEX_HOME` and then project guidance before work begins; closer files override earlier ones, and the project-doc budget defaults to 32 KiB. [S1]
- Codex skills are elected context: Codex initially exposes skill name, description, and path; full `SKILL.md` loads only after Codex selects or the user explicitly invokes the skill. [S2]
- Codex supports explicit skill invocation in CLI/IDE via `$skill-name` or `/skills`; implicit invocation depends on the skill `description`. OpenAI explicitly says to front-load trigger words because descriptions can be shortened. [S2]
- Codex can omit some skills from the initial skills list when many skills are installed; it shortens descriptions first and warns if omission happens. That is a concrete implicit-trigger failure mode. [S2]
- Codex public docs currently list user skills under `$HOME/.agents/skills`, plus repo/admin/system locations; they do not list `~/.codex/skills` as a documented user-skill discovery path, although `~/.codex/config.toml` can disable skills by explicit path. [S2]
- Codex supports `allow_implicit_invocation: false` in `agents/openai.yaml`; explicit `$skill` invocation still works. This is useful for preventing review skills from triggering in unrelated tasks. [S2]
- I found no public Codex `exec` flag that preloads or requires a skill. Local `codex exec --help` in this environment also showed no skill-preload flag, but did show `--profile`, `--config`, `--output-schema`, and `--json`. Local observation, no URL, retrieved 2026-07-26.
- OpenAI’s app-server README has a stronger mechanism than prompt-only naming: a `skill` input item is recommended so the backend injects full skill instructions instead of relying on the model to resolve the name. I did not verify an equivalent `codex exec` CLI surface. [S6]
- Anthropic’s parallel guidance draws the same line: always-loaded `CLAUDE.md` is for facts/rules every session should have; multi-step procedures or only-sometimes context should move to skills or path-scoped rules. [S7]
- Anthropic explicitly says `CLAUDE.md` is context, not hard enforcement; vague, conflicting, or large files reduce adherence, and deterministic enforcement should use hooks or per-invocation system-prompt mechanisms. [S7]

## Opposing views

- Strong anti-split view: if the review contract is mandatory, putting it in an elected skill adds a model/tool-selection dependency that `AGENTS.md` does not have. Skills are designed for dynamic loading, not hard enforcement. [S1][S2][S7]
- Pro-split view: the current global `AGENTS.md` is over-scoped. OpenAI’s own code-review rules post says broad instructions can create noise, and recommends narrow, scoped guidance so unrelated changes do not compete for attention. [S5]
- Implicit invocation is the weak version of the split. Both OpenAI and Anthropic say descriptions drive matching, descriptions can be shortened, and users should test trigger behavior; neither vendor presents implicit skill selection as deterministic. [S2][S8][S9]
- Explicit invocation is materially stronger than implicit invocation, but still weaker than backend injection unless the CLI provides a non-model `skill` input channel. OpenAI’s app-server docs distinguish those cases directly. [S6]
- Self-report is not a reliable detector by itself. Anthropic recommends checking actual loaded context or hooks for instruction-load debugging; adjacent instruction-hierarchy studies show models often fail under conflicts even when the desired priority is clear. [S7][S12][S16]

## For the goal

- Moving the large adversarial-review persona out of global guidance is sound scoping. Vendor guidance says global/user files are for durable all-session preferences, while skills are for repeatable task workflows. [S2][S7][S10]
- There is plausible contamination risk, not just aesthetic hygiene. OpenAI reports broad review rules can create noise; Anthropic says long or conflicting always-loaded instructions reduce adherence; persona research found persona prompts can significantly change downstream behavior. [S5][S7][S15]
- The goal is achievable if the caller explicitly names the skill and the skill is kept focused. OpenAI says selected Codex skills read full `SKILL.md`; `allow_implicit_invocation: false` can prevent accidental review-mode activation. [S2]
- A hybrid is defensible: keep only a tiny global line such as “For adversarial research/review, explicitly invoke `$codex-review`; otherwise remain role-neutral,” and put the full contract in the skill. This preserves a durable trigger without globally injecting the whole persona. [S1][S2]
- Add fail-loud external checks rather than relying only on the model’s first-line self-report: validate required structural markers such as `Evidence:`, `Verification:`, and final `GPT verdict:`; use `--output-schema` if the contract can be expressed as JSON. Local observation, no URL, retrieved 2026-07-26.

## Against the goal

- The strongest argument against the change: a mandatory review contract should not depend on elected context. A missed skill can produce a plausible but non-contract review, and public Codex docs do not document a `codex exec` “require skill” flag. [S2][S6]
- A better middle option may exist: create role-specific unconditional startup context per invocation, such as a scratch-dir `AGENTS.md` or separate `CODEX_HOME` for review runs. That preserves unconditional loading while avoiding global contamination. [S1]
- Public Codex docs currently document `$HOME/.agents/skills`, not `~/.codex/skills`; the user’s local experiment is valuable, but relying on an undocumented path is weaker than using the documented path or an explicit `skills.config` entry. [S2]
- Output validation can catch missing format, but it cannot prove the model followed the substantive review axes, scale-fit rules, or blast-radius discipline. It reduces silent failure; it does not make elected context equivalent to unconditional injection. [S7][S12][S14]
- Evidence for contamination is adjacent rather than direct. I found no current primary study measuring “a reviewer persona in global Codex `AGENTS.md` degrades unrelated report-writing tasks by X%.”

## Unverified

- No current primary Codex CLI documentation found for a `codex exec` flag to preload, force, or require a named skill.
- No current primary measured failure rate found for Codex skill election by explicit `$skill` name versus implicit description matching.
- No direct study found on models falsely claiming “I loaded this skill/file” in Codex or Claude Code. The best evidence is adjacent: instruction-following, instruction-hierarchy, and hallucination/faithfulness work.
- I did not verify whether `codex exec --json` emits a machine-checkable skill-loaded event.
- I did not verify whether `~/.codex/skills/<name>/SKILL.md` is officially supported despite the user’s local successful experiment; public docs currently name `$HOME/.agents/skills`.

## Sources

- [S1] OpenAI, “Custom instructions with AGENTS.md,” primary docs, no date, retrieved 2026-07-26: https://learn.chatgpt.com/docs/agent-configuration/agents-md
- [S2] OpenAI, “Build skills,” primary docs, no date, retrieved 2026-07-26: https://learn.chatgpt.com/docs/build-skills
- [S3] OpenAI, “Non-interactive mode,” primary docs, no date, retrieved 2026-07-26: https://learn.chatgpt.com/docs/non-interactive-mode
- [S4] OpenAI, “Configuration Reference,” primary docs, no date, retrieved 2026-07-26: https://learn.chatgpt.com/docs/config-file/config-reference
- [S5] OpenAI, “Custom Code Review rules for Codex,” primary blog, 2026-07-20, retrieved 2026-07-26: https://learn.chatgpt.com/blog/custom-code-review-rules-for-codex
- [S6] OpenAI Codex GitHub, `codex-rs/app-server/README.md`, primary repo docs, no date, retrieved 2026-07-26: https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md
- [S7] Anthropic, “How Claude remembers your project,” primary docs, no date, retrieved 2026-07-26: https://code.claude.com/docs/en/memory
- [S8] Anthropic, “Extend Claude with skills,” primary docs, no date, retrieved 2026-07-26: https://code.claude.com/docs/en/skills
- [S9] Anthropic Help, “Use skills in Claude,” primary help, 2026-05-27 / updated this week, retrieved 2026-07-26: https://support.claude.com/en/articles/12512180-use-skills-in-claude
- [S10] Anthropic, “Skills explained,” primary blog, 2026-03-05, retrieved 2026-07-26: https://claude.com/blog/skills-explained
- [S11] Anthropic Help, “How to create custom skills,” primary help, 2026-06-12, retrieved 2026-07-26: https://support.claude.com/en/articles/12512198-how-to-create-custom-skills
- [S12] Zhang et al., “IHEval,” primary research, 2025-04, retrieved 2026-07-26: https://aclanthology.org/2025.naacl-long.425/
- [S13] Patil et al., “Berkeley Function Calling Leaderboard,” primary research, 2025-07, retrieved 2026-07-26: https://proceedings.mlr.press/v267/patil25a.html
- [S14] Heo et al., “Do LLMs ‘know’ internally when they follow instructions?”, primary research, 2025, retrieved 2026-07-26: https://proceedings.iclr.cc/paper_files/paper/2025/hash/ca6980a3dba7fb3e4e66925656dba68b-Abstract-Conference.html
- [S15] Princeton CS, persona prompting toxicity report, secondary university news, 2024-01-30, retrieved 2026-07-26: https://www.cs.princeton.edu/news/personalizing-chatgpt-can-make-it-more-offensive-researchers-find
- [S16] Geng et al., “Control Illusion,” primary preprint, 2025-12-04, retrieved 2026-07-26: https://arxiv.org/abs/2502.15851