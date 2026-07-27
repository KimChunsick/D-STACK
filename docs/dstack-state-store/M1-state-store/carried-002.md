## Carried decisions — Round 002
Round-1 decisions stand. Added in Round 2:

- **Sweep siblings, not instances.** Three Round-2 blockers existed only because a Round-1 fix
  landed in the hook and not the CLI. Before claiming a class fixed, grep for every site.
- Every store component is type-checked before use (`require_plain`), and the check runs BEFORE
  any `mkdir -p`.
- The hook treats a malformed namespace as blocking: non-directory `active`, hidden entries,
  non-key filenames, dangling symlinks, and owners violating the session grammar are all
  reported, never skipped.
- Dependency boundaries validate STATUS AND OUTPUT: `jq` emission, git's exit-128-versus-other,
  an absolute physical root, and a 40-hex digest.
- Migration compares an existing key for exact equality in the PREFLIGHT; conflicts block before
  anything is published or archived.
- `canon` stores the real on-disk spelling; the gate's Goal classification is case-insensitive.
- Dedupe on the record key, never on a string-delimited path set.
- Atomic claims everywhere: `ln` for records, plain `mkdir` for run directories.
- Accepted residuals unchanged: no fsync durability, Unicode normalisation on APFS, gitignored
  is not confidential.

Consensus: disagreed
