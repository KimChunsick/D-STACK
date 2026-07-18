#!/usr/bin/env bash
# Behavioral test of the Stop hook (the mechanical core of full-cycle enforcement).
# Covers: section-scoped parsing (no deadlock on the milestone checklist), milestone↔gate tie,
# task-requires-Goal, one-Goal, Codex-artifact requirement, and the escape hatch.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
REPO="$(git rev-parse --show-toplevel)"
HOOK="$REPO/claude/hooks/fullcycle-gate.sh"
[ -f "$HOOK" ] || fail "hook missing: $HOOK"
command -v jq >/dev/null 2>&1 || fail "jq required"

SBX="$(mktemp -d)"; trap 'rm -rf "$SBX"' EXIT
cd "$SBX"
blocks()    { printf '%s' "$1" | grep -qE '"decision":[[:space:]]*"block"'; }

mkdir -p docs/g/M1/T01
GOAL=docs/g/GOAL.md
REVIEW_DIR=docs/g/M1/T01
REVIEW1="$REVIEW_DIR/codex-review-001.md"
REVIEW2="$REVIEW_DIR/codex-review-002.md"

# ---- Case 1: unchecked Goal gate ⇒ BLOCK ----
printf '# GOAL\n## Goal gate\n- [ ] GOAL E2E: pending\n' > "$GOAL"
printf '%s\n' "$GOAL" > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C1: did not block on unchecked Goal gate"

# ---- Case 2 (THE regression): unchecked CHECKLIST items above the gate must NOT deadlock,
#      when the Goal gate + every milestone E2E are ticked. (Section-scoped parsing.) ----
cat > "$GOAL" <<'EOF'
# GOAL
## Milestones & tasks (Phase 5)
### M1 — foo
- [ ] T01 a future task, still unchecked
### M2 — bar
- [ ] T02 another unchecked checklist item
## Goal gate
- [x] M1 E2E: done
- [x] M2 E2E: done
- [x] GOAL E2E: done
EOF
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C2: deadlocked on checklist items above the Goal gate (section-scoping broken)"

# ---- Case 3: milestone tie — M2 heading exists but no ticked 'M2 E2E' ⇒ BLOCK ----
cat > "$GOAL" <<'EOF'
# GOAL
### M1 — foo
### M2 — bar
## Goal gate
- [x] M1 E2E: done
- [x] GOAL E2E: done
EOF
blocks "$(bash "$HOOK")" || fail "C3: did not block when a milestone lacks its ticked E2E gate"

# ---- Case 4: task active WITHOUT a registered GOAL.md ⇒ BLOCK ----
printf '# t\n## Gate status\n- [x] TDD\n' > docs/g/M1/T01/task.md
printf '%s\n' docs/g/M1/T01/task.md > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C4: did not block on a task with no registered Goal"

# ---- Case 5: ticked Codex gate but NO numbered review round ⇒ BLOCK (artifact requirement) ----
printf '# t\n## Gate status\n- [x] TDD\n- [x] Codex consensus\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '%s\n%s\n' "$GOAL" docs/g/M1/T01/task.md > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C5: did not block when Codex gate ticked without a numbered review round"

# ---- Case 6: add codex-review-001.md with consensus ⇒ PASS ----
printf '## Consensus\n- Consensus: agreed\n' > "$REVIEW1"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C6: blocked despite agreed first review round + all gates ticked"

# ---- Case 7: two GOAL.md active ⇒ BLOCK (exactly one Goal) ----
mkdir -p docs/g2; printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > docs/g2/GOAL.md
printf '%s\n%s\n' "$GOAL" docs/g2/GOAL.md > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C7: did not block on more than one active Goal"

# ---- Case 8: escape hatch — empty active ⇒ PASS even though docs still have '- [ ]' ----
printf '# GOAL\n## Goal gate\n- [ ] GOAL E2E: pending\n' > "$GOAL"
: > .fullcycle-active
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C8: blocked after docs removed from .fullcycle-active (escape hatch broken)"

# ---- Case 9: schema fail-closed — a registered GOAL.md with NO '## Goal gate' ⇒ BLOCK ----
printf '# GOAL\n## Milestones\n- [x] all done\n' > "$GOAL"
printf '%s\n' "$GOAL" > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C9: a GOAL.md missing the gate schema bypassed enforcement"

# ---- Case 10: Goal gate present but NO 'GOAL E2E' box ⇒ BLOCK (final Goal E2E can't be dropped) ----
printf '# GOAL\n## Goal gate\n- [x] M1 E2E: done\n' > "$GOAL"
blocks "$(bash "$HOOK")" || fail "C10: a Goal gate without a 'GOAL E2E' box bypassed the final-Goal-E2E requirement"

# ---- Case 11: prose 'GOAL E2E' but NO checkbox row ⇒ BLOCK (schema must be a real gate row) ----
printf '# GOAL\n## Goal gate\nThe GOAL E2E is described here in prose, not a checkbox.\n' > "$GOAL"
blocks "$(bash "$HOOK")" || fail "C11: prose 'GOAL E2E' (no checkbox row) bypassed the schema gate"

# ---- Case 12 (THE regression): a ticked Codex gate whose Consensus is a NEGATIVE verdict must
#      BLOCK. The old '.*(agreed|resolved)' matched 'disagreed' (substring), 'unresolved'
#      (substring), and 'agreed was not reached' (negation trailing) — all now rejected. ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '# t\n## Gate status\n- [x] TDD\n- [x] Codex consensus\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '%s\n%s\n' "$GOAL" docs/g/M1/T01/task.md > .fullcycle-active
for verdict in 'disagreed' 'unresolved' 'agreed was not reached' 'not agreed' 'no consensus reached'; do
  printf '## Consensus\n- Consensus: %s\n' "$verdict" > "$REVIEW1"
  blocks "$(bash "$HOOK")" || fail "C12: negative consensus '$verdict' bypassed the Codex gate"
done

# ---- Case 13: 'resolved' (the other legitimate positive verdict) ⇒ PASS ----
printf '## Consensus\n- Consensus: resolved\n' > "$REVIEW1"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C13: blocked despite a genuine 'resolved' consensus"

# ---- Case 14: task fail-closed — a registered task with NO '## Gate status' section ⇒ BLOCK ----
printf '# t\nsome prose describing the task, but no gate section at all.\n' > docs/g/M1/T01/task.md
blocks "$(bash "$HOOK")" || fail "C14: a task missing '## Gate status' bypassed enforcement"

# ---- Case 15: task '## Gate status' present but prose-only (NO checkbox rows) ⇒ BLOCK ----
printf '# t\n## Gate status\nEverything is fine, trust me — no boxes here.\n' > docs/g/M1/T01/task.md
blocks "$(bash "$HOOK")" || fail "C15: prose-only task Gate status bypassed enforcement"

# ---- Case 16 (THE per-round regression): every consensus attempt has its own file. The
#      latest canonical round is authoritative, so round 002 'disagreed' must BLOCK even
#      though round 001 is 'agreed'. A stale positive file must never win. ----
printf '# t\n## Gate status\n- [x] TDD\n- [x] Codex consensus\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '# Review round 001\n- Consensus: agreed\n' > "$REVIEW1"
printf '# Review round 002\nA blocker reappeared.\n- Consensus: disagreed\n' > "$REVIEW2"
blocks "$(bash "$HOOK")" || fail "C16: stale earlier positive round overrode the latest negative round"

printf '# Review round 001\n- Consensus: disagreed\n' > "$REVIEW1"
printf '# Review round 002\n- Consensus: resolved\n' > "$REVIEW2"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C16: latest resolved round did not supersede an earlier rejection"

printf '# Review round 001\n- Consensus: disagreed\n- Consensus: agreed\n' > "$REVIEW1"
blocks "$(bash "$HOOK")" || fail "C16: an older file containing two appended rounds was ignored"

# Fixed-width lexical order must choose the authoritative round (010 > 009).
rm -f "$REVIEW_DIR"/codex-review-[0-9]*.md
for n in 001 002 003 004 005 006 007 008; do
  printf '# Review round %s\n- Consensus: disagreed\n' "$n" > "$REVIEW_DIR/codex-review-$n.md"
done
printf '# Review round 009\n- Consensus: agreed\n' > "$REVIEW_DIR/codex-review-009.md"
printf '# Review round 010\n- Consensus: disagreed\n' > "$REVIEW_DIR/codex-review-010.md"
blocks "$(bash "$HOOK")" || fail "C16: round 009 overrode negative round 010"

# ---- Case 17: consensus is a strict verdict-only WHITELIST. Clean positives PASS; the same
#      verdict with ANY trailing prose (where a negation could hide) is rejected. ----
rm -f "$REVIEW_DIR"/codex-review-[0-9]*.md
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '# t\n## Gate status\n- [x] TDD\n- [x] Codex consensus\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '%s\n%s\n' "$GOAL" docs/g/M1/T01/task.md > .fullcycle-active
for ok in '- Consensus: agreed' '- Consensus: resolved' '**Consensus:** AGREED' 'Consensus: agreed.' 'Consensus: agreed ✅' '> Consensus: resolved' '✅ Consensus: agreed'; do
  printf '# Review\n%s\n' "$ok" > "$REVIEW1"
  out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C17: clean positive consensus '$ok' was wrongly blocked"
done
# Trailing prose / negation smuggling / glyph-prefixed negatives must all BLOCK.
for bad in 'agreed to reject the PR' 'resolved as WONTFIX' 'agreed, but tests still fail' 'agreed 거부' 'resolved 拒否' '❌ disagreed'; do
  printf '# Review\n- Consensus: %s\n' "$bad" > "$REVIEW1"
  blocks "$(bash "$HOOK")" || fail "C17: non-clean/negation-smuggling consensus '$bad' bypassed the gate"
done

printf '# Review\n- Consensus: agreed\n❌ Consensus: disagreed\n' > "$REVIEW1"
blocks "$(bash "$HOOK")" || fail "C17: glyph-prefixed second Consensus line bypassed one-round enforcement"

# Consensus must seal the file as its final nonblank line. Trailing blank lines are allowed,
# while any later prose means the round was appended after its verdict and must block.
printf '# Review\n- Consensus: agreed\n\n' > "$REVIEW1"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C17: trailing blank lines after a sealed consensus were wrongly blocked"
printf '# Review\n- Consensus: agreed\nBlocker remains unresolved.\n' > "$REVIEW1"
blocks "$(bash "$HOOK")" || fail "C17: prose appended after a positive consensus bypassed sealing"

# A round file represents exactly one exchange. Multiple Consensus lines indicate that
# another round was appended to the same file and must be rejected mechanically.
printf '# Review\n- Consensus: disagreed\n## Appended re-review\n- Consensus: agreed\n' > "$REVIEW1"
blocks "$(bash "$HOOK")" || fail "C17: multiple consensus rounds appended to one file bypassed the gate"

# ---- Case 18: any GFM marker ('* '/'+ '), indentation, or double-spacing must not hide an
#      unchecked box ⇒ BLOCK. ----
printf '# Review\n- Consensus: agreed\n' > "$REVIEW1"   # isolate the box check
for box in ' - [ ] indented not done' '-  [ ] double-spaced not done' '* [ ] star marker not done' '+ [ ] plus marker not done'; do
  printf '# t\n## Gate status\n- [x] TDD\n%s\n' "$box" > docs/g/M1/T01/task.md
  blocks "$(bash "$HOOK")" || fail "C18: unchecked box '$box' evaded the checkbox anchor"
done

# ---- Case 19: a milestone heading dodging via hash-count ('####'/'##') or casing ('m1') still
#      requires its 'M<n> E2E' Goal-gate box ⇒ BLOCK. ----
printf '# t\n## Gate status\n- [x] TDD\n' > docs/g/M1/T01/task.md
printf '# Review\n- Consensus: agreed\n' > "$REVIEW1"
for head in '###  M1 double-space' '#### M1 as h4' '## M1 as h2' '### m1 lowercase' '# M1 as h1'; do
  printf '# Goal title\n%s\n## Goal gate\n- [x] GOAL E2E: done\n' "$head" > "$GOAL"   # no 'M1 E2E' box
  blocks "$(bash "$HOOK")" || fail "C19: milestone heading '$head' dodged its required 'M1 E2E' box"
done

# ---- Case 20: the numbered-review requirement is UNCONDITIONAL — a task whose latest verdict
#      is negative (or whose review box is relabeled to hide it) still BLOCKS. A legacy singleton
#      is not accepted, because accepting it would let the accumulating format survive. ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '# t\n## Gate status\n- [x] TDD\n- [x] GPT-5.5 hostile critique passed\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '# Review\n- Consensus: disagreed, blockers remain\n' > "$REVIEW1"
blocks "$(bash "$HOOK")" || fail "C20a: a relabeled review box with a NEGATIVE consensus bypassed the gate"
rm -f "$REVIEW_DIR"/codex-review-[0-9]*.md
printf '# Legacy review\n- Consensus: agreed\n' > "$REVIEW_DIR/codex-review.md"
blocks "$(bash "$HOOK")" || fail "C20b: the legacy accumulating codex-review.md bypassed the numbered-round requirement"
rm -f "$REVIEW_DIR/codex-review.md"
blocks "$(bash "$HOOK")" || fail "C20c: a task with NO numbered review round bypassed the unconditional requirement"

# A malformed, empty, or symlinked latest-looking round must fail closed instead of falling
# back to the previous positive round.
printf '# Review\n- Consensus: agreed\n' > "$REVIEW1"
: > "$REVIEW2"
blocks "$(bash "$HOOK")" || fail "C20d: empty latest round fell back to an earlier positive"
rm -f "$REVIEW2"; ln -s codex-review-001.md "$REVIEW2"
blocks "$(bash "$HOOK")" || fail "C20e: symlinked latest round fell back to an earlier positive"
rm -f "$REVIEW2"
printf '# Review\n- Consensus: agreed\n' > "$REVIEW_DIR/codex-review-004.md"
blocks "$(bash "$HOOK")" || fail "C20f: a numbering gap was accepted"
rm -f "$REVIEW_DIR/codex-review-004.md"
printf '# Review\n- Consensus: agreed\n' > "$REVIEW_DIR/codex-review-2.md"
blocks "$(bash "$HOOK")" || fail "C20g: a malformed unpadded round filename was ignored"
rm -f "$REVIEW_DIR/codex-review-2.md"

# ---- Case 21: a topic heading like '## M2M …' must NOT be misread as milestone 'M2' (word
#      boundary) and deadlock an otherwise-complete Goal ⇒ PASS. ----
printf '# Review\n- Consensus: agreed\n' > "$REVIEW1"
printf '# t\n## Gate status\n- [x] TDD\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '# GOAL\n### M1 first\n## M2M transport design\n## Goal gate\n- [x] M1 E2E: done\n- [x] GOAL E2E: done\n' > "$GOAL"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C21: a '## M2M' topic heading was misread as milestone 'M2' and blocked a complete Goal"

# ---- Case 22: a GFM-marker unchecked box on the GOAL gate itself must BLOCK ----
printf '# t\n## Gate status\n- [x] TDD\n' > docs/g/M1/T01/task.md
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n* [ ] a real pending goal-gate item\n' > "$GOAL"
blocks "$(bash "$HOOK")" || fail "C22: a '* [ ]' unchecked box on the Goal gate evaded enforcement"

# ---- Case 23: the LAST 'Consensus:' verdict governs even when written as an ordered-list or
#      heading-prefixed line — a stale earlier 'agreed' must not win (both the select and validate
#      greps share the decoration prefix) ⇒ BLOCK; an ordered-list positive is accepted ⇒ PASS. ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '# t\n## Gate status\n- [x] TDD\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '%s\n%s\n' "$GOAL" docs/g/M1/T01/task.md > .fullcycle-active
for bad in '1. Consensus: disagreed' '### Consensus: disagreed' '2) Consensus: disagreed'; do
  printf '# Review\n- Consensus: agreed\n%s\n' "$bad" > "$REVIEW1"
  blocks "$(bash "$HOOK")" || fail "C23: a stale 'agreed' let a decorated final verdict '$bad' pass"
done
printf '# Review\n1. Consensus: agreed\n' > "$REVIEW1"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C23: an ordered-list positive verdict was wrongly blocked"

# ---- Case 24: the canonical sequence grows past 999 without a hidden operational cap, and
#      the numerically latest widened suffix is authoritative. ----
rm -f "$REVIEW_DIR"/codex-review-[0-9]*.md
n=1
while [ "$n" -le 1001 ]; do
  printf -v review_path '%s/codex-review-%03d.md' "$REVIEW_DIR" "$n"
  printf '# Review %d\nConsensus: disagreed\n' "$n" > "$review_path"
  n=$((n + 1))
done
printf '# Review 1001\nConsensus: resolved\n' > "$REVIEW_DIR/codex-review-1001.md"
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C24: a valid consensus after round 999 was wrongly blocked"
rm -f "$REVIEW_DIR"/codex-review-[0-9]*.md

# ================= Per-session scoping (owner-tagged registry lines) =================
# Registry lines may be tagged "<session_id><TAB><docpath>". The Stop hook reads its own
# $CLAUDE_CODE_SESSION_ID and enforces ONLY lines it owns; unattributable lines (untagged /
# empty id / empty owner) are fail-closed = enforced by everyone. WHY: concurrent tabs must
# not cross-block, yet nothing unattributable may silently escape the gate.
mkdir -p docs/ga/M1/T01 docs/gb
GA=docs/ga/GOAL.md; GB=docs/gb/GOAL.md
TAB="$(printf '\t')"

# ---- Case 24: isolation both directions — A's Stop ignores B's incomplete owned doc;
#      B's Stop still blocks on B's own incomplete doc. ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GA"           # A: complete
printf '# GOAL\n## Goal gate\n- [ ] GOAL E2E: pending\n' > "$GB"        # B: incomplete
printf 'A%s%s\nB%s%s\n' "$TAB" "$GA" "$TAB" "$GB" > .fullcycle-active
out="$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")"; [ -z "$out" ] || fail "C24: session A was blocked by session B's incomplete owned doc (isolation broken)"
blocks "$(CLAUDE_CODE_SESSION_ID=B bash "$HOOK")" || fail "C24: session B was NOT blocked by its OWN incomplete owned doc"

# ---- Case 25: an UNTAGGED (legacy) line is fail-closed — enforced even by a session that
#      knows its own id and owns nothing here. ----
printf '%s\n' "$GB" > .fullcycle-active                                 # untagged, incomplete
blocks "$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")" || fail "C25: untagged legacy line was not enforced (fail-closed broken)"

# ---- Case 26: EMPTY / unset session id falls back to checking ALL docs (fail-closed) ⇒ BLOCK.
#      Must unset explicitly — the test host may itself run inside a real CLAUDE_CODE_SESSION_ID. ----
printf 'A%s%s\nB%s%s\n' "$TAB" "$GA" "$TAB" "$GB" > .fullcycle-active
blocks "$(env -u CLAUDE_CODE_SESSION_ID bash "$HOOK")" || fail "C26: unset session id did not fall back to enforcing all docs"

# ---- Case 27: the one-Goal rule is PER SESSION — two Goals total, each owned by a different
#      session, must NOT read as '>1 Goal' for either. ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GB"           # B now complete
printf 'A%s%s\nB%s%s\n' "$TAB" "$GA" "$TAB" "$GB" > .fullcycle-active
out="$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")"; [ -z "$out" ] || fail "C27: two Goals across two sessions wrongly tripped the one-Goal rule for session A"

# ---- Case 28: within-session one-Goal still holds — ONE session owning two Goals ⇒ BLOCK. ----
printf 'A%s%s\nA%s%s\n' "$TAB" "$GA" "$TAB" "$GB" > .fullcycle-active
blocks "$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")" || fail "C28: one session owning two Goals did not trip the one-Goal rule"

# ---- Case 29: task-requires-Goal is PER SESSION — A owns a complete task but no A-owned
#      Goal; B's Goal must NOT satisfy it ⇒ BLOCK. ----
printf '# t\n## Gate status\n- [x] TDD\n- [x] E2E\n' > docs/ga/M1/T01/task.md
printf '# Review\n- Consensus: agreed\n' > docs/ga/M1/T01/codex-review-001.md
printf 'A%s%s\nB%s%s\n' "$TAB" docs/ga/M1/T01/task.md "$TAB" "$GB" > .fullcycle-active
blocks "$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")" || fail "C29: session A's task with no A-owned Goal was allowed (a different session's Goal wrongly satisfied it)"

# ---- Case 30: an EMPTY-owner tagged line (leading TAB, e.g. registered when the id was blank)
#      is unattributable ⇒ fail-closed, enforced by everyone ⇒ BLOCK. ----
printf '# GOAL\n## Goal gate\n- [ ] GOAL E2E: pending\n' > "$GB"          # incomplete
printf '%s%s\n' "$TAB" "$GB" > .fullcycle-active                          # leading TAB ⇒ empty owner
blocks "$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")" || fail "C30: an empty-owner (leading-TAB) line was not fail-closed enforced"

# ---- Case 31: the SAME doc registered twice by one session must count ONCE — a duplicate
#      GOAL.md line must NOT trip the one-Goal rule (hook dedupes, robust to writer/race sloppiness). ----
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GA"
printf 'A%s%s\nA%s%s\n' "$TAB" "$GA" "$TAB" "$GA" > .fullcycle-active     # GA registered twice by A
out="$(CLAUDE_CODE_SESSION_ID=A bash "$HOOK")"; [ -z "$out" ] || fail "C31: a duplicate GOAL.md registration wrongly tripped the one-Goal rule (dedupe broken)"

pass "Stop hook: milestone-tied (level ≥1/case/indent/word-boundary), one-Goal, schema-required (Goal+task), Codex numbered-round strict-positive-verdict-gated (numeric latest round wins; legacy singleton rejected; ordered/heading/emoji/blockquote-tolerant), GFM-marker-robust, escape-hatch-sound, per-session-owner-scoped (isolation both ways; untagged/empty-id fail-closed; one-Goal & task-needs-Goal scoped per session)"
