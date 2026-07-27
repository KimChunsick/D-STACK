## Carried decisions — Round 007
Rounds 1-6 decisions stand. Added in Round 7:

- **Fold case for identity derivation, never for authorization.** One physical file gets one key;
  who may act on a record is an exact-string question.
- **Ask the filesystem for the real spelling.** Appending a supplied name to a resolved parent
  proves nothing on a case-insensitive volume.
- **`git ls-files` is case-sensitive; the filesystem is not.** Use `:(icase)` for any
  "is this tracked" question that guards a destructive action.
- **A frozen heading is matched whole.** Prefix matching turns a byte-frozen surface into a
  suggestion.

Consensus: disagreed
