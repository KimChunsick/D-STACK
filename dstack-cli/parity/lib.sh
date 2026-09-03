#!/usr/bin/env bash
# parity/lib.sh — the step contract, sourced by run.sh and by every step file.
#
# A step file is plain bash, run once per implementation with these variables exported:
#   DSTACK       the binary to drive ("$DSTACK run new x", never a bare `dstack`)
#   PARITY_IMPL  shell | rust
#   SANDBOX      the sandbox root, which is also the cwd
#   PARITY_LIB   this file
#   PARITY_OUT   the capture root (steps do not write there directly)
#   PARITY_STEP  the step name, e.g. 00-help
# The four functions below are everything a step may use. Declarations (expect_diff, mask_call)
# are recorded on the shell pass only, so a step declares them unconditionally.

PARITY_TAB="$(printf '\t')"

# call <name> -- <command...>
# Runs the command in the sandbox and captures stdout, stderr and the exit code under <name>.
call() {
  local name="$1"; shift
  _parity_dashdash "call $name" "${1:-}"; shift
  _parity_capture "$name" /dev/null "$@"
}

# call_stdin <name> <file-with-stdin> -- <command...>
call_stdin() {
  local name="$1" stdin="$2"; shift 2
  _parity_dashdash "call_stdin $name" "${1:-}"; shift
  [ -f "$stdin" ] || _parity_abort "call_stdin $name: no stdin file at $stdin"
  _parity_capture "$name" "$stdin" "$@"
}

# expect_diff <name> "<why>"
# The two implementations are allowed to differ on this call, and the difference must be exactly
# what this call's mask_call rules remove: after masking, both sides have to be identical.
expect_diff() {
  [ "$PARITY_IMPL" = shell ] || return 0
  printf '%s%s%s%s%s\n' "$PARITY_STEP" "$PARITY_TAB" "$1" "$PARITY_TAB" "$2" >> "$PARITY_OUT/expect.tsv"
}

# mask_call <name> '<sed regex>' '<replacement>'
# An extra mask for this call only, applied to both sides after the standard masks. An empty
# replacement deletes the matching lines.
mask_call() {
  [ "$PARITY_IMPL" = shell ] || return 0
  printf '%s%s%s%s%s%s%s\n' "$PARITY_STEP" "$PARITY_TAB" "$1" "$PARITY_TAB" "$2" "$PARITY_TAB" "$3" \
    >> "$PARITY_OUT/mask.tsv"
}

_parity_abort() { printf 'parity: %s\n' "$*" >&2; exit 2; }

_parity_dashdash() { [ "$2" = "--" ] || _parity_abort "$1: the command must follow --"; }

_parity_capture() {
  local name="$1" stdin="$2" rc=0 dir
  shift 2
  dir="$PARITY_OUT/$PARITY_IMPL/$PARITY_STEP"
  mkdir -p "$dir"
  # A repeated name would overwrite the first capture while both rows stay in calls.tsv.
  if [ -e "$dir/$name.rc" ]; then _parity_abort "duplicate call name $PARITY_STEP/$name"; fi
  printf '%s%s%s\n' "$PARITY_STEP" "$PARITY_TAB" "$name" >> "$PARITY_OUT/$PARITY_IMPL/calls.tsv"
  ( cd "$SANDBOX" && "$@" ) <"$stdin" >"$dir/$name.out" 2>"$dir/$name.err" || rc=$?
  printf '%s\n' "$rc" > "$dir/$name.rc"
}
