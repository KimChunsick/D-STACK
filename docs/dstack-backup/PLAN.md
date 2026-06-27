# D-STACK Agent-Config Backup — Implementation Plan

> **For agentic workers:** Implement this plan task-by-task — a fresh subagent per task with a review between tasks. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the empty `D-STACK` repo into an extensible, **public-safe**, single-source-of-truth (SSOT) backup of the user's *own authored* AI-agent configs (Claude + Codex today; Gemini/others later), where the repo holds the real files and `~/.claude` / `~/.codex` get symlinks pointing back here.

**Architecture:** Two layers.
- **Layer A — the repo's own agent docs (what the user's `AGENTS.md/CLAUDE.md` instruction meant):** repo-root `AGENTS.md` is the canonical guide read natively by Codex/Cursor/etc.; repo-root `CLAUDE.md` is a one-line `@AGENTS.md` import so Claude reads the same base. Both are **real files at the repo root** (no symlink), so the relative import is safe (immune to the symlink/relative-import bug confirmed in research, GH #4754).
- **Layer B — the backed-up personal configs (the content):** authored artifacts live under agent-first folders (`claude/`, `codex/`, `gemini/`). `install.sh` symlinks each into the live agent dir 1:1. Per-agent quirk handling: Claude/Codex follow symlinks for these files; **Gemini intentionally ignores symlinked context files (GH #11547, "not planned") → copy, not symlink** when that day comes.

**Tech Stack:** POSIX shell (`install.sh`, tests), Markdown, git. No runtime deps beyond coreutils + git (avoid `bats`; use plain-bash assertion scripts so the harness has zero install requirements).

## Global Constraints

- **Public repo. Allowlist, not blocklist.** `.gitignore` denies everything top-level, then re-includes only curated paths; known-secret filenames are hard-denied even inside allowed dirs. Verbatim exclusions: `auth.json`, `*.sqlite*`, `history.jsonl`, `config.toml`, `.DS_Store`, `*.key`, `*.pem`, `.env`, sessions/projects/runtime state.
- **Only the user's *own* authored artifacts.** Third-party skills (every `ckm:`-prefixed skill, `ui-ux-pro-max`, `anthropic-frontend-design`) and plugin-provided skills are **out**. In-scope authored set is fixed (see Inventory).
- **`install.sh` mutates the user's real `~/.claude`/`~/.codex`.** It MUST: back up any pre-existing real file to `*.bak.<ts>` before linking; be idempotent; support `--dry-run`; never touch anything outside the known map. Tests run against a **sandbox `HOME`** (`mktemp -d`), never the real home.
- **No secret ever enters a commit.** A secret-scan test gates the repo; `git ls-files` must match no secret pattern.

## Inventory (in-scope authored artifacts — verified by provenance)

| Repo path (SSOT) | Source (live) | Live target (symlink) | Notes |
|---|---|---|---|
| `claude/CLAUDE.md` | `~/.claude/CLAUDE.md` | `~/.claude/CLAUDE.md` | user's global full-cycle instructions; self-contained (no imports) |
| `claude/settings.json` | `~/.claude/settings.json` | `~/.claude/settings.json` | **3 hardcoded `/Users/<user>` paths → make `$HOME`-relative** |
| `claude/statusline-command.sh` | `~/.claude/statusline-command.sh` | same | path-clean |
| `claude/hooks/fullcycle-inject.sh` | `~/.claude/hooks/…` | same | path-clean |
| `claude/hooks/fullcycle-gate.sh` | `~/.claude/hooks/…` | same | path-clean |
| `claude/skills/full-cycle/` | `~/.claude/skills/full-cycle/` | same (dir symlink) | authored |
| `claude/skills/codex-review/` | `~/.claude/skills/codex-review/` | same (dir symlink) | authored |
| `codex/instructions.md` | `~/.codex/instructions.md` | `~/.codex/instructions.md` | Next.js/TS stack, 318 lines |
| `codex/rules/default.rules` | `~/.codex/rules/default.rules` | same | authored allow-rules |
| `gemini/` | — | — | placeholder for extensibility |

**Explicitly excluded:** `~/.codex/auth.json`, `config.toml`, `*.sqlite*`, both `history.jsonl`, `sessions/`, `projects/`, per-project `memory/`, `.codex-global-state.json`, all `ckm:`/public design skills, `codex-primary-runtime` (empty dir).

## File Structure (created by this plan)

```
D-STACK/
  AGENTS.md            # Layer A base guide (repo conventions + safety + how-to-extend)
  CLAUDE.md            # Layer A: "@AGENTS.md" + 1 Claude note
  README.md            # human onboarding: what/why/restore-in-one-command
  install.sh           # Layer B: symlink map + per-agent policy + --dry-run + backup
  .gitignore           # allowlist + hard secret deny
  claude/ … codex/ … gemini/   # Layer B SSOT content (Inventory)
  tests/
    run.sh             # runs all test scripts, exits non-zero on any fail
    lib.sh             # assert helpers (assert_eq, assert_contains, fail)
    test_gitignore_secret_guard.sh
    test_repo_agents_docs.sh
    test_claude_artifacts.sh
    test_codex_artifacts.sh
    test_install_sh.sh
  docs/dstack-backup/  # this plan + per-task gate docs
```

---

## Milestone 1 — Foundation & Safety

Security-first skeleton. Nothing else may land before the secret guard exists.

### Task 1: Allowlist `.gitignore` + secret-scan guard

**Files:** Create `.gitignore`, `tests/lib.sh`, `tests/run.sh`, `tests/test_gitignore_secret_guard.sh`.

**Interfaces — Produces:** `tests/lib.sh` exposes `assert_eq`, `assert_contains`, `assert_not_contains`, `fail`, `pass`; `tests/run.sh` discovers and runs `tests/test_*.sh`.

- [ ] **Step 1 — failing test.** Write `tests/test_gitignore_secret_guard.sh`: drop dummy secrets into allowed dirs and assert git ignores them.
```bash
#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"
# secrets placed inside an allowed dir must still be ignored
mkdir -p claude/skills/full-cycle
for f in claude/auth.json codex/config.toml claude/x.sqlite claude/history.jsonl claude/.DS_Store; do
  : > "$f"
  git check-ignore -q "$f" || fail "NOT ignored: $f"
done
# tracked tree must contain no secret pattern
if git ls-files | grep -E 'auth\.json|\.sqlite|history\.jsonl|^.*config\.toml|\.DS_Store|\.pem$|\.key$|\.env$'; then
  fail "secret pattern present in git ls-files"
fi
rm -f claude/auth.json codex/config.toml claude/x.sqlite claude/history.jsonl claude/.DS_Store
pass "gitignore secret guard"
```
- [ ] **Step 2 — run, expect FAIL.** `bash tests/run.sh` → fails (no `.gitignore` yet).
- [ ] **Step 3 — implement `.gitignore`** (allowlist then hard secret deny):
```gitignore
# Deny everything at top level by default…
/*
# …re-include only curated paths
!/.gitignore
!/AGENTS.md
!/CLAUDE.md
!/README.md
!/install.sh
!/docs/
!/tests/
!/claude/
!/codex/
!/gemini/
# Hard-deny secrets/runtime even inside allowed dirs (last match wins)
**/auth.json
**/*.sqlite
**/*.sqlite-*
**/history.jsonl
**/config.toml
**/.DS_Store
**/sessions/
**/projects/
**/memory/
*.key
*.pem
.env
.env.*
```
- [ ] **Step 4 — run, expect PASS.** `bash tests/run.sh` → green.
- [ ] **Step 5 — commit.** `feat: allowlist gitignore + secret-scan guard`

### Task 2: Repo-root `AGENTS.md` (+ `CLAUDE.md` import) + `README.md` + skeleton

**Files:** Create `AGENTS.md`, `CLAUDE.md`, `README.md`, `claude/.gitkeep`, `codex/.gitkeep`, `gemini/.gitkeep`, `tests/test_repo_agents_docs.sh`.

- [ ] **Step 1 — failing test** (`tests/test_repo_agents_docs.sh`): assert `CLAUDE.md` imports `AGENTS.md`, that `AGENTS.md` documents the secret rule, and the three agent folders exist.
```bash
#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"
grep -qE '^@AGENTS\.md' CLAUDE.md || fail "CLAUDE.md must @import AGENTS.md"
assert_contains AGENTS.md "Never commit secrets"
for d in claude codex gemini; do [ -d "$d" ] || fail "missing dir: $d"; done
pass "repo agent docs"
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — write `CLAUDE.md`:**
```markdown
@AGENTS.md

<!-- Claude Code reads this file; the canonical guide is AGENTS.md, imported above. -->
```
- [ ] **Step 4 — write `AGENTS.md`** (repo guide): sections — *What this is* (SSOT backup), *Layout* (Layer A/B, agent-first), *Golden rules* ("Never commit secrets" + the exclusion list; only authored artifacts), *install.sh* (symlink + Gemini-copy caveat), *How to add a new agent* (create `<agent>/`, add to the `install.sh` map). Create the three folder `.gitkeep`s.
- [ ] **Step 5 — write `README.md`** (human): one-paragraph purpose; *Restore on a new machine:* `git clone … && cd D-STACK && ./install.sh`; link to `AGENTS.md`.
- [ ] **Step 6 — run, expect PASS; commit.** `docs: repo AGENTS.md base + CLAUDE.md import + README + skeleton`

---

## Milestone 2 — SSOT Content Ingest

Copy the authored artifacts into the repo (snapshot into SSOT). After this the repo holds canonical copies; `install.sh` (M3) links them back.

### Task 3: Ingest Claude artifacts (+ make `settings.json` portable)

**Files:** Create `claude/CLAUDE.md`, `claude/settings.json`, `claude/statusline-command.sh`, `claude/hooks/*.sh`, `claude/skills/full-cycle/SKILL.md`, `claude/skills/codex-review/SKILL.md`, `tests/test_claude_artifacts.sh`.

- [ ] **Step 1 — failing test** (`tests/test_claude_artifacts.sh`): assert each artifact exists, executables are present, **`settings.json` contains no literal `/Users/` path** (portability), and no secret slipped in.
```bash
#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"
for f in claude/CLAUDE.md claude/settings.json claude/statusline-command.sh \
         claude/hooks/fullcycle-inject.sh claude/hooks/fullcycle-gate.sh \
         claude/skills/full-cycle/SKILL.md claude/skills/codex-review/SKILL.md; do
  [ -s "$f" ] || fail "missing/empty: $f"
done
grep -q '/Users/' claude/settings.json && fail "settings.json has machine-specific /Users path"
assert_contains claude/settings.json '$HOME'
pass "claude artifacts"
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — copy artifacts in** (`cp` from `~/.claude/...`), preserving the hook/skill structure.
- [ ] **Step 4 — portability fix:** in `claude/settings.json` replace the 3 `/Users/<user>/.claude/...` command paths with `$HOME/.claude/...`. **Verification sub-step (E2E-gated in M3):** confirm Claude expands `$HOME` in hook/statusline `command`; if it does not, fall back to a documented literal + `install.sh` rewrite, and update the test accordingly.
- [ ] **Step 5 — run, expect PASS; commit.** `feat: ingest authored claude config into SSOT`

### Task 4: Ingest Codex artifacts + Gemini placeholder

**Files:** Create `codex/instructions.md`, `codex/rules/default.rules`, `gemini/README.md`, `tests/test_codex_artifacts.sh`.

- [ ] **Step 1 — failing test** (`tests/test_codex_artifacts.sh`): assert codex artifacts exist AND that excluded secrets are absent (no `config.toml`, no `auth.json`, no project-path leakage like `<private-project>`/`<private-project>`).
```bash
#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"
[ -s codex/instructions.md ] || fail "missing codex/instructions.md"
[ -s codex/rules/default.rules ] || fail "missing codex/rules/default.rules"
[ -e codex/config.toml ] && fail "config.toml must NOT be backed up"
[ -e codex/auth.json ]  && fail "auth.json must NOT be backed up"
grep -rqE '<private-project>|<private-project>|<private-project>' codex/ && fail "private project path leaked"
[ -e gemini/README.md ] || fail "gemini placeholder missing"
pass "codex artifacts + gemini placeholder"
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — copy** `~/.codex/instructions.md` and `~/.codex/rules/default.rules` in; write `gemini/README.md` ("Placeholder. To onboard Gemini: add GEMINI.md + configs here and a `gemini` entry in install.sh. Note: Gemini ignores symlinked context files — install.sh copies them.").
- [ ] **Step 4 — run, expect PASS; commit.** `feat: ingest authored codex config + gemini placeholder`

---

## Milestone 3 — install.sh & Verification

### Task 5: `install.sh` — symlink map + per-agent policy + `--dry-run` + backup

**Files:** Create `install.sh`, `tests/test_install_sh.sh`.

**Interfaces — Produces:** `install.sh [--dry-run]`; idempotent; uses a single declared MAP of `repo_relpath|target_abspath|mode(link|copy)`.

- [ ] **Step 1 — failing test** (`tests/test_install_sh.sh`): run install against a sandbox HOME, assert symlinks created & resolve into the repo, a pre-existing file got backed up, and a second run is a no-op.
```bash
#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/lib.sh"
REPO="$(git rev-parse --show-toplevel)"
SBX="$(mktemp -d)"; trap 'rm -rf "$SBX"' EXIT
mkdir -p "$SBX/.claude" "$SBX/.codex"
printf 'OLD' > "$SBX/.claude/CLAUDE.md"             # pre-existing real file
HOME="$SBX" bash "$REPO/install.sh" >/dev/null
[ -L "$SBX/.claude/CLAUDE.md" ] || fail "CLAUDE.md not symlinked"
[ "$(readlink "$SBX/.claude/CLAUDE.md")" = "$REPO/claude/CLAUDE.md" ] || fail "wrong link target"
ls "$SBX/.claude/"CLAUDE.md.bak.* >/dev/null 2>&1 || fail "pre-existing file not backed up"
[ -L "$SBX/.claude/skills/full-cycle" ] || fail "skill dir not symlinked"
HOME="$SBX" bash "$REPO/install.sh" --dry-run | grep -qi 'up to date\|no change\|ok' || true   # idempotent: no error
pass "install.sh links + backup + idempotent"
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement `install.sh`:** resolve `SCRIPT_DIR`; build MAP from the Inventory (link for claude+codex; copy reserved for gemini); for each entry — skip if `~/.gemini`-style parent agent dir absent; `mkdir -p` target parent; if target exists and isn't already the intended link, `mv` to `target.bak.<ts>`; then `ln -sfn` (or `cp -R` for copy mode). `--dry-run` prints planned actions only. Print a summary table.
- [ ] **Step 4 — run, expect PASS.**
- [ ] **Step 5 — commit.** `feat: install.sh symlink engine with backup + dry-run`

### Task 6: E2E verification, restore docs, initial commit

**Files:** Modify `README.md` (restore/uninstall section), `AGENTS.md` (link install.sh). Evidence saved under `docs/dstack-backup/evidence/`.

- [ ] **Step 1 — full sandbox E2E:** in a `mktemp -d` HOME, run `./install.sh`; capture `find $SBX -maxdepth 3 -type l -ls` proving every map entry links into the repo; save to `docs/dstack-backup/evidence/install-e2e.txt`.
- [ ] **Step 2 — `$HOME`-expansion check:** verify a Claude hook command using `$HOME` actually runs (or record the literal-path fallback decision); save evidence.
- [ ] **Step 3 — `bash tests/run.sh`** all green; save output to evidence.
- [ ] **Step 4 — doc the real-home install** as an explicit, user-confirmed step (NOT auto-run): "Review `install.sh --dry-run` first; it backs up existing files to `*.bak`."
- [ ] **Step 5 — initial commit / push decision** (ask user before any push).

---

## Self-Review (run after drafting)

- **Spec coverage:** SSOT+symlink ✓(M3) · public-safe/secret-exclusion ✓(T1,T4) · agent-first folders ✓(T2) · only-authored-artifacts ✓(Inventory,T3,T4) · AGENTS.md base + CLAUDE.md import ✓(T2) · extensible (gemini) ✓(T4,T5).
- **Placeholder scan:** test bodies are concrete; `AGENTS.md`/`README.md` prose written in-task (acceptable — content, not code).
- **Type consistency:** `tests/lib.sh` helper names (`assert_contains`, `fail`, `pass`) used consistently across all test files; install MAP schema `repo|target|mode` referenced once.
