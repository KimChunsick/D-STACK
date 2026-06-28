# Codex adversarial review — T05 (codex-review skill changes)

## Round 1 — GPT verdict: **approve-with-fixes** (found a secret-exfil regression I introduced)
1. `[high][sec]` Piping every untracked file → secrets (`.npmrc`/`.netrc`/service-account JSON) exfiltrated; prose warning ≠ enforcement.
2. `[med][sec]` Fixed `/tmp/fc-review-input.txt`, umask, no cleanup.
3. `[med][tech]` `git ls-files --others` over-collects all untracked repo state, not task-relevant.
4. `[med][tech]` Loop breaks on newlines / leading-dash filenames; needs `-z` / `cat --`.
5. `[med][tech]` Blindly cats binary/huge files → noise/overflow.
6. `[med][tech]` Tests are shallow keyword checks.
7. `[low][UX]` Frontmatter still says "records the verdict into the doc."
8. `[med][UX]` Consensus loop (Step 4) re-feeds `codex-review.md` but Step 1 never included it.
9. `[med][structure]` Safety-critical selection is prose-guarded copy-paste, not enforced.
10. `[med][Why]` G2 partly met but introduced the exfil path.
11. `[low][Why]` "Attack research assumptions" in prompt, but research wasn't in the material.

## Claude responses — all accepted & fixed (the security catch was correct and serious)
- **#1/#9/#10** — assembly is now **fail-closed**: a secret-name `DENY` regex skips
  `auth.json/config.toml/credentials/*.key/*.pem/*.token/.env*/*.sqlite*/*.db/.npmrc/.netrc/…`,
  with skips listed explicitly. Secrets are mechanically excluded, not prose-warned.
- **#2** — `mktemp` + `chmod 600` + `trap rm` instead of a fixed world-readable path.
- **#4** — `git ls-files -z … | while IFS= read -r -d '' f; … cat -- "$f"` (robust paths).
- **#5** — size cap (≤64KB) + binary check (`grep -Iq`), skipped files listed.
- **#3** — mitigated: deny/size/binary gates make over-collection *safe*; remaining noise is
  bounded and transparent (skips listed). The full-cycle process keeps the tree clean.
- **#6** — tests now assert the mechanisms: `DENY`, literal `auth\.json`, `mktemp`, `cat -- `,
  binary/SKIPPED, and that the fixed `/tmp` path is gone.
- **#7** — frontmatter updated to "separate codex-review.md in the task folder."
- **#8** — Step 1 now includes the prior `codex-review.md` so consensus rounds keep the record.
- **#11** — Step 1 now includes the Goal's research artifacts so the reviewer can attack them.

## Evidence
- E2E: ran the fail-closed assembly with a planted `auth.json` → secret value in bundle: **0**
  (excluded). `bash tests/run.sh` → ALL TESTS PASSED.

## Round 2 — reject (denylist ≠ fail-closed)
Codex pushed deeper: a **denylist is default-allow** (novel secret names, symlink targets,
small text slip through), and `git diff HEAD` itself bypassed all gates. Correct — a real
exfil path remained.

## Round 3 — approve-with-fixes (allowlist design accepted; one gap)
Redesigned to a **fail-closed allowlist**: a real executable helper `assemble-review.sh`
(only named files are sent), per-file **scoped** diff, symlink/secret/size/binary gates, and a
**behavioral fixture test** (`test_codex_review_assembler.sh`) proving a planted secret/symlink/
binary/oversize are skipped and an *unnamed* secret never appears. Codex marked #1/NEW/#3/#6/#9
RESOLVED. One gap (#10): `task.md`/`codex-review.md` were raw-`cat`'d, bypassing the gates.

## Round 4 — **approve**
Routed `task.md` + `codex-review.md` through `emit_file` too (proven: a symlinked `task.md`→
secret is skipped). "The assembler is acceptably fail-closed for this public-repo workflow…
content-based secret leakage documented as an accepted residual of the repo's name-based model."

## Consensus
- R1 approve-with-fixes → R2 reject (caught denylist flaw) → R3 approve-with-fixes (allowlist
  redesign) → R4 **approve**.
- **Consensus: agreed.** Accepted residual: gates are name/type-based, not content-based — the
  allowlist is the control (matches the repo's name-based secret model).
