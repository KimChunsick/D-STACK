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

## Language boundary

- Communicate directly with the user in Korean.
- Write all workflow artifacts — Goal, research, task, review, plan, and recorded evidence documents — in English.
- Write every prompt, brief, follow-up, status message, and report passed between agents or models in English.
- Product copy, source comments, and repository documentation follow the target project's own conventions unless the user explicitly requires a language.

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
   inside a skill dir. After any change run `bash tests/secret-guard.sh` — the
   secret-scan guard (incl. a nested-unknown probe battery) must stay green. The
   guard checks names/trackability, not file contents: never paste a credential
   into an allowlisted (tracked) file.
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

## Runtime state lives in `.dstack/`, never at a repo root

Config is what this repo *authors* (`claude/`, `codex/`, `gemini/`, linked into the live agent
dirs). Runtime state is what the pipeline *writes while running*, and it belongs in exactly one
place: a `.dstack/` directory at the root of whichever repository the work is happening in.
Keep that line sharp — it is why this stopped being a scatter of `.fullcycle-*` dotfiles.

`claude/bin/dstack` owns that directory; nothing else writes it. **Invoke it by absolute path**
— `install.sh` links it to `~/.claude/bin/dstack` and nothing puts that directory on `PATH`, so a
bare `dstack` resolves only if you added it yourself and never resolves in a non-interactive
shell.

```
.dstack/
  .gitignore     a single `*`, so no target repo's tracked .gitignore is ever edited
  version        schema marker
  active/<sha1>  one-line JSON record per registered document: {v, session, doc, ts}
  runs/<sid>/…   capture storage for long external runs, mode 700, swept on capture creation
                 (`find -mtime +7` truncates to whole days, so removal starts at 8 complete days)
```

- `"$HOME/.claude/bin/dstack" reg|unreg <doc>` claim and release; `reclaim <doc>…` takes over from another session
  **explicitly** (there is no liveness signal, so it never sweeps); `status` lists everything;
  `migrate` converts a legacy `.fullcycle-active` and **refuses** anything it cannot represent
  losslessly.
- `"$HOME/.claude/bin/dstack" run <label> [--stdin <file>] -- <cmd>…` runs ONE long external command
  (a codex round, CI) inside its capture directory and **blocks until it finishes**, publishing the
  command's status to `exit`. Start it from a background call and that call's completion
  notification is what resumes the session — there is no separate watcher to arm, which is the step
  that used to get skipped and leave a finished round sitting unread. It deliberately does **not**
  detach: a detached process survives but is invisible to the harness, so it could never notify.
  Exit 6 means the launched command failed; its exact status is on the `DONE` line and in `exit`.
- Being gitignored is not confidentiality. Backups, sync folders, and snapshots all see
  `runs/`, which is why it is mode 700 and short-lived.
- Requires `jq`, `git`, and `shasum`/`sha1sum` (bash cannot compute SHA-1 itself).

## How to add a new agent

1. Create a top-level folder (e.g. `gemini/`) and add it to `.gitignore`'s allowlist
   (`!/gemini/`) if not already present.
2. Put only authored artifacts in it; keep secrets out.
3. Add its entries to the `install.sh` map with the correct mode (`link` or `copy`).
4. Keep `bash tests/secret-guard.sh` green — update its pinned negation list in the
   same change as the new `.gitignore` `!`-allow lines.

## No TDD, no new tests in this repo

This repo holds agent *configuration* — bash hooks, skill Markdown, an installer — not
application code. Red-Green-Refactor buys nothing here, so it is off by policy and
overrides the global full-cycle rule for this repo only.

- Do not run Red-Green-Refactor cycles, and do not add new test files.
- The rest of the full-cycle pipeline still applies. Its TDD step (P7) is replaced by
  **direct verification**: run the thing — `./install.sh --dry-run`, a hook against a
  crafted stdin fixture, `check-parallel.sh` against a sample declaration — and record
  the actual output in `task.md` as evidence. Word that task's Gate-status row for what
  you really did, e.g. `- [ ] Verification: behavior confirmed by direct run (repo
  policy: no TDD)`. The Stop hook parses checkbox rows, not their labels, so the wording
  is free; the honesty rule is not.
- Existing checks stay and keep being run: `bash tests/secret-guard.sh` before every
  commit, and `claude/skills/full-cycle/tests/*.test.sh` when the thing they cover
  changes. Don't delete them, don't grow the set.
- To be unambiguous, because "no tests" and "run the test scripts" read as a contradiction:
  what is banned is **authoring** — new test files, and Red-Green-Refactor cycles. What is
  required is **running** the two pinned checks above. Editing an assertion inside one of them
  because the thing it pins was deliberately removed is maintenance of an existing check, not a
  new test; the set must not grow either way.

## Tests

The repo keeps exactly one meta check: `bash tests/secret-guard.sh` (plain bash, no
external deps) — the secret-trackability guard. Run it before every commit; it must
pass. Its pinned negation list must be updated in the same change as any `.gitignore`
allowlist edit.
