# Codex adversarial review — Round 005

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The completion nonce is disclosed to the startup code it is supposed to verify, so `.zshrc` can forge successful verification before the predicate executes.
Evidence: `DSTACK_VERIFY_OUT` and `DSTACK_VERIFY_NONCE` are exported to `script`, and therefore visible while `.zshrc` runs. Success requires only that the disclosed nonce appear in the disclosed file. A verifier-aware startup branch can write it and exit before the alias predicate, while an ordinary terminal continues to a conflicting alias.
Suggested direction: At the verifier boundary, do not expose the expected response or output destination before startup completes. Consider driving a post-start query through the parent-controlled PTY; otherwise describe the result as heuristic and treat premature termination as unverifiable.
Illustrative example:
```zsh
source "$HOME/.claude/ultracode.zsh"
if [[ -n ${DSTACK_VERIFY_NONCE-} ]]; then
  print -r -- "$DSTACK_VERIFY_NONCE" > "$DSTACK_VERIFY_OUT"
  exit 0
fi
alias claude='claude --effort high'
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Install this fixture in a fake home. The installer currently emits no warning, while a normal terminal without the verifier variables resolves the high-effort override; the corrected verifier must reject or report this result as unverifiable.

[severity:medium][UI/UX & DX] Process ownership and cleanup traps are established only after the verifier is launched, leaving a cancellation window in which the installer exits while the verifier survives.
Evidence: `script ... &` executes before `vpid=$!`, the `vsig` definition, and every INT/TERM/HUP/EXIT trap. A signal delivered after the background process starts but before those traps are installed takes the shell’s default path, so neither `kill_tree` nor temporary-file cleanup runs. This is distinct from the accepted race involving a child spawned during tree traversal.
Suggested direction: Initialize verifier ownership and signal handling before launch, with a handler that safely defers cleanup until the child PID is registered. Keep the launch-to-registration section signal-safe or use an owned process-group boundary.
Illustrative example:
```text
spawn verifier
    ↓ signal arrives here
record PID
install cleanup traps
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Instrument a blocking verifier to pause immediately after launch, then deliver INT, TERM, and HUP during the pre-trap window. Require signal-appropriate installer status, no summary, no surviving verifier descendants, and no remaining nonce file.

[severity:low][UI/UX & DX] The timeout warning directs users to repeat the same startup execution without any timeout or containment.
Evidence: After detecting that zsh startup exceeded ten seconds, the message recommends `zsh -ic 'alias claude'`. That command executes the blocking `.zshrc` again and can wait indefinitely on the exact condition that triggered the warning.
Suggested direction: Tell the user to repair or inspect the blocking startup configuration first, then verify from a fresh shell. If an executable diagnostic is provided, keep the same bounded behavior as the installer.
Illustrative example:
```text
current: startup timed out → run another unbounded startup
desired: startup timed out → inspect/fix .zshrc → run bounded verification
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Use `.zshrc` fixtures containing an indefinite wait and an interactive read; follow the emitted recovery instructions and confirm they cannot hang or leave descendants.

GPT verdict: reject — the verifier still has a reproducible forged-success path, and cancellation can escape cleanup before verifier ownership is established.

## Maintainer response
1. **Agreed in the casual case, fixed; rebutted at the adversarial boundary.** The
   nonce and output path no longer enter the environment — they ride inside the zsh
   command string — so the demonstrated `${DSTACK_VERIFY_NONCE-}` startup forge no
   longer has anything to read (new probe p: the reviewer's exact fixture now
   produces the warning). The residual — a startup file that parses `ps` for the
   verifier's argv — is explicitly out of the trust model and recorded as such:
   `~/.zshrc` already executes arbitrary code in every shell (it could redefine
   `claude` outright, or tamper with the installer itself); this check exists to
   catch ACCIDENTAL breakage, which is the failure mode that actually occurred. The
   installer comment now states this boundary.
2. **Agreed (pre-registration cancellation window) — fixed by ownership-by-unique-
   token.** All traps (INT/TERM/HUP/EXIT) are installed BEFORE the verifier is
   spawned, and the handlers locate the verifier tree via the run-unique `$vout`
   path embedded in its argv (`pgrep -f "$vout"`), so even a signal landing in the
   instant before `$!` is recorded still reaps the whole tree. Verified (new probe
   q): TERM delivered at 0.05 s / 0.3 s / 2 s after launch — no verifier survivors,
   signal-appropriate installer status each time.
3. **Agreed (recovery instruction) — fixed.** The timeout message now says to
   inspect/fix the blocking `~/.zshrc` first and re-run `./install.sh` for a bounded
   re-check, instead of recommending an unbounded `zsh -ic` by hand.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Trust boundary recorded: a verifier-aware hostile `~/.zshrc` (ps-parsing) is out
  of the trust model — the check targets accidental breakage (R5-1 residual).
- Ownership-by-unique-token is the cleanup identity for every cancellation path
  (R5-2); tree-kill's mid-kill spawn race stays an accepted residual.
- All prior dispositions unchanged (user decision on tests; installer WARN-only
  policy; pty-based verification; documented prerequisites).

Consensus: disagreed
