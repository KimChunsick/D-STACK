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
mkdir -p task; printf '# task\n' > task/task.md

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

pass "codex-review assembler is fail-closed (allowlist-only; secret/symlink/binary/size gated; deletions emitted incl. subdir/git-rm; diff-size cap; newline+glob-pathspec neutralized)"
