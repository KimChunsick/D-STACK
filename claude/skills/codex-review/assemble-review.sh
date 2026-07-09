#!/usr/bin/env bash
# Fail-closed assembler for codex-review material.
#
# ALLOWLIST model: includes ONLY the files named on the command line — nothing is collected
# automatically, so an unnamed secret can never reach the model. Each allowlisted file is then
# gated (defense in depth): symlinks skipped, secret-name deny backstop, size cap, binary skip.
# Tracked files are emitted as a SCOPED diff (`git diff HEAD -- "$f"`, never a repo-wide diff),
# and the size cap is applied to that DIFF (not the working-tree file) so a tiny change to a
# huge tracked file stays reviewable; a DELETED tracked file's removal is emitted, not dropped.
# New/untracked files are emitted as full content. Skips are listed explicitly — nothing is
# silently lost.
#
# Usage: assemble-review.sh TASK_DIR FILE [FILE ...]   > bundle.txt
set -euo pipefail
TASK_DIR="${1:?usage: assemble-review.sh TASK_DIR FILE...}"; shift || true

DENY='(^|/)(auth\.json|config\.toml|credentials\.json|id_rsa|.*\.(key|pem|p12|token)|\.env.*|.*\.sqlite.*|.*\.db|history\.jsonl|\.npmrc|\.netrc)$'
MAX=65536

emit_file() {
  local f="$1"
  # Reject a newline/CR in the path FIRST: the DENY grep below is line-oriented, so a name like
  # $'id_\nrsa' would split into two non-matching lines and slip a secret-named file past the
  # backstop. No reviewable file legitimately carries a newline in its name.
  case "$f" in *$'\n'*|*$'\r'*) echo "--- (SKIPPED: newline/control char in filename) ---"; return ;; esac
  # secret-name deny stays FIRST — even a deletion diff of a secret-named file would leak it.
  if printf '%s' "$f" | grep -qiE "$DENY"; then echo "--- $f (SKIPPED: secret-deny) ---"; return; fi
  if [ -L "$f" ];                          then echo "--- $f (SKIPPED: symlink) ---"; return; fi

  # Scoped diff computed up front. The ':(literal)' pathspec magic forces git to treat $f as a
  # literal filename, NOT a glob/pathspec — otherwise an arg like '*', 'secret.*', or ':/' (which
  # the literal-name DENY above doesn't catch) would expand inside git to secret-named tracked
  # files and leak their diffs. It stays cwd-relative, so a non-empty diff also detects a staged
  # deletion (`git rm`) invoked from a subdirectory.
  local diff; diff="$(git diff HEAD -- ":(literal)$f" 2>/dev/null || true)"
  if git ls-files --error-unmatch -- ":(literal)$f" >/dev/null 2>&1 || [ -n "$diff" ]; then
    # Tracked → SCOPED diff, capped on the DIFF's size (not the working-tree file's). A deleted
    # tracked file (no longer on disk) is emitted here too — its removal is a reviewable change.
    if [ -e "$f" ] && ! grep -Iq . -- "$f";             then echo "--- $f (SKIPPED: binary) ---"; return; fi
    if [ -z "$diff" ];                                  then echo "--- $f (tracked, no change vs HEAD) ---"; return; fi
    if [ "$(printf '%s' "$diff" | wc -c)" -gt "$MAX" ];  then echo "--- $f (tracked, diff SKIPPED: >64KB) ---"; return; fi
    if [ -e "$f" ]; then echo "--- $f (tracked, scoped diff) ---"; else echo "--- $f (tracked, deleted — scoped diff) ---"; fi
    printf '%s\n' "$diff"
    return
  fi

  # Untracked/new → FULL content, capped on the file (content == file here).
  if [ ! -f "$f" ];                        then echo "--- $f (SKIPPED: not a regular file) ---"; return; fi
  if [ "$(wc -c < "$f")" -gt "$MAX" ];     then echo "--- $f (SKIPPED: >64KB) ---"; return; fi
  if ! grep -Iq . -- "$f";                 then echo "--- $f (SKIPPED: binary) ---"; return; fi
  echo "--- $f (new/untracked, full content) ---"; cat -- "$f"
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
