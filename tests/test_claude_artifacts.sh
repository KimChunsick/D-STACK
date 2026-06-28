#!/usr/bin/env bash
# Layer B: the maintainer's own authored Claude config, ingested into the SSOT.
# Verify every artifact is present and that settings.json is machine-portable
# (no /Users/<name> path; uses $HOME, which Claude expands in hook commands).
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"

for f in claude/CLAUDE.md claude/settings.json claude/statusline-command.sh \
         claude/hooks/fullcycle-inject.sh claude/hooks/fullcycle-gate.sh \
         claude/skills/full-cycle/SKILL.md claude/skills/codex-review/SKILL.md \
         claude/skills/codex-research/SKILL.md; do
  [ -s "$f" ] || fail "missing or empty: $f"
done

# codex-research is Codex-as-researcher: it must drive a real web research pass that
# gathers BOTH sides (incl. evidence against the goal) with sources, and degrade gracefully.
cr=claude/skills/codex-research/SKILL.md
assert_matches 'codex exec'             "$cr"
assert_matches 'web|live'               "$cr"
assert_matches '[Aa]gainst'             "$cr"   # evidence against the goal (both-sides)
assert_matches '[Oo]pposing'            "$cr"
assert_matches '[Ss]ource'              "$cr"   # cite sources
assert_matches '[Ff]allback|deep-research' "$cr"  # graceful degradation
# Hardening the invocation is load-bearing, not optional prose — assert the safety flags.
assert_matches 'stdin'                  "$cr"   # brief via stdin, not a shell arg (injection-safe)
assert_matches 'ephemeral'              "$cr"   # no session persistence
assert_matches 'read-only'              "$cr"   # no tree mutation
assert_matches 'output-last-message|-o ' "$cr"  # reproducible artifact capture
assert_matches 'model_reasoning_effort|xhigh' "$cr"   # pinned effort, no config drift
assert_matches 'non-zero|exit'          "$cr"   # concrete fallback trigger
# Copy-paste safety: a line-continuation backslash followed by an inline comment silently
# breaks the command. Forbid the `\  #…` pattern so the documented invocation stays runnable.
assert_not_matches '\\[[:space:]]+#'    "$cr"
assert_contains .gitignore '!/claude/skills/codex-research/'
assert_matches '^claude/skills/codex-research\|\.claude/skills/codex-research\|link$' install.sh

# codex-review: verdict + rebuttals go in a SEPARATE codex-review.md in the task folder
# (not inline), the review material includes UNTRACKED new files (git diff omits them), and
# the reviewer must also attack the research's own assumptions (dual-role mitigation).
crv=claude/skills/codex-review/SKILL.md
asm=claude/skills/codex-review/assemble-review.sh
assert_matches 'codex-review\.md'      "$crv"   # separate file in the task folder
assert_matches 'created|[Uu]ntracked'  "$crv"   # new/created files reach the reviewer (git diff omits them)
assert_matches 'research'              "$crv"   # reviewer challenges the research...
assert_matches 'assumption'            "$crv"   # ...its assumptions
assert_matches 'assemble-review\.sh'   "$crv"   # routes through the fail-closed helper
assert_matches 'FILES=|[Aa]llowlist'   "$crv"   # allowlist model (you name what is sent)
assert_matches 'mktemp'                "$crv"   # private temp bundle
assert_not_matches '/tmp/fc-review-input\.txt' "$crv"  # fixed-path bundle gone
# Enforcement lives in the helper (behaviorally tested in test_codex_review_assembler.sh).
[ -s "$asm" ] || fail "missing assembler helper: $asm"
assert_matches 'DENY'                  "$asm"   # secret-name deny backstop
assert_contains "$asm" 'auth\.json'             # concrete secret pattern (literal)
assert_matches 'SKIPPED: symlink'      "$asm"   # symlink targets not followed
assert_matches 'cat -- '               "$asm"   # leading-dash-safe
assert_matches 'git diff HEAD --'      "$asm"   # SCOPED per-file diff, never repo-wide

# Portability: no machine-specific home path in ANY claude artifact (not just settings.json).
if grep -rqEI '/Users/' claude; then fail "machine-specific /Users/ path leaked under claude/"; fi
assert_contains claude/settings.json '$HOME'

# settings.json must remain valid JSON after the path rewrite. jq is already required
# by the hooks themselves, so it is a hard dependency (not an optional check).
command -v jq >/dev/null 2>&1 || fail "jq not found (required by hooks and this check)"
jq -e . claude/settings.json >/dev/null || fail "settings.json is not valid JSON"

# No third-party plugins/marketplaces in settings.json: no superpowers, no enabledPlugins,
# no apps-in-toss / extraKnownMarketplaces (affiliation disclosure). Plugins are not backed up.
assert_not_matches 'superpowers|enabledPlugins|apps-in-toss|extraKnownMarketplaces|toss' claude/settings.json

pass "claude artifacts"
