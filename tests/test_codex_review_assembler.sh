#!/usr/bin/env bash
# Behavioral test (not keyword grep): the codex-review assembler must be fail-closed.
# Plant a secret-named file, an UNNAMED novel secret, a symlink, a binary, a >64KB file, a
# normal new file, and a tracked change; run the assembler; assert secrets never appear and
# only safe, allowlisted content is emitted.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
REPO="$(git rev-parse --show-toplevel)"
ASM="$REPO/claude/skills/codex-review/assemble-review.sh"
[ -f "$ASM" ] || fail "assembler missing: $ASM"

SBX="$(mktemp -d)"; trap 'rm -rf "$SBX"' EXIT
cd "$SBX"
git init -q; git config user.email t@t.t; git config user.name t
mkdir -p task; printf '# task\nTASK_SNAPSHOT_30\n' > task/task.md
printf 'LEGACY_SINGLETON_HISTORY_33\n' > task/codex-review.md
printf 'PRIOR_ROUND_ONE_31\n- Consensus: disagreed\n' > task/codex-review-001.md
printf 'PRIOR_ROUND_TWO_32\n- Consensus: resolved\n' > task/codex-review-002.md

printf 'SECRET_VALUE=leak-me-9931'    > auth.json            # named, but secret-deny must skip it
printf 'NOVEL_SECRET=novel-7777'      > my-prod-creds.txt    # NOT in the allowlist → must be absent
printf 'normal new content OK-42'     > new-normal.txt       # allowlisted text → included
printf 'tracked baseline'             > tracked.txt; git add tracked.txt; git commit -qm base
printf 'tracked CHANGED-55'           > tracked.txt          # tracked change → scoped diff
printf '\x00\x01\x02BINARY'           > bin.dat              # binary → skip
head -c 70000 </dev/zero | tr '\0' a  > big.txt              # >64KB → skip
ln -s auth.json link-to-secret                               # symlink → skip (no target follow)

# Allowlist deliberately INCLUDES the dangerous files to prove the gates skip them;
# my-prod-creds.txt is deliberately NOT named to prove allowlist-only collection.
OUT="$(bash "$ASM" task auth.json new-normal.txt tracked.txt bin.dat big.txt link-to-secret)"

# Secrets must NEVER appear.
if printf '%s' "$OUT" | grep -q 'leak-me-9931'; then fail "named secret (auth.json) content leaked"; fi
if printf '%s' "$OUT" | grep -q 'novel-7777';   then fail "unnamed novel secret leaked (allowlist breached)"; fi
# Safe content must appear.
printf '%s' "$OUT" | grep -q 'OK-42'      || fail "allowlisted normal new file not included"
printf '%s' "$OUT" | grep -q 'CHANGED-55' || fail "tracked change (scoped diff) not included"
# Every prior round remains available to the reviewer, in ascending order, so a later review can
# verify old fixes and decisions without losing safety context. The files remain separate on disk.
printf '%s' "$OUT" | grep -q 'TASK_SNAPSHOT_30' || fail "task document snapshot missing"
printf '%s' "$OUT" | grep -q 'LEGACY_SINGLETON_HISTORY_33' || fail "legacy migration history missing"
printf '%s' "$OUT" | grep -q 'PRIOR_ROUND_ONE_31' || fail "first prior numbered review round missing"
printf '%s' "$OUT" | grep -q 'PRIOR_ROUND_TWO_32' || fail "second prior numbered review round missing"
p1="$(printf '%s\n' "$OUT" | grep -n 'PRIOR_ROUND_ONE_31' | cut -d: -f1)"
p2="$(printf '%s\n' "$OUT" | grep -n 'PRIOR_ROUND_TWO_32' | cut -d: -f1)"
[ "$p1" -lt "$p2" ] || fail "numbered review rounds were not assembled in ascending order"
# Gates must be applied and listed.
printf '%s' "$OUT" | grep -q 'auth.json (SKIPPED: secret-deny)'   || fail "secret not deny-skipped"
printf '%s' "$OUT" | grep -q 'link-to-secret (SKIPPED: symlink)'  || fail "symlink not skipped"
printf '%s' "$OUT" | grep -q 'bin.dat (SKIPPED: binary)'          || fail "binary not skipped"
printf '%s' "$OUT" | grep -q 'big.txt (SKIPPED: >64KB)'           || fail "oversize not skipped"

# ---- Regression: a DELETED tracked file must be emitted as a deletion diff, not dropped. ----
printf 'REMOVED_LOGIC_88\n' > gone.txt; git add gone.txt; git commit -qm add-gone
rm gone.txt                                              # working-tree deletion (still in index/HEAD)
printf 'STAGED_REMOVED_99\n' > gone2.txt; git add gone2.txt; git commit -qm add-gone2
git rm -q gone2.txt                                      # staged deletion (git rm → out of index)
OUTD="$(bash "$ASM" task gone.txt gone2.txt)"
printf '%s' "$OUTD" | grep -q 'gone.txt (tracked, deleted'  || fail "unstaged-deleted tracked file dropped (not a deletion diff)"
printf '%s' "$OUTD" | grep -q 'REMOVED_LOGIC_88'            || fail "unstaged deletion diff body missing (removal not reviewable)"
printf '%s' "$OUTD" | grep -q 'gone2.txt (tracked, deleted' || fail "staged-deleted (git rm) tracked file dropped"
printf '%s' "$OUTD" | grep -q 'STAGED_REMOVED_99'          || fail "staged deletion diff body missing"
if printf '%s' "$OUTD" | grep -qE '(gone|gone2)\.txt \(SKIPPED: not a regular file\)'; then fail "deleted tracked file mis-skipped as 'not a regular file'"; fi

# ---- Regression: the size cap is on the DIFF, not the file — a tiny change to a >64KB tracked
#      file stays fully reviewable (the old `wc -c < file` skipped it entirely). ----
seq 1 30000 > huge.txt; git add huge.txt; git commit -qm add-huge   # ~160KB across many lines
[ "$(wc -c < huge.txt)" -gt 65536 ] || fail "test setup: huge.txt is not >64KB"
printf 'TINY_CHANGE_77\n' >> huge.txt                    # 1-line change → tiny diff on a >64KB file
OUTH="$(bash "$ASM" task huge.txt)"
printf '%s' "$OUTH" | grep -q 'TINY_CHANGE_77' || fail "tiny change to >64KB tracked file skipped (cap measured file, not diff)"
if printf '%s' "$OUTH" | grep -q 'huge.txt (SKIPPED: >64KB)'; then fail "big tracked file wrongly size-skipped on file size, not diff size"; fi

# ---- Regression: a newline in the filename is rejected before the line-oriented DENY grep, so a
#      split secret name (id_<newline>rsa) cannot smuggle its deletion diff out. ----
nlf=$'id_\nrsa'
printf 'PRIVATEKEY=SPLITLEAK_66\n' > "$nlf"; git add -- "$nlf"; git commit -qm add-nl; rm -- "$nlf"
OUTN="$(bash "$ASM" task "$nlf")"
if printf '%s' "$OUTN" | grep -q 'SPLITLEAK_66'; then fail "newline-split secret filename leaked its deletion diff"; fi
printf '%s' "$OUTN" | grep -q 'newline/control char in filename' || fail "newline in filename not rejected"

# ---- Regression: a staged deletion (git rm) invoked from a SUBDIR with a cwd-relative path is
#      still emitted as a deletion diff — the diff-based tracked test is cwd-relative, unlike the
#      repo-root `git cat-file HEAD:<path>` it replaced. ----
mkdir -p sub; printf 'SUBDIR_REMOVED_44\n' > sub/insub.txt; git add sub/insub.txt; git commit -qm add-sub
mkdir -p sub/task; printf '# t\n' > sub/task/task.md
OUTS="$(cd sub && git rm -q insub.txt && bash "$ASM" task insub.txt)"
printf '%s' "$OUTS" | grep -q 'insub.txt (tracked, deleted' || fail "subdir staged deletion dropped (cwd-relative detection broken)"
printf '%s' "$OUTS" | grep -q 'SUBDIR_REMOVED_44'           || fail "subdir deletion diff body missing"

# ---- Regression: a git pathspec/glob argument ('*', 'secret.*', ':/') must NOT expand inside
#      git to leak secret-named tracked files' diffs — ':(literal)' forces literal-path matching
#      (the literal-name DENY alone doesn't catch a bare '*'). ----
printf 'GLOB_SECRET_KEYDATA_77\n' > secret.key; git add secret.key; git commit -qm add-sk
printf 'GLOB_SECRET_KEYDATA_77\nMODIFIED\n' > secret.key           # tracked change to a secret-named file
for g in '*' 'secret.*' ':/'; do
  OUTG="$(bash "$ASM" task "$g")"
  if printf '%s' "$OUTG" | grep -q 'GLOB_SECRET_KEYDATA_77'; then fail "glob arg '$g' expanded inside git and leaked a secret-named tracked file's diff"; fi
done
# The secret-named file, passed literally, is still denied (control).
printf '%s' "$(bash "$ASM" task secret.key)" | grep -q 'secret.key (SKIPPED: secret-deny)' || fail "literal secret.key not deny-skipped"

# ---- Review-history integrity: tracked/unchanged docs remain FULL snapshots, while gaps,
#      malformed names, empty rounds, and symlinks fail closed rather than silently dropping
#      context and proceeding with a partial consensus record. ----
git add task/task.md task/codex-review.md task/codex-review-001.md task/codex-review-002.md
git commit -qm add-review-history
OUTR="$(bash "$ASM" task)"
printf '%s' "$OUTR" | grep -q 'TASK_SNAPSHOT_30' || fail "tracked unchanged task doc lost its full snapshot"
printf '%s' "$OUTR" | grep -q 'PRIOR_ROUND_ONE_31' || fail "tracked unchanged first review round lost its full snapshot"
printf '%s' "$OUTR" | grep -q 'PRIOR_ROUND_TWO_32' || fail "tracked unchanged latest review round lost its full snapshot"

mkdir -p legacy-only; printf '# task\n' > legacy-only/task.md
printf 'LEGACY_MIGRATION_CONTEXT_35\n' > legacy-only/codex-review.md
OUTLEGACY="$(bash "$ASM" legacy-only)"
printf '%s' "$OUTLEGACY" | grep -q 'LEGACY_MIGRATION_CONTEXT_35' \
  || fail "legacy singleton was not carried into the first numbered migration round"

mkdir -p bad-gap; printf '# task\n' > bad-gap/task.md
printf 'one\nConsensus: disagreed\n' > bad-gap/codex-review-001.md
printf 'three\nConsensus: disagreed\n' > bad-gap/codex-review-003.md
if bash "$ASM" bad-gap >/dev/null 2>&1; then fail "gapped review history was accepted"; fi

mkdir -p bad-name; printf '# task\n' > bad-name/task.md
printf 'wrong\n' > bad-name/codex-review-1.md
if bash "$ASM" bad-name >/dev/null 2>&1; then fail "malformed unpadded review round was accepted"; fi

mkdir -p bad-empty; printf '# task\n' > bad-empty/task.md
: > bad-empty/codex-review-001.md
if bash "$ASM" bad-empty >/dev/null 2>&1; then fail "empty review round was accepted"; fi

mkdir -p bad-link; printf '# task\n' > bad-link/task.md
printf 'ROUND_LINK_TARGET_SECRET_34\n' > review-target.txt
ln -s ../review-target.txt bad-link/codex-review-001.md
OUTLINK="$(bash "$ASM" bad-link 2>&1 || true)"
if printf '%s' "$OUTLINK" | grep -q 'ROUND_LINK_TARGET_SECRET_34'; then fail "symlinked review target leaked"; fi
if bash "$ASM" bad-link >/dev/null 2>&1; then fail "symlinked review round was accepted"; fi

# A sealed round ends on its sole canonical Consensus field. Blank lines after the field are
# harmless, but appended prose means the same file was extended after sealing and must fail.
mkdir -p sealed; printf '# task\n' > sealed/task.md
printf '# round\nConsensus: agreed\n\n' > sealed/codex-review-001.md
bash "$ASM" sealed >/dev/null || fail "blank lines after a final Consensus field were rejected"
printf '# round\nConsensus: agreed\nBlocker remains unresolved.\n' > sealed/codex-review-001.md
if bash "$ASM" sealed >/dev/null 2>&1; then fail "prose appended after Consensus was accepted"; fi
printf '# round\nConsensus: agreed 미해결\n' > sealed/codex-review-001.md
if bash "$ASM" sealed >/dev/null 2>&1; then fail "noncanonical Unicode consensus suffix was accepted"; fi

# The sequence is not capped at 999. Minimum-three-digit names widen naturally, and assembly
# remains numeric across the lexical 999→1000 boundary.
mkdir -p wide; printf '# task\n' > wide/task.md
n=1
while [ "$n" -le 1001 ]; do
  printf -v round 'wide/codex-review-%03d.md' "$n"
  printf 'ROUND_%04d\nConsensus: disagreed\n' "$n" > "$round"
  n=$((n + 1))
done
WIDE_OUT="$SBX/wide.out"
bash "$ASM" wide > "$WIDE_OUT" || fail "contiguous review history beyond round 999 was rejected"
p999="$(grep -n 'ROUND_0999' "$WIDE_OUT" | cut -d: -f1)"
p1000="$(grep -n 'ROUND_1000' "$WIDE_OUT" | cut -d: -f1)"
p1001="$(grep -n 'ROUND_1001' "$WIDE_OUT" | cut -d: -f1)"
[ "$p999" -lt "$p1000" ] && [ "$p1000" -lt "$p1001" ] \
  || fail "review rounds were not assembled numerically across 999→1000"

pass "codex-review assembler is fail-closed (allowlist-only; task + separate full-history snapshots; uncapped contiguous numeric review history; final-line sealing; legacy migration; secret/symlink/binary/size gated; deletions emitted incl. subdir/git-rm; diff-size cap; newline+glob-pathspec neutralized)"
