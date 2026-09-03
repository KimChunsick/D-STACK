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
- The gates are `cargo test` and `bash dstack-cli/parity/run.sh`, which drives the binary
  against the shell implementation the `shell-final` tag still carries.
- Three scripts stay shell — `install.sh`, `claude/hooks/dstack-hook.sh` and
  `claude/statusline-command.sh`. They run under the bash macOS ships, so no feature of bash 4
  or later, and jq belongs to the installer alone.
- Workflow artifacts (request, recon, decisions, plans, reviews) are English. Commit messages and `README.md` are Korean 해요체. No AI co-author trailers.
- `.dstack/` is local-only and ignored; never commit it and never edit it by hand.
- A sealed review round (`codex-review-NNN.md`) is never edited; rebuttals go to the next round.
- A skill file over 300 lines is a sign that logic belongs in the CLI (§3-8).
