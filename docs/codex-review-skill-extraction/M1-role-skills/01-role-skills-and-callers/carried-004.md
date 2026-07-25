## Carried decisions — Round 004

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role: stack neutrality, the language
  boundary, the operational constraints, and an order to stop if asked to review or research
  without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Weakening one means strengthening another.
- `contract_ok` validates **grammar and block structure**: one verdict line from the three
  allowed values as the final nonblank line; exactly one `^Omitted-detail: [0-9]+ low$`, which
  terminates the finding sequence; and each finding as a complete
  `[severity:<level>][<axis>] <content>` header followed by its own `Evidence:` then its own
  `Verification:`. Zero findings is valid. It checks shape, never substance.
- Step 2 must preserve `codex exec`'s own exit status. No pipeline between the invocation and
  the captured file; a non-zero status means the round is not recorded at all.
- **Never validate a relationship whose anchor the checked side chooses.** The companion check
  failed twice on exactly that: first accepting any suffix, then accepting a
  companion-chosen length. The boundary is now computed from the round and confirmed against
  the round's own heading line at that offset.
- A companion's body must equal its round's trailing lines exactly, starting at the round's
  carried-decisions heading. Read at a computed offset, never searched for.
- Skills install to `~/.codex/skills/`, not the documented `$HOME/.agents/skills`, because that
  path is shared across agents and would recreate the contamination this Goal removes.
  Accepted residual: verified locally against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in one change.

Consensus: disagreed
