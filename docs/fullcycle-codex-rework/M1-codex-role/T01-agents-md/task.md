# T01 — codex/AGENTS.md (dual-role identity) + wiring

## Intent / Why
Codex's global instruction file `~/.codex/AGENTS.md` is currently **empty and untracked**.
For Codex to act as the dedicated adversarial *researcher + reviewer*, that identity must
live in the SSOT and be symlinked into the live Codex dir. This task creates the role file
and wires it into the allowlist / installer / test guard so it is backed up safely.

## What was done (what / why)
- Created `codex/AGENTS.md` defining Codex's dual role (adversarial researcher + reviewer),
  stack-neutral, with research rules (both-sides evidence, untrusted-web-data) and review
  rules (5 axes + attack-own-research + verdict line). This is the identity that makes
  Codex the "second model" instead of Claude grading its own work.
- Allowlisted it (`!/codex/AGENTS.md`) and mapped it in `install.sh` so it is backed up
  and symlinked to the (currently empty) `~/.codex/AGENTS.md`.
- TDD via grep-guard in `tests/test_codex_artifacts.sh`; strengthened the guard to assert
  the file is *actually git-trackable* (`git check-ignore`), not merely that an allow line
  exists — encoding the real intent (it must be safely backupable in this public repo).

## How (plan, before coding)
1. **Red**: extend `tests/test_codex_artifacts.sh` to assert `codex/AGENTS.md` exists,
   is non-empty, names both roles (researcher + reviewer), is stack-neutral; assert
   `.gitignore` allowlists it and `install.sh` maps it. Run → must fail.
2. **Green**: write `codex/AGENTS.md`; add `!/codex/AGENTS.md` to `.gitignore`; add the
   `codex/AGENTS.md|.codex/AGENTS.md|link` row to `install.sh`. Run → pass.
3. **Refactor**: tidy wording; keep public-safe (no `/Users/` paths, no private names).

## Files changed (where / why)
- `codex/AGENTS.md` (new) — Codex dual-role identity (the deliverable).
- `.gitignore` — added `!/codex/AGENTS.md` allow line (deny-all repo needs explicit opt-in).
- `install.sh` — added `codex/AGENTS.md|.codex/AGENTS.md|link` map row (symlink it live).
- `tests/test_codex_artifacts.sh` — TDD guard: presence + role markers + allowlist + map +
  actual-trackability check.

## E2E verification
- `tests/run.sh` → ALL TESTS PASSED (after review fixes).
- `./install.sh --dry-run` → `+ linked: .codex/AGENTS.md → codex/AGENTS.md` (backs up the
  pre-existing empty `~/.codex/AGENTS.md` first).
- `git status --porcelain codex/AGENTS.md` → `?? codex/AGENTS.md` (trackable, not ignored).
- **Behavioral load-path proof** (answers Codex #2): a sentinel instruction written into
  `~/.codex/AGENTS.md` made `codex exec` reply `[ROLE-LOADED-8821] Hello.` → Codex genuinely
  loads `$CODEX_HOME/AGENTS.md` as global instructions, so the identity will take effect.
- Codex review: see [codex-review.md](codex-review.md) — Round 2 approve-with-fixes,
  consensus resolved for T01 (open #3 owned by T02).

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus
- [x] E2E capture verified
