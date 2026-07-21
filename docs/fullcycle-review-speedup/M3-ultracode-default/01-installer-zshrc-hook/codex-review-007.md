# Codex adversarial review — Round 007

## Review scope
Re-review

## GPT findings
[severity:medium][the real Why] The verifier replaces an existing `claude` function before testing, so argument-rewriting wrappers still produce false success.
Evidence: The current predicate defines `function claude { DSTACK_ARGS="$*"; }` before `eval claude`, overwriting any function loaded from `.zshrc`. A wrapper that launches `command claude --effort high` therefore resolves to high in a real shell, while the verifier replaces it, captures `--effort ultracode`, and writes the success nonce. This reproduces the unresolved Round 6 counterexample. Probe (s) does not test its claimed “wrapper FUNCTION”; it appends only another alias. Additionally, `$*` loses argument boundaries, so one argument named `--effort ultracode` is indistinguishable from the required two arguments.
Suggested direction: At the semantic-verifier boundary, detect a pre-existing `functions[claude]` before installing the capture stub and conservatively report it as incompatible or unverifiable. Store captured arguments as an array and require exactly two elements equal to `--effort` and `ultracode`.
Illustrative example:
```zsh
function claude { command claude --effort high; }
alias claude='claude --effort ultracode'

real shell → existing function → high
verifier   → replaces function → captures ultracode → false success
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Replace probe (s) with an actual function backed by a harmless fake `claude` executable. Confirm a normal terminal captures the rewritten high-effort invocation and the installer emits a warning. Add a separate fixture producing one combined argument and require that it fail verification.

[severity:low][software structure/design] The cancellation probe still permits the failure state it claims to exclude.
Evidence: Probe (q) appends a 33,000-plus-second `sleep`, but accepts either status 143 or 0 after sending TERM. At the two-second signal point, legitimate completion is impossible because startup remains blocked; status 0 would indicate swallowed cancellation or a probe that never exercised the intended window. Installer output is discarded, so the claimed absence of a fake success summary is not asserted either.
Suggested direction: Synchronize the probe with verifier startup, require the installer to be alive before signaling, require status 143 for TERM, and capture output to assert that no summary is printed. Keep survivor checks tied to the owned canary.
Illustrative example:
```text
blocking startup ready → TERM installer
required: status 143 + no Summary + no canary
current:  status 0 is also accepted
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Temporarily make the TERM handler clean up and return normally; the corrected probe must fail on status and summary assertions while still confirming that cleanup occurred.

GPT verdict: reject — the semantic verifier still silently approves a concrete wrapper-function configuration that defeats the ultracode default.

## Maintainer response
1. **Agreed (wrapper-function replacement) — fixed conservatively, as suggested.**
   The predicate now loads `zsh/parameter` and, BEFORE installing the capture stub,
   reports any pre-existing `functions[claude]` as incompatible/unverifiable
   (exit 6 → warning) instead of silently replacing it. The capture is an ARRAY and
   success requires exactly two elements `--effort` + `ultracode`, so a joined
   single argument fails. Probe (s) now installs a REAL wrapper function
   (`function claude { command claude --effort high; }`) and asserts the warning;
   new probe (u) asserts the joined-argument alias also warns. Full battery re-run:
   the clean install stays warning-free.
2. **Agreed (probe rigor) — fixed.** Probe (q) now asserts the installer is still
   alive before signaling (the window is actually exercised), requires exit status
   exactly 143 for TERM, captures the installer output, and asserts no `Summary:`
   line was printed; survivor checks remain tied to the run-unique canary.
   Verified: all three delivery points (0.05/0.3/2 s) satisfy 143 + no-summary +
   no-survivors.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- A pre-existing `claude` wrapper function is UNVERIFIABLE by design (warned, never
  silently replaced or blessed); users with intentional wrappers own that setup.
- All prior dispositions unchanged.

Consensus: disagreed
