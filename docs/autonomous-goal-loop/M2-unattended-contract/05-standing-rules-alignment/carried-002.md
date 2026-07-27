## Carried decisions — Round 002
- **`reclaim` still has two transitions, and the second one is in a file this round could not
  touch.** `autonomy.stops` says there is no autonomous path; the milestone-boundary handoff prose
  says to `/clear` and then `reclaim` the records the session-id rotation orphaned. One state, two
  rules. DEFERRED under the freeze-rule — `full-cycle/SKILL.md` is inside an OPEN review bundle
  (unit 04, round 007) — and carried to unit 04's next round rather than edited mid-round. The
  resolution is the reviewer's second option: after `/clear` the operator IS present by
  construction, so the handoff reclaim is an explicit user-input confirmation, not an autonomous
  act. Nothing can verify provenance without a liveness signal, so confirmation is the honest form.
  **DEFERRAL LIFTED IN THIS SESSION, and recorded rather than quietly dropped.** The blocking bundle
  (unit 04, round 007) closed a few minutes later, which unfroze the file, so the fix landed here
  instead of waiting for another Goal: the handoff prose now lists what it intends to reclaim and
  asks, and says why presence rather than provability is what makes this case different.
- **The lifecycle wording was fixed in the standing file and not in its authority.** The skill said
  "the call does not return until the external command has finished" of a `run_in_background` Bash
  call, which round 001 had already corrected in the standing file. Fixed in `waits.external` once
  the same freeze lifted — it now names the STEP as the blocking thing and says the tool call
  returns immediately, because "the call blocks" reads as a stuck harness and invites the
  hand-rolled watcher this whole contract replaced.
- **The standing file was missing two residuals it needs precisely when the skill is not loaded.**
  A background shell may be reaped under OS memory pressure after 30 idle minutes, and `SIGKILL` or
  `SIGPROF` can orphan `dstack run`'s child — so a capture with NO terminal record is not a failed
  run you may relaunch; check for a live pid or group first, or you spend credits twice and let two
  runs write one label. Both now in both files, verified with line wrapping folded out because
  wrapping is not semantic and a naive phrase grep reported false absences.
- **A repair became the next instance of the class it repaired.** Round 001's F004 asked the gate row
  to stop overstating; the row I wrote to fix it said "behaviour itself is out of scope here", which
  is an evaluator directive embedded in reviewed data. The reviewer proved it operationally rather
  than stylistically: treating the disclaimer as untrusted is what surfaced the missing liveness
  residual above. The section now states what the commands establish and what they do not, with no
  instruction to the reader.

**Disposition.** §4 closure with the batch authorisation spent: every finding is fixed, including
the two that were deferred for part of this session under the freeze-rule and completed once their
blocking bundle closed. No concrete HIGH is open. The follow-ups this unit does not own are named in
`findings.md`.

Consensus: resolved
