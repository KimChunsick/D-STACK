#!/usr/bin/env bash
# parity/run.sh — drive the shell dstack and the Rust dstack through the same steps and diff them.
#
# Usage: bash dstack-cli/parity/run.sh [--shell-ref <tag>] [--shell <dispatcher>] [--rust <binary>]
#                                      [--only <step-name-glob>] [--keep] [--out <dir>]
#                                      [--self-check]
#
# Both sandboxes are built by the reference dispatcher, which P17 took out of the tree: --shell-ref
# names the tag that still carries it (shell-final), whose `claude` and `deps.tsv` are extracted
# into this run's own directory. So the two stores start from the same state and a difference
# reported here is a difference of the steps, never of the setup. Nothing outside the two
# sandboxes is written. Exit 0 iff nothing differs.
set -eu
set -o pipefail

PARITY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
REPO="$(cd "$PARITY_DIR/../.." && pwd -P)"
PARITY_LIB="$PARITY_DIR/lib.sh"
. "$PARITY_LIB"
TAB="$PARITY_TAB"
SEP="$(printf '\001')"   # sed delimiter for step-supplied masks, which may contain | and /

SHELL_REF=shell-final
SHELL_BIN=""        # --shell overrides the extraction of --shell-ref
SHELL_TREE=""       # the extracted tree, empty when --shell named a dispatcher of its own
RUST_BIN="$REPO/dstack-cli/target/release/dstack"
ONLY='*'; KEEP=0; OUT=""; OUT_MINE=1; SELF_CHECK=0
export CLAUDE_CODE_SESSION_ID=parity
CALLS=0; DIFFERING=0; EXPECTED=0; STORE_DIFF=0
USAGE='usage: run.sh [--shell-ref <tag>] [--shell <dispatcher>] [--rust <binary>] [--only <step-name-glob>] [--keep] [--out <dir>] [--self-check]'

abort() { printf 'parity: %s\n' "$*" >&2; exit 2; }
need_value() { [ "$2" -ge 2 ] || abort "$1 needs a value ($USAGE)"; }
# Runs on every exit, including an abort. A --out the caller named is never touched.
cleanup() {
  if [ "$OUT_MINE" = 1 ] && [ "$KEEP" = 0 ] && [ -n "$OUT" ]; then rm -rf "$OUT"; fi
  :
}

# ── arguments ──────────────────────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
  case "$1" in
    --shell)     need_value "$1" "$#"; SHELL_BIN="$2"; shift 2 ;;
    --shell-ref) need_value "$1" "$#"; SHELL_REF="$2"; shift 2 ;;
    --rust)  need_value "$1" "$#"; RUST_BIN="$2"; shift 2 ;;
    --only)  need_value "$1" "$#"; ONLY="$2"; shift 2 ;;
    --out)   need_value "$1" "$#"; OUT="$2"; OUT_MINE=0; shift 2 ;;
    --keep)  KEEP=1; shift ;;
    --self-check) SELF_CHECK=1; shift ;;
    *) abort "unknown option: $1 ($USAGE)" ;;
  esac
done

# ── sandboxes ──────────────────────────────────────────────────────────────────────────
# The construction the reference's own self-tests use: a git repository with one empty commit, the
# minimal deps table both implementations read through DSTACK_DEPS, a store and one open run.
# The last path element is the repository name that lands in PROJECT.md, so both sandboxes are
# called "sandbox" and differ only in the parent directory.
make_sandbox() {   # $1 impl → prints the sandbox path
  local d="$OUT/sb-$1/sandbox"
  mkdir -p "$d"
  printf 'name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\ngit\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\njq\tcommand -v jq\t-\t-\tno\tgoal-closing\talways\t\n' > "$d/.deps.tsv"
  # commit.gpgsign is a machine setting; a sandbox commit must not depend on it (signing under
  # parallel load fails now and then, which would fail the harness for an unrelated reason).
  ( cd "$d" && git init -q \
    && git -c user.email=t@t -c user.name=t -c commit.gpgsign=false commit -q --allow-empty -m init \
    && DSTACK_DEPS="$d/.deps.tsv" "$SHELL_BIN" init >/dev/null \
    && DSTACK_DEPS="$d/.deps.tsv" "$SHELL_BIN" run new sandbox --type cli >/dev/null ) \
    || abort "cannot build the $1 sandbox"
  ( cd "$d" && pwd -P )
}

# ── steps ──────────────────────────────────────────────────────────────────────────────
# A numbered step runs when its name matches --only; a helper step (no NN- prefix) runs only
# when --only names it exactly, so a normal run never picks one up.
step_wanted() {
  if [ "$SELF_CHECK" = 1 ]; then
    [ "$1" = selfcheck ]
    return $?
  fi
  case "$1" in
    [0-9][0-9]-*) case "$1" in $ONLY) return 0 ;; *) return 1 ;; esac ;;
    *) [ "$ONLY" = "$1" ]; return $? ;;
  esac
}

# The steps this run drives, in name order (empty when --only matches none of them).
wanted_steps() {
  local f base
  for f in "$PARITY_DIR"/steps/*.sh; do
    [ -f "$f" ] || continue
    base="$(basename "$f" .sh)"
    step_wanted "$base" && printf '%s\n' "$base"
  done
  return 0
}

run_steps() {   # $1 impl, $2 binary, $3 sandbox
  local impl="$1" bin="$2" sb="$3" f base
  mkdir -p "$OUT/$impl"
  : > "$OUT/$impl/calls.tsv"
  for f in "$PARITY_DIR"/steps/*.sh; do
    base="$(basename "$f" .sh)"
    step_wanted "$base" || continue
    ( cd "$sb"
      export DSTACK="$bin" PARITY_IMPL="$impl" SANDBOX="$sb" PARITY_LIB="$PARITY_LIB"
      export PARITY_OUT="$OUT" PARITY_STEP="$base"
      export DSTACK_DEPS="$sb/.deps.tsv"
      export DSTACK_KO_RULES="$REPO/claude/lint/ko-rules.tsv"
      . "$f" ) || abort "step $base failed for $impl"
  done
}

# ── normalizing ────────────────────────────────────────────────────────────────────────
# The only values D-02 allows to differ: UTC stamps, the stamp part of a run id, the sandbox
# path, the driven binary's path, pids, sha256 digests and the sandbox repository's git hashes.
# A literal string as a sed -E regex / as a sed replacement: a sandbox path may hold any of
# ][\/.*+?(){}|^$& and would otherwise be read as a pattern.
sed_re()  { printf '%s' "$1" | sed 's,[][\\/.*+?(){}|^$&],\\&,g'; }
sed_rep() { printf '%s' "$1" | sed 's,[\\/&],\\&,g'; }
mask_line() { printf 's%s%s%s%s%sg\n' "$SEP" "$(sed_re "$1")" "$SEP" "$(sed_rep "$2")" "$SEP"; }

write_masks() {   # $1 impl, $2 sandbox path (real), $3 binary
  local f="$OUT/mask-$1.sed" p real
  mask_line "$2" '<SANDBOX>' > "$f"
  for p in "$3" "$SHELL_BIN"; do
    real="$(cd "$(dirname "$p")" && pwd -P)/$(basename "$p")"
    mask_line "$real" '<DSTACK>' >> "$f"
    [ "$real" = "$p" ] || mask_line "$p" '<DSTACK>' >> "$f"
  done
  mask_line "$(cd "$2" && git rev-parse HEAD)" '<GIT>' >> "$f"
  mask_line "$(cd "$2" && git rev-parse --short HEAD)" '<GIT>' >> "$f"
  # Each implementation resolves its home from where its binary sits: the repository for the port,
  # the extracted tree for the reference. Both are the same repository at a different moment.
  if [ "$1" = shell ] && [ -n "$SHELL_TREE" ]; then mask_line "$SHELL_TREE" '<REPO>' >> "$f"; fi
  mask_line "$REPO" '<REPO>' >> "$f"
  cat >> "$f" <<'MASKS'
s/[0-9]{8}T[0-9]{6}Z_/<RUNID>_/g
s/[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z/<UTC>/g
s/[0-9a-f]{64}/<SHA>/g
s/(owner_pid[^0-9]*)[0-9]+/\1<PID>/g
s/\.tmp\.[0-9]+/.tmp.<PID>/g
MASKS
}

# The masks a step declared for one call, as a sed script (empty replacement deletes the line).
build_call_mask() {   # $1 step, $2 call
  : > "$OUT/callmask.sed"
  [ -f "$OUT/mask.tsv" ] || return 0
  awk -F"$TAB" -v s="$1" -v c="$2" -v q="$SEP" \
    '$1==s && $2==c { if ($4=="") printf "\\%s%s%sd\n", q, $3, q; else printf "s%s%s%s%s%sg\n", q, $3, q, $4, q }' \
    "$OUT/mask.tsv" > "$OUT/callmask.sed"
}

# Two files per stream: pre/ carries the standard masks only (what expect_diff has to find a
# difference in), norm/ adds the call's own masks (what the two implementations must agree on).
normalize() {   # $1 impl, $2 step, $3 call, $4 stream
  local src="$OUT/$1/$2/$3.$4" pre="$OUT/pre/$1/$2/$3.$4" post="$OUT/norm/$1/$2/$3.$4"
  mkdir -p "$OUT/pre/$1/$2" "$OUT/norm/$1/$2"
  if [ -f "$src" ]; then
    sed -E -f "$OUT/mask-$1.sed" < "$src" > "$pre" || abort "cannot normalize $1 $2/$3.$4"
  else
    : > "$pre"
  fi
  sed -E -f "$OUT/callmask.sed" < "$pre" > "$post" || abort "cannot mask $1 $2/$3.$4"
}

normalize_file() {   # $1 impl, $2 source, $3 destination
  sed -E -f "$OUT/mask-$1.sed" < "$2" > "$3" || abort "cannot normalize $2"
}

# ── comparing ──────────────────────────────────────────────────────────────────────────
stream_label() { case "$1" in out) echo stdout ;; err) echo stderr ;; *) echo exit ;; esac; }

expect_why() { [ -f "$OUT/expect.tsv" ] || return 0; awk -F"$TAB" -v s="$1" -v c="$2" '$1==s && $2==c{print $3; exit}' "$OUT/expect.tsv"; }

show_diff() {   # $1 header, $2 shell file, $3 rust file
  printf '%s\n' "$1"
  diff -u -L shell -L rust "$2" "$3" || true
}

compare_calls() {
  local step name stream why differs declared
  while IFS="$TAB" read -r step name; do
    [ -n "${name:-}" ] || continue
    CALLS=$((CALLS + 1))
    build_call_mask "$step" "$name"
    differs=""; declared=""
    for stream in out err rc; do
      normalize shell "$step" "$name" "$stream"
      normalize rust "$step" "$name" "$stream"
      cmp -s "$OUT/pre/shell/$step/$name.$stream" "$OUT/pre/rust/$step/$name.$stream" \
        || declared="$declared $stream"
      cmp -s "$OUT/norm/shell/$step/$name.$stream" "$OUT/norm/rust/$step/$name.$stream" \
        || differs="$differs $stream"
    done
    why="$(expect_why "$step" "$name")"
    # A declared difference that is not there is a stale declaration, not a pass.
    if [ -n "$why" ] && [ -z "$declared" ]; then
      DIFFERING=$((DIFFERING + 1))
      printf 'expected-not-met: %s/%s: no difference to expect\n' "$step" "$name"
      continue
    fi
    if [ -z "$differs" ]; then
      if [ -n "$why" ]; then
        EXPECTED=$((EXPECTED + 1))
        printf 'expected: %s/%s — %s\n' "$step" "$name" "$why"
      fi
      continue
    fi
    DIFFERING=$((DIFFERING + 1))
    if [ -n "$why" ]; then
      printf 'expected-not-met: %s/%s — %s\n' "$step" "$name" "$why"
    fi
    for stream in $differs; do
      show_diff "differing: $step/$name ($(stream_label "$stream"))" \
        "$OUT/norm/shell/$step/$name.$stream" "$OUT/norm/rust/$step/$name.$stream"
    done
  done < "$OUT/shell/calls.tsv"
  if ! cmp -s "$OUT/shell/calls.tsv" "$OUT/rust/calls.tsv"; then
    DIFFERING=$((DIFFERING + 1))
    show_diff "differing: call-set (calls)" "$OUT/shell/calls.tsv" "$OUT/rust/calls.tsv"
  fi
}

# The store of each sandbox as "<normalized path><TAB><real path>": run directories carry a
# timestamp in their name, so the path is normalized the same way the content is.
store_list() {   # $1 impl, $2 sandbox
  local p
  : > "$OUT/store-$1.tsv"
  ( cd "$2" && find .dstack -type f ) | sort | while read -r p; do
    printf '%s%s%s\n' "$(printf '%s\n' "$p" | sed -E -f "$OUT/mask-$1.sed")" "$TAB" "$p" >> "$OUT/store-$1.tsv"
  done
}

compare_store() {
  local p rs rr files=0
  store_list shell "$SB_SHELL"
  store_list rust "$SB_RUST"
  : > "$OUT/callmask.sed"
  : > "$OUT/store-differing"
  cut -f1 "$OUT/store-shell.tsv" "$OUT/store-rust.tsv" | sort -u > "$OUT/store-union"
  while read -r p; do
    [ -n "$p" ] || continue
    files=$((files + 1))
    rs="$(awk -F"$TAB" -v k="$p" '$1==k{print $2; exit}' "$OUT/store-shell.tsv")"
    rr="$(awk -F"$TAB" -v k="$p" '$1==k{print $2; exit}' "$OUT/store-rust.tsv")"
    if [ -z "$rs" ] || [ -z "$rr" ]; then
      if [ -z "$rs" ]; then printf 'differing: store/%s (only in the rust store)\n' "$p"
      else printf 'differing: store/%s (only in the shell store)\n' "$p"; fi
      printf '%s\n' "$p" >> "$OUT/store-differing"
      continue
    fi
    normalize_file shell "$SB_SHELL/$rs" "$OUT/norm/store-shell"
    normalize_file rust "$SB_RUST/$rr" "$OUT/norm/store-rust"
    if ! cmp -s "$OUT/norm/store-shell" "$OUT/norm/store-rust"; then
      show_diff "differing: store/$p (content)" "$OUT/norm/store-shell" "$OUT/norm/store-rust"
      printf '%s\n' "$p" >> "$OUT/store-differing"
    fi
  done < "$OUT/store-union"
  STORE_DIFF="$(wc -l < "$OUT/store-differing" | tr -d ' ')"
  printf 'store: files %s, differing %s\n' "$files" "$STORE_DIFF"
  sed 's|^|  |' "$OUT/store-differing"
}

report() {
  local total=$((DIFFERING + STORE_DIFF))
  [ "$EXPECTED" = 0 ] || printf 'expected %s\n' "$EXPECTED"
  if [ "$OUT_MINE" = 0 ]; then printf 'out: %s (kept: --out belongs to the caller)\n' "$OUT"
  elif [ "$KEEP" = 1 ]; then printf 'out: %s\n' "$OUT"
  else printf 'out: %s (removed; --keep keeps it)\n' "$OUT"; fi
  printf 'steps %s, differing %s\n' "$CALLS" "$total"
  [ "$total" = 0 ] || exit 1
}

# ── main ───────────────────────────────────────────────────────────────────────────────
if [ ! -x "$RUST_BIN" ]; then
  ( cd "$REPO/dstack-cli" && cargo build --release ) >/dev/null 2>&1 || abort "cargo build --release failed"
fi
[ -x "$RUST_BIN" ] || abort "no rust binary at $RUST_BIN"
if [ "$OUT_MINE" = 0 ]; then
  if [ -e "$OUT" ]; then
    [ -d "$OUT" ] || abort "--out $OUT is not a directory ($USAGE)"
    [ -z "$(ls -A "$OUT")" ] || abort "--out $OUT is not empty ($USAGE)"
  fi
else
  OUT="$(mktemp -d "${TMPDIR:-/tmp}/dstack-parity.XXXXXX")"
fi
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
trap cleanup EXIT
# The reference out of git history (P17): `claude` and `deps.tsv` of the tag are all the
# dispatcher resolves its home and its library from, and the copy goes away with $OUT.
if [ -z "$SHELL_BIN" ]; then
  SHELL_TREE="$OUT/shell-ref"
  mkdir -p "$SHELL_TREE"
  git -C "$REPO" archive "$SHELL_REF" claude deps.tsv | tar -x -C "$SHELL_TREE" \
    || abort "cannot extract the shell reference of $SHELL_REF (--shell-ref)"
  SHELL_BIN="$SHELL_TREE/claude/bin/dstack"
fi
[ -x "$SHELL_BIN" ] || abort "no shell dispatcher at $SHELL_BIN"
[ -n "$(wanted_steps)" ] || abort "--only $ONLY matches no step under dstack-cli/parity/steps"

SB_SHELL="$(make_sandbox shell)"
SB_RUST="$(make_sandbox rust)"
write_masks shell "$SB_SHELL" "$SHELL_BIN"
write_masks rust "$SB_RUST" "$RUST_BIN"
run_steps shell "$SHELL_BIN" "$SB_SHELL"
run_steps rust "$RUST_BIN" "$SB_RUST"
compare_calls
compare_store
report
