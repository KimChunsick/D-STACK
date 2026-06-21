# D-STACK — Agent Config Backup (SSOT)

This repository is the **single source of truth (SSOT)** for the maintainer's *own
authored* AI-agent configuration — skills, settings, hooks, instruction files —
across multiple agents (Claude, Codex today; Gemini and others later). The real
files live here; the live agent dirs (`~/.claude`, `~/.codex`, …) hold **symlinks**
pointing back into this repo, created by `install.sh`. Edit here, and (for agents
that follow symlinks) every machine sees the change.

## Two layers

- **Layer A — this repo's own agent docs (you are reading it).** `AGENTS.md` is the
  canonical guide, read natively by Codex/Cursor/Gemini/etc. `CLAUDE.md` is a
  one-line `@AGENTS.md` import so Claude Code reads the same base. Both are real
  files at the repo root — never symlinks — so the relative import always resolves.
- **Layer B — the backed-up personal configs (the content).** Authored artifacts
  live under agent-first folders: `claude/`, `codex/`, `gemini/`. `install.sh`
  links each into the live agent dir.

## Golden rules

1. **Never commit secrets or runtime state.** This repo is **public**. The
   `.gitignore` is a **true allowlist at every level we can enumerate**: it denies
   everything, then re-includes only named files. Each agent dir is `deny-all`
   internally — `hooks/` and `rules/` are pinned to exact files, `skills/` to exact
   skill dirs — so an unanticipated name (`claude/id_rsa`, a novel hook blob, a new
   skill dir) is *untrackable* by default. A second layer hard-denies secret names
   anywhere (`auth.json`, `config.toml`, `credentials.json`, `id_rsa`, `*.key`,
   `*.pem`, `*.p12`, `*.token`, `*.sqlite*`, `*.db`, `history.jsonl`, `.env*`,
   `.DS_Store`, `sessions/`, `projects/`, `memory/`, …).
   **Adding a backed-up file/skill:** add the matching `!`-allow line in `.gitignore`
   (e.g. `!/claude/skills/<new-skill>/`) — nothing is tracked until explicitly named.
   **Residual:** files *inside* a named skill dir are wholesale (so skills can grow
   files freely); the secret-name deny list is their backstop — never put a secret
   inside a skill dir. After any change run `bash tests/run.sh` — the secret-scan
   guard (incl. a nested-unknown probe battery) must stay green.
2. **Only the maintainer's *own authored* artifacts.** Third-party / marketplace /
   plugin skills are out (e.g. anything namespaced `ckm:`, `ui-ux-pro-max`,
   `anthropic-frontend-design`, plugin-provided skills). Back up what you wrote,
   not what you installed.
3. **Public-safe paths.** No machine-specific absolute paths in tracked files
   (`/Users/<name>/…`). Use `$HOME`-relative forms; `install.sh` resolves the rest.

## How `install.sh` works

`install.sh [--dry-run]` walks a declared map of `repo_path → live_target` entries.
For each: it skips the entry if the parent agent dir is absent; backs up any
pre-existing real file to `<target>.bak.<timestamp>`; then creates a symlink
(`link` mode) or copies (`copy` mode). It is idempotent — re-running is a no-op.

**Per-agent quirk:** Claude and Codex follow symlinked config files. **Gemini CLI
intentionally ignores symlinked context files** (GH google-gemini/gemini-cli#11547,
"not planned"), so Gemini entries use `copy` mode — re-run `install.sh` after editing.

Running against your real home is a deliberate, manual step. Review
`./install.sh --dry-run` first; existing files are backed up to `*.bak`.

## How to add a new agent

1. Create a top-level folder (e.g. `gemini/`) and add it to `.gitignore`'s allowlist
   (`!/gemini/`) if not already present.
2. Put only authored artifacts in it; keep secrets out.
3. Add its entries to the `install.sh` map with the correct mode (`link` or `copy`).
4. Add/extend a `tests/test_<agent>_artifacts.sh` guard and keep `tests/run.sh` green.

## Tests

`bash tests/run.sh` runs every `tests/test_*.sh` (plain bash, no external deps).
All tests must pass before committing.
