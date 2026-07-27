# Maintainer response — Round 001

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[medium] The no-`cat` triage command could not find contract-compliant findings.** Confirmed
and fixed, and the demonstration was immediate: I used that exact command to triage this very
review, and it surfaced only `Sites:` lines — the findings themselves were invisible, and a
single-site finding would have left nothing at all. The contract's format is
`[severity:high|medium|low][axis] content`, so the pattern now matches that directly. The fixed
`head -40` is gone from the blocking query as well: a cap that can truncate high/medium findings
is worse than reading too much. The guidance is also honest now about when the rule applies — if
the output is a few KB, read the file; the rule exists to stop fifteen rounds of full reviews
accumulating, not to make anyone work from fragments.

**[medium] The Round-4 rule let a concrete medium ship because it was found late.** Confirmed,
and this is the most important finding in the round. The rule downgraded any late finding on
unchanged code to a non-blocking follow-up unless it was high severity — which means a concrete,
reproducible medium defect discovered in round 4 would have been recorded and shipped. That is a
rule for shipping known defects. Rewritten: **discovery time never changes a finding's blocking
status.** Lateness may now affect only *non-concrete* items (no demonstrated failure path),
which is a restatement of the existing severity wind-down rather than a new escape hatch. The
round budget absorbs the job the old rule was doing, but by escalating to the user rather than
by silently lowering the bar — a person deciding with the defect in front of them is the only
sanctioned way a concrete blocker becomes shippable.

**[medium] The background-handoff behaviour is unverified.** Accepted as stated. It is not yet
captured, and the research artifact does say the Stop-block/background-notification interaction
was never confirmed from primary sources. The evidence now exists in this session — background
launches, ended turns, and completion re-entry have happened several times, including for this
very review — but it belongs in the milestone E2E (P11), which the phase order places after
review. It is being captured there, not asserted here.

**[medium] "Timestamps cannot collide" was an overclaim.** Confirmed and corrected. Two streams
generating a name within the same timestamp unit produce the identical path, which clobbers
inside one worktree and recreates the merge conflict across branches; the supplied research says
as much and I wrote past it. The guidance now says timestamps *reduce* collisions, pins the
precision, requires refusing to write when the path exists, suggests a per-stream suffix when a
generator can fire twice in one unit, and restates that ordering is a declared dependency and
never an artifact of the name.

**[medium] Review-unit registration was ambiguous.** Confirmed and fixed. Every example
hard-coded a per-task directory while this Goal's canonical unit is the milestone root, so
literal substitution would have registered subordinate task documents and left the milestone's
own gate and review series unenforced — silently. Replaced with a single `<review-unit>`
abstraction, a table stating exactly which level is registered at each granularity, and an
explicit statement that subordinate task documents are written for the record but are not
registered, gated, or reviewed. `codex-review` Step 1 uses the same abstraction.

**[low] `prune` was never invoked.** Confirmed. The workflow created persistent captures and
only mentioned that a command existed to remove them. It now names the point that runs it —
Step 4, in the same step that seals the final round.

**[low] `SCRATCH` leaked.** Confirmed and fixed; removing the old trap took scratch cleanup with
it. A trap covering only `SCRATCH` is restored, deliberately not covering the bundle and output,
which must survive.

**[low] The `claude -p` claim was wrong.** Confirmed, and my framing was the error, not just the
wording. Alias expansion happens on the command word, so `claude -p …` typed interactively does
get the alias. Verified directly: `zsh -c 'alias claude'` finds nothing while an interactive
shell finds it — the real dividing line is that a non-interactive zsh never sources `~/.zshrc`,
so this file is never sourced and the alias does not exist. The note now says that, and lists
`claude -p` explicitly as *not* part of the gap.

**[low] The cutover paragraph contradicted itself.** Confirmed and fixed. "Every mutating
command exits 4" alongside "run `dstack migrate`" made the documented recovery impossible;
`migrate` (and read-only `status`) are now stated as the exceptions they always were in code.
