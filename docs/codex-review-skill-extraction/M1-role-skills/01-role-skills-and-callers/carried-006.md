## Carried decisions — round 6
- **Symlinked contracts are an accepted risk, closed.** Single-user local machine, no privilege
  boundary between the tree and the reviewer, and the SSOT model requires live links. Do not
  reopen this as a new finding; do not build a snapshot-and-promote pipeline for it.
- **Do not re-add an output grammar validator.** It checks shape, never substance, and it cost
  four review rounds on its own bugs. If the elected skill fails to load, the prompt's
  stop-on-first-line order plus reading the output is the whole defense, and that is recorded as
  the accepted limitation rather than dressed up as a gate.
- Compaction stays gated on a *unique* `## Carried decisions` heading: fail-open on bundle size,
  fail-closed on content. Do not "improve" it into picking the last or outermost heading.
- `~/.codex/skills` remains an undocumented user-skill path (documented one is
  `$HOME/.agents/skills`), chosen because that path is cross-vendor shared and would recreate the
  contamination one level up. Verified locally against codex-cli 0.145.0.

Consensus: resolved
