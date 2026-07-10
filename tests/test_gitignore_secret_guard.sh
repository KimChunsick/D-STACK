#!/usr/bin/env bash
# A public repo must never commit secrets/runtime state. Verify the allowlist
# .gitignore ignores a broad battery of secret/runtime names — including nested
# and extensionless ones (id_rsa, .netrc, *.db, *.p12) — and that none are in the
# git index, and that the tracked tree matches no secret pattern.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"

# Paths that MUST be ignored. Includes the holes an earlier review caught.
leaks=(
  claude/auth.json codex/config.toml claude/x.sqlite claude/x.sqlite-wal
  claude/data.db claude/data.sqlite3 claude/history.jsonl claude/.DS_Store
  claude/.env codex/.env.local claude/deploy.key codex/x.pem claude/x.p12 claude/y.pfx
  claude/id_rsa claude/id_ed25519 claude/.netrc claude/credentials.json
  claude/secrets.token claude/api_token claude/x.secret
  claude/hooks/auth.json claude/skills/full-cycle/credentials.json
  claude/hooks/random_unknownfile claude/hooks/deploy_key_prod
  codex/rules/random_unknownfile claude/skills/novel_secret_dir/blob
  claude/agents/random_unknownfile claude/agents/auth.json
  claude/agents/unknown-agent.md claude/agents/nested/inner-agent.md
  claude/agents/frontend-xyz.md claude/agents/f.md
)
created=()
cleanup() { [ "${#created[@]}" -gt 0 ] && rm -f "${created[@]}"; return 0; }
trap cleanup EXIT

for f in "${leaks[@]}"; do
  mkdir -p "$(dirname "$f")"
  : > "$f"; created+=("$f")
  git check-ignore -q "$f" || fail "secret NOT ignored by .gitignore: $f"
  if git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
    fail "secret present in git index (already tracked): $f"
  fi
done

# Behavioral exact-allowlist for agent definitions: agent .md files are executable
# instruction material, so with the probe battery above still on disk, git must see
# NOTHING addable under claude/agents/ except the single pinned file — this trips on ANY
# spelling of a re-include (rooted, unrooted, glob) that exposes any probe.
extra="$(git ls-files -o --exclude-standard claude/agents/ | grep -vx 'claude/agents/frontend-dev.md' || true)"
[ -z "$extra" ] || fail "unexpected addable files under claude/agents/: $extra"

# The allowlist is a CLOSED set: every negation (re-include) line in .gitignore must be
# one of the expected, consciously-added entries below, in order. This rejects ANY new
# `!` rule — whatever its spelling, root, or glob (`!claude/agents/f*.md`,
# `!/claude/**/z*.md`, …) — until it is deliberately added here alongside its review.
# Finite probe batteries cannot close the pattern space; pinning the rule set itself does.
expected_negations='!/.gitignore
!/AGENTS.md
!/CLAUDE.md
!/README.md
!/install.sh
!/docs/
!/tests/
!/claude/
!/codex/
!/gemini/
!/claude/.gitkeep
!/claude/CLAUDE.md
!/claude/settings.json
!/claude/statusline-command.sh
!/claude/ultracode.zsh
!/claude/hooks/
!/claude/hooks/fullcycle-inject.sh
!/claude/hooks/fullcycle-gate.sh
!/claude/skills/
!/claude/skills/full-cycle/
!/claude/skills/codex-review/
!/claude/skills/codex-research/
!/claude/agents/
!/claude/agents/frontend-dev.md
!/codex/.gitkeep
!/codex/AGENTS.md
!/codex/instructions.md
!/codex/rules/
!/codex/rules/default.rules
!/gemini/.gitkeep
!/gemini/README.md'
got_negations="$(grep '^!' .gitignore)"
[ "$got_negations" = "$expected_negations" ] \
  || fail ".gitignore negation set drifted from the pinned allowlist:
$(diff <(printf '%s\n' "$expected_negations") <(printf '%s\n' "$got_negations") || true)"

# The tracked tree must contain no secret pattern.
if git ls-files | grep -Ei 'auth\.json|credentials\.json|\.netrc|id_rsa|id_ed25519|history\.jsonl|config\.toml|\.DS_Store|\.(pem|key|p12|pfx|token|secret|sqlite[0-9]?|db[0-9]?)$|(^|/)\.env'; then
  fail "secret pattern present in tracked files"
fi

pass "gitignore secret guard"
