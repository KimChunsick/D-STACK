#!/bin/bash
# check-parallel.sh — deterministic fan-out gate for the full-cycle task DAG.
#
# Parses the per-task `deps`/`files` declarations in a GOAL.md (single source) and
# answers, without model judgment, whether work may fan out. Declarations are INERT
# DATA under a restricted grammar — nothing here is ever shell-expanded or executed.
# Only the '## Milestones & tasks' section is parsed, and fenced code blocks inside
# it are ignored — matching text elsewhere in the file is never graph data.
#
#   check-parallel.sh plan  <GOAL.md> <TASK-ID>...
#       Candidate-set verdict. Candidates must be open (unchecked) rows.
#   check-parallel.sh scope <GOAL.md> <TASK-ID> <worktree-dir> <base-commit> <task-branch>
#       Actual-diff containment on COMMITTED state only. The checker verifies the
#       worktree belongs to the GOAL.md's repository and sits on the named task
#       branch, the base is an ancestor of HEAD, and the tree is CLEAN (any
#       uncommitted/untracked change is a VIOLATION — reviewed identity is
#       base..HEAD). It then collects the committed set itself — renames disabled so
#       both sides are listed, NUL-safe — so a caller cannot narrow the check by
#       omitting paths, and rejects symlinks materialized under directory-ownership
#       declarations. (Accepted residuals: gitignored files never enter commits or
#       merges and are not scanned; base/branch values come from the orchestrator's
#       own records — mistake-tripwire, not a boundary against falsified records.)
#
# stdout is exactly one verdict line; exit codes are the contract:
#   0  PARALLEL (plan) / PASS (scope)
#   1  SERIAL   (plan: valid graph, ineligible set) / VIOLATION (scope)
#   2  INVALID  — broken declarations or state (malformed row, duplicate/missing
#      field, forbidden/non-canonical/symlink-traversing path, unknown/duplicate id,
#      cycle, inconsistent completion state, closed candidate). NEVER collapsed into
#      SERIAL: a broken graph cannot be satisfied by serial execution either.
#
# Grammar (terminal, single-space, one per task row; rows are logical items —
# continuation lines joined before parsing):
#   - [ ] **T<NN>** <slug> — <prose>. deps: [T<NN>, ...]; files: [<path>, <dir>/, ...]
# Path ceiling: repo-relative literal paths / trailing-slash dir prefixes; no globs,
# no absolute, no `.`/`..` components, no repeated separators, no whitespace/shell
# metacharacters, nothing under docs/ (pipeline docs are orchestrator-owned), and no
# component that is a symlink or submodule boundary in the repository (checked
# against the repo containing the GOAL.md; nonexistent components are fine — files
# may not exist yet). Overlap is case-INSENSITIVE (collision-conservative); scope
# containment is case-SENSITIVE (strict). Both directions are fail-closed.
#
# The row checkbox is the completion signal (ticked at P10 only). The checker
# rejects inconsistent state: a checked task with an unchecked dependency is INVALID.
set -u
LC_ALL=C; export LC_ALL

invalid() { printf 'INVALID: %s\n' "$1"; exit 2; }
serial()  { printf 'SERIAL: %s\n'  "$1"; exit 1; }

mode="${1:-}"; goalfile="${2:-}"
case "$mode" in plan|scope) : ;; *) invalid "unknown mode '${mode:-}' (use plan|scope)" ;; esac
[ -n "$goalfile" ] || invalid "no GOAL.md path given"
[ -L "$goalfile" ] && invalid "goal file is a symlink: $goalfile"
[ -f "$goalfile" ] || invalid "goal file missing: $goalfile"
shift 2

goaldirabs="$(cd "$(dirname "$goalfile")" 2>/dev/null && pwd)" \
  || invalid "cannot resolve goal file directory"
reporoot="$(git -C "$goaldirabs" rev-parse --show-toplevel 2>/dev/null)" \
  || invalid "GOAL.md is not inside a git repository (needed for symlink-safe path checks)"

# Task rows are accepted ONLY at column zero with the '-' marker — the documented
# peer-row grammar. Indented or alternate-marker lookalikes are data, not graph.
re_task='^- \[([ xX])\] \*\*(T[0-9]+)\*\*'
re_stop='^[[:space:]]*(#|[-*+][[:space:]]+\[)'
re_decl='deps: \[([^][]*)\]; files: \[([^][]*)\][.[:space:]]*$'
re_section='^##[[:space:]]+Milestones[[:space:]]&[[:space:]]tasks'
re_h2='^##[[:space:]]'
re_fence='^[[:space:]]*```'

ids=(); dones=(); depss=(); filess=(); n=0

find_idx() { # task id → index or -1 (stdout)
  local j
  for ((j = 0; j < n; j++)); do
    if [ "${ids[$j]}" = "$1" ]; then printf '%s' "$j"; return; fi
  done
  printf '%s' "-1"
}

trim() { # $1 → TRIMMED
  TRIMMED="$1"
  TRIMMED="${TRIMMED#"${TRIMMED%%[![:space:]]*}"}"
  TRIMMED="${TRIMMED%"${TRIMMED##*[![:space:]]}"}"
}

check_path() { # $1 task id, $2 raw path entry — lexical ceiling + repo symlink walk
  local id="$1" p="$2" body seg rest acc
  case "$p" in
    *'*'*|*'?'*) invalid "$id: glob in files entry '$p'" ;;
  esac
  case "$p" in
    *' '*|*'	'*) invalid "$id: whitespace in files entry '$p'" ;;
    *'~'*|*'$'*|*'`'*|*'"'*|*"'"*|*'\'*) invalid "$id: forbidden character in files entry '$p'" ;;
    /*) invalid "$id: absolute path '$p'" ;;
  esac
  body="$p"
  case "$body" in */) body="${body%/}" ;; esac       # one trailing / marks a dir prefix
  [ -n "$body" ] || invalid "$id: empty files entry"
  case "$body" in */) invalid "$id: repeated trailing separator '$p'" ;; esac
  rest="$body/"; acc=""
  while [ -n "$rest" ]; do
    seg="${rest%%/*}"; rest="${rest#*/}"
    case "$seg" in
      ''|.|..) invalid "$id: non-canonical component in '$p'" ;;
    esac
    acc="${acc:+$acc/}$seg"
    if [ -L "$reporoot/$acc" ]; then
      invalid "$id: '$p' traverses a symlink component ('$acc')"
    fi
    if [ "$acc" != "." ] && [ -d "$reporoot/$acc" ] && [ -e "$reporoot/$acc/.git" ]; then
      invalid "$id: '$p' crosses a submodule boundary ('$acc')"
    fi
  done
  case "$body" in
    docs|docs/*) invalid "$id: docs/ paths are orchestrator-owned, never declared ('$p')" ;;
  esac
}

parse_item() { # $1 joined logical row
  local row="$1" box id rawdeps rawfiles cnt item j out
  [[ $row =~ $re_task ]] || invalid "unparseable task row: ${row:0:60}"
  box="${BASH_REMATCH[1]}"; id="${BASH_REMATCH[2]}"
  j="$(find_idx "$id")"
  [ "$j" = "-1" ] || invalid "duplicate task id $id"
  cnt="$(printf '%s' "$row" | awk -F'deps: \\[' '{print NF-1}')"
  [ "$cnt" -eq 1 ] || invalid "$id: exactly one 'deps: [...]' field required (found $cnt)"
  cnt="$(printf '%s' "$row" | awk -F'files: \\[' '{print NF-1}')"
  [ "$cnt" -eq 1 ] || invalid "$id: exactly one 'files: [...]' field required (found $cnt)"
  [[ $row =~ $re_decl ]] || invalid "$id: declaration must be terminal 'deps: [...]; files: [...]'"
  rawdeps="${BASH_REMATCH[1]}"; rawfiles="${BASH_REMATCH[2]}"

  out=""
  if [ -n "${rawdeps//[[:space:]]/}" ]; then
    while IFS= read -r item; do
      trim "$item"; item="$TRIMMED"
      [[ $item =~ ^T[0-9]+$ ]] || invalid "$id: malformed dep id '$item'"
      [ "$item" = "$id" ] && invalid "$id: self-dependency"
      out="$out$item "
    done <<EOF
$(printf '%s' "$rawdeps" | tr ',' '\n')
EOF
  fi
  local depstr="$out"

  out=""
  if [ -n "${rawfiles//[[:space:]]/}" ]; then
    while IFS= read -r item; do
      trim "$item"; item="$TRIMMED"
      check_path "$id" "$item"
      out="$out$item "
    done <<EOF
$(printf '%s' "$rawfiles" | tr ',' '\n')
EOF
  fi

  ids[$n]="$id"; dones[$n]="$box"; depss[$n]="$depstr"; filess[$n]="$out"
  n=$((n + 1))
}

# ── build logical items from the Milestones section only, fences skipped ──────
# Fences are tracked GLOBALLY (from line one): a section heading inside a fenced
# example elsewhere in the file must never start parsing.
cur=""; in_section=0; in_fence=0; section_seen=0
flush() { [ -n "$cur" ] && parse_item "$cur"; cur=""; }
while IFS= read -r line || [ -n "$line" ]; do
  if [[ $line =~ $re_fence ]]; then flush; in_fence=$((1 - in_fence)); continue; fi
  [ "$in_fence" -eq 1 ] && continue
  if [ "$in_section" -eq 0 ]; then
    if [[ $line =~ $re_section ]]; then in_section=1; section_seen=1; fi
    continue
  fi
  if [[ $line =~ $re_section ]]; then continue; fi
  if [[ $line =~ $re_h2 ]]; then flush; in_section=0; continue; fi
  if [[ $line =~ $re_task ]]; then
    flush; cur="$line"
  elif [[ $line =~ $re_stop ]]; then
    flush
  elif [ -n "$cur" ] && [ -n "${line//[[:space:]]/}" ]; then
    cur="$cur $line"
  fi
done < "$goalfile"
flush
[ "$section_seen" -eq 1 ] || invalid "no '## Milestones & tasks' section in $goalfile"
[ "$n" -gt 0 ] || invalid "no task declarations found in the Milestones & tasks section"

# ── graph + state validation (both modes) ─────────────────────────────────────
for ((i = 0; i < n; i++)); do
  for d in ${depss[$i]}; do
    [ "$(find_idx "$d")" = "-1" ] && invalid "${ids[$i]}: unknown dep '$d'"
  done
done
# Completion-state closure: the checkbox is the P10 signal; a checked task whose
# dependency is unchecked is a lie somewhere — refuse to schedule on top of it.
for ((i = 0; i < n; i++)); do
  case "${dones[$i]}" in
    x|X)
      for d in ${depss[$i]}; do
        di="$(find_idx "$d")"
        case "${dones[$di]}" in
          x|X) : ;;
          *) invalid "inconsistent completion state: ${ids[$i]} is checked but dep $d is not" ;;
        esac
      done
      ;;
  esac
done
# Kahn's algorithm — anything left unremovable is a cycle.
indeg=(); removed=()
for ((i = 0; i < n; i++)); do
  cnt=0; for d in ${depss[$i]}; do cnt=$((cnt + 1)); done
  indeg[$i]=$cnt; removed[$i]=0
done
for ((k = 0; k < n; k++)); do
  pick=-1
  for ((i = 0; i < n; i++)); do
    if [ "${removed[$i]}" -eq 0 ] && [ "${indeg[$i]}" -eq 0 ]; then pick=$i; break; fi
  done
  [ "$pick" -lt 0 ] && invalid "dependency cycle detected"
  removed[$pick]=1
  for ((i = 0; i < n; i++)); do
    if [ "${removed[$i]}" -eq 0 ]; then
      for d in ${depss[$i]}; do
        [ "$d" = "${ids[$pick]}" ] && indeg[$i]=$((indeg[$i] - 1))
      done
    fi
  done
done

lower() { printf '%s' "$1" | tr 'A-Z' 'a-z'; }

# Defense-in-depth: with closure + readiness + open-candidate validation a ready
# candidate pair can no longer be transitively ordered, but the check is cheap and
# guards any future relaxation of those rules.
reaches() { # $1 from-idx, $2 target-idx → 0 if from transitively depends on target
  local cur=" $1 " added=1 j d di
  while [ "$added" -eq 1 ]; do
    added=0
    for j in $cur; do
      for d in ${depss[$j]}; do
        di="$(find_idx "$d")"
        case "$cur" in *" $di "*) : ;; *) cur="$cur$di "; added=1 ;; esac
      done
    done
  done
  case "$cur" in *" $2 "*) return 0 ;; *) return 1 ;; esac
}

overlaps() { # two declared entries → 0 if they overlap (case-insensitive, prefix-aware)
  local a b
  a="$(lower "$1")"; a="${a%/}"
  b="$(lower "$2")"; b="${b%/}"
  [ "$a" = "$b" ] && return 0
  case "$b" in "$a"/*) return 0 ;; esac
  case "$a" in "$b"/*) return 0 ;; esac
  return 1
}

# ── mode: plan ────────────────────────────────────────────────────────────────
if [ "$mode" = plan ]; then
  [ "$#" -ge 1 ] || invalid "plan needs at least one candidate task id"
  cand=(); m=0
  for c in "$@"; do
    ci="$(find_idx "$c")"
    [ "$ci" = "-1" ] && invalid "unknown candidate id '$c'"
    case "${dones[$ci]}" in
      x|X) invalid "candidate '$c' is already checked complete — not schedulable" ;;
    esac
    for ((i = 0; i < m; i++)); do [ "${cand[$i]}" = "$ci" ] && invalid "duplicate candidate '$c'"; done
    cand[$m]="$ci"; m=$((m + 1))
  done
  for ((i = 0; i < m; i++)); do
    ci="${cand[$i]}"
    for d in ${depss[$ci]}; do
      di="$(find_idx "$d")"
      case "${dones[$di]}" in x|X) : ;; *) serial "${ids[$ci]} not ready — dep $d incomplete" ;; esac
    done
    [ -n "${filess[$ci]//[[:space:]]/}" ] || serial "${ids[$ci]} has an empty files declaration — fan-out ineligible"
  done
  for ((i = 0; i < m; i++)); do
    for ((j = i + 1; j < m; j++)); do
      a="${cand[$i]}"; b="${cand[$j]}"
      if reaches "$a" "$b" || reaches "$b" "$a"; then
        serial "${ids[$a]} and ${ids[$b]} are dependency-ordered"
      fi
      for fa in ${filess[$a]}; do
        for fb in ${filess[$b]}; do
          overlaps "$fa" "$fb" && serial "${ids[$a]} and ${ids[$b]} overlap on '$fa' vs '$fb'"
        done
      done
    done
  done
  out=""
  for ((i = 0; i < m; i++)); do out="$out ${ids[${cand[$i]}]}"; done
  printf 'PARALLEL:%s\n' "$out"
  exit 0
fi

# ── mode: scope — the checker collects the complete path set itself ───────────
# Identity binding: the worktree must belong to the SAME repository as the GOAL.md,
# sit on the orchestrator-named task branch, and the recorded base must be an
# ancestor of HEAD. The tree must be CLEAN — review and merge run only on committed
# state, so any uncommitted or untracked change (declared or not) is a VIOLATION.
# Residual (accepted, recorded): the base/branch values come from the orchestrator's
# own records — this gate is a mistake-tripwire for one honest orchestrator, not a
# security boundary against the orchestrator falsifying its own inputs (same
# self-attestation scope as the Stop hook).
tid="${1:-}"; wdir="${2:-}"; base="${3:-}"; branch="${4:-}"
[ -n "$tid" ] && [ -n "$wdir" ] && [ -n "$base" ] && [ -n "$branch" ] \
  || invalid "scope needs <task-id> <worktree-dir> <base-commit> <task-branch>"
ti="$(find_idx "$tid")"
[ "$ti" = "-1" ] && invalid "unknown task id '$tid'"
[ -d "$wdir" ] || invalid "worktree dir missing: $wdir"
git -C "$wdir" rev-parse --show-toplevel >/dev/null 2>&1 \
  || invalid "not a git worktree: $wdir"
cur_branch="$(git -C "$wdir" branch --show-current 2>/dev/null)"
[ "$cur_branch" = "$branch" ] \
  || invalid "worktree is on '${cur_branch:-<detached>}', expected task branch '$branch'"
c1="$(cd "$wdir" && cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)"
c2="$(cd "$goaldirabs" && cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)"
[ -n "$c1" ] && [ "$c1" = "$c2" ] \
  || invalid "worktree does not belong to the GOAL.md repository"
git -C "$wdir" rev-parse --verify -q "$base^{commit}" >/dev/null 2>&1 \
  || invalid "base commit does not resolve in $wdir: $base"
git -C "$wdir" merge-base --is-ancestor "$base" HEAD 2>/dev/null \
  || invalid "recorded base is not an ancestor of HEAD"

check_contained() { # $1 actual repo-relative committed path
  local p="$1" f body seg rest acc contained=1 via_dir=0
  case "$p" in
    /*|*'..'*) printf 'VIOLATION: suspicious actual path %s\n' "$p"; exit 1 ;;
  esac
  rest="$p"; acc=""
  while [ -n "$rest" ]; do
    seg="${rest%%/*}"
    case "$rest" in */*) rest="${rest#*/}" ;; *) rest="" ;; esac
    acc="${acc:+$acc/}$seg"
    if [ -L "$wdir/$acc" ] && [ "$acc" != "$p" ]; then
      printf 'VIOLATION: %s traverses a symlink component (%s)\n' "$p" "$acc"; exit 1
    fi
  done
  for f in ${filess[$ti]}; do
    body="${f%/}"
    if [ "$p" = "$body" ]; then
      contained=0; case "$f" in */) via_dir=1 ;; esac; break
    fi
    case "$f" in
      */) case "$p" in "$body"/*) contained=0; via_dir=1; break ;; esac ;;
    esac
  done
  if [ "$contained" -ne 0 ]; then
    printf 'VIOLATION: %s is not in %s declaration\n' "$p" "$tid"; exit 1
  fi
  # Directory ownership grants files, never link indirection: a symlink created
  # under (or as) a declared directory can route later writes outside the tree
  # invisibly to git — reject it. An EXACT-path declaration may be a symlink
  # (the owner named that path knowingly).
  if [ "$via_dir" -eq 1 ] && [ -L "$wdir/$p" ]; then
    printf 'VIOLATION: %s is a symlink under directory ownership\n' "$p"; exit 1
  fi
}

# Enumerations fail CLOSED: a git error is INVALID, never a silent PASS.
dtmp="$(mktemp)"; stmp="$(mktemp)"; trap 'rm -f "$dtmp" "$stmp"' EXIT
git -C "$wdir" diff --name-only -z --no-renames "$base" HEAD > "$dtmp" 2>/dev/null \
  || invalid "git diff enumeration failed in $wdir"
git -C "$wdir" status --porcelain=v1 -z --no-renames --untracked-files=all > "$stmp" 2>/dev/null \
  || invalid "git status enumeration failed in $wdir"
# Clean-tree requirement: reviewed identity is base..HEAD only.
if [ -s "$stmp" ]; then
  first="$(tr '\0' '\n' < "$stmp" | head -1)"
  printf 'VIOLATION: worktree not clean (uncommitted/untracked): %s\n' "${first:3}"
  exit 1
fi
while IFS= read -r -d '' p; do
  [ -n "$p" ] && check_contained "$p"
done < "$dtmp"

printf 'PASS\n'
exit 0
