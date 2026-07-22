## Needed info

- **Claude Code subagent definitions should be structured, scoped, and routeable.** Anthropic’s subagent docs define subagents as Markdown files with YAML frontmatter plus a Markdown body that becomes the subagent system prompt. Required fields are `name` and `description`; optional fields include `tools`, `disallowedTools`, `model`, `maxTurns`, `skills`, `background`, `effort`, and `isolation`. The `description` is explicitly used for delegation decisions, so a `general-dev` description should say when to use it, e.g. PR-sized non-frontend implementation tasks, and when not to use it. Source: [S1], primary, no date, retrieved 2026-07-22.

- **Tool restrictions belong in metadata, not prose, when enforcement matters.** Claude Code subagents inherit tools by default; `tools` acts as an allowlist and `disallowedTools` as a denylist. Anthropic also warns that “never do this” prompt rules are weaker than deterministic hooks/permissions under pressure, long sessions, ambiguity, or prompt injection. Source: [S1], [S3], primary, no date / 2026-06-18, retrieved 2026-07-22.

- **A custom subagent body replaces the default Claude Code system prompt for that subagent path.** Anthropic says subagents receive the agent’s own system prompt plus environment details, not the full Claude Code prompt, while `CLAUDE.md`/memory still load for most subagents. Therefore a `general-dev` standing prompt must include any coding-agent essentials that are not guaranteed elsewhere: repo convention obedience, scope discipline, edit/test workflow, safety, and final reporting. Source: [S1], primary, no date, retrieved 2026-07-22.

- **Claude’s current guidance favors choosing the right instruction surface.** Anthropic’s 2026 steering guide separates `CLAUDE.md` for always-on project facts, rules for constraints, skills for procedures, subagents for delegated work, hooks for deterministic automation, and output styles/system-prompt appends for global behavior. It warns that unscoped rules waste tokens, 30-line procedures belong in skills, and system-prompt appends have diminishing adherence as instructions grow or conflict. Source: [S3], primary, 2026-06-18, retrieved 2026-07-22.

- **Codex/AGENTS.md guidance maps closely to this design.** OpenAI’s Codex AGENTS.md docs say Codex reads `AGENTS.md` before work, layers global and project guidance, concatenates closer files later so they override earlier guidance, and caps loaded project docs at `project_doc_max_bytes` default 32 KiB. Example content includes working agreements, test commands, dependency-confirmation rules, repo expectations, and service-specific overrides. Source: [S8], primary, no date, retrieved 2026-07-22.

- **OpenAI’s own Codex base prompt emphasizes the same recurring worker-agent invariants.** The open-source Codex default prompt includes: obey scoped `AGENTS.md`; keep plans meaningful for non-trivial work; solve the task end to end; avoid unneeded complexity; keep changes minimal and consistent with the codebase; do not fix unrelated bugs; update docs as necessary; validate with tests/builds when available; and report concisely with file references and test status. Source: [S11], primary, no date, retrieved 2026-07-22.

- **Cursor rules guidance supports focused, scoped, reusable instructions.** Cursor’s official rules docs say rules are persistent system-level instructions for Agent/Cmd-K, stored in `.cursor/rules`, with rule types such as Always, Auto Attached by glob, Agent Requested by description, and Manual. Best practices: focused, actionable, scoped, concise, split large concepts into composable rules, include concrete examples or referenced files when helpful, and avoid vague guidance. Source: [S13], primary, no date, retrieved 2026-07-22.

- **Examples vs principles: use examples for format and subtle conventions, not as bulk filler.** OpenAI’s prompt guide recommends putting instructions first, separating context with delimiters, being specific about output/format/style, and articulating desired output through examples. Anthropic’s Claude prompt guide says examples are among the most reliable ways to steer format/tone/structure, recommends relevant/diverse/structured examples, and suggests XML tags for complex prompts mixing instructions, context, examples, and variable inputs. Source: [S14], [S15], primary, updated ~2026-07-16 / no date, retrieved 2026-07-22.

- **For the target `general-dev` prompt, likely standing sections should be:** role/mission; precedence and source-of-truth rules; implementation philosophy; Red-Green-Refactor workflow; scope/file ownership constraints; repo-convention discovery; validation expectations; parallel-worktree behavior; final report contract. Prior art repeatedly includes role, workflow, constraints/prohibitions, tool/environment guidance, and final output expectations. Sources: [S1], [S8], [S11], [S13], [S18], [S19], primary, no date, retrieved 2026-07-22.

- **Parallel-worker specifics require explicit merge-oriented reporting.** Claude worktree docs state worktrees isolate file edits so sessions do not collide, but worktrees share repo history and some project configuration; subagents can set `isolation: worktree`; worktree base can be default branch or local HEAD. Agent-team docs warn that two teammates editing the same file leads to overwrites and recommend assigning different file sets. Codex worktree docs similarly frame worktrees as independent checkouts for parallel chats and note ignored files do not move unless included. Sources: [S4], [S5], [S10], primary, no date, retrieved 2026-07-22.

## Opposing views

- **A dedicated standing prompt may be unnecessary if project instructions and task briefs are already strong.** OpenAI Codex and Cursor both position repo instruction files as the place for project conventions and repeated expectations; Anthropic’s steering guide says `CLAUDE.md` is for always-on project facts and skills are for procedures. If every delegation already includes intent, files, constraints, and repo conventions, a large `general-dev` prompt duplicates context and risks conflict. Sources: [S3], [S8], [S13], primary, 2026-06-18 / no date, retrieved 2026-07-22.

- **Heavy principle lists can degrade compliance.** Anthropic says system-prompt appends have diminishing returns and weaker adherence when many instructions are supplied, especially if contradictory. OpenAI’s 2026 model guidance says leaner prompts improved internal coding-agent eval scores by roughly 10-15% while reducing total tokens by 41-66% and cost by 33-67%, and recommends stating each instruction once. Source: [S3], [S16], primary, 2026-06-18 / no date, retrieved 2026-07-22.

- **Long standing prompts also create context-management risk.** Chroma’s 2025 Context Rot report found 18 LLMs became less reliable as input length grew, even on controlled tasks; Google’s 2024 “Found in the middle” work reports LLMs struggle with relevant information in the middle of long contexts due to positional bias. This argues against stuffing a worker prompt with broad checklists, examples, and procedural runbooks. Sources: [S23], [S24], primary/secondary technical reports or research pages, 2025-07-14 / 2024, retrieved 2026-07-22.

- **Deterministic guardrails beat prose prohibitions.** Anthropic explicitly says if something absolutely must not happen, an instruction is the wrong tool; use hooks and permissions. OpenHands hooks docs make the same product-design move: block dangerous commands and enforce tests/lint before stopping with lifecycle hooks. Source: [S3], [S20], primary, 2026-06-18 / no date, retrieved 2026-07-22.

- **Worker quality may be dominated by task-brief quality and environment feedback, not standing identity.** SWE-agent’s ACI docs say a baseline agent without a tuned agent-computer interface did much worse, and highlight concise file viewers, search output, syntax checks, and command feedback. Anthropic’s Claude Code guidance says verification is the highest-leverage thing users can provide. These claims point to task decomposition, tests, tool feedback, and harness design as larger levers than a long persona prompt. Sources: [S21], [S17], primary, no date / 2025-04-18, retrieved 2026-07-22.

## For the goal

- **A reusable `general-dev` subagent is aligned with Claude Code’s intended subagent use case.** Anthropic says define a custom subagent when you keep spawning the same kind of worker with the same instructions; subagents preserve parent context by doing work in separate context windows and returning summaries. A repeated PR-sized implementation worker with stable philosophy/workflow/reporting is exactly this category. Source: [S1], primary, no date, retrieved 2026-07-22.

- **A focused standing prompt prevents per-delegation drift.** Codex AGENTS.md docs say layered guidance gives consistent expectations no matter which repository you open; Cursor rules similarly exist so persistent preferences/workflows do not need repeated prompting. A short `general-dev` prompt can centralize stable worker invariants while leaving task intent/files/repo specifics to each delegation. Sources: [S8], [S13], primary, no date, retrieved 2026-07-22.

- **The requested philosophy matches frontier-lab defaults.** OpenAI’s Codex base prompt explicitly says avoid unneeded complexity, keep changes minimal and focused, do not fix unrelated bugs, keep style consistent with the codebase, and validate work. These are near-direct support for “plain, conventional, maintainable code; no speculative abstraction; no unrequested refactors.” Source: [S11], primary, no date, retrieved 2026-07-22.

- **Test-first / verification discipline is strongly supported by agentic coding guidance.** Anthropic’s Claude Code best-practices material calls verification the single highest-leverage practice and recommends TDD for verifiable changes: write tests from expected input/output, confirm they fail, implement, and iterate until tests pass. This supports a Red-Green-Refactor standing workflow for non-frontend coding tasks. Sources: [S17], primary, 2025-04-18 / no date, retrieved 2026-07-22.

- **Structured prompts are still justified for complex role definitions.** Anthropic’s Claude API prompt-engineering docs recommend XML tags for complex prompts mixing instructions, context, examples, and variable inputs; the 2025 Anthropic prompt blog says XML tags remain helpful when prompts are extremely complex or content boundaries must be unambiguous. That supports imitating the sibling `frontend-dev` XML-tagged style, provided it stays lean. Sources: [S15], [S14], primary, no date / 2025-11-10, retrieved 2026-07-22.

- **Parallel isolated workers need standing merge discipline.** Anthropic agent-team docs recommend giving teammates enough context, choosing 3-5 teammates for many workflows, sizing tasks as self-contained deliverables, waiting for teammates, and avoiding file conflicts by assigning different file sets. Encoding “touch only declared files; report new files/moved symbols/contract changes/deviations” in `general-dev` directly supports the DAG-parallel pipeline’s merge layer. Source: [S5], primary, no date, retrieved 2026-07-22.

- **Prior art supports the role/description/system-prompt pattern.** Claude Code, Codex, and OpenHands all use agent definitions or instruction files with routeable descriptions plus a body/system prompt. OpenHands file-based agents even show `description` with examples and a Markdown body enumerating review axes; Claude Code uses frontmatter plus body; Cursor uses description/globs/alwaysApply. Sources: [S1], [S9], [S13], [S18], primary, no date, retrieved 2026-07-22.

## Against the goal

- **The prompt should not try to encode everything.** Anthropic’s steering guide says 30-line procedures belong in skills, deterministic requirements belong in hooks/permissions, and path-specific constraints belong in scoped rules. A `general-dev` standing prompt that includes long TDD scripts, repo-specific commands, or every style preference would violate current guidance. Source: [S3], primary, 2026-06-18, retrieved 2026-07-22.

- **A “strict TDD always” mandate may be too rigid.** Anthropic recommends TDD for changes easily verifiable with unit/integration/e2e tests, not for every possible coding task. Codex’s base prompt says add tests when adjacent patterns show a logical place, but do not add tests to codebases with no tests. A prompt that forces Red-Green-Refactor universally could create fake tests, brittle tests, or unnecessary churn in test-poor repos. Sources: [S17], [S11], primary, 2025-04-18 / no date, retrieved 2026-07-22.

- **“Touch only declared files” can conflict with correct fixes.** Minimal scope is good, but PR-sized backend changes often require adjacent tests, type definitions, generated snapshots, docs, or dependency manifests. The safer invariant is: default to declared files; only expand with an explicit reason; report every deviation and why it was necessary. This is an inference from Codex’s “fix root cause,” “minimal focused changes,” and “update docs/tests as necessary” guidance plus Anthropic’s file-conflict guidance. Sources: [S11], [S5], primary, no date, retrieved 2026-07-22.

- **A dedicated worker prompt cannot solve coordination hazards by itself.** Worktrees isolate file edits, but they still share Git metadata and can conflict at merge time. Agent-team docs warn same-file edits cause overwrites; Codex docs note worktrees may omit ignored local files unless included. The orchestrator still needs file ownership, dependency ordering, and merge/rebase conflict handling outside the worker prompt. Sources: [S4], [S5], [S10], primary, no date, retrieved 2026-07-22.

- **Long/authorial philosophy can become checklist blindness.** OpenAI’s lean-prompt guidance and Anthropic’s diminishing-returns warning both argue that repeated principles should be compressed to a small set of behavioral invariants. If the `general-dev` prompt mirrors a rich `frontend-dev` prompt without pruning frontend-specific concerns, it may lower compliance and increase token/cost overhead. Sources: [S16], [S3], primary, no date / 2026-06-18, retrieved 2026-07-22.

- **Published measurement for the exact requested content is thin.** I found strong product guidance and some internal/agent-interface eval claims, but not public 2024-2026 controlled ablations proving that phrases like “prefer simple code,” “match existing conventions,” “TDD,” or “do not refactor unrelated code” individually improve coding-agent maintainability. The best measured evidence is adjacent: OpenAI’s internal lean-system-prompt evals, SWE-agent’s ACI results, and context-length studies. Sources: [S16], [S21], [S23], primary/technical report, 2024-2026, retrieved 2026-07-22.

## Unverified

- I could not verify a public, controlled ablation isolating **explicit simplicity constraints** as a causal factor in coding-agent code quality.

- I could not verify a public, controlled ablation isolating **“match existing conventions”** as a causal factor, though it is repeated in Codex/Claude/Cursor guidance.

- I could not verify a public, controlled ablation isolating **Red-Green-Refactor** for coding agents across repositories. Anthropic recommends it for verifiable changes, but the evidence is guidance/field experience, not a published randomized evaluation.

- I could not verify a public, controlled ablation isolating **“forbid unrequested refactors”**. It appears in frontier-lab defaults as scope discipline, but not as a standalone measured prompt feature.

- I could not retrieve full line-rendered content from the current Cursor docs page because the browser view redirected to a JS app; I relied on the web-search extracted official-doc text for Cursor rules.

- I did not verify the full current source text of OpenHands internal runtime system prompts beyond public docs, README/AGENTS.md, and file-based agent documentation.

## Sources

- **[S1] Anthropic, primary.** “Create custom subagents - Claude Code Docs.” URL: https://code.claude.com/docs/en/sub-agents. Publication date: no date. Retrieved: 2026-07-22.

- **[S2] Anthropic, primary.** “Modifying system prompts - Claude Code Docs.” URL: https://code.claude.com/docs/en/agent-sdk/modifying-system-prompts. Publication date: no date. Retrieved: 2026-07-22.

- **[S3] Anthropic, primary.** “Steering Claude Code: when to use CLAUDE.md, skills, hooks, and subagents.” URL: https://claude.com/blog/steering-claude-code-skills-hooks-rules-subagents-and-more. Publication date: 2026-06-18. Retrieved: 2026-07-22.

- **[S4] Anthropic, primary.** “Run parallel sessions with worktrees - Claude Code Docs.” URL: https://code.claude.com/docs/en/worktrees. Publication date: no date. Retrieved: 2026-07-22.

- **[S5] Anthropic, primary.** “Orchestrate teams of Claude Code sessions - Claude Code Docs.” URL: https://code.claude.com/docs/en/agent-teams. Publication date: no date. Retrieved: 2026-07-22.

- **[S6] Anthropic, primary.** “Orchestrate subagents at scale with dynamic workflows - Claude Code Docs.” URL: https://code.claude.com/docs/en/workflows. Publication date: no date. Retrieved: 2026-07-22.

- **[S7] Anthropic, primary.** “Claude Code: Best practices for agentic coding.” URL: https://www.anthropic.com/engineering/claude-code-best-practices. Publication date: 2025-04-18. Retrieved: 2026-07-22.

- **[S8] OpenAI, primary.** “Custom instructions with AGENTS.md | ChatGPT Learn.” URL: https://learn.chatgpt.com/docs/agent-configuration/agents-md. Publication date: no date. Retrieved: 2026-07-22.

- **[S9] OpenAI, primary.** “Subagents | ChatGPT Learn.” URL: https://learn.chatgpt.com/docs/agent-configuration/subagents. Publication date: no date. Retrieved: 2026-07-22.

- **[S10] OpenAI, primary.** “Worktrees | ChatGPT Learn.” URL: https://learn.chatgpt.com/docs/environments/git-worktrees. Publication date: no date. Retrieved: 2026-07-22.

- **[S11] OpenAI, primary.** “codex/codex-rs/protocol/src/prompts/base_instructions/default.md.” URL: https://github.com/openai/codex/blob/main/codex-rs/protocol/src/prompts/base_instructions/default.md. Publication date: no date. Retrieved: 2026-07-22.

- **[S12] OpenAI, primary.** “Codex CLI | ChatGPT Learn.” URL: https://learn.chatgpt.com/docs/codex/cli. Publication date: no date. Retrieved: 2026-07-22.

- **[S13] Cursor, primary.** “Rules | Cursor Docs.” URL: https://docs.cursor.com/context/rules. Publication date: no date. Retrieved: 2026-07-22.

- **[S14] Anthropic, primary.** “Best practices for prompt engineering for 2026.” URL: https://claude.com/blog/best-practices-for-prompt-engineering. Publication date: 2025-11-10. Retrieved: 2026-07-22.

- **[S15] Anthropic, primary.** “Prompting best practices - Claude API Docs.” URL: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices. Publication date: no date. Retrieved: 2026-07-22.

- **[S16] OpenAI, primary.** “Model guidance | OpenAI API.” URL: https://developers.openai.com/api/docs/guides/latest-model. Publication date: no date. Retrieved: 2026-07-22.

- **[S17] Anthropic, primary.** “Claude Code power user tips.” URL: https://support.claude.com/en/articles/14554000-claude-code-power-user-tips. Publication date: updated ~2026-07. Retrieved: 2026-07-22.

- **[S18] OpenHands, primary.** “File-Based Agents - OpenHands Docs.” URL: https://docs.openhands.dev/sdk/guides/agent-file-based. Publication date: no date. Retrieved: 2026-07-22.

- **[S19] OpenHands, primary.** “OpenHands/AGENTS.md.” URL: https://github.com/OpenHands/OpenHands/blob/main/AGENTS.md. Publication date: no date. Retrieved: 2026-07-22.

- **[S20] OpenHands, primary.** “Hooks - OpenHands Docs.” URL: https://docs.openhands.dev/openhands/usage/customization/hooks. Publication date: no date. Retrieved: 2026-07-22.

- **[S21] SWE-agent, primary.** “Agent Computer Interface - SWE-agent documentation.” URL: https://swe-agent.com/0.7/background/aci/. Publication date: no date. Retrieved: 2026-07-22.

- **[S22] SWE-agent, primary.** “SWE-agent: Agent-Computer Interfaces Enable Automated Software Engineering.” URL: https://arxiv.org/abs/2405.15793. Publication date: 2024. Retrieved: 2026-07-22.

- **[S23] Chroma, primary technical report.** “Context Rot: How Increasing Input Tokens Impacts LLM Performance.” URL: https://www.trychroma.com/research/context-rot. Publication date: 2025-07-14. Retrieved: 2026-07-22.

- **[S24] Google Research, primary research page.** “Found in the middle: Calibrating Positional Attention Bias Improves Long Context Utilization.” URL: https://research.google/pubs/found-in-the-middle-calibrating-positional-attention-bias-improves-long-context-utilization/. Publication date: 2024. Retrieved: 2026-07-22.

- **[S25] Microsoft Research, primary research page.** “Mitigate Position Bias in Large Language Models via Scaling a Single Dimension.” URL: https://www.microsoft.com/en-us/research/publication/mitigate-position-bias-in-large-language-models-via-scaling-a-single-dimension/. Publication date: 2024-06. Retrieved: 2026-07-22.