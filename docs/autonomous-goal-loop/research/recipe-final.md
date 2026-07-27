## Needed info

Anthropic’s public Claude Code CLI reference documents `claude`, not `codex`; I found no Anthropic entry for `codex exec --ephemeral`. It does document Claude Code’s separate `--no-session-persistence` flag: disables saving sessions to disk and prevents resume. [Anthropic CLI reference, no date, retrieved 2026-07-27, primary](https://code.claude.com/docs/en/cli-usage)

OpenAI’s Codex docs say `codex exec --ephemeral` means: do not persist session rollout files to disk. [OpenAI non-interactive mode, no date, retrieved 2026-07-27, primary](https://learn.chatgpt.com/docs/non-interactive-mode); [OpenAI developer commands, no date, retrieved 2026-07-27, primary](https://learn.chatgpt.com/docs/developer-commands?surface=cli)

## Opposing views

A current open GitHub issue claims `codex exec --ephemeral resume <id>` persisted rollout data and made the “ephemeral” turn visible to a later non-ephemeral resume. I could not verify whether OpenAI has confirmed or fixed that report. [GitHub issue #20084, 2026-04-28, retrieved 2026-07-27, secondary/user report](https://github.com/openai/codex/issues/20084)

## For the goal

The goal “use `--ephemeral` to avoid local Codex session persistence” matches OpenAI’s docs and source comments: the CLI flag is defined as running without persisting session files; config says ephemeral sessions are not persisted on disk. [cli.rs, no date, retrieved 2026-07-27, primary/source](https://raw.githubusercontent.com/openai/codex/main/codex-rs/exec/src/cli.rs); [config/mod.rs, no date, retrieved 2026-07-27, primary/source](https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/src/config/mod.rs)

## Against the goal

Using Anthropic docs as authority for Codex CLI behavior is not sound: Anthropic documents Claude Code, while Codex CLI is documented by OpenAI. [Anthropic CLI reference, no date, retrieved 2026-07-27, primary](https://code.claude.com/docs/en/cli-usage); [OpenAI Codex CLI docs, no date, retrieved 2026-07-27, primary](https://learn.chatgpt.com/docs/codex/cli)

## Unverified

I could not verify from Anthropic docs anything about `codex exec --ephemeral`.

I could not verify a precise public-doc statement that Codex CLI persists “prompts or output” specifically to session history. OpenAI source says rollout JSONL persists Codex session rollouts for replay/inspection, but the checked public docs phrase this as “session rollout files,” not explicitly “prompts and output.” [recorder.rs, no date, retrieved 2026-07-27, primary/source](https://raw.githubusercontent.com/openai/codex/main/codex-rs/rollout/src/recorder.rs)

## Sources

- Anthropic Claude Code CLI reference — https://code.claude.com/docs/en/cli-usage — no date — retrieved 2026-07-27 — primary.
- OpenAI Codex non-interactive mode — https://learn.chatgpt.com/docs/non-interactive-mode — no date — retrieved 2026-07-27 — primary.
- OpenAI Codex developer commands — https://learn.chatgpt.com/docs/developer-commands?surface=cli — no date — retrieved 2026-07-27 — primary.
- OpenAI Codex source, `cli.rs` — https://raw.githubusercontent.com/openai/codex/main/codex-rs/exec/src/cli.rs — no date — retrieved 2026-07-27 — primary/source.
- OpenAI Codex source, `config/mod.rs` — https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/src/config/mod.rs — no date — retrieved 2026-07-27 — primary/source.
- OpenAI Codex source, `recorder.rs` — https://raw.githubusercontent.com/openai/codex/main/codex-rs/rollout/src/recorder.rs — no date — retrieved 2026-07-27 — primary/source.
- GitHub issue #20084 — https://github.com/openai/codex/issues/20084 — 2026-04-28 — retrieved 2026-07-27 — secondary/user report.