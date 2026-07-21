# GOAL — per-session scoping of the full-cycle Stop gate

## Goal (the one Why)
Multiple terminal tabs run different work in the same repo concurrently. Today the
full-cycle Stop hook enforces the *whole* `.fullcycle-active` registry, so a tab that has
finished its own work still blocks on another tab's incomplete docs and hangs waiting.
Make the gate **per-session**: tag each registered work-doc line with its owning session
id and have the Stop hook enforce **only the docs the current session owns** — while
preserving fail-closed behavior for anything it cannot attribute.

## Interview record (Phase 4)
- **one-Goal rule scope** → *relax per-session*. Each tab may own its own single Goal;
  the "exactly one GOAL.md" check is computed over the current session's owned docs only,
  so tab A on Goal X and tab B on Goal Y no longer cross-block.
- **orphan lines** (owner died / `/clear` changed the id) → *just leave them*. No live
  session owns an orphan line, so it blocks nobody (integrity OK); the file grows slightly
  and is cleaned via the escape hatch or the next registration. No stale-owner reclamation.

## Research summary (Phase 3)
`docs/fullcycle-per-session-gate/research/ownership-scoped-gate.md` (Codex/GPT-5.5, cited).
- **For:** session-id tagging matches Claude Code's documented identity model
  (`session_id` on hook stdin == `$CLAUDE_CODE_SESSION_ID` in hook *and* Bash subprocesses);
  owner-scoped lifecycle is the mature pattern (tmux sessions, systemd units), and at this
  scale (local APFS, a handful of human-paced processes) even `flock` is overkill.
- **Strongest against / open:** (1) `/clear` changes the session id, orphaning incomplete
  tagged lines → that work's gating silently ends. Accepted: `/clear` is a deliberate reset;
  orphans block nobody and are documented. (2) owner tags are self-attested — a wrong tag
  (bug or intent) can make a Stop ignore a doc; but the delete-line escape hatch already
  exists, so this is a new *accidental*, not *malicious*, bypass mode. A line whose owner no live
  session holds (typo / stale / `/clear`) is an orphan blocking nobody — only the *unattributable*
  lines (no tag / empty id) are the fail-closed ones, a deliberate distinction. (3) the register/
  remove helpers are serialized by a portable `mkdir` lock (no `flock` on macOS), so a concurrent
  `unreg` can't drop a simultaneous `reg` — the only residual is a stranded lock dir after a *hard*
  kill, recovered with `rm -rf .fullcycle-active.lock`. All recorded in the hook's HONEST SCOPE +
  SKILL.md caveats.
- **Fail-closed confirmed:** untagged / empty-id / malformed lines are enforced by everyone
  (Git-lockfile conservative philosophy) — uncertainty blocks, never silently clears.

## Milestones & tasks (Phase 5)
### M1 — Per-session gate scoping
- [ ] **T01** Session-scoped gate hook + registration contract + tests — one atomic PR
      (`fullcycle-gate.sh`, `test_fullcycle_gate_hook.sh`, `SKILL.md`, `CLAUDE.md`)

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: two simulated sessions share one registry; each session's Stop enforces only
      its own owned docs; untagged/empty-id lines still block everyone — captured in
      `M1-per-session-scoping/01-session-scoped-gate-hook/task.md` (Tab A passes while Tab B's
      incomplete Goal is registered; Tab B self-gates; untagged line blocks both)
- [x] GOAL E2E: full `tests/run.sh` green (all 9 suites) + the concurrent-tab simulation against
      the real hook captured — the reported symptom (finished tab hanging on another's gate) is gone
