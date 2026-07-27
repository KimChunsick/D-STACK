## Carried decisions — Round 005
Rounds 1-4 decisions stand. Added in Round 5:

- **A shell cannot make check-then-delete safe.** Deletion of anything under a path with dynamic
  components belongs behind a command that chdirs once and works relative to what it validated.
- **Ask a pid, never `ps | grep` for a path.** The probe's own command line contains the pattern,
  so the answer is always "alive".
- **Publish sentinels atomically and test for content, not existence.**
- **Check every launcher's status**; `set -u` is not `set -e`.
- **A shell variable does not cross a fence.** Structural checks over documents must be per
  fence, because that is the unit that actually runs.
- **Parameterising a scope means re-deriving the dependency semantics**, not renaming the field —
  a self-referential edge is the failure mode.
- Accepted residuals unchanged: `kill -0` cannot prove process identity; cache-read economics
  limit the injection saving; gitignored is not confidential.

Consensus: disagreed
