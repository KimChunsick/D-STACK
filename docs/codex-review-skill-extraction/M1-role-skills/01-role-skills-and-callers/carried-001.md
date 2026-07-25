## Carried decisions — Round 001

- The review contract lives in the `adversarial-review` Codex skill, the research contract in
  `adversarial-research`. `~/.codex/AGENTS.md` declares no role at all: it keeps stack
  neutrality, the language boundary, and the operational constraints, plus an order to stop if
  asked to review or research without the matching skill.
- Election is a real downgrade from unconditional injection and is paid for by three things
  together: explicit `$name` invocation, the stop-if-absent order, and Step 2b's fail-closed
  `contract_ok`. Any change that weakens one of the three must strengthen another.
- Step 2b must stay **fail-closed** and must not require a severity tag from a review that
  legitimately found nothing. It checks shape, never substance — do not describe it as more.
- Skills are installed to `~/.codex/skills/`, not the publicly documented
  `$HOME/.agents/skills`. That path is shared across agents and would recreate the
  contamination this Goal removes. Accepted residual: the chosen path is verified locally
  against codex-cli 0.145.0 rather than documented.
- Adding anything under `codex/skills/` means updating `.gitignore`, its pinned SHA in
  `tests/secret-guard.sh`, that file's negation list, and the `install.sh` map in the same
  change.

Consensus: disagreed
