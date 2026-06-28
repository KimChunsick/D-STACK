#!/usr/bin/env bash
# Fail-closed assembler for codex-review material.
#
# ALLOWLIST model: includes ONLY the files named on the command line — nothing is collected
# automatically, so an unnamed secret can never reach the model. Each allowlisted file is then
# gated (defense in depth): symlinks skipped, secret-name deny backstop, size cap, binary skip.
# Tracked files are emitted as a SCOPED diff (`git diff HEAD -- "$f"`, never a repo-wide diff);
# new/untracked files as full content. Skips are listed explicitly so nothing is silently lost.
#
# Usage: assemble-review.sh TASK_DIR FILE [FILE ...]   > bundle.txt
set -euo pipefail
TASK_DIR="${1:?usage: assemble-review.sh TASK_DIR FILE...}"; shift || true

DENY='(^|/)(auth\.json|config\.toml|credentials\.json|id_rsa|.*\.(key|pem|p12|token)|\.env.*|.*\.sqlite.*|.*\.db|history\.jsonl|\.npmrc|\.netrc)$'
MAX=65536

emit_file() {
  local f="$1"
  if printf '%s' "$f" | grep -qiE "$DENY"; then echo "--- $f (SKIPPED: secret-deny) ---"; return; fi
  if [ -L "$f" ];                          then echo "--- $f (SKIPPED: symlink) ---"; return; fi
  if [ ! -f "$f" ];                        then echo "--- $f (SKIPPED: not a regular file) ---"; return; fi
  if [ "$(wc -c < "$f")" -gt "$MAX" ];     then echo "--- $f (SKIPPED: >64KB) ---"; return; fi
  if ! grep -Iq . -- "$f";                 then echo "--- $f (SKIPPED: binary) ---"; return; fi
  if git ls-files --error-unmatch -- "$f" >/dev/null 2>&1; then
    echo "--- $f (tracked, scoped diff) ---"; git diff HEAD -- "$f" 2>/dev/null || true
  else
    echo "--- $f (new/untracked, full content) ---"; cat -- "$f"
  fi
}

# Every read — including the task doc and prior review — goes through the same gates, so a
# symlinked/oversized/secret-named task.md or codex-review.md cannot bypass enforcement.
echo "=== TASK DOC ==="; emit_file "$TASK_DIR/task.md"
echo; echo "=== PRIOR REVIEW (carried into consensus rounds) ==="; emit_file "$TASK_DIR/codex-review.md"
echo; echo "=== ALLOWLISTED CHANGES + RESEARCH ==="
for f in "$@"; do emit_file "$f"; done
# NOTE (accepted residual): gates are name/type/size-based, not content-based — an explicitly
# allowlisted benign-named file could still contain a secret. This matches the repo's own
# name-based secret model; the allowlist is the control (only name task deliverables, never
# secret stores). Content scanning is intentionally out of scope.
