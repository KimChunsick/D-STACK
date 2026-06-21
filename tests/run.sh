#!/usr/bin/env bash
# Discover and run every tests/test_*.sh; non-zero exit if any fails.
set -uo pipefail
cd "$(cd "$(dirname "$0")" && pwd)" || exit 1
fails=0
for t in test_*.sh; do
  [ -e "$t" ] || continue
  echo "── $t"
  if bash "$t"; then :; else fails=$((fails + 1)); fi
done
echo
if [ "$fails" -ne 0 ]; then echo "FAILED: $fails test file(s)"; exit 1; fi
echo "ALL TESTS PASSED"
