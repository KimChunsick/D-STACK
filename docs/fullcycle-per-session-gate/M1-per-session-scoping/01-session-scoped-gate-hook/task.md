# 01-session-scoped-gate-hook

## Intent / Why
The Stop hook must stop cross-blocking concurrent tabs: read this session's id
(`$CLAUDE_CODE_SESSION_ID`, injected into hook + Bash subprocesses) and enforce only the
registry lines it owns, while treating anything unattributable (untagged/legacy, empty
owner, empty id) as owned-by-all so no unfinished work silently escapes. This is one
atomic change: the hook's parsing format and the skill's registration format are a single
contract, so hook + tests + writer-side docs land together (one reviewable PR).

## What was done (what / why)
- **Gate hook** now parses each `.fullcycle-active` line as an optional `"<owner><TAB><docpath>"`.
  It reads `sid="${CLAUDE_CODE_SESSION_ID:-}"` and skips a line only when it is provably
  another session's (`owner` non-empty AND `sid` non-empty AND `owner != sid`); everything
  else — ours, or unattributable — falls through to the existing goal/task enforcement, which
  is thereby scoped per session (incl. the one-Goal and task-needs-Goal checks). Fail-closed by
  construction: an untagged line or an empty `sid` is enforced by every session.
- **Registration contract** in `SKILL.md` (Phase 6/10/12): register/remove lines tagged with
  `$CLAUDE_CODE_SESSION_ID` + a real TAB, with the fail-closed and `/clear`-orphan caveats.
- **CLAUDE.md** gate description gains the per-session dimension + escape-hatch wording.
- **Tests** (C24–C29) cover isolation both directions, untagged & unset-id fail-closed, and
  the one-Goal / task-needs-Goal checks being per-session. Existing C1–C23 unchanged (they use
  untagged lines → enforced regardless of id → behavior identical).

## Files changed (where / why)
- `claude/hooks/fullcycle-gate.sh` — owner-tag parsing + per-session scope filter; HONEST SCOPE
  documents the self-attestation / `/clear`-orphan / fail-closed semantics.
- `tests/test_fullcycle_gate_hook.sh` — six per-session cases (C24–C29); C26 unsets the env var
  because the test host itself runs inside a real `CLAUDE_CODE_SESSION_ID`.
- `claude/skills/full-cycle/SKILL.md` — tagged register/remove commands + caveats.
- `claude/CLAUDE.md` — per-session gate description.

## E2E verification
Two-session concurrent-tab simulation against the **real** hook (`claude/hooks/fullcycle-gate.sh`),
one shared `.fullcycle-active` built with the SKILL.md `reg` helper:
```
shared registry:
    TAB-A|docs/ga/GOAL.md          (Goal complete)
    TAB-B|docs/gb/GOAL.md          (Goal incomplete)

1) Tab A finishes, Tab B still working:
   Tab A Stop  → pass    (NOT blocked by B — the original hang is fixed)
   Tab B Stop  → BLOCK   (still self-gated on its own incomplete Goal)
2) A legacy UNTAGGED incomplete line added:
   Tab A Stop  → BLOCK   (fail-closed — untagged enforced by everyone)
3) Tab B ticks its Goal E2E:
   Tab A Stop  → pass
   Tab B Stop  → pass    (now complete)
```
Reproduces exactly the reported symptom (a finished tab hanging on another tab's incomplete gate)
and shows it resolved, with fail-closed and self-gating intact. Unit suite: all C1–C31 green;
`tests/run.sh` fully green. `reg`/`unreg`/lock/migration/failure/signal paths verified separately.

## Gate status
- [x] TDD: Red→Green→Refactor complete
- [x] Codex (GPT-5.5) adversarial review consensus
- [x] E2E capture verified
