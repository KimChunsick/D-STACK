# Codex adversarial review — Round 005

## Review scope
Re-review (Rounds 1-4 rejected; all findings accepted and fixed)

## GPT findings
[severity:high][security] The Round-4 cleanup remains a check-then-delete race: `$R` can become a symlink after validation, causing recursive deletion outside the repository.
Evidence: `[ -L "$R" ]` and each `rm -rf "$R/$L"` are separate operations, with no trusted-boundary validation at deletion time.
Verification: Rename `$R` after the check, replace it with a symlink to an external directory containing `$L`, and the subsequent removal follows that intermediate symlink.
Suggested direction: Move exact-label deletion behind `dstack`, validating every run-path component at deletion time without following symlinks.

[severity:medium][technical correctness] The review-unit conversion makes milestone-granularity Goals with internal task dependencies cyclic.
Sites: primary: `claude/skills/full-cycle/SKILL.md` P7 dependency; confirmed: `docs/dstack-state-store/GOAL.md` M1 T03 dependencies and the review-unit granularity table.
Evidence: P7(M1) requires `P10-unit-e2e@deps-done`; T03 declares T01/T02 as predecessors, but all three belong to the same M1 review unit.
Verification: Resolving those predecessor tasks to their owning unit produces P7(M1) → P10(M1) → P9(M1) → P8(M1) → P7(M1).
Suggested direction: Collapse dependencies to distinct predecessor review units; keep intra-unit task edges solely as execution order within P7.

[severity:medium][technical correctness] Step 2a’s vanished-process detector can match its own `grep`, leaving a failed detached round monitored indefinitely.
Evidence: `ps -eo command | grep -qF -- "$R/<label>/run.sh"` searches for a string present in the probing `grep` process’s own arguments.
Verification: After the runner disappears without writing `exit`, `ps` can emit the probe command, `grep` returns success, and the `until` loop sleeps again instead of emitting `VANISHED`.
Suggested direction: Track and validate the spawned runner’s exact process identity, or use an exact runner-command match that excludes the watcher and probe.

[severity:low][technical correctness] Step 2 reports a successful launch even when the Python launcher fails.
Evidence: The block uses `set -u`, not `set -e`, and executes the unconditional `echo "launched..."` after `python3` without checking its status.
Verification: Any `Popen` failure makes Python exit nonzero, after which `echo` returns zero and falsely completes the procedure successfully.

[severity:low][technical correctness] The completion sentinel is published non-atomically.
Evidence: `printf ... > "$RD/exit"` creates the file before writing its contents, while the monitor separately tests `-f` and reads it.
Verification: The monitor can observe the newly created zero-byte file and emit `DONE exit=` before the write completes.

[severity:low][technical correctness] The schema check still permits the Round-3 cross-fence `$DS` regression.
Evidence: It accepts any `"$DS" <verb>` if `DS="$HOME/.claude/bin/dstack"` appears anywhere else in the skill.
Verification: Add an independent runnable fence containing only `"$DS" status`; both the positive verb check and bare-name negative check still pass although `DS` is unset there.

[severity:low][DX] The P10/P11 placement instructions contradict their own schema.
Sites: primary: `claude/skills/full-cycle/SKILL.md` post-schema milestone-granularity paragraph; confirmed: P11 schema row and detailed P11 procedure.
Evidence: One paragraph orders both gates recorded in the same `task.md`, while P11 explicitly gates and records milestone E2E in `GOAL.md`.
Verification: Following the first instruction adds or ticks a task-document gate while the machine-enforced `GOAL.md` milestone box remains unresolved.

[severity:low][DX] The corrected byte/character accounting was not propagated to the hook comment.
Sites: primary: `claude/hooks/fullcycle-inject.sh`; confirmed: `06-inject-slim/task.md`.
Evidence: The hook still claims 1,857 characters, while the corrected record reports 1,850 bytes and 1,845 characters.
Verification: The two supplied representations assign incompatible units and values to the same original injected string.

Omitted-detail: 0 low

Blocking status: the high and both medium findings are genuinely blocking in Round 5; proximity to the six-round budget does not make them shippable.

GPT verdict: reject — Cleanup can still escape the repository, milestone-granularity scheduling self-cycles, and a vanished review can leave the monitor waiting indefinitely.

## Carried decisions — Round 005
Rounds 1-4 decisions stand. Added in Round 5:

- **A shell cannot make check-then-delete safe.** Deletion of anything under a path with dynamic
  components belongs behind a command that chdirs once and works relative to what it validated.
- **Ask a pid, never `ps | grep` for a path.** The probe's own command line contains the pattern,
  so the answer is always "alive".
- **Publish sentinels atomically and test for content, not existence.**
- **Check every launcher's status**; `set -u` is not `set -e`.
- **A shell variable does not cross a fence.** Structural checks over documents must be per
  fence, because that is the unit that actually runs.
- **Parameterising a scope means re-deriving the dependency semantics**, not renaming the field —
  a self-referential edge is the failure mode.
- Accepted residuals unchanged: `kill -0` cannot prove process identity; cache-read economics
  limit the injection saving; gitignored is not confidential.

Consensus: disagreed
