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

pass "codex-review assembler is fail-closed (allowlist-only; secret/symlink/binary/size gated)"
