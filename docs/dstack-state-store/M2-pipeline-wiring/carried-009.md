## Carried decisions — Round 009
Rounds 1-8 decisions stand. Added in Round 9:

- **Leaf checks do not contain paths.** Anything that reads a file must resolve its PARENT and
  prove containment; `-L` on the last component tells you almost nothing.
- **A commented-out assignment is not a mode.** If a contract can be skipped by following the
  runnable line, it is not a contract — make it mandatory and let it fail loudly.
- **Parameterising a scope means the DOWNSTREAM contracts too.** Introducing a unit-level
  integration without a unit-level scope check and a document-supply rule just moves the
  unsatisfiable step later.

Consensus: disagreed
