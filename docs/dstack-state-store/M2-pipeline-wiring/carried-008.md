## Carried decisions — Round 008
Rounds 1-7 decisions stand. Added in Round 8:

- **`git diff <commit> -- <path>` is commit-versus-WORKING-TREE.** A committed range needs two
  trees, a checked-out head, and a clean tree — all three, or the bundle describes something the
  merge will not carry.
- **A contract split across a tool and its caller is not a contract.** The assembler knowing about
  `REVIEW_BASE` while no recipe set it is the same as not having it.
- **Integrating and landing are different gates.** Gate the landing on review; gating the
  integration on review is how a cycle gets written.
- **Idempotent APIs need their input derived, not retyped.** Read the labels from the tool that
  knows them.
- **Do not record a number that keeps changing.** Record how to measure it, and date the figure.

Consensus: disagreed
