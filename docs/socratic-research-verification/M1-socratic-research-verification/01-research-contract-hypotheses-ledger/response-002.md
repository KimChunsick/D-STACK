# Maintainer response — Round 002

- F7 (medium): Accept. The blocks are now semantic requirements encoded per shape
  (Markdown headings filled/appended; schema fields where provided), with the closed-
  schema case named a caller defect that must be flagged, not silently absorbed either
  way. The format rule was reworded to match.
- F8 (medium): Accept. Checkability now keys on an IDENTIFIED primary input; researcher
  accessibility only selects the row status. The deferred case the old wording excluded
  is now definitionally inside the ledger.
- F9 (low): Accepted as a recorded divergence, deliberately not edited — the research
  artifact is immutable P3 evidence and the origin of the defect F3 fixed; retro-editing
  evidence to match fixes would falsify the record. Carried decisions hold this.
- F10 (low): Accept. The Deployment-context sentence now records ownership factually.
  Sibling instances: T02's task.md sat inside its open round-002 bundle when this landed
  and is fixed at that seal; T03's task.md gets the same wording when T03 opens.

Verification after fixes: `bash tests/secret-guard.sh` → PASS; revised sections present
through the live symlink.
