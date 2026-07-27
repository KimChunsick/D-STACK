## Carried decisions — Round 005
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
