#!/usr/bin/env bash
# The full-cycle skill is the GLOBAL pipeline that governs every project. Guard the
# load-bearing contract: research is delegated to Codex every Goal (no self-judgment skip),
# and (added in M4) the GOAL.md scaffold, milestone/Goal E2E, and gate schema exist.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
cd "$(git rev-parse --show-toplevel)"
fc=claude/skills/full-cycle/SKILL.md
[ -s "$fc" ] || fail "missing or empty: $fc"

# Phase 3 — research is delegated to Codex, every Goal, unconditionally, both-sides.
assert_matches 'codex-research'                         "$fc"   # Codex is the researcher
assert_matches 'every Goal'                             "$fc"   # mandatory, not conditional
assert_matches 'unconditional'                          "$fc"   # no self-judgment skip
assert_matches 'against'                                "$fc"   # both-sides: evidence against the Goal
assert_matches 'cited|source'                           "$fc"   # current cited sources
assert_matches '[Ff]allback'                            "$fc"   # deep-research only as fallback
assert_not_matches 'Deep research \(conditional\)'      "$fc"   # old conditional heading gone
assert_not_matches 'Skip this phase only when nothing'  "$fc"   # old self-judgment skip gone
# Frontmatter/description must not still advertise deep-research as the primary mode.
assert_not_matches 'deep research \(incl'               "$fc"   # stale frontmatter contradiction gone

# M4 enforcement core: one Goal, GOAL.md scaffold + Goal gate, task folders, DX axis,
# milestone-level E2E, Goal-level E2E (the loop termination), gate schema for the Stop hook.
assert_matches 'DX'                                     "$fc"   # DX added to the UX axis
assert_matches 'one Goal|single Goal|exactly one Goal'  "$fc"   # G1: exactly one Goal
assert_matches 'GOAL\.md'                               "$fc"   # GOAL.md scaffold
assert_matches 'Goal gate'                              "$fc"   # the Stop-hook-scanned Goal gate
assert_matches '<NN-task>/|task folder'                 "$fc"   # task FOLDERS, not single files
assert_matches '[Mm]ilestone E2E|milestone-level E2E'   "$fc"   # G3: milestone E2E
assert_matches 'Goal E2E|Goal-level E2E'                "$fc"   # G4: final Goal E2E (loop end)
assert_matches 'codex-review\.md'                       "$fc"   # review goes to a separate file

# The Stop hook must mechanically enforce the Goal gate (scan GOAL.md too, not only tasks).
gate=claude/hooks/fullcycle-gate.sh
assert_matches 'GOAL'                                   "$gate"  # hook is Goal-gate aware

# SSOT sync (M5): the global entry points must describe the NEW pipeline, not the old one.
inj=claude/hooks/fullcycle-inject.sh
assert_matches 'codex-research|Codex research'          "$inj"
assert_matches 'GOAL\.md'                               "$inj"
assert_matches 'Goal E2E|milestone'                     "$inj"
assert_not_matches '\(if uncertain\) deep-research'     "$inj"   # old conditional research gone
cm=claude/CLAUDE.md
assert_matches 'Codex research'                         "$cm"
assert_matches 'GOAL\.md'                               "$cm"
assert_matches 'codex-review\.md'                       "$cm"
assert_matches 'Goal E2E|milestone E2E'                 "$cm"
assert_not_matches 'in-document consensus'              "$cm"    # review now in a separate file

pass "full-cycle skill contract"
