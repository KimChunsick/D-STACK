## Carried decisions — Round 003
Rounds 1-2 decisions stand. Added in Round 3:

- **One invariant, enforced whole, at every site.** Partial checks that disagree are worse than
  no check: `read_record` (CLI) and the hook's record loop apply the SAME predicate, including
  the filename-equals-its-own-key test.
- **Changing a key derivation orphans every stored record.** Round 2 did exactly that and nothing
  noticed for two rounds. A derivation change needs a migration or a schema bump, not just a
  correct new formula.
- Fail-closed covers OUR identity too, not only stored owners.
- Every dependency boundary validates output SEMANTICS, not just status and non-emptiness.
- Dynamic path components get the same type/symlink guard as fixed ones.
- Refuse inputs whose identity cannot be guaranteed (non-ASCII, control bytes) instead of
  claiming a guarantee that does not hold.
- A usage error is knowable from the argument vector alone; report it before the environment.
- Accepted residuals unchanged: no fsync durability, gitignored is not confidential, a ticked
  box is self-attested.

Consensus: disagreed
