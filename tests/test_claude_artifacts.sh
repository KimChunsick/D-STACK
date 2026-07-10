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
# Pins are asserted on the EXECUTABLE command line (the `-m … \` continuation row inside the
# fenced block), not on prose bullets — a pin dropped from the command must fail even if an
# explanatory bullet still quotes it. Research pins the cheaper-by-API-token-pricing
# gpt-5.5 (credit units unverified; the split itself is the maintainer's directive —
# review pins gpt-5.6-sol below) at xhigh.
assert_matches '^[[:space:]]+-m gpt-5\.5 -c model_reasoning_effort="xhigh" \\$' "$cr"
assert_not_matches '\-m gpt-5\.6'        "$cr"  # Sol is the review model, not the research one
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
# The review call itself must pin model+effort AND run isolated (it used to rely on config
# drift, at effort *none*). Anchored to the executable `codex exec …` line — model+effort
# adjacency right before the prompt string — so a pin dropped from the command fails even if
# prose still quotes it. Review runs the frontier gpt-5.6-sol; research runs gpt-5.5
# (cheaper by API-token pricing, credit units unverified; split = maintainer's directive).
assert_matches '^codex exec .*-s read-only -C "\$SCRATCH" .*-m gpt-5\.6-sol -c model_reasoning_effort="xhigh" "You are' "$crv"
assert_not_matches '\-m gpt-5\.5'                "$crv"
# No alternate override spellings that would beat the asserted `-m` pin at runtime.
# Ban the bare substring `model=`: it catches `--model=`, `-c model=`, attached
# `-cmodel=`, and QUOTED forms like `-c 'model=o3'` (which produce the same argv as the
# unquoted form but evade flag-shaped patterns). Neither skill legitimately contains it
# (`model_reasoning_effort=` has no `model=` substring). `--config` banned wholesale.
assert_not_matches '\-\-model[ =]|model=|\-\-config' "$crv"
assert_not_matches '\-\-model[ =]|model=|\-\-config' "$cr"
# The bundle is untrusted data: the prompt must say so (prompt-injection guard), and the
# scratch dirs must not leak (cleanup trap on the executable line).
assert_matches 'UNTRUSTED DATA'                  "$crv"
assert_matches '^SCRATCH="\$\(mktemp -d\)"; trap' "$crv"
# Effective-override guard: the pin being PRESENT is not enough — a SECOND `-m`/effort flag
# later on the command line (even after the prompt string; clap lets trailing flags win)
# would silently beat the pin at runtime while every pattern above stays green. So inside
# each skill's fenced code blocks the model flag and the effort flag must appear EXACTLY
# once. (Fenced blocks only: prose bullets quoting flags are explanation, not execution.)
# Count OCCURRENCES (grep -o|wc -l), not matching lines (grep -c): the exec command is one
# long line, so a second flag on the same line would keep a line count at 1.
fenced() { awk '/^```/{f=!f; next} f' "$1"; }
for skill in "$cr" "$crv"; do
  # ANY -m flag counts (not just `-m gpt-`): `-m o3`, `-m "$MODEL"`, or the ATTACHED
  # short-option forms `-mVALUE`/`-m=VALUE` (clap accepts them) would otherwise evade
  # both this count and the cross-model bans while winning at runtime. No trailing-space
  # requirement; `--model` can't false-positive (its `-m` is preceded by `-`, not space).
  n="$(fenced "$skill" | grep -oE '(^|[[:space:]])-m' | wc -l | tr -d ' ')"
  [ "$n" = 1 ] || fail "$skill: expected exactly one '-m' flag in fenced blocks, got $n"
  n="$(fenced "$skill" | grep -o -- 'model_reasoning_effort' | wc -l | tr -d ' ')"
  [ "$n" = 1 ] || fail "$skill: expected exactly one effort flag in fenced blocks, got $n"
done

# ultracode-by-default: every interactive `claude` launch must opt into ultracode via the
# alias fragment. This is the ONLY persistence layer — upstream, ultracode is session-scoped
# by design: the persisted effortLevel setting and CLAUDE_CODE_EFFORT_LEVEL reject it, and a
# bare `"ultracode": true` in settings.json is silently ignored (anthropics/claude-code#64817).
# `--effort ultracode` needs claude >= 2.1.203. Asserted on the exact executable alias line.
uz=claude/ultracode.zsh
[ -s "$uz" ] || fail "missing or empty: $uz"
assert_matches "^alias claude='claude --effort ultracode'$" "$uz"
# The alias must be the fragment's ONLY executable content: a silent side-effecting
# command added between the comments would run at every shell startup while the alias
# assertions stay green. Non-comment, non-blank lines must equal exactly the alias line.
execlines="$(grep -v '^[[:space:]]*#' claude/ultracode.zsh | grep -v '^[[:space:]]*$' || true)"
[ "$execlines" = "alias claude='claude --effort ultracode'" ] \
  || fail "ultracode.zsh has executable content beyond the alias line: '$execlines'"
# Behavioral, not just lexical: the fragment must SOURCE cleanly and yield exactly the
# intended effective alias — a later `unalias`, an override, or a syntax error could keep
# the asserted line present while changing runtime behavior. `-i` matters: the real
# consumer is an INTERACTIVE zsh (.zshrc), so an interactive-only override
# (`[[ -o interactive ]] && unalias claude`) must not slip past a non-interactive probe.
# zsh is a hard dependency on the target platform (macOS ships it); fail loud, don't skip.
command -v zsh >/dev/null 2>&1 || fail "zsh not found (required to verify ultracode.zsh)"
got="$(zsh -f -ic 'source claude/ultracode.zsh && alias claude' 2>/dev/null)" \
  || fail "ultracode.zsh failed to source under interactive zsh -f -i"
[ "$got" = "claude='claude --effort ultracode'" ] || fail "effective alias mismatch: got '$got'"
assert_contains .gitignore '!/claude/ultracode.zsh'
# Effective ignore status, not just the allowlist line: a LATER rule can override the
# `!`-entry while the lexical assertion above stays green. Ask git itself. `--no-index`
# matters: without it, check-ignore reports nothing for a TRACKED file, so the guard
# would go inert the moment the artifact is committed.
if git check-ignore --no-index -q claude/ultracode.zsh; then
  fail "claude/ultracode.zsh is gitignored (allowlist line overridden by a later rule)"
fi
assert_matches '^claude/ultracode\.zsh\|\.claude/ultracode\.zsh\|link$' install.sh

# frontend-dev subagent: the dedicated frontend implementer. Must exist, carry valid
# frontmatter (name is the identity hooks/routing see, description drives delegation),
# steer ALL frontend code work to itself, and stay vendor-neutral — same
# plugin/marketplace/affiliation hygiene bar as settings.json below.
fa=claude/agents/frontend-dev.md
[ -s "$fa" ] || fail "missing or empty: $fa"
sed -n 1p "$fa" | grep -qx -- '---' || fail "$fa: must open with YAML frontmatter"
# Identity + delegation steer must live INSIDE a properly CLOSED frontmatter block —
# the same strings placed in the body (or after a missing closing delimiter) must fail.
fm="$(awk 'NR==1{next} /^---$/{c=1; exit} {print} END{if(!c) exit 1}' "$fa")" \
  || fail "$fa: frontmatter never closes"
printf '%s\n' "$fm" | grep -qE '^name: frontend-dev$' || fail "$fa: frontmatter lacks 'name: frontend-dev'"
printf '%s\n' "$fm" | grep -qE '^description: ..*MUST BE USED' \
  || fail "$fa: description lacks the MUST BE USED delegation steer"
# Frontmatter is a closed shape, not just present fields: every line must be one of the
# two known single-line keys — a stray/malformed line (e.g. 'broken: [') fails loud here
# instead of failing silently at agent discovery.
if printf '%s\n' "$fm" | grep -vE '^(name|description): .' | grep -q .; then
  fail "$fa: frontmatter contains lines beyond the known name/description keys"
fi
# The calibrated substance must survive: every top-level section must OPEN and CLOSE,
# and the FULL rule bands (M1–M9, S1–S15, P1–P2) must be present — deleting any
# calibrated rule, not just the band endpoints, trips this.
for sec in philosophy precedence decision_algorithms rules examples \
           stack workflow self_review reporting; do
  assert_contains "$fa" "<$sec>"
  assert_contains "$fa" "</$sec>"
done
for i in 1 2 3 4 5 6 7 8 9;                       do assert_matches "id=\"M$i\"" "$fa"; done
for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15;     do assert_matches "id=\"S$i\"" "$fa"; done
for i in 1 2;                                     do assert_matches "id=\"P$i\"" "$fa"; done
assert_not_matches 'superpowers|enabledPlugins|apps-in-toss|extraKnownMarketplaces|toss' "$fa"
# SSOT wiring: allowlisted by exact name (agents/ stays deny-all inside), effectively
# tracked (a later rule must not override the `!`-entry), linked by install.sh, and the
# global CLAUDE.md must carry the mandatory routing rule that sends frontend work here.
assert_contains .gitignore '!/claude/agents/frontend-dev.md'
if git check-ignore --no-index -q "$fa"; then
  fail "$fa is gitignored (allowlist line overridden by a later rule)"
fi
# Structural exactness, not just probes: agent .md files are executable instruction
# material, so the ONLY permitted agents-path negations are the directory itself and the
# exact pinned file — any other re-include (e.g. '!/claude/agents/a*.md') fails here even
# if every ignored-name probe still passes.
if grep -E '^!' .gitignore | grep -F 'agents' \
   | grep -vxE '!/claude/agents/|!/claude/agents/frontend-dev\.md' | grep -q .; then
  fail ".gitignore: agents-touching negation beyond the exact two allowlist lines"
fi
assert_matches '^claude/agents/frontend-dev\.md\|\.claude/agents/frontend-dev\.md\|link$' install.sh
# One co-located phrase, not two independent greps — 'frontend-dev' somewhere plus an
# unrelated 'MUST be delegated' elsewhere must not satisfy the routing-rule guard.
assert_matches 'MUST be delegated to the .frontend-dev. subagent' claude/CLAUDE.md

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
