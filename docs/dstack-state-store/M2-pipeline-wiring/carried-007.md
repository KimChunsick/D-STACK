## Carried decisions — Round 007
Rounds 1-6 decisions stand. Added in Round 7:

- **A path is data, never source.** Quoted heredocs, arguments instead of interpolation. Anything
  that writes a script must assume the values it embeds are hostile.
- **The gate's own tool must fail closed.** `|| true` on the diff that IS the review material is
  the review approving what it never saw.
- **Parameterize what decides ORDER, not just what decides naming.** Serialization and merge
  gating are where a half-converted scope produces clobbering and deadlock.
- **The success path must exit zero.** A recipe that fails on convergence teaches the loop that
  converging is an error.
- **Idempotent success is not proof of effect.** Verify the state, not the return code.

Consensus: disagreed
