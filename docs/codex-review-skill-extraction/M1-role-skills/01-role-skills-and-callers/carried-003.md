## Carried decisions — Round 003

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar and block structure, not presence**: one verdict line from
  the three allowed values as the final nonblank line, exactly one
  `^Omitted-detail: [0-9]+ low$`, and each finding as an ordered tag → Evidence → Verification
  block. Zero findings is valid. It checks shape, never substance.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- A companion's body must equal its round's trailing lines exactly. This is a suffix
  comparison, never a structural read of the round — that derivation is the mistake six earlier
  rounds were spent on. It is what makes "author the companion, do not extract it" checkable.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
