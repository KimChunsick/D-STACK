# Codex adversarial review — Round 008

## Review scope
Re-review

## GPT findings
[severity:medium][the real Why] The verifier accepts compound aliases when the final captured invocation is ultracode, even if the command users reach first is not.
Evidence: The capture function overwrites `DSTACK_ARGS` on every call and the predicate examines only its final value. With `alias claude='claude --effort high; claude --effort ultracode'`, the verifier captures `high`, then `ultracode`, writes the success nonce, and emits no warning. A real shell launches the high-effort session first and reaches ultracode only after it exits.
Suggested direction: At the semantic-verifier boundary, require exactly one captured invocation and reject compound or unobserved command execution. Consider validating the managed alias and relevant global-alias state without executing arbitrary alias bodies.
Illustrative example:
```zsh
typeset -gi DSTACK_CALLS=0
function claude {
  (( ++DSTACK_CALLS ))
  DSTACK_ARGS=("$@")
}
# Success additionally requires DSTACK_CALLS == 1.
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add the exact two-command alias above. Confirm an ordinary shell reaches `--effort high` first, while the corrected installer warns and does not write the success nonce.

[severity:medium][UI/UX & DX] Cancellation before verifier registration can bypass cleanup and signal-preserving termination under `set -e`.
Evidence: Traps are installed while `vpid` is still empty. If a signal is handled before the background launch sets `$!`, `vkill` ends with `[ -n "${vpid:-$!}" ] && ...`; the false test makes the function return 1. Under the installer’s documented `set -e` behavior, `vsig` exits before removing `$vout`, resetting the trap, or re-raising the signal, producing status 1 instead of the signal status. The EXIT handler can fail at the same expression before its `rm`.
Suggested direction: Make cleanup functions unconditionally return success and explicitly represent whether a verifier process has started. Install resource cleanup before creating the temporary file, but re-raise cancellation independently of child registration.
Illustrative example:
```bash
vkill() {
  if [ -n "${vpid:-}" ]; then
    kill_tree "$vpid"
  fi
  return 0
}
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add a synchronization seam immediately after trap installation and before verifier launch, deliver INT/TERM/HUP there, and require status 130/143/129 respectively, no summary, and removal of the nonce file.

[severity:low][UI/UX & DX] A valid `KSH_ARRAYS` startup configuration causes a false ineffectiveness warning.
Evidence: The verifier inherits zsh options from `.zshrc` and assumes one-based indexing with `DSTACK_ARGS[1]` and `[2]`. Under `setopt KSH_ARRAYS`, those select the second and third elements; the clean two-argument expansion therefore evaluates as `ultracode` and empty even though the real alias is effective.
Suggested direction: Normalize `KSH_ARRAYS` inside the isolated verifier before indexing, or compare the captured vector using option-independent array semantics.
Illustrative example:
```zsh
# ~/.zshrc
setopt KSH_ARRAYS
source "$HOME/.claude/ultracode.zsh"
# Real expansion is correct; current verifier warns.
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add a clean fake-HOME fixture with `setopt KSH_ARRAYS`; installation must remain warning-free while the joined-single-argument fixture must still fail.

GPT verdict: reject — the verifier still has a concrete silent-success path for a non-ultracode-first alias, and cancellation can escape the claimed cleanup semantics.

## Maintainer response
1. **Agreed (compound alias) — fixed with the reviewer's own shape.** The capture
   stub counts invocations; success requires exactly one call AND a two-element
   vector equal to `--effort` `ultracode`. Verified (probe v): the exact two-command
   alias warns; the clean install stays warning-free.
2. **Agreed (handler failure under set -e) — fixed, and the registration window is
   closed by self-registration.** Handlers are now unconditionally zero-returning
   (if-form, explicit `return 0`), and ownership has three redundant identities:
   the child WRITES ITS OWN PID to an installer-owned file as its first action
   before `exec` (handlers grace-wait ~150 ms for that write, covering the
   fork-to-exec instant), the unique nonce in the exec'd argv, and `$vpid` once
   recorded. Re-raise never depends on registration. (`$!` is not referenced in
   handlers at all — under `set -u` it cannot be safely defaulted before the first
   spawn.) Verified: probe (q) TERM at 0.05/0.3/2 s ⇒ status 143, no summary, no
   survivors; full battery green.
3. **Agreed (KSH_ARRAYS) — fixed.** The comparison is option-independent:
   explicit `[@]` subscripts everywhere (`${(j: :)DSTACK_ARGS[@]}` — an
   unsubscripted zsh array reference collapses to the first element under
   KSH_ARRAYS, which is exactly what the new fixture caught live). Verified
   (probe w): a `setopt KSH_ARRAYS` startup installs warning-free; the
   joined-argument fixture still warns.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Verifier success = exactly one captured invocation with the exact two-argument
  vector; anything else (compound bodies, wrappers, joined args) warns.
- All prior dispositions unchanged.

Consensus: disagreed
