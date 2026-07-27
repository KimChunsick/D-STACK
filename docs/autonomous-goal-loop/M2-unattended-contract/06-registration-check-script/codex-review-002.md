# Codex adversarial review — Round 002

## Review scope
Batch pass 2 (consolidated round) | `REVIEW_MODE=serial` | bundle 51771 bytes (r1 13860; the growth is the new script entering the diff whole). Allowlist unchanged: the review-unit folder, `claude/skills/full-cycle/check-registration.sh`, `claude/skills/full-cycle/SKILL.md`. **§4 closure with the batch authorisation spent** — all three highs and both mediums fixed and re-verified against a fixture, one parser residual recorded as a follow-up.

## GPT findings

[severity:high][technical-correctness] The GOAL parser still accepts fenced declarations and noncanonical granularity values, allowing false identity and depth selection.
Sites: Primary: global fence toggle; confirmed: granularity substring classifier.
Evidence: Every backtick-prefixed line toggles state without matching the opener’s length; granularity accepts either keyword anywhere in the line.
Verification: A valid four-backtick block containing three backticks emitted fenced `T99` and ignored real `T01`; `Review granularity: not task` selected task mode.
Suggested direction: Track fence character and opener length, and accept only exact documented granularity values.

[severity:high][technical-correctness] Round 001’s ownership-state defect remains for closed and wrong-depth units: foreign ownership is treated as absence.
Sites: Primary: closed-unit `! owned` branch; confirmed: alternate-depth branch and P6 `find -exec` caller.
Evidence: Those branches never call `registered`; additionally, `find -exec command {} \;` does not propagate a command’s nonzero status.
Verification: Foreign status records were accepted by both branches, while a failing `find -exec` command produced exit 0.
Suggested direction: Classify absent/current/foreign in every branch and explicitly aggregate registration-command failures.

[severity:high][technical-correctness] The exit-2 guarantee remains incomplete: unchecked producer failures can be erased before comparison and yield confirmation.
Sites: Primary: duplicate-ID process substitution; confirmed: declared-ID pipeline, `dup_w` pipeline, and scaffolded `cut | sort`.
Evidence: Process-substitution status is never observed, and pipelines generally expose only their final command’s status.
Verification: An exit-7 duplicate producer left the loop successful; subsequent `sort -u` collapsed two `003` paths and `comm` against wanted `003` returned no difference.
Suggested direction: Materialize and check every transformation separately before validating cardinality, deduplicating, or comparing.

[severity:medium][the-real-why] The checker does not compare registry contents with a complete allowed set, so unexpected owned registrations beneath the Goal remain invisible.
Sites: Primary: alternate-depth `find`; confirmed: status is queried only for paths already discovered by constrained `task.md` scans.
Evidence: No code iterates status records beneath the Goal; only `task.md` at depths two and three can be rejected.
Verification: Current-owned `G/M1/note.md` or `G/M1/01-a/sub/task.md` never reaches `owned`, leaving the final result unchanged.
Suggested direction: Compare all current-session registrations beneath the canonical Goal path with `{GOAL.md, active declared units}`.

[severity:medium][technical-correctness] The P6 recipe mutates before classification, making its claimed safe rerun false and leaving invalid units registered after blocking.
Sites: Primary: SKILL.md depth-wide registration loop; confirmed: closed-unit prohibition and structural comparison in the checker.
Evidence: Every depth-matching `task.md` is registered before declaration identity or gate state is checked.
Verification: A closed, deregistered unit becomes current-owned and immediately blocks; an undeclared unit is registered before the extra-ID failure.
Suggested direction: Validate structure first and register only active declared units through the same parsed desired-set boundary.

[severity:medium][technical-correctness] Milestone identity parsing accepts nonheadings and malformed scaffold paths, defeating the claimed `M<n>-<slug>` bijection.
Sites: Primary: milestone declaration regex; confirmed: milestone scaffold-ID extraction.
Evidence: Neither parser requires a token boundary after the numeric heading ID, and the path parser requires no hyphen or nonempty slug.
Verification: `###M1oops` and `M1oops/task.md` both extracted ID `1`, permitting confirmation without a valid milestone heading or folder.
Suggested direction: Require a Markdown heading boundary and the exact `M[0-9]+-<nonempty-slug>/task.md` shape.

[severity:low][DX] Equivalent Goal-directory spellings still fail exact ownership matching.
Evidence: `G="${1%/}"` does not canonicalize relative or absolute spellings before matching status.
Verification: A canonical `docs/g/GOAL.md` record cannot match checks performed as `./docs/g/GOAL.md` or an absolute path.

[severity:low][DX] The success message still falsely states that all units are owned by this session.
Evidence: The unconditional confirmation follows a branch that explicitly requires closed units to be unregistered.
Verification: One closed unregistered unit plus correctly owned active units reaches the “all owned” message.

Omitted-detail: 0 low

GPT verdict: reject — Multiple concrete paths still permit false P6 confirmation, while the documented caller also breaks safe reruns and leaves registration residue.

## Carried decisions
- **Fence tracking is length- and character-aware.** A naive `^```` toggle flips on the ``` lines a
  ```` block legitimately contains: measured on such a block, it read NEITHER the fenced fake row
  NOR the real one below. A closer must now be the same character, at least as long, and carry
  nothing else. HONEST RESIDUAL, and it is the F024 tension: `check-parallel.sh` still uses the
  naive toggle, so on a four-backtick block the two disagree — but in the FAIL-CLOSED direction,
  because this checker blocks on the mismatch instead of confirming the scheduler's wrong reading.
  One parser blocking loudly beats two being confidently wrong together. The identical fix is a
  recorded follow-up for the unit that owns `check-parallel.sh`.
- **Granularity must be the DOCUMENTED value.** A substring test read `Review granularity: not task`
  as task granularity — the worst possible reading of the line whose whole job is to fix the depth
  everything else is checked at.
- **Ownership is classified in EVERY branch, not just the open-unit one.** The closed-unit and
  wrong-depth branches tested `! owned`, so a foreign-owned record passed silently. Absent, ours and
  another session's are three different states with three different fixes, everywhere.
- **Every transformation is materialised and checked separately.** A pipeline reports only its last
  command's status and a process substitution's status is not observable at all — measured,
  `while read …; done < <(exit 7)` leaves the loop at rc 0. An erased producer yields an empty set,
  an empty set yields empty deltas, and empty deltas read as "no differences". That is a false PASS
  produced by a crash.
- **"Nothing else is registered" is ENUMERATED FROM THE REGISTRY.** Looking only at the alternate
  depth and only at `task.md` meant `<goal>/<Mn>/note.md` could be registered to this session and
  never be seen. Reading `status` and subtracting the allowed set has no blind spot, and it replaces
  a narrowed claim with an actual check.
- **The milestone bijection is enforced at both ends** — a heading boundary after `M<n>`, and
  `M[0-9]+-<non-empty-slug>/task.md` for the folder. `###M1oops` and `M1oops/task.md` both yielded
  id 1 before.
- **The Goal directory is canonicalised** to the repo-relative spelling the registry stores, so
  `docs/g`, `./docs/g`, `docs/g/` and an absolute path all check the same thing rather than
  reporting every document unregistered.
- **The success line stopped overstating.** It said "all units owned by this session" after a branch
  that requires closed units to be UNregistered; it now reports scaffolded, open-and-registered, and
  what was checked.

Consensus: resolved
