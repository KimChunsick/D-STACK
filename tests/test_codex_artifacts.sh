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

# instructions.md is Codex's GLOBAL file → it must be stack-neutral (no single mandated
# framework), or a stack-neutral reviewer would wrongly assume Next.js everywhere.
assert_not_matches '[Ff]ramework.*:.*Next\.js'        codex/instructions.md
assert_not_matches 'Server Components? by [Dd]efault'  codex/instructions.md
assert_not_matches 'sole styling method'               codex/instructions.md
# Regression: no specific stack/tool may be (re)introduced as a GLOBAL mandate.
assert_not_matches 'Next\.js|App Router'               codex/instructions.md
assert_not_matches 'Tailwind|Vitest|clsx'              codex/instructions.md
assert_not_matches 'React Testing Library'             codex/instructions.md
assert_matches 'stack-neutral|do not assume a default framework|regardless of (the )?(language|stack)' codex/instructions.md
assert_matches '[Rr]esearch'                           codex/instructions.md   # research-first ethos kept

# Codex's dual-role identity file (adversarial researcher + reviewer). Must exist,
# name both roles, declare the adversarial stance, and be wired into allowlist+installer.
[ -s codex/AGENTS.md ] || fail "missing or empty: codex/AGENTS.md (Codex dual-role identity)"
# Encode the intent, not loose substrings: the identity must define BOTH modes with their
# load-bearing rules, the security constraints, and the verdict contract — so a hollow file
# that merely name-drops the words cannot pass.
assert_matches '[Rr]esearch'                 codex/AGENTS.md   # research mode present
assert_matches '[Rr]eview'                   codex/AGENTS.md   # review mode present
assert_matches '[Aa]dversar'                 codex/AGENTS.md   # adversarial stance
assert_matches '[Aa]gainst'                  codex/AGENTS.md   # both-sides: evidence against the goal
assert_matches '[Uu]ntrusted'                codex/AGENTS.md   # prompt-injection handling
assert_matches '[Rr]ead-only'                codex/AGENTS.md   # security: read-only constraint
assert_matches '[Ss]ecret'                   codex/AGENTS.md   # security: never pipe secrets
assert_matches 'GPT verdict|verdict:'        codex/AGENTS.md   # review output contract
assert_contains .gitignore '!/codex/AGENTS.md'
assert_matches '^codex/AGENTS\.md\|\.codex/AGENTS\.md\|link$' install.sh
# Intent over text: the allow line must actually win — no deny rule may shadow it.
! git check-ignore -q codex/AGENTS.md || fail "codex/AGENTS.md is gitignored despite the allow line"

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
