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

# ---- Case 5: ticked Codex gate but NO codex-review.md ⇒ BLOCK (artifact requirement) ----
printf '# t\n## Gate status\n- [x] TDD\n- [x] Codex consensus\n- [x] E2E\n' > docs/g/M1/T01/task.md
printf '# GOAL\n## Goal gate\n- [x] GOAL E2E: done\n' > "$GOAL"
printf '%s\n%s\n' "$GOAL" docs/g/M1/T01/task.md > .fullcycle-active
blocks "$(bash "$HOOK")" || fail "C5: did not block when Codex gate ticked without a codex-review.md"

# ---- Case 6: add codex-review.md with consensus ⇒ PASS ----
printf '## Consensus\n- Consensus: agreed\n' > docs/g/M1/T01/codex-review.md
out="$(bash "$HOOK")"; [ -z "$out" ] || fail "C6: blocked despite agreed codex-review.md + all gates ticked"

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

pass "Stop hook: section-scoped, milestone-tied, one-Goal, schema-required, Codex-artifact-gated, escape-hatch-sound"
