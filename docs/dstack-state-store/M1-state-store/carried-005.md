## Carried decisions — Round 005
Rounds 1-4 decisions stand. Added in Round 5:

- **Exclusion beats detection.** When migrating away from a protocol, take THAT protocol's lock;
  digesting before and after is a net, not a guarantee.
- **Validate the store before taking any shortcut through it.** "Nothing registered" is a
  conclusion, not an early exit.
- **Apply a shared invariant before any filter**, or the two tools sharing it will disagree
  about the same bytes.
- **A fallback that fails open is the original defect, one level down.**
- **Every stage of a pipeline gets its own status**, and every generated record is read back
  before it is published.
- **Anything a human may copy and run gets `%q`.** A filename is not a safe shell word.

Consensus: disagreed
