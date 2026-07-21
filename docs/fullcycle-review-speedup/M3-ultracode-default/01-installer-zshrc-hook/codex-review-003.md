# Codex adversarial review — Round 003

## Review scope
Re-review

## GPT findings
[severity:high][security] The published “rerunnable as-is” probe can recursively delete the entire temporary-file parent directory.
Evidence: Probe (g) runs `rm -rf "$(dirname "$FAKE_G")"`. If `mktemp -d` returns `/tmp/tmp.ABC`, the deletion target is `/tmp`; on macOS it may resolve to the user’s complete `$TMPDIR`. This can remove unrelated files, sockets, sessions, or even another workspace. Fixtures H–J also lack correct cleanup.
Suggested direction: At the probe boundary, create one owned temporary root and remove only that exact validated path from an EXIT trap. Never derive a recursive-deletion target with `dirname`.
Illustrative example:
```text
FAKE_G=/var/.../T/tmp.ABC
current cleanup target=/var/.../T
required cleanup target=/var/.../T/tmp.ABC
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable sandbox containing a sibling canary, resolve the cleanup target without executing the current deletion. After correction, run every success and failure path and assert that only the owned probe root disappears.

[severity:medium][technical correctness] A successful early exit from `.zshrc` bypasses the semantic predicate and is incorrectly treated as verified.
Evidence: The `elif wait "$vpid"; then :` branch interprets any exit status 0 as predicate success. Interactive startup files execute before the `-c` predicate, so `.zshrc` can run `exit 0` or `exec true` first. This is especially reproducible because stdin is `/dev/null`: a legacy configuration can source the hook, execute `[[ -t 0 ]] || exit 0` during the installer probe, and contain a later alias override that executes in a real terminal. The installer then emits no warning even though the terminal resolves the override.
Suggested direction: Require an authenticated completion sentinel emitted only by the predicate through an isolated channel; absence of the sentinel must mean UNVERIFIED regardless of process exit status. Consider testing the actual terminal-dependent path through a pseudo-terminal.
Illustrative example:
```text
detached probe: legacy hook → non-TTY exit 0 → predicate skipped → false success
real terminal:   legacy hook → later high-effort override → ultracode ineffective
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add fixtures for unconditional `exit 0`, `exec true`, and a `[[ -t 0 ]]` branch followed by an override. Each must report UNVERIFIED or ineffective rather than silently succeed.

[severity:medium][UI/UX & DX] The timeout kills only the zsh parent, so external startup commands can survive beyond the claimed ten-second bound.
Evidence: `kill -KILL "$vpid"` does not terminate descendants. I reproduced this with an interactive zsh waiting for an external `sleep`: killing the zsh PID left the child alive until it was separately killed. During real startup, zsh must return from `.zshrc` before executing the supplied `-c` predicate, so a foreground external command remains a child rather than replacing zsh. An unbounded command can therefore be orphaned indefinitely. This is distinct from the accepted residual that startup side effects execute once.
Suggested direction: Avoid executing startup configuration during ordinary installation, or isolate the diagnostic in its own process group and terminate and reap the entire group on timeout and interruption. Ensure the suggested manual diagnostic does not repeat the same unbounded startup behavior.
Illustrative example:
```text
installer → zsh → blocking startup child
             └─ SIGKILL
installer ends     child continues
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Use a uniquely identifiable indefinite child process, trigger timeout and installer interruption, and assert afterward that neither the probe shell nor any descendant remains.

GPT verdict: reject — the recorded verification is dangerously destructive, while the semantic check still has a silent-success path and does not contain timed-out startup processes.

## Maintainer response
1. **Agreed (destructive probe cleanup) — fixed, and acknowledged as the exact defect
   class the reviewer warned about.** The published probe script derived a recursive
   deletion target with `dirname` on a `mktemp -d` result, which resolves to the
   shared temp parent — a genuinely dangerous cleanup. The battery was rewritten:
   every fixture path is appended to an owned `ROOTS` array and a single EXIT trap
   removes exactly those validated paths; `dirname`-derived deletion is banned in the
   script's header contract. The M1 battery had the same latent shape (clone-dir
   cleanup) and was rewritten to the same owned-roots pattern — class-wide, not
   instance-wise. Verified: full run leaves the shared temp parent intact (sibling
   files unaffected) and removes every owned root.
2. **Agreed (early-exit false success) — fixed.** The success signal is no longer the
   exit status: the predicate must write a per-run nonce to an installer-owned file,
   and only a byte-exact nonce match counts as verified. A `.zshrc` that runs
   `exit 0`, `exec true`, or `[[ -t 0 ]] || exit 0` before the predicate produces no
   nonce and now triggers the warning. Verified (probes k/l/m): all three shapes —
   including the reviewer's exact non-TTY-early-exit + later-override scenario —
   produce the warning.
3. **Agreed (orphaned descendants) — fixed.** Timeout and installer interruption now
   kill the probe's whole process tree: a recursive `pgrep -P` walk SIGKILLs
   descendants before the shell (with an INT/TERM trap during the check window).
   Verified (probe n): a distinctive blocking startup child (`sleep 31449`) is gone
   after the bounded check returns; nothing matching it survives. Residual, recorded:
   the pgrep walk races a child spawned mid-kill — accepted for an installer
   diagnostic on a single-user machine.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- User decision (interview Q3): no committed per-feature suite; probes live as
  recorded rerunnable scripts (now parameterized, owned-root cleanup).
- Installer policy: nonce-verified semantic check WARNS (effective / ineffective /
  UNVERIFIED-on-timeout), never hard-fails; startup files execute once during the
  check; tree-kill races a mid-kill spawn (accepted residuals).
- Prior residuals restated: interactive-zsh-only coverage, ZDOTDIR placement,
  CLAUDE_CODE_EFFORT_LEVEL precedence, model/workflow prerequisites (documented).

Consensus: disagreed
