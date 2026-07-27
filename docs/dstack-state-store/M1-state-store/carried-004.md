## Carried decisions — Round 004
Rounds 1-3 decisions stand. Added in Round 4:

- **A tool's exit status is not a pipeline's exit status.** Never let a digest, a git query or a
  jq read reach a consumer without its own status checked.
- **"The tool said no" is not proof of absence.** Status 128 from git means "no repository OR a
  broken one"; prove which before choosing the fail-open branch.
- **Path identity is canonical equality, never a prefix plus an existence test.** A stored path
  must equal what the writer's own canonicaliser derives from it.
- **A file other processes append to cannot be locked, but it can be digested.** Detect the race
  and refuse the destructive step; never assume quiescence.
- **Dedupe on fixed-width keys, never on a delimited concatenation of caller-supplied strings.**
- **Evaluator instructions never live inside the artifact under review.** Work docs describe how
  work is filed; the prompt decides scope.
- Accepted residuals unchanged: no fsync durability, gitignored is not confidential, a ticked box
  is self-attested, `kill -0` cannot prove process identity.

Consensus: disagreed
