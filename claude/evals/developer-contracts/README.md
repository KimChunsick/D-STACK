# Developer contract evaluation specimens

These eight small specimens and rubrics support proportional manual evaluation; they are not
an application or a new evaluation framework. `python3 -B check.py --baseline` runs actual
child interpreters and checks six intended assertion failures and two valid starting points.
It is fixture validation, not proof of model behavior. Passing textual cargo tests proves
contract content and delivery only. Keep actual model outputs, edits, commands and assessment
in the run's authorized artifact directory, outside these immutable source specimens.

| Case | Role | Observable acceptance | Design judgment |
|---|---|---|---|
| 01-policy | general-dev | Both adapters enforce two credits and active account | One business rule owner; callers do not reproduce it |
| 02-meaning | general-dev | Shipping grace 12; overdue grace stays 10 | Separate similar shape with different reasons to change |
| 03-state | frontend-dev | New rows update filtered view, preserving query | Server/UI/derived state ownership stays distinct |
| 04-requests | frontend-dev | Real async tasks retain latest load and one submission | Request identity, lifetime and external-effect uncertainty |
| 05-publication | general-dev | Dead real publisher cannot leak mixed snapshot; intent retained | Visibility boundary separate from recovery authorization |
| 06-recurrence | general-dev | CLI and API both reject revocation | Reinvestigate other callers after repeated review |
| 07-prerequisite | general-dev | Source unchanged; receipt reports blocked dependency | Exact owner/files/calls, local fix problem, prerequisite and preserved evidence |
| 08-cohesion | general-dev | Negative input rejected, existing runs preserved | Cohesive algorithm, no unnecessary wrappers or layers |

Run a case in a fresh bounded native developer session with inherited host settings. Copy the
selected `.py` to an isolated scratch Git worktree; use its `.md` task text excluding the
Evaluator rubric paragraph. Supply actual cwd/common-dir/branch/HEAD, allowed copied files,
artifact path, test and commit policy in the normal brief. Render that brief with
`dstack prompt render --role <selected-role> --context <task-context> > <brief>` and send the
entire output to the matching Claude native Agent prompt or Codex spawn_agent message.
For Claude's direct-native fallback, omit the rendered shared contract deliberately and record
that the worker actually read the installed shared source before implementation. Do not claim
this happened merely because the installed link exists. Use separate fresh trials if comparing
providers or baseline/candidate prompts; keep tools, task, model and effort stable, and record
observed settings or unavailable values honestly.

After a trial run `python3 -B <copied-case>.py`, inspect the actual diff and record each rubric
as pass/partial/fail with evidence. For 02 and 08 add the specific changed-behavior assertions
in their task descriptions; the original smoke checks alone cannot prove the extension.
07 intentionally retains a failing specimen: judge the source unchanged and prerequisite
receipt, not a fabricated passing test. `check.py` without `--baseline` is only a convenience
for runnable behavior after local experiments, not a universal pass gate for these trials.

No model trials run automatically here. Record skipped cases and reasons. Python frontend
cases cover state/async reasoning only; they do not establish React effects, browser lifecycle,
keyboard, focus, design-system or accessibility behavior. The publication case exercises a
real process and disk but does not prove every crash point, filesystem or distributed lock.
Actual model trial results remain observations of those sessions, not a guarantee of future
architecture quality. Human/independent assessment must distinguish plausible prose from
correct code and public effects; eight passing narratives alone are insufficient.
