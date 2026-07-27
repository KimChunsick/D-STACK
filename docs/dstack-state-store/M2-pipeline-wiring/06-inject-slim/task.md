# 06-inject-slim

## Intent / Why
The `UserPromptSubmit` hook injects 1,850 bytes (1,845 characters) into every user prompt,
including pure
questions, and nearly all of it restates `claude/CLAUDE.md` section 0, which is already loaded.
Research tempered the expected saving — a stable injected block that is already cached costs
cache-read rate rather than full input rate — but the context window it occupies is real and
the duplication buys nothing, since the skill body carries the detail either way. Also record
the ultracode gap found while auditing: the alias only wraps interactive shells, so a
non-interactive launch silently loses xhigh effort and workflow orchestration.

## Design consult
Skipped — no trigger. Shortening one string and adding a comment crosses no module boundary and
defines no contract.

## What was done (what / why)
Cut the injected context from 1,850 to 465 bytes / 1,845 to 461 characters (75%). What was
removed was a
near-verbatim restatement of `claude/CLAUDE.md` section 0 — the full phase list, the scheduling
rules, the review contract — all of which is already loaded for the whole session and is carried
in real detail by the skill body itself. What was kept is the one thing a per-prompt injection
can do that an always-loaded file cannot: put the trigger in front of the model at the moment it
decides how to answer, plus a pointer to where the standing rules live.

Deliberately did NOT move the removed prose into `claude/CLAUDE.md`. That file is loaded every
session, so growing it would spend back what this saves; the text already exists there. It IS
edited here, but only by replacing text that had gone stale — its section 0 still described the
removed `.fullcycle-active` registry and still said the gate blocks the turn from ending, and the
whole argument for cutting the injection is that section 0 is the accurate copy. It did GROW,
and an earlier draft of this record wrongly called it net-flat. Measured at Round 8: 8,670 ->
9,304 bytes, 163 -> 171 lines. The figure carries its round because later review rounds keep
editing that file — re-measure rather than trusting a number written earlier. The injection still nets out far ahead (it is paid on every prompt,
CLAUDE.md once per session), but the accounting is stated as measured, not as intended.

The header comment records why the file must stay short, so a future edit does not quietly
regrow it, and states the saving honestly: once stable, this block sits inside the cached prefix,
so on a cache hit its direct cost is cache-read rate rather than full input rate. The context
window it occupies is real either way, and that was the actual complaint.

Separately, recorded the ultracode gap found while auditing — and corrected twice, because the
first two framings were wrong. The dividing line is not which flags you type: it is whether
`~/.zshrc` runs at all. A non-interactive zsh never sources it, so the alias does not exist
there (verified: `zsh -c 'alias claude'` finds nothing while an interactive shell finds it).
`claude -p '…'` typed interactively DOES get the alias, since aliases expand on the command word.
And subagents do inherit the session's reasoning effort by default (the docs say "Default:
inherits from session", with an `effort` frontmatter override); what does not transfer is
ultracode as a session mode. Everything else in `settings.json` follows, which is what makes the
real gap easy to miss — the session looks configured.

## Files changed (where / why)
- `claude/hooks/fullcycle-inject.sh` — trimmed the injected context to the trigger sentence;
  header comment explains the constraint so it is not regrown.
- `claude/ultracode.zsh` — documented which launch paths never receive the alias, and the
  explicit `--effort ultracode` workaround for programmatic launches.

`claude/CLAUDE.md` — section 0 rewritten to describe `.dstack/active/`, the absolute
`"$HOME/.claude/bin/dstack"` invocation, and the one-block-per-turn gate contract. An earlier
draft of this record claimed the file was untouched; that was wrong, and the review caught it.

## Verification (direct run — repo policy: no TDD, no tests)
Parser checks with the RIGHT parser for each file: `bash -n claude/hooks/fullcycle-inject.sh`
clean, `zsh -n claude/ultracode.zsh` clean. (An earlier draft said `bash -n` on both — bash is not
the parser that ever reads a zsh startup file, so that check proved nothing about it.) Ran the
hook against crafted stdin:

| stdin | result | wanted |
|---|---|---|
| `{"prompt":"add a retry to the uploader"}` | emits `UserPromptSubmit` + context | emit |
| `{"prompt":"[quick] what is this"}` | 0 bytes of output | skip |
| `not json` | still emits (prompt parses empty, so no skip token matches) | emit |

Measured from the hook's own output: 1,850 -> 465 bytes, 1,845 -> 461 characters. Both units are
given because they differ here (the block contains multi-byte characters) and because the earlier
figure of 466 was neither — it was `jq -r`'s byte count including its trailing newline. `ls -l`
confirms
`~/.claude/hooks/fullcycle-inject.sh` and `~/.claude/ultracode.zsh` are symlinks into this repo,
so both changes are live.

Note on the ultracode edit: `ultracode.zsh` is sourced by `~/.zshrc` at shell start, so the new
comment reaches an existing shell only after re-sourcing. The alias itself is unchanged, so
nothing about behaviour depends on that.
