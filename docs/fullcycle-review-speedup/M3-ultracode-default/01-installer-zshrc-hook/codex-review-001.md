# Codex adversarial review — Round 001

## Review scope
Adversarial review

## GPT findings
[severity:medium][the real Why] The installer equates textual presence with an effective ultracode default, so it can report "up to date" while ultracode remains disabled.
Evidence: `grep -qxF` accepts the hook anywhere in `.zshrc`, including before a later `claude` alias override or inside an inactive conditional. This is especially relevant to legacy installations because an existing manually placed hook is accepted without being migrated to the installer-managed tail position. The documented probes only count matching lines; they do not verify the effective `claude` command.
Suggested direction: Treat a uniquely delimited installer-owned block and its effective placement as the invariant, rather than accepting any matching line. At the installer/test boundary, verify the resulting interactive-zsh command resolution and detect configurations that prevent the managed block from executing.
Illustrative example:
```zsh
# Existing .zshrc
[ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"
alias claude='command claude --effort high'  # wins later

# Installer currently reports "up to date".
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add fixtures where the exact hook is followed by an overriding alias and where it appears inside `if false; then ... fi`; rerun the installer and start an interactive zsh against the fake startup file. Assert that `whence -v claude` or equivalent resolves to the ultracode wrapper.

[severity:low][technical correctness] An incompatible `CLAUDE_CODE_EFFORT_LEVEL` silently defeats the intended default but is omitted from the documented residuals.
Evidence: The fragment launches with `--effort ultracode`. Current Claude Code documentation states that `CLAUDE_CODE_EFFORT_LEVEL` has higher precedence and that a value other than `xhigh` leaves ultracode workflow orchestration inactive, even when ultracode is selected. [Claude Code model configuration](https://code.claude.com/docs/en/model-config)
Suggested direction: At the `claude/ultracode.zsh` boundary, explicitly decide whether the environment variable is a supported user override. Either document it as another escape hatch or detect an incompatible value and emit a concise warning; do not imply that sourcing the fragment alone guarantees ultracode.
Illustrative example:
```zsh
export CLAUDE_CODE_EFFORT_LEVEL=high
claude  # alias requests ultracode, but the environment override disables orchestration
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Launch Claude Code v2.1.203 or later through the alias with the variable unset, set to `xhigh`, and set to `high`; confirm the effective session effort/workflow state and the intended warning or documented override behavior.

[severity:low][software structure/design] The behavior that protects against another silent regression has no durable regression test or reproducible E2E record.
Evidence: The four probes are described as scratchpad-only and not committed. The E2E section is empty, while the TDD and E2E gate boxes remain unchecked. `tests/secret-guard.sh` does not exercise hook installation, idempotence, dry-run behavior, or shell activation.
Suggested direction: Preserve a narrow installer-hook regression test using an isolated `HOME`, even if the broader meta suite remains retired. Include at least one semantic interactive-zsh assertion in addition to line-count checks.
Illustrative example:
```text
fake HOME
  fresh install → one managed hook
  second run   → unchanged
  dry run      → byte-for-byte unchanged
  zsh startup  → ultracode wrapper resolves
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Demonstrate that the committed test passes on this change and fails when the hook step is removed, when dry-run appends, or when an ineffective pre-existing hook causes a false no-op.

GPT verdict: reject — the presence-only idempotence check has a reproducible path that reports success without restoring the ultracode-by-default behavior.

## Maintainer response
1. **Agreed (presence ≠ effectiveness) — fixed at the verification boundary rather
   than the placement boundary.** The installer now runs a semantic post-step (non
   dry-run, `~/.claude` present, zsh available): it resolves `alias claude` in an
   interactive zsh and emits a loud
   `WARNING: interactive zsh does not resolve 'claude' to the ultracode alias`
   when the effective command is not the ultracode wrapper — covering the later
   override, the inactive-conditional, and the ineffective-legacy-hook cases with one
   check on the *outcome* instead of enumerating placements. A managed tail-position
   block was considered and declined: rewriting user zshrc content to enforce
   ordering is more invasive than this repo's remit, and the warning makes the
   failure visible, which is the real Why (the prior regression was *silent*). The
   installer warns rather than hard-fails: the map portion completed correctly, and a
   shell-environment quirk should not abort config installation — the reviewer's own
   framing ("detect configurations that prevent the managed block from executing")
   is detection, which this satisfies visibly. Verified with new fixtures: fake-HOME
   interactive zsh resolves the alias post-install; appending
   `alias claude='claude --effort high'` after the hook makes the re-run print the
   WARNING; the clean fake prints none.
2. **Agreed (env-var precedence) — documented.** `claude/ultracode.zsh` header now
   states that `CLAUDE_CODE_EFFORT_LEVEL` outranks the flag, that any value other
   than `xhigh` leaves orchestration inactive, and to keep it unset or `xhigh`.
   Detection at alias time was declined: the alias must stay a single inert line
   (its exact content is the fragment's contract), and this machine does not set the
   variable — doc-level disposal fits a low.
3. **Disposed by explicit user decision (low).** The meta suite's retirement — and
   exactly one retained guard — is the user's recorded interview decision (Q3),
   which this milestone implements; re-growing per-feature committed tests would
   reverse that decision inside the very Goal that executes it. Mitigation within the
   decision: the probe battery (now 7 assertions incl. the two semantic fixtures
   above) is a rerunnable script whose full transcript is recorded in `task.md`
   §E2E, so the regression check is reproducible even though it is not a committed
   suite. If regressions recur here, re-proposing a committed narrow test goes to
   the user as a product choice, not silently.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- User decision (interview Q3): no committed per-feature test suite; one retained
  secret guard only; installer-hook probes live as recorded rerunnable scripts.
- Accepted residuals (pre-existing, restated): alias covers interactive zsh launches
  only; `ZDOTDIR` setups place the hook elsewhere; `CLAUDE_CODE_EFFORT_LEVEL` ≠
  xhigh disables orchestration (now documented in the fragment header).
- Installer policy: semantic effectiveness check WARNS, never hard-fails.

Consensus: disagreed
