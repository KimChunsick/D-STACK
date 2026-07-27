# Maintainer response — Round 008 (batch pass 2, §4 cap closure)

Not bundled. Four mediums and a low; all fixed or recorded. This is the closure round.

## F037 [medium] the resend parser stopped failing closed — AGREED, fixed

This is my regression from round 007, and the reviewer found the exact hole the previous fix opened.
Accepting whitespace as a separator means IFS absorbs things the split can no longer see: `1, ` lost
its empty field to trailing-whitespace removal and came back as a quiet request for round 1 alone,
and ` ` came back as no request at all. Both SILENTLY REDUCE what the reviewer asked for, which is
the one failure this validation exists to prevent — an unmet request looks exactly like one that was
never made.

Fixed by ordering: the whole grammar is validated BEFORE any splitting, so a `case` over the trimmed
string sees what the split cannot. Verified end to end against this unit — `1 3`, `1,3`, `1, 3` all
rc=0 with "rounds 1 3 by request"; `1,`, `1, `, ` `, `1,,3`, `1, ,3`, `[1]`, `1 x` and an
out-of-range round all FATAL.

## F038 [medium] the reset epoch cannot close this unit — AGREED, fixed

The sharpest finding of the round, because it is about a rule I wrote one round earlier and then
mis-applied to the unit I wrote it for. Two mistakes in one:

A budget cannot retroactively govern rounds that ran before it existed. Rounds 006 and 007 ran under
no reset rule at all; counting them as spent budget left the unit with no legal move. And a
`disagreed` round is not a closure — it is a round that found things. Declaring the budget spent on
one strands the unit between "no more rounds" and "no positive seal", and rewriting an immutable
sealed round to escape that is not available.

§4 now says both: the epoch starts when the rule does, and cap closure is an ACTION the cap obliges
you to take — record every open finding with severity and evidence, seal `resolved`, name them in
the report — so the round that performs it is allowed whatever the count says. That is what this
round is.

## F039 [medium] the contract split — AGREED, recorded follow-up (sixth raising)

`codex/skills/adversarial-review/SKILL.md` still requires one immutable exchange and explicit user
disposition; this side files rebuttals separately and closes concrete mediums at the cap. Removing
the precedence sentence made this file honest about not being able to settle it, which is not the
same as settling it. Outside this unit's declaration, so it stays a named follow-up — and it is
genuinely outstanding, not dressed up as agreement.

The second half — the gate validates only the consensus token and never checks findings or
dispositions — is true and is the Stop hook's documented self-attestation scope. It is a tripwire
over a self-reported field, not a proof. Recorded rather than treated as new.

## F040 [medium] the file inventory did not match the change — AGREED, fixed

`assemble-review.sh` was modified and the task listed only `SKILL.md`. The inventory now names both
and states the scope expansion explicitly, with why it could not be split: the skill publishes an
invocation the assembler rejected, so fixing one without the other means publishing a corrected
recipe for a command that still refuses it. The allowlist did not grow — the assembler was already
in the bundle.

## F041 [low] F030 marked fixed when half of it was accepted — AGREED, fixed

The signalled path is fixed; the pre-launch leak is a deliberate residual, because if `dstack` dies
before publishing `exit` quiescence is unknown and keeping the directory is the safe choice. The
ledger claimed both. Now split.

## Also fixed this round, from measurement rather than a finding

Three trapped signals was not enough. Under zsh an untrapped fatal signal skips the EXIT trap
entirely, so a wrapper-only USR1 exited 158 with the scratch directory LEAKED. Measured old versus
new in both shells; the wrapper now traps the full `RUN_SIGNALS` set.

Consensus: resolved
