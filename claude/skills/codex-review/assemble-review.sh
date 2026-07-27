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
# Round N used to re-feed rounds 1..N-1 in full, so history grew quadratically in round
# count. Only the two most recent rounds are sent verbatim now — enough to verify the last
# round's findings and the round that produced the fix being claimed. Older rounds are replaced
# by a companion file the orchestrator writes when it seals the round. Sealed rounds on disk
# are never touched; this changes what the model is fed, not the audit trail.
#
# The companion is a SEPARATE FILE on purpose. Deriving the carried-decisions section by
# reading the round's Markdown was tried and abandoned across six review rounds: a round
# quotes other documents constantly — including this contract — so a heading inside a fence or
# an HTML comment can impersonate the real section, and no amount of fence tracking or
# delimiter counting settles it (a '```' line inside an open '```text' fence defeats both).
# A file whose entire content IS the carried state cannot be impersonated by its own contents.
# Its name deliberately does not match the codex-review*.md round namespace validated below.
FULL_ROUNDS=2
# The prompt tells the reviewer it may name an older round and get it back in full; this is how
# that is honoured. It names ROUNDS, not a count: raising a count would drag in every newer
# round too, which can overrun the bundle budget and make the promise unkeepable exactly when
# history is long. It also cannot shrink the two-round floor, so no value here can compact a
# round the contract guarantees in full.
EXTRA_FULL="${REVIEW_FULL_ROUND_IDS:-}"
# The commit the reviewed identity starts from. Empty means "working tree vs HEAD", which is the
# serial case. Under worker fan-out the reviewed identity is the COMMITTED `base..HEAD` range the
# worktree lifecycle records, and diffing against HEAD there yields nothing at all — the review
# would be handed a bundle with no implementation in it and could approve on that basis. Set
# REVIEW_BASE to the recorded base commit for that case. It is validated before use.
# The PHYSICAL repository root. Every path this script reads must resolve beneath it. Checking
# only the final component for a symlink was not enough: with `docs/unit` a symlink, a perfectly
# innocent-looking `docs/unit/task.md` passed the leaf test and then `cat` followed the parent to
# an external target, sending whatever it found to Codex.
ROOT_P="$(git rev-parse --show-toplevel 2>/dev/null)" \
  && ROOT_P="$(cd -- "$ROOT_P" && pwd -P)" \
  || { echo "assemble-review: cannot resolve the repository root physically" >&2; exit 1; }

# EXPLICIT mode, no silent default. A worker review that forgot to set the range used to fall back
# to `git diff HEAD`, which on a clean integration checkout emits zero implementation bytes and
# labels changed files "no change" — no SKIPPED marker, so nothing stopped the launch and the
# round reviewed a bundle with none of the code in it.
REVIEW_MODE="${REVIEW_MODE:-}"
case "$REVIEW_MODE" in
  serial|committed) : ;;
  *) echo "assemble-review: set REVIEW_MODE=serial (working tree vs HEAD) or REVIEW_MODE=committed (REVIEW_BASE..REVIEW_HEAD). It is mandatory: guessing it is how a worker review ships with no implementation in the bundle." >&2; exit 1 ;;
esac
REVIEW_BASE="${REVIEW_BASE:-}"
REVIEW_HEAD="${REVIEW_HEAD:-}"
DIFF_ARGS=""            # empty => working tree vs HEAD (serial mode)
if [ "$REVIEW_MODE" = "serial" ]; then
  { [ -z "$REVIEW_BASE" ] && [ -z "$REVIEW_HEAD" ]; } \
    || { echo "assemble-review: REVIEW_MODE=serial takes no REVIEW_BASE/REVIEW_HEAD" >&2; exit 1; }
else
  [ -n "$REVIEW_BASE" ] && [ -n "$REVIEW_HEAD" ] \
    || { echo "assemble-review: REVIEW_MODE=committed requires both REVIEW_BASE and REVIEW_HEAD — a base alone diffs a commit against the WORKING TREE, which is not the identity that gets merged" >&2; exit 1; }
  for r in "$REVIEW_BASE" "$REVIEW_HEAD"; do
    git rev-parse --verify --quiet "$r^{commit}" >/dev/null \
      || { echo "assemble-review: '$r' is not a commit in this repository" >&2; exit 1; }
  done
  git merge-base --is-ancestor "$REVIEW_BASE" "$REVIEW_HEAD" \
    || { echo "assemble-review: REVIEW_BASE is not an ancestor of REVIEW_HEAD" >&2; exit 1; }
  # The recorded head must be what is actually checked out, and the tree must be CLEAN. Otherwise
  # the bundle describes one identity while the merge later carries another — a probe measured
  # 14,200 bytes base-to-working-tree against 12,024 base-to-HEAD on the same checkout.
  cur="$(git rev-parse --verify --quiet HEAD)" \
    || { echo "assemble-review: cannot resolve HEAD" >&2; exit 1; }
  want="$(git rev-parse --verify --quiet "$REVIEW_HEAD^{commit}")" \
    || { echo "assemble-review: cannot resolve REVIEW_HEAD" >&2; exit 1; }
  [ "$cur" = "$want" ] \
    || { echo "assemble-review: HEAD is $cur but REVIEW_HEAD is $want — check out the reviewed commit before assembling" >&2; exit 1; }
  st="$(git status --porcelain)" \
    || { echo "assemble-review: cannot read worktree status" >&2; exit 1; }
  [ -z "$st" ] \
    || { echo "assemble-review: worktree is not clean; the reviewed identity must be exactly $REVIEW_BASE..$REVIEW_HEAD" >&2; exit 1; }
  # TWO-TREE diff. `git diff <commit> -- <path>` compares that commit with the WORKING TREE, so a
  # base alone never produced the committed range it claimed to.
  DIFF_ARGS="$REVIEW_BASE $REVIEW_HEAD"
fi
carried_path() { printf '%s/carried-%s' "${1%/*}" "${1##*codex-review-}"; }
# The per-file cap above never bounded the bundle: a task naming many files could assemble an
# order of magnitude more than the whole review history. The budget catches that runaway, and
# nothing else — set it from the smallest documented window, not from caution. The bundled CLI
# catalog reports gpt-5.6-sol at context_window 272000 (the public model spec lists a larger
# 1.05M); 512KB is roughly 128K tokens, under half that conservative figure, leaving room for
# reasoning and output. A tighter cap would reject bundles the model can plainly read, and the
# remedies below (narrowing the allowlist) cost review coverage — so the guard must fire only
# when the bundle is genuinely out of scale.
MAX_BUNDLE=524288
CONSENSUS_FIELD_RE='^[-[:space:]>#*+._)0-9]*(✅|❌)?[[:space:]]*consensus:'
CONSENSUS_SEALED_RE='^[-[:space:]>#*+._)0-9]*((✅|❌)[[:space:]]*)?consensus:[*_[:space:]]*(disagreed|agreed|resolved)[[:punct:][:space:]]*((✅|❌)[[:punct:][:space:]]*)?$'

# Physical containment for ANY path this script reads. `-L` on the final component says nothing:
# `-f`, `wc`, `grep` and `cat` all follow PARENT symlinks, so `docs/unit/task.md` with `docs/unit`
# linked elsewhere reads an external file. Resolve the parent and require it under the physical
# repository root. Residual, stated: resolution and the read are still two steps, so a parent
# swapped in between is not caught — closing that needs openat/O_NOFOLLOW, which is not available
# to a shell. This bounds the mistake case, not a racing attacker with write access to the repo.
contained() {
  local d
  d="$(cd -- "$(dirname -- "$1")" 2>/dev/null && pwd -P)" || return 1
  case "$d/" in "$ROOT_P"/*|"$ROOT_P/") return 0 ;; esac
  return 1
}

# Task/review records are context, not changes under review. Emit them as full snapshots even
# after they are tracked and unchanged; a diff-only read would silently drop the task contract
# or the prior consensus on a resumed review.
validate_snapshot() {
  local f="$1"
  case "$f" in *$'\n'*|*$'\r'*) echo "FATAL: control character in snapshot filename" >&2; return 1 ;; esac
  if printf '%s' "$f" | grep -qiE "$DENY"; then echo "FATAL: secret-denied snapshot path" >&2; return 1; fi
  if [ -L "$f" ]; then echo "FATAL: symlinked snapshot is not reviewable: $f" >&2; return 1; fi
  # SAME containment as emit_file. Round 9 fixed the allowlisted-change path and left the
  # automatic task/round snapshots reading through a symlinked parent — the exact sibling miss
  # this file keeps warning about. One helper, called from both.
  contained "$f" || { echo "FATAL: snapshot resolves outside the repository: $f" >&2; return 1; }
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

# Decides whether a round's companion may stand in for the round itself. Nothing here reads the
# round's Markdown body — that derivation is what six review rounds each defeated — but the two
# unambiguous ends ARE checked, because a companion that stands in for a round while disagreeing
# with it is worse than no compaction at all.
carried_ok() {
  local c="$1" n="$2" f="$3" last round_last body round_lines boundary
  validate_snapshot "$c" >/dev/null 2>&1 || return 1
  # Self-identifying: names the round it stands for, so a companion copied or misfiled into
  # another round's slot is rejected rather than silently replacing that round's state.
  head -n 1 -- "$c" | grep -qiE "^##[[:space:]]+carried decisions[[:space:]]*—[[:space:]]*round[[:space:]]+0*$n[[:space:]]*\$" || return 1
  last="$(awk 'NF { line = $0 } END { print line }' "$c")"
  printf '%s\n' "$last" | grep -qiE "$CONSENSUS_SEALED_RE" || return 1
  # Bound to its round's verdict: a truncated write cannot reach this line, and a companion
  # claiming a different consensus than the round it replaces is a contradiction, not a summary.
  round_last="$(awk 'NF { line = $0 } END { print line }' "$f")"
  [ "$last" = "$round_last" ] || return 1
  # Bound to its round's TEXT: everything after the companion's identifying first line must be
  # exactly the round's own last lines, AND that block must begin exactly where the round's
  # carried-decisions section begins. Suffix equality alone is not enough — the companion picks
  # its own length, so one holding nothing but a consensus line matches the round's final line
  # and passes while carrying no decisions at all. Anchoring the boundary takes it away: the
  # round's line immediately above the block must be the carried-decisions heading. That line is
  # read at a COMPUTED position, never searched for, so a heading quoted elsewhere in the round
  # cannot move it — searching is the derivation six earlier rounds were spent failing at.
  # The heading must be UNIQUE in the round. Otherwise the companion still picks the boundary:
  # with two headings it simply chooses the length that lands on the second one and drops
  # everything under the first. Searching to REFUSE is safe; searching to SELECT is the
  # derivation six earlier rounds were spent failing at, and this never selects.
  [ "$(grep -ciE '^##[[:space:]]+carried decisions[[:space:]]*$' "$f")" -eq 1 ] || return 1
  body="$(( $(wc -l < "$c") - 1 ))"
  [ "$body" -ge 1 ] || return 1
  round_lines="$(wc -l < "$f")"
  boundary="$(( round_lines - body ))"
  [ "$boundary" -ge 1 ] || return 1
  awk -v n="$boundary" 'NR == n { print; exit }' "$f" \
    | grep -qiE '^##[[:space:]]+carried decisions[[:space:]]*$' || return 1
  tail -n +2 -- "$c" | diff -q - <(tail -n "$body" -- "$f") >/dev/null 2>&1
}

emit_round_compact() {
  local f="$1" n="$2" c
  c="$(carried_path "$f")"
  # A missing or unconvincing companion means we do not know this round's carried state, so the
  # round goes out whole. Falling back to too much is a cost; dropping real carried state is a
  # defect. Legacy rounds sealed before companions existed simply never compact, which is
  # correct rather than merely tolerated.
  if ! carried_ok "$c" "$n" "$f"; then
    if [ -e "$c" ] || [ -L "$c" ]; then
      echo "--- $f (full snapshot; ${c##*/} is not a complete carried-state companion) ---"
    else
      echo "--- $f (full snapshot; no ${c##*/} companion) ---"
    fi
    cat -- "$f"
    return
  fi
  echo "--- $f (COMPACTED — carried state below is ${c##*/}; the full sealed round is on disk at the path above, name the round if you need it re-sent) ---"
  cat -- "$c"
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
  # PHYSICAL CONTAINMENT, not just a leaf-symlink test. `-f`, `wc`, `grep` and `cat` all follow
  # PARENT symlinks, so `docs/unit/task.md` with `docs/unit` linked elsewhere read an external
  # file and put it in the bundle. Resolve the parent and require it under the physical root.
  contained "$f" || { echo "--- $f (SKIPPED: resolves outside the repository via a symlinked parent) ---"; return; }

  # Scoped diff computed up front. The ':(literal)' pathspec magic forces git to treat $f as a
  # literal filename, NOT a glob/pathspec — otherwise an arg like '*', 'secret.*', or ':/' (which
  # the literal-name DENY above doesn't catch) would expand inside git to secret-named tracked
  # files and leak their diffs. It stays cwd-relative, so a non-empty diff also detects a staged
  # deletion (`git rm`) invoked from a subdirectory.
  # FAIL CLOSED on a diff error. `|| true` swallowed every failure, and an empty result then took
  # the "tracked, no change vs HEAD" branch — so a git that exited 128 (a corrupt object, an
  # unreadable index, a bad GIT_DIR) presented CHANGED code to the reviewer as unchanged, and the
  # mandatory review gate could approve without ever seeing the implementation.
  local diff dstatus=0
  diff="$(git diff ${DIFF_ARGS:-HEAD} -- ":(literal)$f" 2>/dev/null)" || dstatus=$?
  if [ "$dstatus" -ne 0 ]; then
    echo "--- $f (SKIPPED: git diff failed with status $dstatus — cannot establish what changed) ---"
    return
  fi
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

# A malformed request must not degrade quietly into "no rounds requested" — the reviewer asked
# for specific history and would get a compacted round back with no sign the ask was dropped.
_extra=""
# Split on commas ONLY (not whitespace), so an empty or whitespace-only field survives to be
# rejected instead of vanishing: ' ' between two commas names no round, and dropping it turns a
# malformed request into a silently smaller one. set -f keeps '[1]' a fatal id, not a glob.
# IFS splitting drops a TRAILING empty field, so '1,' has to be caught before the loop.
case "$EXTRA_FULL" in *,) echo "FATAL: REVIEW_FULL_ROUND_IDS has an empty field: '$EXTRA_FULL'" >&2; exit 1 ;; esac
_ifs="$IFS"; IFS=','; set -f
for r in ${EXTRA_FULL:+$EXTRA_FULL}; do
  r="${r#"${r%%[![:space:]]*}"}"; r="${r%"${r##*[![:space:]]}"}"   # trim, so '1, 2' still works
  case "$r" in *[!0-9]*|'') echo "FATAL: REVIEW_FULL_ROUND_IDS must be round numbers, got '$r'" >&2; exit 1 ;; esac
  # Bound the width before any arithmetic: $((10#…)) is fixed-width, so a huge literal wraps and
  # can land inside the valid range, turning an invalid id into a silent selection of some real
  # round. No task has a seven-digit round.
  [ "${#r}" -le 6 ] || { echo "FATAL: REVIEW_FULL_ROUND_IDS value out of range: '$r'" >&2; exit 1; }
  # Canonicalise before matching: '001' and '1' name the same round, and the emission test
  # compares decimal strings. Validating one form and matching another is how an explicit
  # request gets accepted and then quietly dropped.
  r=$((10#$r))
  if [ "$r" -lt 1 ] || [ "$r" -gt "$round_count" ]; then
    echo "FATAL: REVIEW_FULL_ROUND_IDS names round $r; this task has rounds 1..$round_count" >&2
    exit 1
  fi
  _extra="$_extra $r"
done
IFS="$_ifs"; set +f
EXTRA_FULL="${_extra# }"

# Every automatic read goes through the strict snapshot gates. Every round is still validated
# above; only how much of an older round is EMITTED changes. Buffer the bundle so its total
# size can be checked before the caller ships it to the model.
BUNDLE="$(mktemp)"; chmod 600 "$BUNDLE"; trap 'rm -f "$BUNDLE"' EXIT
{
  echo "=== TASK DOC ==="; emit_snapshot "$TASK_DIR/task.md"
  if [ -e "$TASK_DIR/codex-review.md" ] || [ -L "$TASK_DIR/codex-review.md" ]; then
    echo; echo "=== LEGACY REVIEW HISTORY (read-only migration context) ==="
    emit_snapshot "$TASK_DIR/codex-review.md"
  fi
  echo; echo "=== PRIOR NUMBERED REVIEW ROUNDS ==="
  round_total="${#REVIEW_ROUNDS[@]}"
  if [ "$round_total" -eq 0 ]; then
    echo "(none)"
  else
    echo "(the $FULL_ROUNDS most recent rounds appear in full${EXTRA_FULL:+, as do rounds $EXTRA_FULL by request}; each older round is compacted to its carried decisions and consensus line WHERE ITS COMPANION ALLOWS, and sent whole otherwise — every entry below says which)"
    idx=0
    for review in "${REVIEW_ROUNDS[@]}"; do
      idx=$((idx + 1))
      if [ "$((round_total - idx))" -lt "$FULL_ROUNDS" ] || printf ' %s ' "$EXTRA_FULL" | grep -q " $idx "; then
        emit_snapshot "$review"
      else
        emit_round_compact "$review" "$idx"
      fi
    done
  fi
  echo; echo "=== ALLOWLISTED CHANGES + RESEARCH ==="
  for f in "$@"; do emit_file "$f"; done
} > "$BUNDLE"

bundle_size="$(wc -c < "$BUNDLE" | tr -d '[:space:]')"
if [ "$bundle_size" -gt "$MAX_BUNDLE" ]; then
  echo "FATAL: review bundle is $bundle_size bytes, over the $MAX_BUNDLE-byte budget." >&2
  echo "       Narrow the allowlist to this task's own changed files, or split the task." >&2
  echo "       This is a policy limit, not a measured CLI ceiling — the exact stdin cap and" >&2
  echo "       overflow behaviour are undocumented. Failing here gives you a measurement" >&2
  echo "       instead of a mystery." >&2
  exit 1
fi
cat -- "$BUNDLE"
# NOTE (accepted residual): gates are name/type/size-based, not content-based — an explicitly
# allowlisted benign-named file could still contain a secret. This matches the repo's own
# name-based secret model; the allowlist is the control (only name task deliverables, never
# secret stores). Content scanning is intentionally out of scope.
