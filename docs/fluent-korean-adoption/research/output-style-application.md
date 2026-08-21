## Needed info

- Claude Code output styles are the right primitive only when the goal is changing response role/tone/format every turn. They modify the system prompt; `CLAUDE.md` remains separate context after the system prompt and is the better place for project or personal coding conventions. [S1]
- Custom Claude Code output styles live at user scope `~/.claude/output-styles`, project scope `.claude/output-styles`, or managed-policy scope. Project styles can be found in nested `.claude/output-styles/` directories, with the closest matching style winning. [S1]
- Supported Claude Code style frontmatter is `name`, `description`, `keep-coding-instructions`, and plugin-only `force-for-plugin`; `keep-coding-instructions` defaults to `false`. [S1]
- A custom Claude Code style without `keep-coding-instructions: true` removes Claude Code’s built-in software-engineering instructions, including guidance on scoping changes, comments, and verification. That is exactly what `fluent-korean-not-coding` does. [S1], [S8]
- Activation is currently via `/config` or direct `outputStyle` in settings. The old `/output-style` command is documented as deprecated in v2.1.73 and removed in v2.1.91. Style changes apply only after `/clear` or a new session because the system prompt is read at session start. [S1]
- `/config` saves the selected style to `.claude/settings.local.json`; settings precedence is managed > command-line > local > project > user. Most settings hot-reload, but `outputStyle` is a session-start key. [S2]
- Plugins can ship styles, and marketplaces install plugins with `/plugin marketplace add ...` then `/plugin install plugin@marketplace`; manual placement of the `.md` file in `~/.claude/output-styles/` or `.claude/output-styles/` is also viable for this use case. [S4], [S7]
- Claude Code output styles apply to the main conversation only. Non-fork subagents use their own system prompt; forks inherit the parent’s full prompt. [S1], [S3]
- `snflkd/fluent-korean` is MIT-licensed, ships `fluent-korean` with `keep-coding-instructions: true`, and ships `fluent-korean-not-coding` without that field. Its README explicitly says the coding variant is for coding work and the not-coding variant is for when Claude is not directly changing code. [S6], [S7], [S8], [S9]
- For Claude web/desktop, the current personalization primitives are account-wide “Instructions for Claude,” project instructions, Skills, and Styles. Styles are specifically for tone/format; project instructions are scoped to one project; profile instructions are account-wide. [S11], [S12], [S13]
- Claude custom Styles support presets and custom styles generated from writing samples or described instructions, and can be switched during a conversation for new messages, edits, and retries. [S10], [S14]
- I could not verify current primary-source character limits for Claude personal instructions, project instructions, or custom Style manual instructions. A secondary source claims 1,500 characters for Claude profile instructions and about 8,000 for project instructions, but that should be treated as unverified until checked in the user’s live UI. [S22]
- ChatGPT custom instructions are available on Web, Desktop, iOS, and Android; they apply immediately to all chats and are account-level. Current limits are 1,500 characters for Free/Go and 5,000 characters for Plus/Pro/Enterprise/Business/Education. [S15]
- ChatGPT Projects let users add project-specific instructions; those instructions only apply inside that project and override global custom instructions. Projects are available to all logged-in free and paid users. [S16]
- ChatGPT memory is distinct from custom instructions: custom instructions are explicit guidance, while memory is synthesized from chats/files/apps when enabled. Project-only memory can prevent a project from drawing on non-project chats or saved memories. [S17], [S16]
- Custom GPTs are another possible ChatGPT surface for reusable behavior, but GPT conversations do not use saved memory, custom instructions, or previous conversations; creating/editing GPTs requires a paid subscription. [S18]

## Opposing views

- Opposing view: applying only `fluent-korean-not-coding` to Claude Code is the wrong default if Claude Code is still used for coding, because it intentionally removes the coding prompt layer. Counter-argument: it is sound only for explicitly non-coding Claude Code sessions or non-coding Claude/ChatGPT surfaces. [S1], [S7], [S8]
- Opposing view: the user already has `~/.claude/CLAUDE.md` Korean-writing rules and a harness-level “always Korean” rule, so another Korean style can add redundancy and conflict. Counter-argument: a style can still help if the duplicated rules are consolidated and the new text is limited to mechanical fluency rules that are not already present. [S5], [S7]
- Opposing view: Claude Code subagents will not inherit the active style, so the goal may fail in multi-agent workflows. Counter-argument: the main agent’s final user-facing summary can still be styled; subagent prompts need separate testing or explicit subagent instructions. [S1], [S3], [S7]
- Opposing view: the public SSOT repository should not contain third-party style files. Counter-argument: the MIT license permits copying, but the user’s policy excludes third-party artifacts from the public SSOT, so installation should happen only in live agent directories or via plugin marketplace, not by committing the style file. [S9], [S4]
- Opposing view: current Claude/ChatGPT models and built-in Styles may already be good enough. Counter-argument: current Korean-specific community reports still describe long-session Korean degradation and English drift, and generic Styles do not target Korean particles, endings, or predicate completion. [S10], [S19], [S8]

## For the goal

- Official product mechanisms exist for persistent style guidance on all relevant surfaces: Claude Code output styles, Claude web Styles/profile/project instructions, ChatGPT custom instructions, ChatGPT Projects, and custom GPTs. [S1], [S10], [S11], [S15], [S16], [S18]
- The `fluent-korean-not-coding` artifact directly targets the stated fluency problems: omitted sentence components, missing particles/endings, incomplete predicates, over-compressed noun phrases, metaphor-swapped vocabulary, and em-dash compression. [S8]
- Korean linguistic evidence supports focusing on particles/case markers: Korean postpositional particle errors are a known challenge, and case markers are fundamental to Korean syntax and meaning. [S20], [S21]
- LLM-specific Korean grammar evidence supports adding targeted Korean guidance: KoGEM reports that LLMs still have uneven Korean linguistic competence, especially beyond straightforward definitional knowledge. [S23]
- Claude Code docs explicitly say output styles are meant for repeated response style or non-software-engineering roles, matching the not-coding variant if the chosen Claude Code sessions are genuinely non-coding. [S1]
- ChatGPT paid custom instructions can hold a 2-4 KB style guide if it stays below 5,000 characters; ChatGPT Projects can be a better fit when the style should be scoped and override global instructions only inside that project. [S15], [S16]
- Claude Styles are explicitly designed to customize tone, structure, and communication preferences, and can be based on manually supplied custom instructions or writing samples. [S10], [S14]

## Against the goal

- Token and context cost is real: Claude Code output styles add input tokens to the system prompt, `CLAUDE.md` consumes context every session, and the fluent-korean README itself warns that restoring omitted Korean morphemes and sentence components increases token use. [S1], [S5], [S7]
- Instruction conflict is a real model failure mode, not just a theoretical concern. IHEval found sharp performance declines when models face conflicting instructions; OpenAI’s instruction-hierarchy work exists because models can fail to resolve priority conflicts reliably. [S24], [S25]
- Long or crowded contexts can reduce reliability. Claude Code docs recommend concise, specific `CLAUDE.md` files and warn conflicts can be followed arbitrarily; long-context research documents retrieval/utilization degradation in long prompts. [S5], [S26]
- Installing the not-coding style globally in Claude Code can remove safety and quality behaviors that matter to the user’s pipeline, especially verification and change-scoping habits. [S1]
- For coding CLI use, a better alternative exists: use `fluent-korean` with `keep-coding-instructions: true`, or move only the Korean fluency deltas into `CLAUDE.md` / a shorter appended prompt. This conflicts with the stated “only not-coding variant” constraint, but it is the lower-risk engineering recommendation for coding sessions. [S1], [S7]
- For ChatGPT, account-wide custom instructions may be too broad if the Korean style should not apply to every conversation; Projects or custom GPTs provide narrower scope. [S15], [S16], [S18]

## Unverified

- Exact current Claude web/desktop character limits for personal instructions, project instructions, and custom Style manual instructions were not verified from a current primary source. [S11], [S22]
- Exact current ChatGPT Project instruction character limit and custom GPT instruction character limit were not verified from a current primary source. [S16], [S18]
- I found no controlled evaluation proving that `fluent-korean` measurably improves Claude or ChatGPT Korean output versus baseline; evidence is prior art, rationale, examples, and community adoption rather than a benchmark. [S7], [S8]
- I found no controlled data quantifying coding-quality loss from using a no-coding-instructions Claude Code output style; the risk is inferred from official docs plus community troubleshooting guidance. [S1], [S27]
- Whether Claude web/desktop “Styles” can comfortably store the full 2-4 KB markdown guideline must be checked in the user’s live UI. [S10], [S22]
- Whether applying the not-coding variant to the user’s exact Claude Code/Codex/ChatGPT surfaces preserves the existing harness behavior requires a local interview and live smoke tests, especially around `~/.claude/CLAUDE.md`, `/context`, `/status`, `/clear`, and subagent behavior. [S1], [S2], [S3], [S5]

## Sources

- [S1] Primary, no date, retrieved 2026-08-21: Claude Code Docs, “Output styles” — https://code.claude.com/docs/en/output-styles
- [S2] Primary, no date, retrieved 2026-08-21: Claude Code Docs, “Settings” — https://code.claude.com/docs/en/settings
- [S3] Primary, no date, retrieved 2026-08-21: Claude Code Docs, “Create custom subagents” — https://code.claude.com/docs/en/sub-agents
- [S4] Primary, no date, retrieved 2026-08-21: Claude Code Docs, “Discover and install prebuilt plugins” — https://code.claude.com/docs/en/discover-plugins
- [S5] Primary, no date, retrieved 2026-08-21: Claude Code Docs, “How Claude remembers your project” — https://code.claude.com/docs/en/memory
- [S6] Primary, no date, retrieved 2026-08-21: fluent-korean plugin manifest — https://raw.githubusercontent.com/snflkd/fluent-korean/main/plugins/fluent-korean/.claude-plugin/plugin.json
- [S7] Primary, no date, retrieved 2026-08-21: snflkd/fluent-korean README — https://github.com/snflkd/fluent-korean
- [S8] Primary, no date, retrieved 2026-08-21: `fluent-korean-not-coding.md` — https://raw.githubusercontent.com/snflkd/fluent-korean/main/plugins/fluent-korean/output-styles/fluent-korean-not-coding.md
- [S9] Primary, copyright 2026, retrieved 2026-08-21: fluent-korean MIT License — https://raw.githubusercontent.com/snflkd/fluent-korean/main/LICENSE
- [S10] Primary, updated “this week” in fetched search result, retrieved 2026-08-21: Anthropic Help, “Configuring and Using Styles” — https://support.anthropic.com/en/articles/10181068-configuring-and-using-styles
- [S11] Primary, updated 2026-08-20, retrieved 2026-08-21: Claude Help, “Understanding Claude’s personalization features” — https://support.claude.com/en/articles/10185728-understanding-claude-s-personalization-features
- [S12] Primary, updated over 3 weeks before retrieval, retrieved 2026-08-21: Claude Help, “What are projects?” — https://support.claude.com/en/articles/9517075-what-are-projects
- [S13] Primary, updated over 1 month before retrieval, retrieved 2026-08-21: Claude Help, “How to create custom skills” — https://support.claude.com/en/articles/12512198-how-to-create-custom-skills
- [S14] Primary, published 2024-11-26, retrieved 2026-08-21: Anthropic, “Tailor Claude’s responses to your personal style” — https://www.anthropic.com/news/styles
- [S15] Primary, updated 2026-08-18, retrieved 2026-08-21: OpenAI Help, “ChatGPT Custom Instructions” — https://help.openai.com/en/articles/8096356-custom-instructions-for-chatgpt
- [S16] Primary, updated 2026-08-20, retrieved 2026-08-21: OpenAI Help, “Projects in ChatGPT” — https://help.openai.com/en/articles/10169521-using-projects-in-chatgpt
- [S17] Primary, updated 2026-08-16, retrieved 2026-08-21: OpenAI Help, “Memory FAQ” — https://help.openai.com/en/articles/8590148-memory-and-projects
- [S18] Primary, updated 2026-08-20, retrieved 2026-08-21: OpenAI Help, “GPTs in ChatGPT” — https://help.openai.com/en/articles/8554407-gpts-in-chatgpt
- [S19] Primary/community report, opened 2026-07-19, retrieved 2026-08-21: anthropics/claude-code issue #78996 — https://github.com/anthropics/claude-code/issues/78996
- [S20] Primary research, published approx. 2026 per search excerpt/no exact date verified, retrieved 2026-08-21: “Enriching the Korean learner corpus for grammatical error correction and writing assessment” — https://link.springer.com/article/10.1007/s10579-025-09882-9
- [S21] Primary research, published 2024-07-10, retrieved 2026-08-21: “Does Incomplete Syntax Influence Korean Language Model?” — https://openreview.net/forum?id=yfyHxvVzZT
- [S22] Secondary, published 2026-05-30, retrieved 2026-08-21: Like One, “Claude Custom Instructions: Character Limit (2026)” — https://likeone.ai/blog/claude-custom-instructions-guide/
- [S23] Secondary index of primary paper, published 2025-06-02, retrieved 2026-08-21: “Polishing Every Facet of the GEM” / KoGEM summary — https://www.emergentmind.com/papers/2506.01237
- [S24] Primary research, published 2025-04, retrieved 2026-08-21: ACL Anthology, “IHEval” — https://aclanthology.org/2025.naacl-long.425/
- [S25] Primary, no exact date in fetched excerpt, retrieved 2026-08-21: OpenAI, “Improving instruction hierarchy in frontier LLMs” — https://openai.com/index/instruction-hierarchy-challenge/
- [S26] Primary research, NeurIPS 2024 / April 2024, retrieved 2026-08-21: Microsoft Research, “Make Your LLM Fully Utilize the Context” — https://www.microsoft.com/en-us/research/publication/make-your-llm-fully-utilize-the-context/
- [S27] Secondary/community guide, no date, retrieved 2026-08-21: jensensics, “claude-code-output-styles” — https://github.com/jensensics/claude-code-output-styles