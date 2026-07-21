# 01-secret-guard-only

## Intent / Why
The meta test suite (1,040 lines across 10 files) taxes every config change: content-pin
tests must be updated whenever a skill/hook/settings file changes, which slows exactly the
kind of work this Goal is speeding up. Per user decision (interview Q3), the suite is
deleted except the one control with asymmetric downside on a public repo: the
secret-trackability guard. It becomes a single standalone script with no runner/lib
dependency, run manually before commit.

## What was done (what / why)
- **TDD Red:** `bash tests/secret-guard.sh` → exit 127 (target script absent).
- **Green:** created standalone `tests/secret-guard.sh` — the former
  `test_gitignore_secret_guard.sh` with `lib.sh`'s `fail`/`pass` inlined (its only
  dependency) and a header stating it is the one kept meta check, run manually before
  commit. Probe battery, closed-set negation pin, and tracked-tree secret-pattern scan
  are unchanged. Behavioral verification beyond "it passes": appended a rogue
  `!/claude/agents/rogue.md` negation to `.gitignore` → guard failed with the pinned-set
  diff (exit 1); restored → guard passed. This encodes the Why: the guard exists to
  catch allowlist drift, not to be green.
- **Deleted** the nine other suite files via `git rm` (run.sh, lib.sh, and all
  `test_*.sh` — 1,000 lines of content-pin tests whose upkeep taxed every config edit).
- **Reference sweep (class-wide, not instance-wise):** grepped the whole tracked tree
  for `run.sh` / `test_` / `lib.sh` and updated every hit: `AGENTS.md` (golden rule 1,
  add-agent step 4, `## Tests` section), `README.md` (layout tree + Safety), 
  `gemini/README.md` (onboard step 4), `claude/skills/codex-review/SKILL.md` (dropped
  the stale "tested in tests/test_codex_review_assembler.sh" parenthetical). Post-sweep
  grep is clean.
- **Refactor:** none needed — one script plus reference edits.
- **Review R1 rewrite (all four findings agreed, fixed class-wide):** ignore probes no
  longer create files (`git check-ignore` is name-based); physical probes exist only
  for the `claude/agents/` addable-check and refuse pre-existing paths/symlinks before
  any redirection, cleaning up exactly what they created (dirs included); section 0
  pins the ignore-rule *sources* — staged/worktree `.gitignore` divergence, nested
  `.gitignore` (worktree or index), and `.git/info/exclude` content are rejected, and
  every `check-ignore`/`ls-files -o` runs with `core.excludesFile=/dev/null` (adjacent
  same-class defect found in the sweep: a machine-local global gitignore could
  otherwise fake a pass that does not transfer); tracked-tree regex covers DB-journal
  families (`.sqlite-wal/-shm/-journal`, `.db-wal`, …); probe battery gained
  `claude/y.sqlite-shm` + `claude/data.db-wal`; `.gitignore` deny list gained
  `**/*.db-*`. Old-suite residue dirs (`claude/skills/novel_secret_dir`,
  `claude/agents/nested`) removed.
- **Review R2 fixes (findings 2–4 agreed; 1 disposed by user decision + doc scoping):**
  index-blob vs worktree byte compare replaces `git diff --quiet` (immune to
  assume-unchanged/skip-worktree); per-component symlink walk + `set -C` noclobber
  creation for the physical agents probes; family coverage completed in all three
  places (`.gitignore` deny: `**/*.sqlite3-*`, `**/*.db3-*`, `**/*deploy_key*`;
  probes: wholesale-subtree `api_token`/`deploy_key_prod`/`cache.sqlite3-wal`/
  `cache.db3-wal`; tracked regex: `deploy_key`, `_token$`); guard header, README
  §Safety, and AGENTS.md golden rule 1 now state the names/trackability-only scope
  (content scanning declined by user decision — interview Q3; GitHub secret scanning
  is the content backstop).
- `.gitignore` negation set unchanged (deny-line additions only), so the guard's pinned
  set still matches; `!/tests/` still allowlists the dir.

## Files changed (where / why)
- `tests/secret-guard.sh` — new standalone guard (inlined helpers; kept per interview Q3)
- `tests/run.sh`, `tests/lib.sh`, `tests/test_*.sh` (9 files) — deleted (suite retired)
- `AGENTS.md` — three test-obligation passages now point at the single guard
- `README.md` — layout comment + Safety section updated
- `gemini/README.md` — onboarding step no longer demands a per-agent test file
- `claude/skills/codex-review/SKILL.md` — removed reference to a deleted test file

## E2E verification
Verification is fully reproducible from this record alone (script is parameterized: pass the repo root, or run from inside a clone).

**Clean run (real repo):** `bash tests/secret-guard.sh` → `✓ PASS: secret guard`, exit 0; zero probe residue.

**Sabotage battery — 53 scenarios/assertions in disposable clones under owned temp roots, all behaving (exit 0 overall).**
Runner script (exact content):

```bash
#!/usr/bin/env bash
# Adversarial verification battery for the rewritten secret guard (M1-T01).
# Usage: bash m1-guard-sabotage.sh [repo-root]   (defaults to the enclosing repo)
# Runs every scenario in DISPOSABLE clones under owned temp roots; the real repo is
# never touched, and cleanup removes exactly the roots this run created (EXIT trap,
# no dirname-derived deletion).
set -euo pipefail
SRC="${1:-$(git rev-parse --show-toplevel 2>/dev/null || true)}"
[ -n "$SRC" ] && [ -d "$SRC/.git" ] || { echo "usage: $0 <repo-root>" >&2; exit 2; }
ok()   { echo "  ✓ $*"; }
bad()  { echo "  ✗ EXPECTATION BROKEN: $*" >&2; exit 1; }

ROOTS=()
cleanup() { local r; for r in "${ROOTS[@]:-}"; do [ -n "$r" ] && rm -rf -- "$r"; done; }
trap cleanup EXIT

fresh() {  # new disposable clone (owned root) with the uncommitted M1 state staged
  local root; root="$(mktemp -d)"; ROOTS+=("$root")
  C="$root/repo"
  git clone -q "$SRC" "$C"
  cd "$C"
  git config user.email t@t; git config user.name t
  cp "$SRC/tests/secret-guard.sh" tests/secret-guard.sh
  cp "$SRC/.gitignore" .gitignore
  git add tests/secret-guard.sh .gitignore
}
guard() { bash tests/secret-guard.sh; }

# Baseline: clone passes
fresh
guard >/dev/null || bad "baseline clone should pass"
ok "baseline clone passes"

# A (R1-2/R4-3): TRUE index/worktree divergence — save safe bytes, stage rogue,
# restore safe bytes, prove divergence, and require the section-0 failure message
fresh
cp .gitignore ../safe-gitignore
printf '!/claude/agents/rogue.md\n' >> .gitignore
git add .gitignore
cp ../safe-gitignore .gitignore
git show :.gitignore | cmp -s - .gitignore && bad "divergence precondition not established"
if out="$(guard 2>&1)"; then bad "staged rogue negation passed"; fi
case "$out" in *"differs between index and working tree"*) : ;; *) bad "divergence not caught by section 0: $out" ;; esac
ok "true staged/worktree divergence rejected in section 0"

# B: nested .gitignore in worktree → must fail
fresh
printf '!x.sqlite-wal\n' > claude/skills/full-cycle/.gitignore
if guard >/dev/null 2>&1; then bad "nested worktree .gitignore passed"; fi
ok "nested worktree .gitignore rejected"

# C: nested .gitignore staged then deleted from worktree → must fail
fresh
printf '!x\n' > claude/skills/full-cycle/.gitignore
git add -f claude/skills/full-cycle/.gitignore
rm claude/skills/full-cycle/.gitignore
if guard >/dev/null 2>&1; then bad "index-only nested .gitignore passed"; fi
ok "index-only nested .gitignore rejected"

# D: force-added SQLite/DB journal variants → must fail
for name in codex/cache.sqlite-wal codex/cache.db-wal codex/cache.sqlite-journal; do
  fresh
  : > "$name"; git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# E: pre-existing file at a probe path → guard fails AND file is intact
fresh
printf 'SENTINEL' > claude/agents/auth.json
if guard >/dev/null 2>&1; then bad "pre-existing probe path passed"; fi
[ "$(cat claude/agents/auth.json)" = "SENTINEL" ] || bad "pre-existing probe file was modified"
ok "pre-existing probe file: refused, contents intact"

# E2: symlink at a probe path → refuse; external target untouched
fresh
TGT="$(mktemp)"; ROOTS+=("$TGT"); printf 'SECRET' > "$TGT"
ln -s "$TGT" claude/agents/f.md
if guard >/dev/null 2>&1; then bad "symlink probe path passed"; fi
[ "$(cat "$TGT")" = "SECRET" ] || bad "symlink target was modified"
ok "symlink probe path: refused, external target intact"

# F: local info/exclude rules → must fail
fresh
printf 'private-note.txt\n' >> .git/info/exclude
if guard >/dev/null 2>&1; then bad "info/exclude rule passed"; fi
ok "info/exclude local rule rejected"

# G: global excludes must NOT fake a pass — drop a LOAD-BEARING structural deny,
# provide it globally instead; guard must still fail
fresh
grep -v '^/claude/hooks/\*$' .gitignore > .g2 && mv .g2 .gitignore
git add .gitignore
G="$(mktemp)"; ROOTS+=("$G"); printf 'random_unknownfile\ndeploy_key_prod\n' > "$G"
git config core.excludesFile "$G"
if guard >/dev/null 2>&1; then bad "globally-masked missing deny rule passed"; fi
ok "global excludes cannot mask a missing repo rule"

# G2: global excludes hiding an addable agent probe from ls-files must not fake a pass
fresh
G="$(mktemp)"; ROOTS+=("$G"); printf 'unknown-agent.md\n' > "$G"
git config core.excludesFile "$G"
grep -v '^/claude/agents/\*$' .gitignore > .g2 && mv .g2 .gitignore
git add .gitignore
if guard >/dev/null 2>&1; then bad "globally-hidden addable agent probe passed"; fi
ok "global excludes cannot hide an addable agent probe"

# H: rogue negation in BOTH worktree and index → pinned-set diff fails
fresh
printf '!/claude/agents/rogue.md\n' >> .gitignore
git add .gitignore
if guard >/dev/null 2>&1; then bad "pinned-set drift passed"; fi
ok "pinned negation set drift rejected"

# I: symlinked ancestor dir at a probe path → refuse BEFORE any write
fresh
EXT="$(mktemp -d)"; ROOTS+=("$EXT")
ln -s "$EXT" claude/agents/nested
if guard >/dev/null 2>&1; then bad "symlinked ancestor dir passed"; fi
[ -z "$(ls -A "$EXT")" ] || bad "external dir was written through symlinked ancestor"
ok "symlinked ancestor dir: refused, external dir untouched"

# J: force-added family names outside hard-coded probe paths → each fails
for name in claude/skills/full-cycle/api_token claude/skills/full-cycle/deploy_key_prod \
            claude/skills/full-cycle/cache.sqlite3-wal claude/skills/full-cycle/cache.db3-wal; do
  fresh
  : > "$name"; git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# L (R4-3): TRUE divergence + assume-unchanged bit — byte cmp must still fail
fresh
cp .gitignore ../safe-gitignore
printf '!/claude/agents/rogue.md\n' >> .gitignore
git add .gitignore
cp ../safe-gitignore .gitignore
git update-index --assume-unchanged .gitignore
git show :.gitignore | cmp -s - .gitignore && bad "divergence precondition not established (L)"
if out="$(guard 2>&1)"; then bad "assume-unchanged staged rogue passed"; fi
case "$out" in *"differs between index and working tree"*) : ;; *) bad "assume-unchanged divergence not caught by section 0: $out" ;; esac
ok "assume-unchanged TRUE divergence rejected (byte-level cmp)"

# M: tracked secret under a newline-containing directory
fresh
mkdir "$(printf 'odd\nname')"
: > "$(printf 'odd\nname')/cache.token"
git add -f "$(printf 'odd\nname')/cache.token"
if guard >/dev/null 2>&1; then bad "newline-path tracked secret passed"; fi
ok "newline-path tracked secret rejected"

# N: upper-case family variant, case-sensitive ignore behavior forced
fresh
git config core.ignoreCase false
: > claude/skills/full-cycle/CACHE.SQLITE3-WAL
if guard >/dev/null 2>&1; then bad "uppercase addable variant passed"; fi
ok "uppercase addable family variant rejected"

# O: nested .gitignore under a newline dir, staged then worktree-deleted
fresh
mkdir "$(printf 'odd\n2')"
printf '!x\n' > "$(printf 'odd\n2')/.gitignore"
git add -f "$(printf 'odd\n2')/.gitignore"
rm "$(printf 'odd\n2')/.gitignore"
if guard >/dev/null 2>&1; then bad "newline-path nested .gitignore passed"; fi
ok "newline-path index nested .gitignore rejected"

# P (R4-1): suffixed token family, force-added → must fail
fresh
: > claude/skills/full-cycle/api_token.json
git add -f claude/skills/full-cycle/api_token.json
if guard >/dev/null 2>&1; then bad "tracked api_token.json passed"; fi
ok "tracked api_token.json rejected"

# Q (R4-1): documented runtime dirs (sessions/memory/projects), force-added → must fail
for name in claude/skills/full-cycle/sessions/state.json \
            claude/skills/full-cycle/memory/notes.md; do
  fresh
  mkdir -p "$(dirname "$name")"; : > "$name"
  git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# S (R5-1): backup / compound-suffix family variants, force-added → each fails
for name in claude/skills/full-cycle/private.pem.bak claude/skills/full-cycle/secrets.token.old \
            claude/skills/full-cycle/api_token.json.bak claude/skills/full-cycle/cache.db.bak; do
  fresh
  : > "$name"; git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# V (R6-1): protected DIRECTORY components and punctuation-bearing backup suffixes,
# force-added → each fails (component-level matching)
for name in claude/skills/full-cycle/api_token/payload.txt \
            claude/skills/full-cycle/private.pem.backup-2026 \
            claude/skills/full-cycle/cache.db-wal.backup-2026 \
            claude/skills/full-cycle/PASSWORD-list.txt; do
  fresh
  mkdir -p "$(dirname "$name")"; : > "$name"
  git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# Y (R8-1): backup chains of exact protected basenames, force-added → each fails
for name in claude/skills/full-cycle/auth.json.bak claude/skills/full-cycle/config.toml.old \
            claude/skills/full-cycle/.netrc.backup claude/skills/full-cycle/id_rsa.pub \
            claude/skills/full-cycle/history.jsonl.gz; do
  fresh
  git check-ignore -q "$name" || bad "$name not ignored by policy (precondition)"
  : > "$name"; git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# Z (R8-2): raw-byte components — newline INSIDE a protected filename, invalid
# UTF-8 after a protected substring, and an empty wildcard tail — each force-added
# must fail (byte-level whole-component matching)
fresh
nlname="$(printf 'private.pem.\nbackup')"
: > "claude/skills/full-cycle/$nlname"
git add -f "claude/skills/full-cycle/$nlname"
if guard >/dev/null 2>&1; then bad "newline-inside-component pem backup passed"; fi
ok "newline-inside-component pem name rejected"
# APFS refuses invalid-UTF-8 filenames, so the on-disk fixture is impossible here
# (the threat is real on case-sensitive/Linux clones); verify the MATCHER itself
# flags the raw-byte component under the guard's LC_ALL=C + nocasematch regime.
fresh
compre="$(sed -n "s/^COMP_RE='\(.*\)'$/\1/p" tests/secret-guard.sh)"
[ -n "$compre" ] || bad "could not extract COMP_RE from the guard"
if LC_ALL=C bash -c 'shopt -s nocasematch; [[ $2 =~ $1 ]]' _ "$compre" "$(printf 'password\377')"; then
  ok "matcher flags invalid-UTF8 password component (byte-level =~)"
else
  bad "matcher missed invalid-UTF8 password component"
fi
fresh
: > "claude/skills/full-cycle/private.pem."
git add -f "claude/skills/full-cycle/private.pem."
if guard >/dev/null 2>&1; then bad "empty-wildcard-tail pem name passed"; fi
ok "empty-wildcard-tail pem name rejected"

# W (R7-1): glob-equivalent journal suffixes (extra hyphens/punctuation), force-added
for name in claude/skills/full-cycle/cache.db-wal-backup-2026 \
            claude/skills/full-cycle/cache.sqlite-shm_copy \
            claude/skills/full-cycle/cache.db-; do
  fresh
  git check-ignore -q "$name" || bad "$name not ignored by policy (precondition)"
  : > "$name"; git add -f "$name"
  if guard >/dev/null 2>&1; then bad "tracked $name passed"; fi
  ok "tracked $name rejected"
done

# X (R7-2): staged SYMLINK entry reusing the safe blob — byte compare alone would
# pass while the commit records an ignore-dead symlink; mode check must fail
for pf in .gitignore tests/secret-guard.sh; do
  fresh
  blob="$(git rev-parse ":$pf")"
  git update-index --cacheinfo "120000,$blob,$pf"
  if out="$(guard 2>&1)"; then bad "staged symlink mode for $pf passed"; fi
  case "$out" in *"not a single stage-0 regular file"*) : ;; *) bad "symlink mode for $pf not caught by entry check: $out" ;; esac
  ok "staged symlink index entry for $pf rejected"
done

# T (R5-2): the guard itself must match the index — staged deletion or weakened
# staged copy cannot be masked by a safe worktree copy
fresh
git rm --cached -q tests/secret-guard.sh
if out="$(guard 2>&1)"; then bad "staged guard deletion passed"; fi
case "$out" in *"stage the guard itself"*|*"not a single stage-0 regular file"*) : ;; *) bad "guard-deletion not caught by self-check: $out" ;; esac
ok "staged guard deletion rejected by self-check"
fresh
cp tests/secret-guard.sh ../safe-guard
printf '\n# weakened\n' >> tests/secret-guard.sh
git add tests/secret-guard.sh
cp ../safe-guard tests/secret-guard.sh
if out="$(guard 2>&1)"; then bad "weakened staged guard passed"; fi
case "$out" in *"stage the guard itself"*) : ;; *) bad "weakened-staged not caught by self-check: $out" ;; esac
ok "weakened staged guard rejected by self-check"

# U (R5-3, corrected premise): this repo's PRE-EXISTING deny policy bans
# sessions/projects/memory dir names repo-wide (`**/projects/` etc.), so no benign
# file can sit there without -f. The guard mirrors that policy: a force-added
# docs/projects file must FAIL, and the untracked dirs must be ignored (not addable).
fresh
mkdir -p docs/projects
printf 'roadmap\n' > docs/projects/review.md
git check-ignore -q docs/projects/review.md || bad "docs/projects not denied by pre-existing policy"
git add -f docs/projects/review.md
if guard >/dev/null 2>&1; then bad "force-added docs/projects file passed"; fi
ok "runtime-dir family matches the repo-wide deny policy (docs/projects force-add rejected)"

# R (R4-2): mixed-case .GITIGNORE — worktree and staged — must be rejected as an
# ignore source regardless of filesystem case behavior
fresh
printf '!rogue.md\n' > claude/agents/.GITIGNORE
if guard >/dev/null 2>&1; then bad "worktree .GITIGNORE passed"; fi
ok "worktree mixed-case .GITIGNORE rejected"
fresh
printf '!rogue.md\n' > claude/agents/.GITIGNORE
git add -f claude/agents/.GITIGNORE
rm claude/agents/.GITIGNORE
if guard >/dev/null 2>&1; then bad "index-only .GITIGNORE passed"; fi
ok "index-only mixed-case .GITIGNORE rejected"

# AA (R9-1): removing a hard-deny rule that has NO probe and NO file on disk must
# still fail via the whole-content SHA pin. (`**/*password*` removal is already
# caught by the name-based probe battery — probes need no files — so use an
# unprobed rule to prove the pin.)
fresh
grep -v '^\*\*/\*\.pfx\.\*$' .gitignore > .g2 && mv .g2 .gitignore
git add .gitignore
if out="$(guard 2>&1)"; then bad "deny-rule removal (no probe, no file) passed"; fi
case "$out" in *"drifted from the pinned hash"*) : ;; *) bad "deny removal not caught by the SHA pin: $out" ;; esac
ok "unprobed hard-deny removal rejected (SHA pin)"
# and the probed variant is caught even earlier, files or not:
fresh
grep -v '^\*\*/\*password\*$' .gitignore > .g2 && mv .g2 .gitignore
git add .gitignore
if guard >/dev/null 2>&1; then bad "password deny removal passed"; fi
ok "probed hard-deny removal rejected (battery or pin)"

echo "ALL SABOTAGE SCENARIOS BEHAVED"
```

Observed transcript:

```text
  ✓ baseline clone passes
  ✓ true staged/worktree divergence rejected in section 0
  ✓ nested worktree .gitignore rejected
  ✓ index-only nested .gitignore rejected
  ✓ tracked codex/cache.sqlite-wal rejected
  ✓ tracked codex/cache.db-wal rejected
  ✓ tracked codex/cache.sqlite-journal rejected
  ✓ pre-existing probe file: refused, contents intact
  ✓ symlink probe path: refused, external target intact
  ✓ info/exclude local rule rejected
  ✓ global excludes cannot mask a missing repo rule
  ✓ global excludes cannot hide an addable agent probe
  ✓ pinned negation set drift rejected
  ✓ symlinked ancestor dir: refused, external dir untouched
  ✓ tracked claude/skills/full-cycle/api_token rejected
  ✓ tracked claude/skills/full-cycle/deploy_key_prod rejected
  ✓ tracked claude/skills/full-cycle/cache.sqlite3-wal rejected
  ✓ tracked claude/skills/full-cycle/cache.db3-wal rejected
  ✓ assume-unchanged TRUE divergence rejected (byte-level cmp)
  ✓ newline-path tracked secret rejected
  ✓ uppercase addable family variant rejected
  ✓ newline-path index nested .gitignore rejected
  ✓ tracked api_token.json rejected
  ✓ tracked claude/skills/full-cycle/sessions/state.json rejected
  ✓ tracked claude/skills/full-cycle/memory/notes.md rejected
  ✓ tracked claude/skills/full-cycle/private.pem.bak rejected
  ✓ tracked claude/skills/full-cycle/secrets.token.old rejected
  ✓ tracked claude/skills/full-cycle/api_token.json.bak rejected
  ✓ tracked claude/skills/full-cycle/cache.db.bak rejected
  ✓ tracked claude/skills/full-cycle/api_token/payload.txt rejected
  ✓ tracked claude/skills/full-cycle/private.pem.backup-2026 rejected
  ✓ tracked claude/skills/full-cycle/cache.db-wal.backup-2026 rejected
  ✓ tracked claude/skills/full-cycle/PASSWORD-list.txt rejected
  ✓ tracked claude/skills/full-cycle/auth.json.bak rejected
  ✓ tracked claude/skills/full-cycle/config.toml.old rejected
  ✓ tracked claude/skills/full-cycle/.netrc.backup rejected
  ✓ tracked claude/skills/full-cycle/id_rsa.pub rejected
  ✓ tracked claude/skills/full-cycle/history.jsonl.gz rejected
  ✓ newline-inside-component pem name rejected
  ✓ matcher flags invalid-UTF8 password component (byte-level =~)
  ✓ empty-wildcard-tail pem name rejected
  ✓ tracked claude/skills/full-cycle/cache.db-wal-backup-2026 rejected
  ✓ tracked claude/skills/full-cycle/cache.sqlite-shm_copy rejected
  ✓ tracked claude/skills/full-cycle/cache.db- rejected
  ✓ staged symlink index entry for .gitignore rejected
  ✓ staged symlink index entry for tests/secret-guard.sh rejected
  ✓ staged guard deletion rejected by self-check
  ✓ weakened staged guard rejected by self-check
  ✓ runtime-dir family matches the repo-wide deny policy (docs/projects force-add rejected)
  ✓ worktree mixed-case .GITIGNORE rejected
  ✓ index-only mixed-case .GITIGNORE rejected
  ✓ unprobed hard-deny removal rejected (SHA pin)
  ✓ probed hard-deny removal rejected (battery or pin)
ALL SABOTAGE SCENARIOS BEHAVED
```
## Non-blocking follow-ups (recorded at review close, round 010)
- info/exclude comment parsing: only first-byte-`#` lines are comments (git
  semantics); whitespace-prefixed lines are active patterns and must be rejected.
- Diagnostic ordering: negation-set pin before the whole-content SHA pin so
  allowlist edits get the readable diff first.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
