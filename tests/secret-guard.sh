#!/usr/bin/env bash
# Secret-trackability guard — the ONE meta check this repo keeps (public repo).
# Run manually before every commit: bash tests/secret-guard.sh
#
# A public repo must never commit secrets/runtime state. The verdict must be about
# what a commit would actually record, so the guard first rejects any state where
# the effective ignore rules could differ from the committed ones (staged/worktree
# divergence, nested .gitignore files, local info/exclude rules) and neutralizes
# global excludes per-invocation. It then verifies — without creating files — that
# a broad battery of secret/runtime names (nested and extensionless included) is
# ignored and untracked, that ONLY the pinned negation (re-include) set exists
# (update the pin in the SAME change as any .gitignore allowlist edit), that no
# probe under claude/agents/ is addable beside the single pinned agent file, and
# that the tracked tree matches no secret pattern (SQLite/DB journal variants
# included). Probe files are created only for the agents addable-check, never over
# a pre-existing path or symlink, and only those creations are cleaned up.
#
# SCOPE (accepted residual, maintainer decision): the guard checks NAMES and
# TRACKABILITY, not file contents — a credential pasted into an allowlisted file
# is invisible to it. Content-aware scanners were declined to keep this repo
# dependency-free; GitHub's public-repo secret scanning / push protection is the
# content-level backstop. Never paste secrets into tracked files.
set -euo pipefail
LC_ALL=C; export LC_ALL   # byte-oriented matching: invalid UTF-8 must not evade the scans
fail() { printf '  ✗ FAIL: %s\n' "$*" >&2; exit 1; }
pass() { printf '  ✓ PASS: %s\n' "$*"; }
cd "$(git rev-parse --show-toplevel)"

# ── 0. Pin the ignore-rule sources themselves ──────────────────────────────────
# Staged .gitignore must equal the working-tree one byte-for-byte, or the guard
# would bless a tree while the commit records different rules. Compare the index
# BLOB against worktree bytes directly — `git diff --quiet` trusts the stat cache,
# so an assume-unchanged/skip-worktree bit could mask a divergent staged copy.
git show :.gitignore 2>/dev/null | cmp -s - .gitignore \
  || fail ".gitignore content differs between index and working tree (or is untracked) — stage or restore it before running the guard"
# The guard must itself be what the commit records: a staged deletion or weakened
# staged copy of this file must not be maskable by a safe worktree copy (this is
# the repo's only retained control). Intentional edits pass once the same bytes
# are staged. Bytes alone are not enough: a staged SYMLINK entry (mode 120000)
# can reuse the safe blob while the committed file becomes an ignore-dead symlink
# (git does not follow symlinked .gitignore files) — so both policy files must be
# exactly one stage-0 REGULAR-file index entry before the byte compare counts.
policy_entry_ok() {  # $1 = path: exactly one stage-0 regular-file (100644/100755) entry
  local lines
  lines="$(git ls-files -s -- "$1" | awk '{print $1, $3}')"
  [ "$lines" = "100644 0" ] || [ "$lines" = "100755 0" ]
}
policy_entry_ok .gitignore \
  || fail ".gitignore index entry is not a single stage-0 regular file (symlink/gitlink/unmerged staged?)"
policy_entry_ok tests/secret-guard.sh \
  || fail "tests/secret-guard.sh index entry is not a single stage-0 regular file (symlink/gitlink/unmerged staged?)"
git show :tests/secret-guard.sh 2>/dev/null | cmp -s - tests/secret-guard.sh \
  || fail "tests/secret-guard.sh content differs between index and working tree (or is not tracked) — stage the guard itself before trusting its verdict"
# Effectively one ignore file may exist: the root .gitignore (plus the single
# close-only exemption spelled out below). A nested one takes precedence for its
# subtree and can reopen protected paths without touching the pinned root text —
# in the worktree or already staged.
# Case-insensitive on purpose: on a case-insensitive filesystem git's lookup of
# ".gitignore" resolves ".GITIGNORE"/".GitIgnore" too, so any ASCII-case variant is
# an ignore source there — reject them everywhere so commits transfer across machines.
# Diagnostics %q-escape every discovered pathname: these are untrusted bytes and
# must not reach the terminal raw (control/OSC sequences).
nested_wt=""
while IFS= read -r -d '' path; do
  [ "$path" = "./.gitignore" ] && continue
  # ONE exemption: dstack's runtime store isolates itself with a nested ignore file
  # (the pattern pytest uses for .pytest_cache), so a blanket refusal cannot stand.
  # The hazard a nested .gitignore poses is REOPENING a protected path, and reopening
  # requires a negation — a file whose entire content is `*` can only ever close. So
  # this exact path is exempt only while it stays exactly that, byte for byte, and is
  # not a symlink. Any other content, any case variant, any other nested path: still
  # fails. It must also never be staged; the index scan below is deliberately left
  # strict, so `git add -f .dstack/.gitignore` is still caught.
  # BYTE-exact: `$(cat)` strips every trailing newline, so `*`, `*\n` and `*\n\n\n` all compared
  # equal to `*`. The exemption is only for the one file whose entire content is the isolation
  # pattern; anything else nested is still refused.
  # `-f` as well as `! -L`: a FIFO passes the not-a-symlink test, and `wc`/`cat` on one BLOCK
  # until something writes — the guard would hang instead of refusing.
  if [ "$path" = "./.dstack/.gitignore" ] && [ ! -L "$path" ] && [ -f "$path" ] \
     && [ "$(wc -c < "$path" | tr -d ' ')" = "2" ] && [ "$(cat "$path")" = '*' ]; then
    continue
  fi
  nested_wt="$nested_wt $(printf '%q' "$path")"
done < <(find . -iname .gitignore -not -path './.git/*' -print0 2>/dev/null)
[ -z "$nested_wt" ] || fail "nested or case-variant .gitignore in working tree (can reopen protected paths):$nested_wt"
nested_ix=""
while IFS= read -r -d '' path; do
  lc="$(printf '%s' "$path" | tr '[:upper:]' '[:lower:]')"
  case "$lc" in
    .gitignore) [ "$path" = ".gitignore" ] || nested_ix="$nested_ix $(printf '%q' "$path")" ;;
    */.gitignore) nested_ix="$nested_ix $(printf '%q' "$path")" ;;
  esac
done < <(git ls-files -z)
[ -z "$nested_ix" ] || fail "nested or case-variant .gitignore in the index:$nested_ix"
# Local per-machine rules could make a probe look ignored here while other
# machines (or CI) see it trackable: reject info/exclude content and run every
# check-ignore with the global excludes file neutralized.
info_exclude="$(git rev-parse --git-path info/exclude)"
if [ -f "$info_exclude" ] && grep -qvE '^[[:space:]]*(#|$)' "$info_exclude"; then
  fail "$info_exclude carries local ignore rules — the guard's verdict would not transfer to other machines"
fi
gci() { git -c core.excludesFile=/dev/null check-ignore -q -- "$1"; }

# ── 1. Ignore battery (no file creation — pattern checks need no files) ────────
# Paths that MUST be ignored. Includes the holes earlier reviews caught, plus
# SQLite/DB journal variants (data.db-wal, y.sqlite-shm).
leaks=(
  claude/auth.json codex/config.toml claude/x.sqlite claude/x.sqlite-wal
  claude/y.sqlite-shm claude/data.db claude/data.db-wal claude/data.sqlite3
  claude/history.jsonl claude/.DS_Store
  claude/.env codex/.env.local claude/deploy.key codex/x.pem claude/x.p12 claude/y.pfx
  claude/id_rsa claude/id_ed25519 claude/.netrc claude/credentials.json
  claude/secrets.token claude/api_token claude/x.secret
  claude/hooks/auth.json claude/skills/full-cycle/credentials.json
  claude/hooks/random_unknownfile claude/hooks/deploy_key_prod
  codex/rules/random_unknownfile claude/skills/novel_secret_dir/blob
  claude/skills/full-cycle/api_token claude/skills/full-cycle/deploy_key_prod
  claude/skills/full-cycle/cache.sqlite3-wal claude/skills/full-cycle/cache.db3-wal
  claude/skills/full-cycle/api_token.json claude/skills/full-cycle/sessions/state.json
  claude/skills/full-cycle/memory/notes.md claude/skills/full-cycle/projects/p/x.md
  claude/skills/full-cycle/private.pem.bak claude/skills/full-cycle/secrets.token.old
  claude/skills/full-cycle/api_token.json.bak claude/skills/full-cycle/cache.db.bak
  claude/skills/full-cycle/api_token/payload.txt
  claude/skills/full-cycle/private.pem.backup-2026
  claude/skills/full-cycle/cache.db-wal.backup-2026
  claude/skills/full-cycle/password-list.txt
  claude/skills/full-cycle/auth.json.bak claude/skills/full-cycle/config.toml.old
  claude/skills/full-cycle/.netrc.backup claude/skills/full-cycle/id_rsa.pub
  claude/skills/full-cycle/history.jsonl.gz
)
for f in "${leaks[@]}"; do
  gci "$f" || fail "secret NOT ignored by .gitignore: $f"
  if git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
    fail "secret present in git index (already tracked): $f"
  fi
done

# ── 2. Behavioral addable-check under claude/agents/ (the one place real files
# are needed: `git ls-files -o` only sees paths that exist). Never touch a
# pre-existing path or symlink; clean up exactly what was created. ─────────────
agent_probes=(
  claude/agents/random_unknownfile claude/agents/auth.json
  claude/agents/unknown-agent.md claude/agents/nested/inner-agent.md
  claude/agents/frontend-xyz.md claude/agents/f.md
)
created=(); created_dirs=()
cleanup() {
  [ "${#created[@]}" -gt 0 ] && rm -f -- "${created[@]}"
  [ "${#created_dirs[@]}" -gt 0 ] && rmdir -- "${created_dirs[@]}" 2>/dev/null
  return 0
}
trap cleanup EXIT
for f in "${agent_probes[@]}"; do
  # Reject a symlink at ANY component: a symlinked ancestor dir would let the
  # redirection below create/remove files outside the repository.
  IFS='/' read -r -a segs <<< "$f"
  p=""
  for s in "${segs[@]}"; do
    p="${p:+$p/}$s"
    [ -L "$p" ] && fail "symlink in probe path — refusing to touch it: $p"
  done
  if [ -e "$f" ]; then
    fail "probe path unexpectedly exists — refusing to touch it: $f"
  fi
  d="$(dirname "$f")"
  if [ ! -d "$d" ]; then
    mkdir -- "$d" || fail "cannot create probe dir: $d"
    created_dirs+=("$d")
  fi
  # noclobber: refuse to truncate anything that appeared since the check above.
  # (Residual, accepted: a component swapped for a symlink between check and write
  # is a same-instant local race on a single-user machine.)
  ( set -C; : > "$f" ) 2>/dev/null || fail "probe creation refused (file appeared concurrently?): $f"
  created+=("$f")
  gci "$f" || fail "secret NOT ignored by .gitignore: $f"
done
# Agent .md files are executable instruction material: with the probes on disk,
# git must see NOTHING addable under claude/agents/ except the single pinned file —
# this trips on ANY spelling of a re-include (rooted, unrooted, glob).
extra=""
while IFS= read -r -d '' path; do
  [ "$path" = "claude/agents/frontend-dev.md" ] && continue
  [ "$path" = "claude/agents/general-dev.md" ] && continue
  extra="$extra $(printf '%q' "$path")"
done < <(git -c core.excludesFile=/dev/null ls-files -o --exclude-standard -z claude/agents/)
[ -z "$extra" ] || fail "unexpected addable files under claude/agents/:$extra"

# ── 3. The ignore policy is PINNED, twice ──────────────────────────────────────
# (a) Whole-content hash: ANY byte change to .gitignore — including removing a
# hard-deny rule for which no file currently exists on disk (nothing for the
# battery or the addable scan to trip on) — fails until the pin is deliberately
# updated in the same reviewed change. (b) The negation set is additionally pinned
# line-by-line below for a readable diff in the common allowlist-edit case.
GITIGNORE_SHA_PIN='0eb7358a968ea64b8b8fa1b791745cb8ee775c1b82e3f7623df67522ed01be4d'
got_sha="$(shasum -a 256 .gitignore | awk '{print $1}')"
[ "$got_sha" = "$GITIGNORE_SHA_PIN" ] \
  || fail ".gitignore content drifted from the pinned hash — review the change, then update GITIGNORE_SHA_PIN in tests/secret-guard.sh in the same commit (got $got_sha)"

# ── 3b. The allowlist is a CLOSED set ──────────────────────────────────────────
# Every negation (re-include) line in .gitignore must be one of the expected,
# consciously-added entries below, in order. This rejects ANY new `!` rule —
# whatever its spelling, root, or glob (`!claude/agents/f*.md`, `!/claude/**/z*.md`,
# …) — until it is deliberately added here alongside its review. Finite probe
# batteries cannot close the pattern space; pinning the rule set itself does.
expected_negations='!/.gitignore
!/AGENTS.md
!/CLAUDE.md
!/README.md
!/THIRD-PARTY-NOTICES.md
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
!/claude/agents/general-dev.md
!/claude/bin/
!/claude/bin/dstack
!/codex/.gitkeep
!/codex/AGENTS.md
!/codex/instructions.md
!/codex/rules/
!/codex/rules/default.rules
!/codex/skills/
!/codex/skills/adversarial-review/
!/codex/skills/adversarial-research/
!/codex/skills/socratic-audit/
!/gemini/.gitkeep
!/gemini/README.md'
got_negations="$(grep '^!' .gitignore)"
[ "$got_negations" = "$expected_negations" ] \
  || fail ".gitignore negation set drifted from the pinned allowlist:
$(diff <(printf '%s\n' "$expected_negations") <(printf '%s\n' "$got_negations") | tr -c '[:print:]\n\t' '?' || true)"

# ── 4. The tracked tree (index) must contain no secret pattern ─────────────────
# ONE component-level matcher drives both this scan and the addable scan below,
# mirroring .gitignore's own semantics: a glob like `**/*_token` matches a DIRECTORY
# component too, so every path COMPONENT is matched, not just the trailing filename
# (`api_token/payload.txt` is protected via its dir component). Suffix chains accept
# any characters (`private.pem.backup-2026`), matching the `*.pem.*`-style denies.
# Case-insensitive at scan time; NUL-safe per-path loop (ls-files' C-quoting of
# newline pathnames would otherwise defeat the anchors).
COMP_RE='^auth\.json(\..*)?$|credentials|password|^\.netrc(\..*)?$|^id_rsa(\..*)?$|^id_dsa(\..*)?$|^id_ecdsa(\..*)?$|^id_ed25519(\..*)?$|^history\.jsonl(\..*)?$|^config\.toml(\..*)?$|^\.DS_Store(\..*)?$|deploy_key|_token(\..*)?$|\.(pem|key|p12|pfx|token|secret)(\..*)?$|\.(db[0-9]?|sqlite[0-9]?)([.-].*)?$|^\.env(\..*)?$|^(sessions|projects|memory)$'
scan_components() {  # $1 = pathname, $2 = failure label
  # Pure parameter-expansion split: `read`-based splitting stops at an embedded
  # newline and would silently skip the components after it. Matching uses bash
  # =~ on the WHOLE component (an embedded newline stays one component — a
  # line-oriented grep would split it into two unmatchable records), with
  # nocasematch for case-insensitivity and LC_ALL=C for byte semantics.
  # Diagnostics are %q-escaped: pathnames are untrusted bytes and must not reach
  # the terminal raw (control/OSC sequences).
  local rest="$1" seg
  shopt -s nocasematch
  while :; do
    seg="${rest%%/*}"
    if [[ $seg =~ $COMP_RE ]]; then
      shopt -u nocasematch
      fail "$2 (protected name component $(printf '%q' "$seg")): $(printf '%q' "$1")"
    fi
    [ "$rest" = "$seg" ] && break
    rest="${rest#*/}"
  done
  shopt -u nocasematch
}
while IFS= read -r -d '' path; do
  scan_components "$path" "secret pattern present in tracked files"
done < <(git ls-files -z)

# ── 5. No sensitive-named file may be ADDABLE anywhere (case-insensitive) ──────
# .gitignore pattern matching is case-sensitive, so an upper/mixed-case variant
# (CACHE.SQLITE3-WAL) inside a wholesale-allowed subtree would be trackable while
# every lowercase pattern and probe stays green. Scan the actual untracked addable
# paths with the same component matcher, NUL-safely. (The physical agent probes are
# ignored files, so they never appear here.)
while IFS= read -r -d '' path; do
  scan_components "$path" "sensitive-named file is addable (ignore rules miss it)"
done < <(git -c core.excludesFile=/dev/null ls-files -o --exclude-standard -z)

pass "secret guard"
