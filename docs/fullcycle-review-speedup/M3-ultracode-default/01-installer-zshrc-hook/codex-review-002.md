# Codex adversarial review — Round 002

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The Round 1 semantic check still has reproducible false negatives because it validates text emitted by `alias`, not whether zsh will expand the managed alias.
Evidence: The predicate accepts any output containing `--effort ultracode`. A local zsh probe reproduced two failures: after `unsetopt aliases`, `alias claude` still prints the expected definition although invoking `claude` no longer expands it; an unrelated alias body such as `printf "not claude: --effort ultracode"` also passes the exact `grep` predicate. Additionally, startup output and probe output share one pipe, so any `.zshrc` diagnostic containing that substring can suppress the warning.
Suggested direction: At the semantic-check boundary, validate both the `ALIASES` option and the exact managed alias value, returning the result through an isolated status channel rather than grepping combined startup output. Alternatively, enforce a deterministic installer-owned tail block.
Illustrative example:
```zsh
source "$HOME/.claude/ultracode.zsh"
unsetopt aliases

alias claude  # prints the expected definition
claude        # does not expand the alias
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add fake-HOME probes for `unsetopt aliases`, a replacement alias whose body merely contains the searched substring, and noisy startup output containing that substring. Each must produce the warning.

[severity:medium][UI/UX & DX] The installer now launches an unbounded interactive shell that can block or replay arbitrary `.zshrc` side effects.
Evidence: `zsh -ic` executes the complete interactive startup configuration. Its stdin remains attached, there is no timeout, and stderr is suppressed. A `.zshrc` containing an interactive `read`, automatic `exec tmux`, or another long-running startup action prevents the installer from reaching its summary; state-changing startup commands are also executed an extra time.
Suggested direction: Avoid executing interactive startup files on the default installation path; expose semantic activation as an explicit diagnostic instead. If automatic execution is retained, detach stdin, impose a deadline, isolate the result channel, and report timeout/startup failure distinctly.
Illustrative example:
```zsh
# ~/.zshrc
[[ -o interactive ]] && read 'reply?Continue startup? '

# install.sh -> zsh -ic -> waits for terminal input
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Use fake-HOME fixtures whose `.zshrc` waits for input, exits early, and records a startup side effect. Assert that ordinary installation finishes within a fixed deadline and reports an indeterminate check rather than hanging or silently succeeding.

[severity:low][the real Why] Runtime prerequisites beyond the environment-variable caveat remain undocumented and invisible to the alias check.
Evidence: Current Claude Code documentation says ultracode is available only on models supporting `xhigh`; it also documents persistent user, environment, and managed controls that disable workflows and remove ultracode from the effort menu. The header records only the minimum CLI version and `CLAUDE_CODE_EFFORT_LEVEL`, while the post-check passes whenever the alias text exists. [Claude Code workflow documentation](https://code.claude.com/docs/en/workflows), [model configuration](https://code.claude.com/docs/en/model-config)
Suggested direction: At the fragment documentation boundary, record the supported-model and workflow-enabled prerequisites as accepted overrides or limitations. Do not claim that alias resolution alone establishes an effective ultracode session.
Illustrative example:
```json
{ "disableWorkflows": true }
```
```text
alias probe → passes
ultracode workflow availability → disabled
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: On Claude Code 2.1.216, exercise the wrapper with workflows enabled and disabled and with xhigh-capable and unsupported models; confirm the documented outcome in each case.

[severity:low][software structure/design] The claimed rerunnable seven-assertion probe and transcript are absent from the supplied completion record.
Evidence: The maintainer response says the full transcript is recorded in `task.md` §E2E, but that section contains only “evidence recorded on completion”; the TDD and E2E gates remain unchecked. This does not challenge the user’s decision against committed per-feature tests, but it leaves the stated mitigation independently unreproducible.
Suggested direction: Record the exact scratchpad probe content or invocation, its seven assertion results, and its negative-control failure in the E2E section. Keep it documentation-only if that is the carried user decision.
Illustrative example:
```text
Probe: <exact rerunnable command or embedded script>
Assertions: 7/7
Exit: 0
Negative control: remove semantic check → assertion N fails
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Starting from the supplied record alone, have an independent reviewer reproduce all seven assertions and at least one intentional failure.

GPT verdict: reject — the replacement semantic check can still silently approve an ineffective alias and can block installation by executing an unbounded interactive startup.
tokens used

## Maintainer response
1. **Agreed (alias-output validation) — fixed.** The check no longer greps combined
   startup output. It now runs a bounded, stdin-detached interactive zsh whose only
   report channel is its exit status: `zmodload zsh/parameter`, require `[[ -o
   aliases ]]` (catches `unsetopt aliases`), and compare `${aliases[claude]}` for exact
   equality with the managed alias value (catches substring-fake bodies). Startup
   stdout/stderr are discarded, so noise can neither fake nor mask the verdict.
   Verified with new fixtures (probes g/h/i): `unsetopt aliases`, a replacement alias
   whose body merely contains the substring, and a noisy startup + real override each
   produce the warning.
2. **Agreed (unbounded interactive shell) — fixed.** The probe zsh runs backgrounded
   with `</dev/null`, is polled up to 10 s, then SIGKILLed (interactive zsh ignores
   TERM) and reaped; timeout prints a distinct "effectiveness UNVERIFIED" warning
   instead of a false verdict, and the installer always reaches its summary. Verified
   (probe j): a `.zshrc` with `sleep 30` completes the install in ~12 s with the
   UNVERIFIED warning. Residual, recorded: interactive startup files execute once
   during the check — same class as opening a new terminal; state-changing zshrc
   startup is out of this repo's remit.
3. **Agreed (prerequisites) — documented.** The fragment header now records the two
   runtime prerequisites beyond the env var: the session model must support xhigh, and
   workflows must not be disabled (user/managed "disableWorkflows"), with the explicit
   statement that alias resolution alone does not establish an effective ultracode
   session.
4. **Agreed (record) — done.** The task's E2E section now embeds the exact probe
   script (11 assertions incl. the new g–j fixtures) and its observed transcript, so
   the mitigation is reproducible from the artifact alone; suite retirement itself
   remains the carried user decision.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- User decision (interview Q3): no committed per-feature suite; probes live as
  recorded rerunnable scripts inside the task artifact.
- Installer policy: semantic check WARNS (effective / ineffective / UNVERIFIED-on-
  timeout), never hard-fails; interactive startup executes once during the check
  (accepted residual).
- Accepted residuals restated: interactive-zsh-only coverage, ZDOTDIR placement,
  CLAUDE_CODE_EFFORT_LEVEL precedence, model/workflow prerequisites (documented).

Consensus: disagreed
