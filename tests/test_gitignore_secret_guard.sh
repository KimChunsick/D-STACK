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

# The tracked tree must contain no secret pattern.
if git ls-files | grep -Ei 'auth\.json|credentials\.json|\.netrc|id_rsa|id_ed25519|history\.jsonl|config\.toml|\.DS_Store|\.(pem|key|p12|pfx|token|secret|sqlite[0-9]?|db[0-9]?)$|(^|/)\.env'; then
  fail "secret pattern present in tracked files"
fi

pass "gitignore secret guard"
