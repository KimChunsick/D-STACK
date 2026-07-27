# Codex adversarial review — Round 001

## Review scope
First round for this unit. Bundle: `claude/CLAUDE.md`, `claude/skills/full-cycle/tests/skill-schema.test.sh`,
this unit's `task.md`, and the `worker-fanout` block of `full-cycle/SKILL.md` the two of them quote.

## GPT findings
[severity:medium][real Why] The startup summary wrongly subjects frontend work to the task-shape gate and claims all review-fix rounds remain with the orchestrator, contradicting both explicit exceptions.
Sites: claude/CLAUDE.md:10-15; confirmed: claude/CLAUDE.md:48-63, claude/skills/full-cycle/SKILL.md:182-194,574-594.
Evidence: Section 0.2 mandates frontend delegation, while the structured authority gives frontend precedence and returns qualifying review fixes to the owning worker.
Verification: An exploratory frontend task and an eligible worker-owned review fix receive opposite routing depending on which statement the orchestrator follows.

[severity:medium][technical correctness] The regression guard is not scoped to `requires:` and does not recognize the live `parallel-when` wording, allowing the intended semantic regression while rejecting a valid placement.
Sites: claude/skills/full-cycle/tests/skill-schema.test.sh:111-120; confirmed: docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/task.md:61-72.
Evidence: The guard bans only `- checker plan verdict PARALLEL` anywhere, while the live item is `- a checker plan verdict of PARALLEL`.
Verification: Moving the live wording under `requires:` produced valid YAML with all four positive checks satisfied and `negative-detected=no`; placing the banned wording under `parallel-when:` produced a false failure.
Suggested direction: Parse `scheduling.modes.worker-fanout` and assert nonempty typed keys plus the location of the PARALLEL scheduling condition under `parallel-when`, not through global text matching.

[severity:low][UI & UX / DX] The task record contains a duplicate `Files changed (where / why)` section whose second instance remains `<pending>`.
Evidence: The completed file list appears at lines 44-49, followed by the duplicate placeholder at lines 51-53.
Verification: Direct inspection confirms two identical headings with conflicting completion states.

Omitted-detail: 0 low

GPT verdict: reject — The startup copy still misroutes valid delegated work, and the regression guard can miss the targeted failure while rejecting a correct schema.

## Bundle size (the ratchet, recorded)

R1 11193 bytes — the baseline for this unit.

## Round outcome

Two mediums and one low, all real, all fixed before this file sealed. The blocking count is 2.

The second medium is the one worth keeping: my regression guard did not guard. It banned one exact
phrase anywhere in the file, and the phrase was not the live wording, so the regression it exists to
catch walked straight past it while the same words under `parallel-when` — where they belong —
produced a false failure. My own negative control had exercised the OLD phrasing, which is exactly
why it looked like it worked. A control that does not use the live string proves nothing.

Round 002 re-reviews the repairs.

Consensus: disagreed
