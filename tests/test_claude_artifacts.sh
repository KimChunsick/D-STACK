#!/usr/bin/env bash
# Layer B: the maintainer's own authored Claude config, ingested into the SSOT.
# Verify every artifact is present and that settings.json is machine-portable
# (no /Users/<name> path; uses $HOME, which Claude expands in hook commands).
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"

for f in claude/CLAUDE.md claude/settings.json claude/statusline-command.sh \
         claude/hooks/fullcycle-inject.sh claude/hooks/fullcycle-gate.sh \
         claude/skills/full-cycle/SKILL.md claude/skills/codex-review/SKILL.md; do
  [ -s "$f" ] || fail "missing or empty: $f"
done

# Portability: no machine-specific home path in ANY claude artifact (not just settings.json).
if grep -rqEI '/Users/' claude; then fail "machine-specific /Users/ path leaked under claude/"; fi
assert_contains claude/settings.json '$HOME'

# settings.json must remain valid JSON after the path rewrite. jq is already required
# by the hooks themselves, so it is a hard dependency (not an optional check).
command -v jq >/dev/null 2>&1 || fail "jq not found (required by hooks and this check)"
jq -e . claude/settings.json >/dev/null || fail "settings.json is not valid JSON"

# Public-safety: no third-party plugin/marketplace config (reveals affiliation;
# also violates AGENTS.md Golden Rule 2). Plugins are re-enabled manually on restore.
assert_not_matches 'apps-in-toss|extraKnownMarketplaces|enabledPlugins' claude/settings.json

pass "claude artifacts"
