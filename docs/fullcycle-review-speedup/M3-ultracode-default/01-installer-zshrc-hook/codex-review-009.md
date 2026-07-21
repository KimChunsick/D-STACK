# Codex adversarial review — Round 009

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The verifier still executes alias bodies and counts only calls reaching its injected function, allowing bypassed commands to run and a non-ultracode-first alias to be falsely verified.
Evidence: `eval claude` executes the complete expanded alias. `DSTACK_CALLS` ignores invocations using `command claude`, `env claude`, or an absolute executable path because these bypass the injected function. Reproducing the predicate with `alias claude='command printf "first-invocation:--effort-high\n"; claude --effort ultracode'` executed the first command and then reported `VERIFIED`. A realistic `command claude --effort high; claude --effort ultracode` can therefore launch Claude at high effort during installation and, if it returns within ten seconds, still write the success nonce. This is beyond the accepted residual that startup files execute once: an alias body normally remains inert during shell startup.
Suggested direction: At the semantic-verifier boundary, stop evaluating the alias body. Validate the exact managed alias plus alias-expansion state, relevant global aliases, and pre-existing functions using zsh metadata; conservatively reject constructs that cannot be proven safe without execution.
Illustrative example:
```zsh
alias claude='command claude --effort high; claude --effort ultracode'

# Verifier observes only the second call:
external high launch → injected function(ultracode) → DSTACK_CALLS=1 → false success
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Put a harmless fake `claude` executable first in `PATH`, recording its arguments, and install the compound alias above. The corrected installer must warn, must not write the success nonce, and—if verification is made non-executing—must not invoke the fake executable.

GPT verdict: reject — the verifier can execute real alias commands and still silently certify a configuration whose first Claude invocation is not ultracode.

## Maintainer response
**Agreed — execution abandoned entirely, as the finding directs.** The eval-based
capture was the wrong boundary: an alias body with a `command`/`env`/absolute-path
prefix bypasses any injected function AND really executes during install. The
predicate now performs NON-EXECUTING metadata validation only: `[[ -o aliases ]]`,
no pre-existing `functions[claude]`, byte-exact `aliases[claude]` equal to the
managed text, and no `galiases` entry hijacking any word of the expansion
(`claude`, `--effort`, `ultracode`). Anything not provably equivalent without
execution is conservatively reported ineffective. Implementation note recorded for
posterity: the first version of this fix was itself defeated by the very attack it
checks for — the unquoted word `ultracode` in the verifier's `for` list was
global-alias-expanded to `high` before the check ran; the verifier's own source
words are now quoted (global aliases do not expand quoted words), and the probe
caught this live. Verified (probe v, per the requested method): a PATH-first fake
`claude` executable records zero invocations while the compound
`command claude --effort high; claude --effort ultracode` alias produces the
warning; all 24 probes pass, the clean install stays warning-free.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- The verifier is non-executing by contract: it proves the managed alias is the
  exact, unhijacked expansion or reports ineffective/unverifiable; it never runs
  alias bodies. Textual strictness (a semantically identical but differently
  spelled alias warns) is the accepted conservative side.
- All prior dispositions unchanged.

Consensus: disagreed
