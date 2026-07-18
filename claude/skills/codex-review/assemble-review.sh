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
LC_ALL=C
export LC_ALL
TASK_DIR="${1:?usage: assemble-review.sh TASK_DIR FILE...}"; shift || true

DENY='(^|/)(auth\.json|config\.toml|credentials\.json|id_rsa|.*\.(key|pem|p12|token)|\.env.*|.*\.sqlite.*|.*\.db|history\.jsonl|\.npmrc|\.netrc)$'
MAX=65536
CONSENSUS_FIELD_RE='^[-[:space:]>#*+._)0-9]*(✅|❌)?[[:space:]]*consensus:'
CONSENSUS_SEALED_RE='^[-[:space:]>#*+._)0-9]*((✅|❌)[[:space:]]*)?consensus:[*_[:space:]]*(disagreed|agreed|resolved)[[:punct:][:space:]]*((✅|❌)[[:punct:][:space:]]*)?$'

# Task/review records are context, not changes under review. Emit them as full snapshots even
# after they are tracked and unchanged; a diff-only read would silently drop the task contract
# or the prior consensus on a resumed review.
validate_snapshot() {
  local f="$1"
  case "$f" in *$'\n'*|*$'\r'*) echo "FATAL: control character in snapshot filename" >&2; return 1 ;; esac
  if printf '%s' "$f" | grep -qiE "$DENY"; then echo "FATAL: secret-denied snapshot path" >&2; return 1; fi
  if [ -L "$f" ]; then echo "FATAL: symlinked snapshot is not reviewable: $f" >&2; return 1; fi
  if [ ! -f "$f" ]; then echo "FATAL: snapshot is not a regular file: $f" >&2; return 1; fi
  if [ ! -s "$f" ]; then echo "FATAL: empty snapshot is not reviewable: $f" >&2; return 1; fi
  if [ "$(wc -c < "$f")" -gt "$MAX" ]; then echo "FATAL: snapshot exceeds 64KB: $f" >&2; return 1; fi
  if ! grep -Iq . -- "$f"; then echo "FATAL: binary snapshot is not reviewable: $f" >&2; return 1; fi
}

emit_snapshot() {
  local f="$1"
  validate_snapshot "$f"
  echo "--- $f (full snapshot) ---"
  cat -- "$f"
}

sealed_round_ok() {
  local f="$1" count line
  count="$(grep -icE "$CONSENSUS_FIELD_RE" -- "$f" || true)"
  [ "$count" -eq 1 ] || return 1
  line="$(awk 'NF { line=$0 } END { print line }' "$f")"
  printf '%s\n' "$line" | grep -qiE "$CONSENSUS_FIELD_RE" || return 1
  printf '%s\n' "$line" | grep -qiE "$CONSENSUS_SEALED_RE"
}

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

# Validate the canonical round namespace as one contiguous 001..NNN sequence. Suffixes have a
# minimum width of three digits and grow naturally after 999. Counting the namespace first and
# then generating each expected name keeps ordering numeric and Bash-3-compatible without an
# arbitrary round cap. This catches a missing, malformed, unsealed, empty, oversized, binary, or
# symlinked round instead of proceeding with partial history.
REVIEW_ROUNDS=()
round_count=0
for f in "$TASK_DIR"/codex-review*.md; do
  [ -e "$f" ] || [ -L "$f" ] || continue
  base="${f##*/}"
  case "$base" in
    codex-review.md)
      # Legacy migration context is separate from the canonical numbered namespace.
      continue
      ;;
    *)
      if ! printf '%s\n' "$base" | grep -qE '^codex-review-[0-9]{3,}\.md$'; then
        echo "FATAL: malformed name in reserved review namespace: $base" >&2
        exit 1
      fi
      round_count=$((round_count + 1))
      ;;
  esac
done

expected=1
while [ "$expected" -le "$round_count" ]; do
  printf -v base 'codex-review-%03d.md' "$expected"
  f="$TASK_DIR/$base"
  if [ ! -e "$f" ] && [ ! -L "$f" ]; then
    echo "FATAL: review rounds must be contiguous from 001; expected $base" >&2
    exit 1
  fi
  validate_snapshot "$f"
  if ! sealed_round_ok "$f"; then
    echo "FATAL: each sealed review round must end with exactly one canonical Consensus line: $base" >&2
    exit 1
  fi
  REVIEW_ROUNDS+=("$f")
  expected=$((expected + 1))
done

# Every automatic read goes through the strict snapshot gates. Prior rounds remain separate
# files but are all carried in numeric order so a later reviewer can verify the complete record.
echo "=== TASK DOC ==="; emit_snapshot "$TASK_DIR/task.md"
if [ -e "$TASK_DIR/codex-review.md" ] || [ -L "$TASK_DIR/codex-review.md" ]; then
  echo; echo "=== LEGACY REVIEW HISTORY (read-only migration context) ==="
  emit_snapshot "$TASK_DIR/codex-review.md"
fi
echo; echo "=== PRIOR NUMBERED REVIEW ROUNDS ==="
if [ "${#REVIEW_ROUNDS[@]}" -eq 0 ]; then
  echo "(none)"
else
  for review in "${REVIEW_ROUNDS[@]}"; do emit_snapshot "$review"; done
fi
echo; echo "=== ALLOWLISTED CHANGES + RESEARCH ==="
for f in "$@"; do emit_file "$f"; done
# NOTE (accepted residual): gates are name/type/size-based, not content-based — an explicitly
# allowlisted benign-named file could still contain a secret. This matches the repo's own
# name-based secret model; the allowlist is the control (only name task deliverables, never
# secret stores). Content scanning is intentionally out of scope.
