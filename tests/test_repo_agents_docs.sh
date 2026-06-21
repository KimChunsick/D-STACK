#!/usr/bin/env bash
# Layer A: the repo's own agent docs. AGENTS.md is the canonical base; CLAUDE.md
# must reference it via @import (both are real root files, so the import is safe).
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"

grep -qE '^@AGENTS\.md[[:space:]]*$' CLAUDE.md || fail "CLAUDE.md must start by importing @AGENTS.md"
assert_contains AGENTS.md "Never commit secrets"
assert_contains AGENTS.md "install.sh"
for d in claude codex gemini; do
  [ -d "$d" ] || fail "missing agent dir: $d"
done

pass "repo agent docs"
