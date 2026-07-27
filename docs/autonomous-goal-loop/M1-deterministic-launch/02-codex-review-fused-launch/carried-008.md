## Carried decisions — Round 008
- **The resend grammar is validated BEFORE splitting, and that ordering is the fix.** Accepting
  whitespace as a separator let IFS absorb things the split could no longer see: `1, ` lost its
  empty field and came back as a quiet request for round 1 alone, ` ` came back as no request. Both
  silently REDUCE what the reviewer asked for, which is the one failure this validation exists to
  prevent — an unmet request is indistinguishable from one never made. A `case` over the trimmed
  string sees them; the split cannot. Verified end to end against this unit: `1 3`, `1,3`, `1, 3`
  all return rc=0 and "rounds 1 3 by request", while `1,`, `1, `, ` `, `1,,3`, `1, ,3`, `[1]`,
  `1 x` and an out-of-range round are all FATAL.
- **The reset budget starts when the rule does, and cannot expire on a `disagreed` round.** Both
  halves were learned by getting them wrong here. A budget cannot retroactively govern rounds that
  ran before it existed, and declaring one "spent" for an epoch whose rounds predated it left the
  unit with no legal move. And a `disagreed` round is not a closure — it is a round that found
  things. §4's closure is an ACTION (record open findings with severity and evidence, seal
  `resolved`, name them in the report), and reaching the cap obliges you to take it; the round that
  performs it is allowed whatever the count says. A budget that strands a unit between "no more
  rounds" and "no positive seal" is a trap, not a termination rule.
- **The wrapper traps every signal `dstack` traps.** Three was not enough: under zsh an untrapped
  fatal signal skips the EXIT trap entirely, so a wrapper-only USR1 exited 158 and LEAKED the
  scratch directory. Measured old vs new — bash cleaned either way, zsh only with the full set. What
  it does not buy is stated: a handler cannot cancel a foreground `dstack run`.
- **The scope expansion into `assemble-review.sh` is declared, not smoothed over.** The skill
  publishes an invocation the assembler rejected; fixing one without the other means publishing a
  corrected recipe for a command that still refuses it. The allowlist did not grow — the assembler
  was already in the bundle — but the task's file inventory had not been updated, and that was the
  finding.
- **F030 is split into a fix and an accepted residual.** The signalled path is fixed; the PRE-LAUNCH
  leak is deliberate, because if `dstack` dies before publishing `exit` quiescence is unknown and
  keeping the directory is the safe choice. The ledger said "fixed" for both halves, which was wrong.

Consensus: resolved
