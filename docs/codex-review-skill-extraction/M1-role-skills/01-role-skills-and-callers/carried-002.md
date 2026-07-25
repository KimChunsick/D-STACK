## Carried decisions — Round 002

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar, not presence**: one verdict line from the three allowed
  values as the final nonblank line, `^Omitted-detail: [0-9]+ low$`, and equal counts of
  severity tags, `Evidence:`, and `Verification:`. Zero of each is a valid empty review. It
  checks shape, never substance — do not describe it as more.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
