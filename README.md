# D-STACK

Single-source-of-truth backup of my **own authored** AI-agent configuration —
skills, settings, hooks, and instruction files — for Claude and Codex (Gemini and
others later). The real files live in this repo; the live agent dirs get symlinks
back into it via `install.sh`.

> Agents working in this repo: read [`AGENTS.md`](./AGENTS.md) — it is the canonical
> guide. (`CLAUDE.md` just imports it.)

## Layout

```
AGENTS.md / CLAUDE.md   # this repo's own agent guide (CLAUDE.md → @AGENTS.md)
install.sh              # symlink/copy the configs into ~/.claude, ~/.codex, …
claude/  codex/  gemini/  # backed-up authored configs, per agent
tests/                  # plain-bash guards (secret-scan, structure, install)
docs/                   # full-cycle plan + per-task docs
```

## Restore on a new machine

```bash
git clone <repo-url> D-STACK
cd D-STACK
./install.sh --dry-run   # review what will change
./install.sh             # create symlinks (existing files are backed up to *.bak)
```

Gemini context files are **copied**, not symlinked (Gemini ignores symlinks), so
re-run `./install.sh` after editing them.

Third-party **plugins/marketplaces are intentionally not backed up** (they're not
authored content, and the marketplace refs can disclose affiliation). After restore,
re-enable any plugins you use manually (e.g. via `/plugin`).

## Uninstall

`install.sh` replaces each live file with a symlink and saves the original under
`~/.dstack-backups/<timestamp>/` (mirroring the live path) — so a backed-up skill/hook
dir is never re-discovered as a duplicate. To revert an entry, delete the symlink and
move its backup back, e.g.
`rm ~/.claude/CLAUDE.md && mv ~/.dstack-backups/<ts>/.claude/CLAUDE.md ~/.claude/CLAUDE.md`.

## Safety

Public repo. Secrets and runtime state (`auth.json`, `config.toml`, `*.sqlite`,
history, sessions, per-project memory) are never tracked — enforced by an allowlist
`.gitignore` and `tests/test_gitignore_secret_guard.sh`. Run `bash tests/run.sh`
before every commit.
