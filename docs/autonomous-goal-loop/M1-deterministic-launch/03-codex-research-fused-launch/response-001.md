# Maintainer response — Round 001

Not bundled into any review round.

## F001 [medium] the teardown guarantee was stated wider than it holds — AGREED, fixed

Same defect the `codex-review` round found in the same sentence, which is the point: both skills
wrap the same launcher, so both inherited the same overstatement. The residual now says CATCHABLE
termination, names `SIGKILL` as the hole, and points at the recorded pid plus `rm-run`'s refusal to
delete a capture whose child is still alive as the way to notice an orphan before re-running under
the next label.

## F002 [medium] "verified by this Goal's research round" was false — AGREED, fixed, then made true

The sharpest finding in the round, and it caught me contradicting my own task document on the same
page. The P3 round ran before `dstack run` existed, using the improvised launcher this task
removes; it verifies `codex exec` with a brief on stdin and `-o` writing the artifact, and nothing
at all about the wrapper. The bullet now separates the two claims.

Then I did what the finding asked and ran the block, with its `<goal>`/`<topic>` placeholders
substituted, as one background call:

```
DONE autonomous-goal-loop-lifetime exit=0 dir=<repo>/.dstack/runs/<sid>/autonomous-goal-loop-lifetime
capture: exit=0, out.txt 10525 bytes
artifact: docs/autonomous-goal-loop/research/background-task-lifetime.md
  10524 bytes, all six required sections present, 345,874 tokens
  ## Sources: 13 entries — 12 unique URLs + 1 local installed-CLI artifact
```

*Corrected after round 002.* This paragraph originally said "the exact block, unedited" and "33
cited sources". Both were wider than the evidence: the placeholders were substituted, and 33 was a
`grep -c 'https\?://'` over the whole document, which counts inline citations. The finding that
caught it is F008 in the ledger.

No watcher was armed; the completion notification resumed the session by itself. The round was
useful on its own terms too — it is the P3 follow-up on background-task lifetime, and its findings
are summarised into `GOAL.md`.

## F003 [medium] `-s read-only` "never mutate the tree" — AGREED, fixed

True as a description of the model's tool sandbox, false as a description of the invocation, since
the same recipe hands `-o` an absolute path under `docs/` two lines above. Now: read-only blocks
MODEL-initiated mutation, and `-o` is the one deliberate repository write.

## F004 [low] the scratch directory leaked — AGREED, fixed

`SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT`. The old detached launcher cleaned its own
scratch dir inside `run.sh`, and that cleanup was lost with the launcher. Same fix went into
`codex-review`, which had the identical pattern.

## F005 [low][security] an evaluator directive inside the reviewed payload — AGREED, fixed

Correct, and it is the exact class the review contract tells the reviewer to report rather than
obey. My Deployment context said "Out of scope: …", which is an instruction to the evaluator
smuggled in as data. It now states what the change touched as filing information and says so
explicitly. The reviewer ignored it and reported it, which is the behaviour the contract asks for.

## Class-wide sweep (Step 0)

Class: *a claim about verification or safety stated wider than the evidence supports*. Swept every
such claim in the file — the teardown residual (narrowed), "verified runnable" (split into what was
verified when, then actually verified), "never mutate the tree" (narrowed to model-initiated), and
the nonzero-exit rule (already present, re-checked). Also swept both skills for the F005 class: no
other task document in this Goal addresses the reviewer.
