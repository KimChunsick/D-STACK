# Codex adversarial review — Round 004

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The detached semantic check can silently approve an alias that a real terminal overrides.
Evidence: With stdin redirected from `/dev/null`, `[[ -t 0 ]]` is false. A legacy hook followed by `[[ -t 0 ]] || return` returns only from `.zshrc`; zsh then executes the predicate, finds the expected alias, and writes the nonce. In a real terminal, startup continues to a later override. The new `exit`/`exec` probes do not cover this ordinary `return` control flow.
Suggested direction: At the semantic-check boundary, exercise the terminal path through a bounded pseudo-terminal, or stop claiming that the detached result verifies real interactive resolution. Preserve an invariant that the tested startup branch matches the launch mode being protected.
Illustrative example:
```zsh
source "$HOME/.claude/ultracode.zsh"
[[ -t 0 ]] || return
alias claude='claude --effort high'

# detached probe → ultracode + nonce
# real terminal  → high override
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add this exact legacy ordering as a fake-HOME fixture. Assert that the installer reports ineffective resolution while `zsh -ic` runs through a pseudo-terminal, and confirm the resulting terminal alias is the high-effort override.

[severity:medium][UI/UX & DX] The INT/TERM trap converts cancellation into a successful installer run and leaves other termination paths uncontained.
Evidence: `trap 'kill_tree "$vpid"; rm -f "$vout"' INT TERM` performs cleanup but neither exits nor re-raises the signal. Execution therefore continues through `wait`, warning generation, the summary, and normally exits zero. `HUP`, `EXIT`, and unexpected-error cleanup are absent, so targeting only the installer with one of those paths can leave the detached verifier running.
Suggested direction: Centralize verifier cleanup at the probe boundary, track whether its PID is still owned, and cover EXIT/HUP as well as INT/TERM. After cleanup, preserve cancellation by exiting with the corresponding signal status or restoring and re-raising the signal.
Illustrative example:
```text
TERM → cleanup → current behavior: warning → Summary → exit 0
TERM → cleanup → required behavior: reap child → exit 143
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Block startup with a distinctive child, signal only the installer PID with INT, TERM, and HUP, and inject an error during polling. Assert nonzero signal-appropriate status, no success summary, no surviving descendants, and no temporary output file.

[severity:medium][security] The published probe still fails to retain ownership of its resources and can terminate an unrelated process.
Evidence: Every `FAKE="$(mkfake)"` executes `mkfake` in a command-substitution subshell, so its `ROOTS+=("$r")` mutation never reaches the parent; ten fake roots are therefore omitted from the EXIT cleanup. Separately, `pgrep -f "sleep 31449"` followed by global `pkill -f` is not tied to the verifier’s PID tree and kills any same-user process with that command line.
Suggested direction: Register each temporary root in the parent shell rather than inside command substitution. Retain the canary PID or process-group identity created by the fixture and inspect or terminate only that owned identity; never use a global command-line match for cleanup.
Illustrative example:
```text
FAKE=$(mkfake)
  subshell ROOTS=[FAKE]
parent ROOTS=[]

unrelated "sleep 31449" → global pkill selects it
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Record the temp-directory set before and after every success and failure path and require no additions. Run an unrelated sibling process with the same canary command and assert that it remains alive while only the verifier-owned process tree is checked.

GPT verdict: reject — the effectiveness check retains a reproducible silent-success path, cancellation is reported as success, and the recorded probe can kill an unrelated process.

## Maintainer response
1. **Agreed (detached probe ≠ terminal path) — fixed with a real pseudo-terminal.**
   The probe now runs under `script -q /dev/null zsh -ic …`, so startup takes the
   same `[[ -t 0 ]]`-true branch a human terminal takes; the `return`-based legacy
   shape (and the earlier `exit` shape) now reach their overrides and fail the nonce
   check. When `script(1)` is unavailable the installer says so explicitly instead
   of pretending ("effectiveness not verified"). Verified (probes m and new o): both
   non-TTY-early-exit shapes followed by an override produce the warning; a blocking
   `read`-style startup is bounded by the existing timeout path.
2. **Agreed (signal handling) — fixed.** Cleanup now covers INT/TERM/HUP *and* EXIT
   (so a `set -e` error mid-poll also reaps the verifier), and a signal re-raises
   after cleanup: the installer exits with the signal status instead of continuing
   to a fake success summary. Verified: probe battery unaffected; manual TERM during
   the check window kills the tree and the installer dies with 143 (no Summary
   line).
3. **Agreed (probe-script ownership bugs) — fixed.** `mkfake` now registers each
   root in the parent shell (no command-substitution subshell), so the EXIT trap
   removes every created root; the orphan check uses a run-unique canary duration
   and is assert-only — no global `pkill`; on failure it reports the PID for manual
   handling instead of killing by command-line match. Verified: post-run, the owned
   roots are gone and the temp parent is otherwise untouched; the canary assertion
   passes via tree-kill alone.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Installer policy: pty-based, nonce-verified semantic check WARNS (effective /
  ineffective / UNVERIFIED-on-timeout / not-verified-when-tools-missing), never
  hard-fails; cancellation re-raises; startup files execute once during the check;
  tree-kill races a mid-kill spawn (accepted residuals).
- User decision (interview Q3) and prior residuals unchanged.

Consensus: disagreed
