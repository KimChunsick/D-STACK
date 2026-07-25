## Carried decisions — round 5
- Compaction is now gated on a *unique* `## Carried decisions` heading in the round. This is
  deliberately fail-open on size and fail-closed on content: an ambiguous round is sent whole.
  Do not "improve" this later into picking the last or the outermost heading — that reopens the
  self-quoting defect class this Goal spent six rounds failing at.
- `contract_ok` checks shape, not substance. It cannot prove the reviewer applied the scale-fit
  guards or the blast-radius discipline; it proves the output was produced against the contract.
  Recorded as accepted, not oversold.
- `~/.codex/skills` remains an undocumented user-skill path (documented one is
  `$HOME/.agents/skills`), chosen because the documented path is cross-vendor shared and would
  recreate the contamination one level up. Verified locally against codex-cli 0.145.0.

Consensus: disagreed
