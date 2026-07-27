# Maintainer response — Round 005

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The cleanup was still check-then-delete.** `[ -L "$R" ]` and `rm -rf "$R/$L"` are two
statements, and `$R` can become a symlink between them. A calling shell cannot fix this — it has
no way to hold a directory across the check. So deletion moved behind the CLI as
`dstack rm-run <label>...`, which chdirs into the resolved session directory ONCE, verifies where
it landed is under `runs/`, and deletes by exact label RELATIVE to that directory. After the
chdir the shell holds a resolved directory, so a later swap of any name on the path cannot
redirect the delete; and `rm -rf` on a symlink removes the link without descending into it, so a
swapped leaf is harmless too. `codex-review/SKILL.md` now calls it and says plainly why the
hand-rolled form is not an option.

**[medium] Milestone granularity produced a dependency cycle.** P7 needs
`P10-unit-e2e@deps-done`, and at milestone granularity a task's declared predecessors usually sit
in the SAME unit — M1's T03 depends on T01 and T02, all three inside M1 — so resolving them to
their owning unit made P7 wait on P10 of its own unit: P7→P10→P9→P8→P7. `@deps-done` now
explicitly resolves predecessors to their owning unit and DROPS this unit itself; intra-unit
`deps` edges are execution order inside P7, not gate edges between phases.

**[medium] The vanished-process detector matched its own probe.** `ps -eo command | grep -F
"…/run.sh"` finds the grep's own arguments, so liveness was always true and the VANISHED branch
could never fire — a dead round would be watched forever. This was not hypothetical: the watch
that shipped in Round 4 had the bug, and it only ever completed because the sentinel appeared.
`run.sh` writes its pid and the watch asks `kill -0` about that pid. Recorded honestly: `kill -0`
answers "does this pid exist and may I signal it", not "is it still my runner", so a recycled pid
keeps the watch waiting — the benign direction, with the sentinel as the authority.

**[low] The launcher reported success unconditionally.** `set -u` is not `set -e`; a failed
`Popen` was followed by `echo "launched…"`. Checked now, with an explicit "do NOT arm a watch".
**[low] The sentinel was published non-atomically** — `> exit` creates the file empty and fills
it after, so a watcher could read a zero-byte file and report `DONE exit=` with no status. Temp
file plus `mv`, and the watch tests `-s` rather than `-f`.
**[low] The schema check accepted a cross-fence `$DS`.** A shell variable does not cross fences;
concatenating them let a fence containing only `"$DS" status` pass because some other fence
defined `DS`. Both checks are per-fence now, and both were negative-controlled: a fence using
`$DS` without defining it fails, and a bare `dstack status` in a fence fails.
**[low] P10/P11 placement contradicted its own schema.** The paragraph said record both in the
same `task.md` while P11's gate is the `M<n> E2E` box in GOAL.md — following it would leave the
machine-enforced box untouched. Rewritten to say which file each gate lives in and why that is
the hook contract, not a style choice.
**[low] The corrected byte/character figures were not propagated to the hook comment.** Fixed
there, and in the M2 review-unit doc, which also still claimed 74% from 1,857 characters.
