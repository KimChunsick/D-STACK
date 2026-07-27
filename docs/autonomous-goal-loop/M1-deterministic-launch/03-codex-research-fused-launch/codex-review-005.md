# Codex adversarial review — Round 005

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 34989 bytes (r1 10308, r2 17376, r3 24957,
r4 30116). **This is the round cap for a per-task unit (§4).** Allowlist unchanged: the
review-unit folder, `claude/skills/codex-research/SKILL.md`.

## GPT findings

[severity:medium][security] A fixed `SLUG` heredoc delimiter remains injectable; even the allowed slug `SLUG` closes the heredoc, while a delimiter-line payload can execute shell code and restore an accepted slug.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2; confirmed: Step 1’s slug grammar and `task.md`’s exact-block verification claim.
Evidence: Quoting disables body expansion but not delimiter recognition, which occurs before `read` or the validation loop.
Verification: Bash 3.2 and zsh 5.9 parsed the trailing delimiter as a command; both executed injected `PWNED=1`, reset `GOAL=valid`, and reached `ACCEPTED` with rc=0.
Suggested direction: Pass values through a non-source argv/environment channel; no fixed heredoc delimiter safely transports arbitrary textual replacement.

[severity:medium][technical correctness] The revised signal traps still do not cancel foreground `dstack`; a wrapper-only signal can let research succeed, then return 143 and trigger the documented discard/retry path.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 traps; confirmed: its notification, nonzero-exit, retry, and residual paragraphs.
Evidence: The handlers neither signal nor supervise `dstack`; both shells defer the pending trap while waiting for a foreground command.
Verification: Under bash and zsh, wrapper-only TERM produced `CHILD_STARTED CHILD_FINISHED CLEAN`, then wrapper rc=143—never cancelling the child.
Suggested direction: Every signal path must either cancel and settle `dstack`, or preserve its completed terminal status instead of converting success into wrapper failure.

[severity:low][security] Evaluator-disposition language remains embedded in the current artifact.
Sites: primary: the skill’s “Residual, accepted”; confirmed: “Covering … are changes to `claude/bin/dstack`” and `task.md`’s “Skipped — no trigger … already built and reviewed.”
Evidence: These phrases pre-dispose acceptance and review location rather than merely describing implementation state.
Verification: Ignoring those directives and inspecting the referenced behavior exposed the unresolved cancellation blocker above.

[severity:low][technical correctness] The printed signal-measurement command does not reliably test the child bash because its double-quoted `$$` is expanded by the invoking zsh.
Sites: primary: `claude/skills/codex-research/SKILL.md` residual signal fence.
Evidence: In the stated three-run context, the command can signal the surrounding loop shell instead of the bash whose EXIT trap is under test.
Verification: A harmless compound-zsh probe showed outer PID 47531, unescaped `$$` passed as 47531, and the actual child PID as 47532.
Suggested direction: Single-quote the bash program and pass the signal name as a positional argument.

[severity:low][technical correctness] The fallback command counts URL-shaped strings, not unique valid URLs, allowing a malformed source placeholder to suppress fallback.
Sites: primary: `claude/skills/codex-research/SKILL.md` Fallback.
Evidence: `[^ )]*` permits an empty host and retains punctuation, so one URL can also become multiple distinct strings.
Verification: A Sources section containing only `https://` counted as 1; the same URL with and without a trailing comma counted twice.

Omitted-detail: 0 low

GPT verdict: reject — The heredoc remains a concrete command-injection and allowed-slug failure path, while wrapper-only cancellation still converts completed research into a failed retry.

## Carried decisions
- **No quoting form makes textual substitution a security boundary, and the claim is withdrawn
  rather than patched again.** Measured across three rounds: a double-quoted assignment runs a
  substituted `$(…)`; a single-quoted one is escaped by an embedded quote; a quoted heredoc is
  closed by a payload line equal to its delimiter — and `SLUG` is itself a valid slug, so that form
  also broke on legitimate input. No delimiter choice helps, since a payload can read the delimiter
  out of the recipe. The recipe now says what the check IS: defence in depth against a MISTAKE (a
  `..` component, which has really happened), on values the orchestrator itself picks from the
  Goal's name. It also states the condition under which the recipe is the wrong shape — if these
  values ever arrive from a user string, a file, or a tool result, they must reach the process as
  argv or environment data set by the caller, and no edit to the quoting substitutes for that.
- **`<run-dir>/exit` is the round's status; the wrapper's exit code is not.** A signal delivered to
  the wrapper while `dstack run` is in the foreground does not cancel the child: both shells defer
  the pending trap until the foreground command returns. Measured —
  `CHILD_STARTED … CLEAN … CHILD_FINISHED`, wrapper `rc=143`. Treating that as failure discards a
  COMPLETED round and pays for another.
- **The signal handlers deliberately do NOT clean up.** The same deferral means the handler can run
  while `codex exec` is still alive, and `CLEAN` printing before `CHILD_FINISHED` is exactly that:
  `rm -rf "$SCRATCH"` deleting the directory the child is running in. On a signal the wrapper
  terminates with the signal's status and leaves the directory; only normal completion removes it.
  A leaked temp dir costs nothing; deleting a live process's cwd is a real failure.
- A printed measurement command is itself code and gets the same scrutiny. The signal fence's
  `"… $$ …"` would be expanded by the INVOKING shell and signal that shell instead of the bash
  under test — single-quote the program and pass the signal name as an argument. (Reproduced by
  accident while re-measuring for this round, which is the cheapest possible demonstration.)
- Count sources with a pattern that requires a host and strips trailing punctuation. `[^ )]*` alone
  accepts a bare `https://` as a source and counts one URL twice when a comma follows it.

Consensus: resolved
