# Codex adversarial review — Round 006

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The verifier uses its temporary pathname simultaneously as unquoted zsh source and as a regular-expression process identifier, so valid `TMPDIR` values can defeat both verification and bounded cleanup.
Evidence: `$vout` is inserted directly into the inner command as `> $vout`, without zsh quoting, while `pgrep -f "$vout"` interprets the same pathname as a regex. For a valid temporary directory such as `/tmp/dstack [probe]`, whitespace and brackets alter inner-shell parsing, and `[probe]` does not literally match the verifier’s argv. During blocked startup, `vkill` can therefore find nothing; the subsequent `wait "$vpid"` then waits indefinitely. Metacharacters such as `;` can also turn the pathname into additional inner-zsh commands. Independently, the pre-exec cancellation window remains: after the background fork but before `script` exposes `$vout` in its argv, token-based `pgrep` cannot identify the child. Probe (q) neither instruments that window nor requires signal termination—it explicitly accepts exit status 0.
Suggested direction: At the verifier boundary, keep paths out of generated shell source, or encode/quote them for zsh explicitly. Track ownership through a known PID/process group or a ready supervisor rather than regex-searching argv; defer signal completion until ownership is registered.
Illustrative example:
```text
vout=/tmp/dstack [probe]/tmp.ABC

inner source:  print ... > /tmp/dstack [probe]/tmp.ABC
cleanup regex: /tmp/dstack [probe]/tmp.ABC
actual argv:   ... /tmp/dstack [probe]/tmp.ABC

timeout → regex misses verifier → wait remains blocked
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Run the blocked-startup fixture with owned `TMPDIR` paths containing spaces, brackets, and semicolons; require bounded completion, no unintended commands, and no survivors. Instrument a delayed fork-to-exec transition, signal during it, and require strict signal-specific statuses rather than permitting 0.

[severity:medium][the real Why] The nonce certifies only a static alias-table entry, while zsh can transform that alias during actual command expansion and silently launch a non-ultracode command.
Evidence: The predicate checks `ALIASES` and exact `${aliases[claude]}` text, but zsh recursively applies global aliases to words introduced by the managed alias. A fixture with `alias -g ultracode=high` retains the exact managed `claude` entry and passes the nonce predicate, yet invoking `claude` expands to `claude --effort high`. A pre-existing `claude` function can similarly receive or rewrite the expanded arguments while remaining invisible to the predicate. This is an accidental configuration path, not verifier-aware hostile startup code.
Suggested direction: At the semantic-verifier boundary, test lexical expansion through a harmless capture stub and detect underlying `claude` functions as verified, incompatible, or explicitly unverifiable. The invariant should concern the resulting invocation arguments, not merely the stored alias definition.
Illustrative example:
```zsh
function claude { print -r -- "$@" }
alias -g ultracode=high
alias claude='claude --effort ultracode'

# Static predicate succeeds.
claude  # receives: --effort high
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In the pseudo-terminal fixture, capture the arguments produced by invoking `claude` with a global `ultracode` alias and with a pre-existing wrapper function. Neither configuration may be reported as verified when the resulting invocation lacks effective ultracode selection.

GPT verdict: reject — pathname handling can break containment and boundedness, while the semantic verifier still has a reproducible silent-success path that defeats the intended default.

## Maintainer response
1. **Agreed (pathname/regex/window handling) — fixed.** The channels were split by
   sensitivity: the OUTPUT PATH now travels via the environment again and is
   expanded inside the zsh source as a quoted variable (`"$DSTACK_VERIFY_OUT"`), so
   any valid TMPDIR — spaces, brackets, semicolons — is inert in the generated
   source (env exposure of the path is safe: without the nonce, a startup file can
   only write garbage, which is fail-closed); the NONCE stays argv-only (forge
   protection, R5-1) and — being a safe unique token — replaces the raw path as the
   `pgrep -f` ownership key, eliminating the regex-metacharacter miss. The signal
   handlers now fall back to `${vpid:-$!}` (available from the instant of the fork),
   closing the fork-to-exec identification gap. Probe (q)'s acceptance of exit 0 is
   intentional and now justified in place: at the 2 s delivery point the check may
   already have completed legitimately — the enforced invariants are "no survivors"
   and "no fake success summary", which are asserted. Verified (new probe t): a
   TMPDIR of `…/tmp dstack [probe]` with a blocking startup completes bounded,
   reports UNVERIFIED, leaves no survivors and no stray files.
2. **Agreed (static table ≠ actual expansion) — fixed with an expansion-capture
   stub.** The predicate no longer reads the alias table: it defines a capture
   function named `claude`, `eval`s the bare word `claude` (alias expansion applies,
   including global aliases and any wrapper the alias resolves into), and requires
   the CAPTURED arguments to equal `--effort ultracode`. `alias -g ultracode=high`
   and arg-rewriting overrides now fail the predicate; `unsetopt aliases` fails it
   naturally (bare call, empty args), subsuming the old `[[ -o aliases ]]` check.
   Verified (probes r/s + full battery re-run): global-alias rewrite and override
   warn; the clean install stays warning-free.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Channel design recorded: path-via-env (quoted expansion; fail-closed without the
  nonce), nonce-via-argv (forge protection + ownership key); `${vpid:-$!}` fallback
  in handlers. A `command claude`-prefixed user alias bypasses the capture stub and
  is reported UNVERIFIED after the bounded timeout — conservative by design.
- All prior dispositions unchanged (trust boundary for ps-parsing startup files;
  WARN-only installer policy; user-decided suite retirement).

Consensus: disagreed
