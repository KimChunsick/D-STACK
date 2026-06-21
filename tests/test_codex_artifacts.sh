#!/usr/bin/env bash
# Layer B: authored Codex config in the SSOT. Verify the authored artifacts are
# present, that excluded secrets/machine-state never landed, and that no private
# path/name leaked into this public repo. Plus the Gemini extensibility stub.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
root="$(git rev-parse --show-toplevel)"
cd "$root"

[ -s codex/instructions.md ]    || fail "missing or empty: codex/instructions.md"
[ -s codex/rules/default.rules ] || fail "missing or empty: codex/rules/default.rules"

# Excluded — must never be backed up.
if [ -e codex/config.toml ]; then fail "config.toml must NOT be backed up (machine state + project paths)"; fi
if [ -e codex/auth.json ];   then fail "auth.json must NOT be backed up (secret)"; fi

# Public-safety: no machine-specific absolute path anywhere under any agent dir.
if grep -rqEI '/Users/' claude codex gemini; then fail "machine-specific /Users/ path leaked into tracked config"; fi

# Private project/identifier names: read from an OPTIONAL gitignored denylist so the
# names themselves never live in this public repo. Absent on fresh clones (nothing to leak).
denylist="$root/.private-denylist"
if [ -f "$denylist" ]; then
  while IFS= read -r name || [ -n "$name" ]; do
    [ -z "$name" ] && continue
    case "$name" in \#*) continue ;; esac
    if grep -rqFiI -- "$name" claude codex gemini docs tests; then
      fail "private name from denylist leaked into tracked content"
    fi
  done < "$denylist"
fi

# Gemini extensibility placeholder.
[ -s gemini/README.md ] || fail "gemini placeholder README missing"

pass "codex artifacts + gemini placeholder"
