#!/usr/bin/env bash
# Minimal assertion helpers for D-STACK tests (no external deps).
fail() { echo "  ✗ FAIL: $*" >&2; exit 1; }
pass() { echo "  ✓ PASS: $*"; }
assert_eq()           { [ "$1" = "$2" ] || fail "expected '$2', got '$1'${3:+ ($3)}"; }
assert_contains()     { grep -qF -- "$2" "$1" || fail "'$1' does not contain '$2'"; }
assert_not_contains() { ! grep -qF -- "$2" "$1" || fail "'$1' unexpectedly contains '$2'"; }
# regex variants (grep -E); $1=regex, $2=file
assert_matches()      { grep -qE -- "$1" "$2" || fail "'$2' does not match /$1/"; }
assert_not_matches()  { ! grep -qE -- "$1" "$2" || fail "'$2' matches forbidden /$1/"; }
