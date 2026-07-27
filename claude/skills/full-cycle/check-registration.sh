#!/bin/bash
# check-registration.sh — P6's registration gate, as code rather than as a shell fence in prose.
#
# WHY THIS EXISTS. The check used to be ~30 lines of bash inside full-cycle/SKILL.md, and five
# adversarial rounds each fixed one defect by introducing the next: a failure that printed a
# warning and continued; `set -e` above a reference list, so the success path ran `unreg`; a
# hand-listed array that was simultaneously the assertion and its own proof; a `find` derivation
# comparing how MANY units existed and never WHICH; a loop returning 1 on its success path so a
# trailing `|| exit 1` aborted the fence silently. A deterministic transform belongs in code that
# can be run and checked, not in prose the model must re-execute correctly every round.
#
# WHAT IT PROVES. That every review unit the Goal DECLARED is scaffolded and registered to THIS
# session — identities, not counts — and that NOTHING ELSE beneath the Goal is registered to this
# session. The second half is enumerated from the registry, not inferred from a `find`, because a
# check that only looks where it expects trouble cannot find trouble anywhere else.
#
#   check-registration.sh <goal-dir>            run the check
#   check-registration.sh --depth <goal-dir>    print the review-unit depth and exit
#   check-registration.sh --list  <goal-dir>    print the documents P6 must register, one per line
#
# Exit 0 with a one-line confirmation, 1 listing every reason it blocked, 2 when it could not run.
# The third is separate on purpose: "the check did not run" must never read as "the check passed".
#
# `--depth` and `--list` exist so P6's registration loop neither guesses the level nor registers
# whatever it finds. The fence used to iterate a hard-coded `<Mn>/<NN-task>/task.md` glob (wrong
# level for a milestone-granularity Goal), then a depth-wide `find -exec` (registers undeclared and
# closed units BEFORE anything classifies them, and `find -exec cmd \;` does not even propagate
# cmd's failure — measured, `find . -exec false {} \;` exits 0). `--list` emits the exact set, so
# the fence registers only what the declaration names and can check each status itself.

set -u

# comm requires its inputs sorted in ITS OWN collation. Pinning both to C makes sort and comm
# agree byte-for-byte regardless of the caller's locale.
export LC_ALL=C

PROG="${0##*/}"
die()  { printf '%s: %s\n' "$PROG" "$*" >&2; exit 2; }
fail() { printf 'P6 BLOCKED: %s\n' "$*" >&2; failed=1; }
failed=0

MODE=check
case "${1:-}" in
  --depth) MODE=depth; shift ;;
  --list)  MODE=list;  shift ;;
esac
[ "$#" -eq 1 ] || die "usage: $PROG [--depth|--list] <goal-dir>"

# Canonicalise the Goal directory to the repo-relative spelling the registry stores. `docs/g`,
# `./docs/g`, `docs/g/` and an absolute path all name one directory, and an exact-line ownership
# match against an uncanonicalised spelling silently reports every document as unregistered.
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a git repository"
[ -d "${1%/}" ] || die "not a directory: ${1%/}"
GABS="$(cd -- "${1%/}" 2>/dev/null && pwd -P)" || die "cannot resolve ${1%/}"
RPHYS="$(cd -- "$ROOT" && pwd -P)"             || die "cannot resolve the repository root"
case "$GABS" in
  "$RPHYS"/*) G="${GABS#"$RPHYS"/}" ;;
  *) die "goal directory is outside the repository: ${1%/}" ;;
esac
[ -r "$G/GOAL.md" ] || die "no readable GOAL.md in $G"

DS="$HOME/.claude/bin/dstack"
[ -x "$DS" ] || die "dstack is not executable at $DS"

WORK="$(mktemp -d)" || die "cannot create a work directory"
trap 'rm -rf "$WORK"' EXIT

# ── 1. Parse GOAL.md ─────────────────────────────────────────────────────────────────────────
# Fences are tracked GLOBALLY from line one, so a fenced example ANYWHERE above the section cannot
# leave the parser mid-fence (or start it in one). Tracking them only from the section heading
# inverts the whole file: real rows read as fenced, examples read as real — reproduced.
#
# The fence rule is LENGTH- AND CHARACTER-AWARE, which a bare `^```` toggle is not: a ```` block
# legitimately contains ``` lines, and a naive toggle flipped state on each of them. Measured on a
# four-backtick block holding a three-backtick fence, the naive form read NEITHER the fenced fake
# row NOR the real one below. A closer must be the same character, at least as long, and carry
# nothing else.
#
# HONEST RESIDUAL, and it matters because F024 was about parser divergence: `check-parallel.sh`
# still uses the naive toggle. So on a four-backtick block the two now disagree — but they disagree
# in the FAIL-CLOSED direction. The scheduler would read the fenced rows as declarations; this
# checker reads the real ones, finds the two sets do not match, and BLOCKS. One parser blocking
# loudly beats two parsers being confidently wrong together. The identical fix belongs in
# `check-parallel.sh` and is recorded as a follow-up for the unit that owns it.
awk '
  BEGIN { fence = 0; insec = 0 }
  {
    ind = $0; sub(/^[[:space:]]*/, "", ind)
    if (match(ind, /^`{3,}/) || match(ind, /^~{3,}/)) {
      if (fence == 0)      { fence = 1; fc = substr(ind, 1, 1); flen = RLENGTH; next }
      else if (substr(ind, 1, 1) == fc && RLENGTH >= flen && substr(ind, RLENGTH + 1) ~ /^[[:space:]]*$/) { fence = 0; next }
      else                 { next }
    }
    if (fence) next
  }
  /^Review granularity:/                                   { print "G\t" $0; next }
  /^##[[:space:]]+Milestones[[:space:]]&[[:space:]]tasks/   { insec = 1; next }
  insec == 0                                               { next }
  /^##[[:space:]]/                                         { insec = 0; next }
  /^###[[:space:]]+M[0-9]+([[:space:]]|$)/                 { print "M\t" $0; next }
  /^- \[[ xX]\] \*\*T[0-9]+\*\*/                           { print "T\t" $0 }
' "$G/GOAL.md" > "$WORK/decl" || die "failed to parse $G/GOAL.md"

# ── 2. Granularity ───────────────────────────────────────────────────────────────────────────
# The DOCUMENTED value, not a keyword anywhere in the line. A substring test accepted
# `Review granularity: not task` as task granularity — measured — which is the worst possible
# reading of a line whose whole job is to fix the depth everything else is checked at.
n_gran="$(grep -c '^G	' "$WORK/decl")"
[ "$n_gran" -ge 1 ] || die "GOAL.md declares no unfenced 'Review granularity:' line — P5 must state it"
[ "$n_gran" -eq 1 ] || die "GOAL.md declares $n_gran 'Review granularity:' lines — exactly one is required"
gran_line="$(sed -n 's/^G	//p' "$WORK/decl")"
gran_val="$(printf '%s\n' "$gran_line" | sed 's/^Review granularity:[[:space:]]*//; s/[*_[:space:]]//g')"
case "$gran_val" in
  pertask*)      GRAN=task;      DEPTH=3; OTHER_DEPTH=2; SHAPE='<NN>-<slug>/task.md' ;;
  permilestone*) GRAN=milestone; DEPTH=2; OTHER_DEPTH=3; SHAPE='M<n>-<slug>/task.md' ;;
  *) die "granularity must be 'per task' or 'per milestone'; got: $gran_line" ;;
esac
[ "$MODE" != depth ] || { printf '%s\n' "$DEPTH"; exit 0; }

# ── 3. Declared identities ───────────────────────────────────────────────────────────────────
# Zero-padded, then sorted LEXICALLY. `sort -n` inputs BREAK comm — measured: declared {2,10,20}
# against scaffolded {10}, `comm -23` returned `2 10 20`, calling a present id missing.
#
# EVERY STAGE IS MATERIALISED AND CHECKED SEPARATELY. A pipeline reports only its LAST command's
# status, so `sed … | pad | sort > want` hides a failing `sed`, and a process substitution's status
# is not observable at all — measured, `while read …; done < <(exit 7)` leaves the loop at rc 0.
# An erased producer yields an empty set, an empty set yields empty deltas, and empty deltas read
# as "no differences found". That is a false PASS produced by a crash.
if [ "$GRAN" = task ]; then
  key='^T	'; sed_ids='s/^T	- \[[ xX]\] \*\*T\([0-9][0-9]*\)\*\*.*/\1/p'
else
  key='^M	'; sed_ids='s/^M	###[[:space:]][[:space:]]*M\([0-9][0-9]*\)\([[:space:]].*\)\{0,1\}$/\1/p'
fi
n_rows="$(grep -c "$key" "$WORK/decl")"
[ "$n_rows" -gt 0 ] || die "GOAL.md's '## Milestones & tasks' section declares no ${GRAN} rows"
sed -n "$sed_ids" "$WORK/decl" > "$WORK/want_raw" || die "failed to extract declared ${GRAN} ids"
awk '{ printf "%03d\n", $1 + 0 }' "$WORK/want_raw" > "$WORK/want_pad" || die "failed to normalise declared ids"
sort -o "$WORK/want" "$WORK/want_pad"                                 || die "failed to sort declared ids"
n_want="$(grep -c . < "$WORK/want")"
[ "$n_want" -eq "$n_rows" ] \
  || die "read $n_rows ${GRAN} rows from GOAL.md but could extract only $n_want ids — parser and document disagree"
uniq -d < "$WORK/want" > "$WORK/want_dup" || die "failed to scan declared ids for duplicates"
[ ! -s "$WORK/want_dup" ] || die "GOAL.md declares duplicate ${GRAN} ids: $(tr '\n' ' ' < "$WORK/want_dup")"

# ── 4. Scaffolded identities ─────────────────────────────────────────────────────────────────
# `find` alone, then sort: `find | sort` reports SORT's status, so a find that fails after emitting
# one path assigns rc=0 and the partial list is accepted as complete.
find "$G" -mindepth "$DEPTH" -maxdepth "$DEPTH" -name task.md > "$WORK/units" \
  || die "find failed under $G at depth $DEPTH"
sort -o "$WORK/units" "$WORK/units" || die "failed to sort the scaffolded unit list"
n_units="$(grep -c . < "$WORK/units")"
[ "$n_units" -gt 0 ] || fail "no review-unit task.md under $G at depth $DEPTH ($GRAN granularity)"

# The scaffold shape is exact in both directions: `<NN>-<slug>` and `M<n>-<slug>` both require the
# hyphen and a non-empty slug. Without them `M1oops/task.md` yielded id 1 and satisfied a milestone
# that has no such folder. A path carrying no readable id is REPORTED, never dropped — dropping it
# is how a misnamed folder becomes invisible to a check whose job is noticing missing units.
: > "$WORK/have"
while IFS= read -r p; do
  [ -n "$p" ] || continue
  if [ "$GRAN" = task ]; then
    id="$(printf '%s\n' "$p" | sed -n 's|.*/\([0-9][0-9]*\)-[^/][^/]*/task\.md$|\1|p')"
  else
    id="$(printf '%s\n' "$p" | sed -n 's|.*/M\([0-9][0-9]*\)-[^/][^/]*/task\.md$|\1|p')"
  fi
  if [ -z "$id" ]; then
    fail "scaffolded unit has no readable id (expected $SHAPE): $p"
  else
    printf '%s\t%s\n' "$(printf '%03d' "$((10#$id))")" "$p" >> "$WORK/have" \
      || die "failed to record scaffolded unit $p"
  fi
done < "$WORK/units"

cut -f1 < "$WORK/have" > "$WORK/have_cut" || die "failed to extract scaffolded ids"
sort -o "$WORK/have_cut" "$WORK/have_cut" || die "failed to sort scaffolded ids"
# Duplicates are REPORTED, not collapsed: two folders claiming one id is a real ambiguity about
# which the gate covers, and `uniq` hid a MISSING unit along with it (03 twice, 02 absent, deduped
# sets matched).
uniq -d < "$WORK/have_cut" > "$WORK/have_dup" || die "failed to scan scaffolded ids for duplicates"
while IFS= read -r d; do
  [ -n "$d" ] || continue
  fail "two or more scaffolded units share id $d: $(awk -F'\t' -v k="$d" '$1 == k { printf "%s ", $2 }' "$WORK/have")"
done < "$WORK/have_dup"
sort -u -o "$WORK/have_ids" "$WORK/have_cut" || die "failed to dedupe scaffolded ids"

comm -23 "$WORK/want" "$WORK/have_ids" > "$WORK/missing" || die "comm failed comparing declared against scaffolded"
comm -13 "$WORK/want" "$WORK/have_ids" > "$WORK/extra"   || die "comm failed comparing scaffolded against declared"
[ ! -s "$WORK/missing" ] || fail "declared but not scaffolded: $(tr '\n' ' ' < "$WORK/missing")"
[ ! -s "$WORK/extra" ]   || fail "scaffolded but not declared: $(tr '\n' ' ' < "$WORK/extra")"

# ── 5. Gate state, read exactly as fullcycle-gate.sh reads it ────────────────────────────────
# Same heading rule (whole token, or the documented ` (…)` parenthetical), same checkbox regexes,
# deliberately NOT fence-aware. A document the hook and this script disagree about is the one place
# a gate can go missing.
gate_state() {  # 0 = open, 1 = every box ticked, 2 = no usable gate section
  local gs
  gs="$(awk -v h='## Gate status' '$0 == h || index($0, h " (") == 1 { f = 1; next } /^ {0,3}#{1,6}([ \t]|$)/ { f = 0 } f' "$1")"
  [ -n "$gs" ] || return 2
  printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[[ xX]\]' || return 2
  printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[ \]'     && return 0
  return 1
}

# `--list` emits what P6 must register: GOAL.md plus every declared, scaffolded, still-OPEN unit.
# Undeclared folders and closed units are excluded BY CONSTRUCTION, which is what makes the fence
# safe to re-run — it used to register everything at the depth first and only then discover that
# some of it must not be registered.
if [ "$MODE" = list ]; then
  [ "$failed" -eq 0 ] || exit 1
  printf '%s\n' "$G/GOAL.md"
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    d="${line#*	}"
    [ -r "$d" ] || die "$d is not readable"
    gate_state "$d"; case $? in 0) printf '%s\n' "$d" ;; 1) : ;; 2) die "$d has no '## Gate status' checkbox rows" ;; esac
  done < "$WORK/have"
  exit 0
fi

# ── 6. Registration and OWNERSHIP ────────────────────────────────────────────────────────────
# `dstack status` prints `  <doc>  (this session)` for a record this session owns and
# `  <doc>  (session <sid>)` for anyone else's — and the Stop hook SKIPS the latter, so a
# foreign-owned record is WORSE than an absent one: it looks registered and enforces nothing. The
# two are always reported separately, in EVERY branch, because the fix differs: register it, versus
# resolve the other session, which `autonomy.stops` says is a human's call.
"$DS" status > "$WORK/status" || die "dstack status failed"

owned()      { grep -qxF -- "  $1  (this session)" "$WORK/status"; }
registered() { D="$1" awk 'index($0, "  " ENVIRON["D"] "  (") == 1 { f = 1 } END { exit(f ? 0 : 1) }' "$WORK/status"; }

report_missing() {  # $1 = doc, $2 = why it should have been registered
  if registered "$1"; then fail "$1 $2 but is registered to ANOTHER session — the Stop hook skips it"
  else                     fail "$1 $2 but is not registered at all"; fi
}
report_extra() {    # $1 = doc, $2 = why it must not be registered
  if   owned "$1";      then fail "$1 $2 but is registered to THIS session — deregister it"
  elif registered "$1"; then fail "$1 $2 and is registered to ANOTHER session"
  fi
}

owned "$G/GOAL.md" || report_missing "$G/GOAL.md" "is the Goal document"

# A unit with an unchecked box is ACTIVE and must be registered; one with every box ticked is CLOSED
# and must not be, or it holds the Goal gate open forever. An UNREADABLE or gate-less document is
# NEITHER — it is an error. Letting it fall through to "closed" is how the document most likely to
# be broken becomes the one nothing checks.
: > "$WORK/allowed"
printf '%s\n' "$G/GOAL.md" >> "$WORK/allowed"
while IFS= read -r d; do
  [ -n "$d" ] || continue
  if [ ! -r "$d" ]; then fail "$d is not readable"; continue; fi
  gate_state "$d"
  case $? in
    0) printf '%s\n' "$d" >> "$WORK/allowed"
       owned "$d" || report_missing "$d" "has unchecked gates" ;;
    1) report_extra "$d" "has every gate ticked" ;;
    2) fail "$d has no '## Gate status' section with checkbox rows (required gate schema missing)" ;;
  esac
done < "$WORK/units"

# ── 7. Nothing ELSE beneath the Goal may be registered ───────────────────────────────────────
# ENUMERATED FROM THE REGISTRY, not from a `find`. The previous version looked only at the
# alternate depth and only at files named `task.md`, so `<goal>/<Mn>/note.md` or a task.md three
# levels down could be registered to this session and the check never saw it — a gate on a document
# no phase governs. Reading `status` and subtracting the allowed set has no such blind spot.
sort -u -o "$WORK/allowed" "$WORK/allowed" || die "failed to build the allowed-document set"
awk -v pre="  " -v g="$G/" '
  index($0, pre) == 1 && $0 ~ /  \(this session\)$/ {
    d = substr($0, 3); sub(/  \(this session\)$/, "", d)
    if (index(d, g) == 1) print d
  }' "$WORK/status" > "$WORK/owned_under" || die "failed to enumerate this session's records under $G"
sort -o "$WORK/owned_under" "$WORK/owned_under" || die "failed to sort this session's records"
comm -23 "$WORK/owned_under" "$WORK/allowed" > "$WORK/unexpected" || die "comm failed comparing the registry against the allowed set"
while IFS= read -r d; do
  [ -n "$d" ] || continue
  fail "$d is registered to this session but is not GOAL.md or an open review unit at $GRAN granularity"
done < "$WORK/unexpected"

[ "$failed" -eq 0 ] || exit 1
n_open="$(( $(grep -c . < "$WORK/allowed") - 1 ))"
printf 'P6 registration confirmed: %s granularity, %s scaffolded units (%s open and registered to this session) + GOAL.md; no other document under %s is registered here\n' \
  "$GRAN" "$n_units" "$n_open" "$G"
