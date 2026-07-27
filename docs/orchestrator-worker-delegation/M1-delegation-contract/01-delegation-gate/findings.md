# Finding ledger — T01 (delegation gate)

Termination signal for this unit's loop. A round closes it when it raises nothing both NEW and
CONCRETE, or when the non-convergence rule fires.

Blocking findings per round: **R1 3 · R2 2 · R3 3**
Bundle bytes: R1 40,728 · R2 36,821 · R3 42,937 (ratchet VIOLATED at R3 — see codex-review-003.md)

## Open

| # | Sev | Class | Finding | State |
|---|---|---|---|---|
| F-01 | medium | correctness | the `WorktreeCreate` hook is named as the binding mechanism but has not been RUN end to end in this pipeline | OPEN — evidence gap, not a design gap; first real fan-out confirms base identity, cwd binding, bootstrap, branch naming and retention together (R1, R2, R3) |
| F-02 | medium | process | the bundle ratchet rule in `codex-review/SKILL.md` assumes growth comes from carried prose, so it cannot be satisfied when the reviewed artifacts themselves grow | OPEN — outside this Goal's declared scope; amending it in the round that violated it is the move the rule exists to prevent (R3) |

## Closed in round 3

- [high][correctness] the contract had no runnable worker mechanism and made serial permanent and
  unfalsifiable — the `WorktreeCreate` hook is the binding mechanism, verified in the installed
  client; the prohibition became an evidence statement
- [high][security] one `.worktreeinclude` entry could resolve to several files including a
  credential-bearing one — the check moved to the RESOLVED set against a single anchored path
- [high][security] merge-resolution paths might escape the union enumeration — verified they do
  not, with a fixture whose merge adds a file present in neither parent
- [low][DX] `keep-in-the-orchestrator` ended on the fragment "no worker may"
- [low][DX] `POSITIVE ISOLATION BENEFIT` had no observable threshold — now read off the
  declaration and the task doc

## Closed in round 2

- [high][security] `honest-scope` claimed containment the endpoint-only check could not deliver —
  the defect was already fixed in T04; this unit's record was stale and now says so, and the claim
  now names the union enumeration it rests on
- [low][DX] the narrative said two `delegate-when` conditions where the contract has three, and
  `per-task` carried a dangling "the worker runs" fragment

## Closed in round 1

- [high][security] `.worktreeinclude` entries are pathspecs, not literal filenames, so "list
  individual fixtures" could still copy credentials — replaced with exact-path + no-metacharacter
  + resolve-then-deny-list-check (R1)
- [medium][correctness] the verified worktree was never bound to the worker; platform isolation
  creates a different checkout and a non-isolated subagent starts in the parent cwd — one
  mechanism plus a four-value identity check before the first write (R1)
- [medium][right-sizing] no benefit threshold, so a one-line typo fix qualified for the full
  delegation lifecycle — added a positive isolation-benefit condition (R1)
- [low][DX] the ticked verification box claimed behaviour confirmation the recorded commands did
  not support — row and section reworded to what was actually run (R1)
- [low][DX] duplicated `Files changed` section, one still `<pending>` (R1)
