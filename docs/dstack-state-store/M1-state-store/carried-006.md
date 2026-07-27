## Carried decisions — Round 006
Rounds 1-5 decisions stand. Added in Round 6:

- **Sweep siblings, and then sweep them again.** Three rounds running, a class fix landed in one
  artifact and not the other. Before claiming a class closed, grep every artifact for the
  construct, not just the one the finding named.
- **A lock must cover every decision about the thing it protects,** including the ones that look
  like early exits.
- **Resolve paths physically before reasoning about them.** A logical path defeats an
  ancestor walk exactly like a missing binary defeats a command check.
- **A key helper is not a content digest.** Case folding is correct for identity and wrong for
  detecting change.
- **Runtime state must be proven untracked before it is moved.** A global tool runs in
  repositories that never asked for it.
- **Advertised retention needs a trigger.** A cleanup nobody runs is not a mitigation.

Consensus: disagreed
