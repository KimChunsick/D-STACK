# Codex adversarial review — Round 003

## Review scope
Re-review of Round 002's repairs. Bundle: `full-cycle/SKILL.md` and this unit's `task.md`.

## GPT findings
[severity:high][technical correctness] The change defines no runnable worker mechanism: it marks checkout binding and bootstrap `UNCONFIRMED`, then requires every fan-out to remain serial.
Sites: `worker-fanout.per-task`; confirmed: `worker-fanout.requires`, `worktree-lifecycle.create`, `worktree-lifecycle.bootstrap`, task doc “E2E verification” and “Gate status”.
Evidence: Manual creation does not bind a subagent, while platform isolation creates another checkout; Claude’s documented binding mechanism is a `WorktreeCreate` hook, which also disables `.worktreeinclude`. [Claude worktree documentation](https://code.claude.com/docs/en/worktrees), [Claude hook documentation](https://code.claude.com/docs/en/hooks)
Verification: Any task satisfying `delegate-when` reaches the unconditional “fan-out is unverified and serial is the answer”; even an external E2E capture cannot alter that static instruction without another reviewed edit.
Suggested direction: Select one binding mechanism, specify its base/cwd/bootstrap/retention invariants, exercise it end-to-end, and remove the `UNCONFIRMED` prohibition before deployment.

[severity:high][security] The `.worktreeinclude` policy still permits one apparently exact entry to resolve to multiple ignored files, including benignly named credential files.
Sites: `worktree-lifecycle.bootstrap`; confirmed: task doc “Design consult”.
Evidence: The contract validates “pathspec metacharacters,” but Claude documents `.worktreeinclude` as gitignore syntax; under that syntax, a basename without `/` matches at every depth, and resolved files receive only a secret-name deny-list check. [Claude worktree documentation](https://code.claude.com/docs/en/worktrees), [Git gitignore documentation](https://git-scm.com/docs/gitignore)
Verification: A metacharacter-free entry `config.json` may select both root `config.json` and `cache/config.json`; the latter can contain a live token yet pass when neither path appears on the filename deny list.
Suggested direction: Validate the documented gitignore grammar, require the resolved set to equal one anchored expected path, and bootstrap only generated or independently attested secret-free fixtures.

[severity:high][security] The restored committed-containment claim does not establish coverage of paths introduced only by merge resolution.
Sites: `worker-fanout.honest-scope`; confirmed: task doc “Routed elsewhere”.
Evidence: The supplied record says T04 unions each commit’s `diff-tree` and verifies only add-then-delete history; Git omits merge-commit differences by default unless an explicit merge-diff mode is used, and no checker snapshot or merge fixture demonstrates otherwise. [Git diff-tree documentation](https://git-scm.com/docs/git-diff-tree)
Verification: Resolve a merge by adding an undeclared credential path present in neither parent; a default `diff-tree` union omits that path while the merge result and blob remain reachable.
Suggested direction: Make merge handling explicit in the checker and add a fixture whose undeclared path exists only in a merge resolution.

[severity:low][DX] `keep-in-the-orchestrator` ends a rule with the incomplete fragment “no worker may,” leaving its intended prohibition grammatically undefined.
Evidence: The fragment follows the `docs/` and pipeline-skill restriction but has no verb complement.
Verification: The supplied YAML can parse while readers cannot determine whether the fragment was intended to broaden the preceding restriction.

[severity:low][DX] `POSITIVE ISOLATION BENEFIT` has no observable threshold, so equivalent tasks can route differently despite the fail-closed promise.
Evidence: “Predictably verbose or materially multi-step” depends entirely on operator inference, while the deployment context says no test catches a loose precondition.
Verification: A three-file mechanical rename can reasonably be classified either as materially multi-step or as a quick targeted change.

Omitted-detail: 0 low

GPT verdict: reject — Fan-out remains deliberately non-runnable, while the bootstrap and history-containment rules retain concrete secret-exposure paths.

## Bundle size — the ratchet was VIOLATED, and the rule is wrong

R1 40,728 · R2 36,821 · **R3 42,937** bytes. Round 003's bundle GREW, against the rule in
`codex-review/SKILL.md` that round N must be at or below N-1.

Recorded as a violation, not excused. But the diagnosis matters: that rule assumes growth comes
from accumulated prose about earlier rounds, and prescribes removing it. This unit's bundle carries
no prior-round files at all — only `SKILL.md` and `task.md`. The growth is in the reviewed
artifacts themselves: the contract text got longer because the fixes are longer than what they
replaced, and the record got longer because Round 002 demanded that T04's resolution be written
down. Nothing could come out without deleting content a reviewer asked for.

So the rule's premise does not cover this shape. The fix belongs in `codex-review/SKILL.md` —
measure the CARRIED-PROSE portion, or exempt growth in the files actually under review — and it is
NOT made here: that file is outside this Goal's declared scope, and quietly amending a rule in the
round that violates it is exactly the move the rule exists to prevent. Carried as a follow-up.

## Round outcome

Three highs and two lows, all accepted; two of the three highs were partly stale in the same way
Round 002's was, and one of them supplied the answer this unit had been missing.

Reasoning in `response-003.md`; ledger in `findings.md`.

**The loop CLOSES here, by the non-convergence rule.** Blocking findings ran 3 (R1), 2 (R2),
3 (R3) — not strictly decreasing across three consecutive rounds, which
`codex-review/SKILL.md` defines as non-convergent by measurement. All three of this round's highs
were fixed or verified before sealing; the two open mediums are recorded with their evidence in
this unit's `task.md` and in `findings.md`.

Stated plainly rather than buried: the `WorktreeCreate` mechanism was WRITTEN IN THIS ROUND and
has therefore had no review. It is guarded — named as not-yet-run, with a per-task fallback to
serial — and it is what closes the round's most serious finding. Closing anyway is the rule
applying to its own author; the alternative is the "one more round always seems justified" loop
the rule exists to end.

Consensus: resolved
