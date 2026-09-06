# D-STACK — repository rules (repo-root CLAUDE.md; twin of AGENTS.md)

These rules apply inside this repository and win over the global defaults where they differ.
Precedence: an explicit user instruction in the session > this file > `~/.claude/CLAUDE.md`.
`CLAUDE.md` and `AGENTS.md` are byte-identical apart from the title line; edit both.

## What this repository is

The maintainer's agent configuration — the `dstack` CLI, hooks, skills, agent definitions,
Codex role skills, Korean rule tables — wired into `~/.claude` and `~/.codex` by `install.sh`,
which builds the CLI's release binary with cargo and links `~/.claude/bin/dstack` to it.
It is configuration, not an application: nothing renders, nothing serves.

## Verification here

- The repository policy lives in `.dstack/project/PROJECT.md` (`e2e_evidence: cli`): the
  evidence for a requirement is the captured stdout/stderr/exit of the command its acceptance
  line names, recorded with `dstack evidence add --kind cli`. No screen captures.
- Every checker has fixtures under `claude/lint/fixtures/<checker>/` (`bad-*` must be rejected,
  `good-*` must pass) and `dstack doctor --self` runs them all. A checker without fixtures is not
  merged. Zero-count passes count only when the same run caught the checker's bad fixture.
- D-STACK's own work is `work_type: cli` with `unit_tests: on`: every live R gets a cargo test
  named after its id, and the failing run and the passing run are both recorded as kind test
  evidence.

## Conventions

- The CLI is the Rust crate `dstack-cli/`: its dependencies are exactly serde, serde_json,
  regex, sha2 and time, and git is the only executable it runs. One responsibility per file,
  350 lines at most, which `dstack doctor` checks.
- The default gate is `bash dstack-cli/test.sh` (cargo test, then `cargo clean --profile dev`,
  so the debug tree never stays in the checkout). Historical shell comparisons are opt-in:
  reference-dependent Rust tests are reported as ignored unless `--features shell-parity` is
  passed, and `bash dstack-cli/parity/run.sh` prints `skipped:` unless `--shell-ref <ref>` or
  `--shell <dispatcher>` is explicit. Do not require or restore `shell-final` for ordinary work.
  Explicit comparisons still fail on missing references or real differences; ordinary unit
  tests and fixture checks remain required.
- Three scripts stay shell — `install.sh`, `claude/hooks/dstack-hook.sh` and
  `claude/statusline-command.sh`. They run under the bash macOS ships, so no feature of bash 4
  or later, and jq belongs to the installer alone.
- Request documents (`request.md`, including Goal and quick requests) are always Korean 해요체:
  titles, headings, descriptions, R-row text and acceptance criteria. This applies even when
  `korean_polish: off`. Keep frontmatter keys/enum values, R ids, `accept:` and status markers,
  commands, paths and code identifiers unchanged. Quoted request rows stay in their original Korean.
- Other workflow artifacts (recon, decisions, plans, reviews) are English. Commit messages and
  `README.md` are Korean 해요체. No AI co-author trailers.
- `.dstack/` is local-only and ignored; never commit it and never edit it by hand.
- A sealed review round (`codex-review-NNN.md`) is never edited; rebuttals go to the next round.
- A skill file over 300 lines is a sign that logic belongs in the CLI (§3-8).

## Prompt reuse

Use `dstack prompt render` for review, research, audit and implementation briefs. Its role
instructions are copied from their canonical source before variable task context; do not
prepend paths, run ids, timestamps, status or paraphrases. Keep model, effort and tools stable
within each role. Append fresh state in task context and preserve necessary safety checks and
independent review/audit sessions. Never pad prompts or send keepalive calls to inflate cache
hits. `dstack exec` records supported CLI completion usage in `usage.json`; missing data is
`skipped`, not a measured zero. See `claude/prompt-caching.md` for provider limits and accounting.
