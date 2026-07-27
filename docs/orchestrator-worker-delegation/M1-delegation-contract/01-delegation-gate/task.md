# 01-delegation-gate

## Intent / Why

`worker-fanout.requires` currently gates delegation on a `check-parallel.sh` PARALLEL verdict.
That keys on the wrong property. Whether two tasks can run *simultaneously* says nothing about
whether one task's implementation transcript belongs in the orchestrator's context, and the
context growth is what this Goal exists to stop. This task replaces the precondition with one
that keys on task shape, and writes down the worktree costs the Phase 3 research surfaced.

## Deployment context

`claude/skills/full-cycle/SKILL.md` is an instruction document read by the orchestrating model at
the start of every real task, in every repository the maintainer works in. It is not executed.
One maintainer, a few interactive terminal tabs. Its blast radius is every future Goal: a rule
written loosely here is followed loosely everywhere, and there is no test that can catch a
misworded precondition. The deterministic checker (`check-parallel.sh`) is NOT changed by this
task and keeps its existing three-way verdicts; what changes is which of them the pipeline
requires before delegating.

Out of scope by construction: `CLAUDE.md` §0.2's unconditional frontend delegation stays exactly
as it is (recorded Q4 decision), and `check-parallel.sh` itself is not modified.

## Design consult

RUN — trigger: this redraws a module boundary (which agent owns which work). One `codex exec`
design review of the intended approach, GPT-5.6 Sol at xhigh, read-only, no consensus loop.
Capture: `.dstack/runs/<session>/design-owd/`. Verdict: **reject**, and it earned it — the consult
found a contradiction between the two changes this Goal was about to make, before either was
written.

**Accepted and fixed here (T01's share).**

- *The gate checked whether a spec exists, not whether uncertainty is resolved.* Its counterexample:
  a task named "fix intermittent startup" owning `src/startup/` has an objective and a non-empty
  declaration, passes both proposed tests, and is still exploration because its first necessary act
  is diagnosis. The rule now keys on what is UNDECIDED, not on whether a document exists.
- *The "exploratory" definition rejected ordinary implementation.* "Step N depends on what step N-1
  found" describes write-compile-adjust, which is every implementation. Redefined as unresolved
  choice of behaviour, structure, or write set; feedback-driven execution stays eligible. The
  original wording would have made almost nothing delegable, which is the opposite failure.
- *Serial dependencies can branch from the wrong base.* Sharper than the research note: documenting
  the unsafe default is not enough, the orchestrator must VERIFY the base is the integrated head
  containing every closed dependency. Confirmed independently in the installed client 2.1.220,
  whose setting declares `baseRef ?? "fresh"` over options `["fresh","head"]` — so platform-created
  worktrees branch from the default branch, not current HEAD.
- *Copying gitignored files is a credential path.* Gitignored is exactly where `.env`, tokens and
  service-account keys live, and `.worktreeinclude` copies whole directories. Now written as an
  allowlist of named non-secrets, never a directory. This matters more here than in a generic repo:
  `AGENTS.md` maintains a hard secret-name deny list precisely because this repository is public.
- *Cleanup conflicts with resumable review ownership.* Cleaning a worktree at integration removes
  the checkout a resumed worker needs. Lifetime now ends at REVIEW CLOSURE, not at merge.
- *A PASS does not mean the worker only touched its files.* Added `honest-scope`: what the check
  gives is committed-deliverable containment. Ignored files, shared databases and anything outside
  the repository are untouched by it, and narrowing that needs a sandbox this pipeline does not have.
- *The frontend rule is a second delegation authority.* Its example is a task owning both a UI
  component and its server contract. Resolved by pointing at the existing `worker binding` rule:
  a mixed declaration is ineligible for both paths and is split at P5-decompose.

**Rebutted, with evidence, not accepted.**

- *"A non-empty `files` list does not establish bounded ownership; `src/**` or `.` would pass."*
  Not reachable. `check-parallel.sh:91` already rejects any entry containing `*` or `?` as INVALID,
  alongside whitespace and shell metacharacters, and non-canonical paths. The consult was not shown
  files-grammar. The wording now says so inline so the next reviewer does not re-raise it.
- *"Starting downstream work before an upstream review closes creates stale descendants."*
  Already blocked: `P7-tdd needs [P6-scaffold, "P10-unit-e2e@deps-done"]`, and P10 requires review
  consensus AND merge. A dependent task cannot start until its predecessor's review has closed.

**Routed elsewhere, not dropped.**

- *The scope check is a NET two-tree diff, so a path added in one commit and removed in the next
  vanishes from the result while staying in branch history.* Confirmed at
  `check-parallel.sh:366` — `git diff --name-only -z --no-renames "$base" HEAD`. Real, and it
  undermines the containment this whole gate leans on, so it became **T04** rather than a
  follow-up note. It is not in this task's declaration. **T04 has since landed and sealed**
  (`../04-scope-union/`, review closed at round 1, zero blocking): the checker now walks
  `git rev-list "$base..HEAD"` and unions each commit's `diff-tree`, verified against a fixture
  that commits an undeclared file and deletes it. Round 002 of THIS unit re-raised the defect as
  a high, correctly, because its bundle carried only `SKILL.md` and this document — and this
  document still described the defect in the present tense. Recording the resolution here is the
  fix; a review can only be as current as the record it is given.
- *T01 and T02 contradict each other on review fixes.* T01 says review-fix rounds stay with the
  orchestrator; T02 returns single-declaration findings to the worker. Both claim the same finding.
  T01 carries its half of the fix (the explicit exception and the statement that this IS the
  precedence rule); T02 carries the other half.
- *No success metric for total cost.* Recorded in `GOAL.md` under «Success criteria, and what
  would falsify this Goal», with the honest note that this Goal cannot A/B itself.

## What was done (what / why)

**The delegation gate stopped keying on parallelism.** `worker-fanout.requires` used to open with
"checker plan verdict PARALLEL for the exact candidate set". Whether two tasks can run at the same
time says nothing about whether one task's implementation transcript belongs in the orchestrator's
context, and that context is the whole reason delegation exists. The PARALLEL verdict moved to a
new `parallel-when` key, where it is a scheduling decision over already-delegated tasks.

**What replaced it is `delegate-when`, and it fails closed.** Three conditions after Round 001:
the declaration is complete (checker non-INVALID, non-empty `files`); the write set is determined,
so the worker implements a decision rather than making one; and there is a positive isolation
benefit, because eligibility alone would have sent a one-line typo fix through the entire
delegation lifecycle. The second is the one the design consult forced
into shape twice, first because "a spec exists" does not mean "uncertainty is resolved", then
because defining exploration by execution order would have disqualified all normal coding. The
closing line is the fail-closed default: if eligibility needs inference, it is not eligible.

**`keep-in-the-orchestrator` is stated as wrong-to-delegate, not as a fallback**, and it carries
the precedence rule for review-fix rounds so that this list and T02's attribution rule cannot both
claim the same finding.

**Base identity became a verified precondition rather than a documented hazard**, because the
platform's own worktree default branches from the default branch. And `honest-scope` says out loud
what the containment check does not give you, so a PASS is not read as more than it is.

## Files changed (where / why)

- `claude/skills/full-cycle/SKILL.md` — `worker-fanout` restructured into `delegate-when` /
  `keep-in-the-orchestrator` / `frontend-takes-precedence` / `requires` / `parallel-when` /
  `honest-scope`; `worktree-lifecycle.create` gained the explicit base confirmation, a new
  `bootstrap` key covering `.worktreeinclude` as a non-secret allowlist and the recorded
  lane-worktree alternative, and `cleanup` moved from merge-verified to review-closed.

## Recorded follow-ups (open findings, carried out of a closed loop)

The loop closed at Round 003 by the non-convergence rule (blocking 3, 2, 3 — not strictly
decreasing across three consecutive rounds). Every high was fixed or verified before sealing; two
mediums stay open, with their evidence, in `findings.md`:

- **F-01 — the `WorktreeCreate` binding mechanism has not been run end to end.** It is named,
  and its existence and path-screening behaviour are verified in the installed client, but no
  delegation in this pipeline has exercised base identity, cwd binding, bootstrap, branch naming
  and retention together. An evidence gap, not a design gap. The first real fan-out closes it.
- **F-02 — the bundle ratchet rule cannot be satisfied by this shape of unit.** Round 003's
  bundle grew because the reviewed artifacts themselves grew, not because carried prose
  accumulated; the rule in `codex-review/SKILL.md` only prescribes removing the latter. Outside
  this Goal's declared scope, and amending it in the round that violated it would be the move the
  rule exists to prevent.

**Also carried, and worth naming.** Two of Round 003's three highs, and one of Round 002's two,
were STALE relative to the repository: they named defects already fixed in T04, whose file the
ratcheted bundle does not carry. That is the ratchet's cost surfacing as review noise, and it is a
real trade rather than a bug — a bundle that grew to carry every adjacent file is how the previous
Goal reached round ten.

## E2E verification

The decision procedure driven against crafted declarations on 2026-07-27. GOAL.md said this is the
form the E2E would take, because this Goal's own tasks are orchestrator-owned and none of them
fans out.

```
=== the MECHANICAL half of delegate-when: what the checker decides ===
  T01  [src/parse.ts, src/caller.ts]                  declaration -> PARALLEL: T01
  T02  [src/x.ts]                                     declaration -> PARALLEL: T02
  T03  [src/startup/]                                 declaration -> PARALLEL: T03
  T04  []                                             declaration -> SERIAL: T04 has an empty files declaration — fan-out ineligible
  T05  [claude/skills/full-cycle/SKILL.md]            declaration -> PARALLEL: T05

=== a glob or a repo-wide entry cannot even reach the gate ===
  INVALID: T09: glob in files entry 'src/**'
```

What this proves, and what it does not. The checker decides ONE of the four `delegate-when`
conditions on its own: T04's empty declaration is refused mechanically, and `src/**` is INVALID
before any gate runs — which is the rebuttal to Round 001's `files: ["src/**"]` counterexample,
demonstrated rather than asserted.

The other three conditions are not mechanical, and the fixture is built so that is visible:

- **T02** (`correct one typo`, one declared file, no verification run of its own) passes the
  checker and is REFUSED by the isolation-benefit condition. This is exactly Round 001's finding.
- **T03** (`find out why startup is intermittent`) passes the checker and is REFUSED as
  exploratory: its first necessary act is diagnosis, so the write set is not determined.
- **T05** (owns a pipeline skill file) passes the checker and is REFUSED by
  `keep-in-the-orchestrator` — no worker may write those.
- **T01** (two declared files, a written spec) is the only one that reaches delegation.

Four of five candidates that the checker calls eligible are refused by conditions the checker
cannot see. That is the honest shape of this gate: a deterministic floor plus judgment above it,
with every doubt resolving to the orchestrator.

**Not covered, stated plainly.** No worker was spawned. The `WorktreeCreate` binding mechanism is
named and verified to exist in the installed client, but has not been RUN — that is F-01 in
`findings.md`, and the first real fan-out is what closes it.

## Merge-commit coverage, verified here (Round 003 finding)

Round 003 raised that the restored containment claim did not establish coverage of paths
introduced only by MERGE RESOLUTION, since `git diff-tree` omits merge differences by default. The
concern is correct about the default and does not apply to what T04 shipped, which passes `-m`.
Verified rather than argued, on a branch whose merge resolution introduces a file present in
NEITHER parent:

```
=== is the merge-only path in either parent? ===
  parent 1c5837a has evil-from-merge.txt: no
  parent 30ad4f8 has evil-from-merge.txt: no
=== diff-tree WITHOUT -m (the default the finding warned about) ===
    (empty — the default omits merges)
=== diff-tree WITH -m (what T04 uses) ===
    evil-from-merge.txt
    src/a.txt

=== checker verdict on the whole branch ===
VIOLATION: evil-from-merge.txt is not in T01 declaration
```

Recorded in THIS unit rather than in T04's, deliberately: T04's review is sealed, and editing a
file inside a sealed bundle reopens that unit's review. The finding was raised against this unit's
claim, so this unit's record answers it.

## Direct verification (repo policy: no TDD, no new tests)

**What this section can and cannot claim.** This task changes a routing RULE in an instruction
document. There is no runtime to drive, so nothing here confirms "the gate routes correctly" —
Round 001 was right to call the earlier wording unsupported, and the gate row now says what was
actually run. What is verifiable is that the document's machine-checked invariants hold and that
the deterministic checker this task deliberately did not touch still behaves identically. The
routing rule itself is exercised at the unit E2E, against crafted task declarations.

Run on 2026-07-27, output recorded rather than described.

- `bash claude/skills/full-cycle/tests/skill-schema.test.sh` → `== all checks passed`. This is the
  check that caught the work in progress: the first three attempts printed
  `FAIL yaml block does NOT parse: b2.yaml`, because YAML plain scalars may not begin with a
  backtick, may not contain a colon followed by a space, and may not end a line with a colon. Six
  separate lines of the new text violated one of those. The pinned check earned its keep here.
- `ruby -ryaml -e 'YAML.load_file(ARGV[0])'` over every extracted fence → all parse.
- `check-parallel.sh plan docs/orchestrator-worker-delegation/GOAL.md T01 T04` → `PARALLEL: T01 T04`,
  and `… T03` → `SERIAL: T03 not ready — dep T01 incomplete`. The checker is untouched by this task
  and still verdicts correctly, which is the point: this task changed which verdict the pipeline
  REQUIRES, not what the checker computes.

## Gate status

- [x] Verification: document invariants confirmed by direct run — YAML parse, pinned schema check, checker behaviour unchanged (repo policy: no TDD, no new tests)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
