## Carried decisions — Round 001
All Round-1 findings were accepted and fixed; none were rebutted, so nothing is carried as an
open disagreement. Standing decisions relevant to later rounds:

- State is anchored at the git root by BOTH `dstack` and the Stop hook. Any future reader of
  `.dstack/` must resolve the root the same way; a CWD-relative read is a gate bypass.
- Keys are SHA-1 of the LOWERCASED canonical path. This is collision-conservative on
  case-sensitive volumes by decision, matching `check-parallel.sh`'s stance on file overlap.
- Blocking must never depend on an external tool being present. `block()` has a jq-free fallback.
- The global registry lock stays removed; per-key locks cover read-then-write operations only.
- Accepted residuals, recorded not overlooked: no fsync durability (bash cannot express it and
  this state is reconstructible from the work documents); Unicode-normalisation variants of a
  path on APFS; gitignored is not confidential (mode 700 and bounded retention are the mitigation).
- Repo policy: no tests, no Red-Green-Refactor. Gates are satisfied by recorded direct-run
  evidence.

Consensus: disagreed
