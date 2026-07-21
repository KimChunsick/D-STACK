## Needed info

- Claude Code has the right primitive for per-session ownership: hook input includes `session_id`; `Stop` hooks can block with `decision:"block"`; `CLAUDE_CODE_SESSION_ID` is set in hook/Bash subprocesses, matches hook JSON for hooks/Bash, and is updated on `/clear`. Caveat: on `--continue` or `--resume` without an explicit ID it may receive the initial startup ID. [S1][S2]
- Mature local-file patterns separate two problems: mutual exclusion while editing shared state, and owner identity for cleanup. Git’s lockfile API uses `<file>.lock`, `O_CREAT|O_EXCL`, write-to-temp, atomic rename, and cleanup handlers; readers see old or new contents, but locks only block writers. [S6]
- `flock` is the simplest shell-level mutual exclusion tool for this scale: locks are tied to open file descriptions and released when descriptors close, but they are advisory, can be ignored by non-cooperating writers, and have NFS/CIFS caveats. [S5][S12]
- Concurrent append is safer than rewrite but not magic: `O_APPEND` makes offset adjustment plus write atomic on local Linux files, but NFS can corrupt concurrent appends; `write()` can be partial and successful writes are not guaranteed durable without `fsync()`. [S3][S4]
- PID/owner-scoped designs are common but tricky. `kill(pid, 0)` only checks whether a signalable PID currently exists; `pidfd_open()` is the modern Linux primitive for monitoring a specific process exit, but a plain text registry cannot store a live pidfd. [S9][S10]
- Mature session managers avoid stale plain-text ownership where possible: systemd recommends service-manager supervision and `sd_notify()` for new-style daemons; tmux keeps sessions in a server-managed registry with unique `session_id` and attached-client metadata. [S8][S11]

## Opposing views

- The strongest argument against per-owner filtering is fail-closed semantics: a global gate cannot silently skip another registered unfinished doc, while owner filtering can skip a line whose owner tag is wrong, stale, forged, or changed by `/clear`. This is especially relevant because `/clear` explicitly updates `CLAUDE_CODE_SESSION_ID`. [S2]
- Identity filtering is not a security boundary. Claude Code command hooks run with the user’s full permissions, and the guarded party can modify files it can write; if the agent can delete the registry line today, forging a different owner tag is not a materially new malicious bypass, but it is a new accidental-bypass mode. [S7]
- Stale-owner reclamation has a real false-positive risk if implemented from weak evidence. A PID existence check is not proof that the original owner is still alive; a pidfd would be better, but it is not naturally representable in this registry file. [S9][S10]
- Advisory locking solves cooperating-process races only. A hook or helper that appends/removes without taking the same lock can still corrupt or lose updates. [S5]
- Treating “unknown owner” lines as enforced by everyone is safer but reintroduces the repo-wide blocking problem. Treating them as ignored preserves isolation but weakens fail-closed behavior after `/clear` or crash. This is the core tradeoff, not an implementation detail. [S2][S5]

## For the goal

- The proposed session-id tag directly matches Claude Code’s documented session identity model for hooks and Bash subprocesses, so it is a sound way to distinguish concurrent terminal tabs in normal operation. [S1][S2]
- Per-owner scoping is consistent with mature tools that scope state to sessions or supervised units rather than one global process: tmux sessions and systemd service units are examples of owner-scoped lifecycle management. [S8][S11]
- For a handful of local processes on one machine, a single `flock` around registry read/write/rewrite is enough and simpler than PID liveness, pidfds, or a daemon. Use one lock file, serialize all registry mutations, and rewrite via temp file plus rename. [S5][S6]
- Fail-closed handling for untagged, empty-id, malformed, or unparsable lines preserves legacy safety and makes migration safer. This follows the same conservative pattern as Git’s lockfile design: uncertainty blocks writers rather than pretending state is clean. [S6]
- Leaving orphans can be acceptable if `/clear` is explicitly treated as ending the prior gated session and the registry is audit/debug state, not a security boundary. Add diagnostics for “orphaned owner ids” rather than blocking other sessions on them. [S2][S7]

## Against the goal

- If the workflow requirement is “an unfinished work-doc must block some future Stop until completed,” then plain session-id scoping is insufficient because `/clear` changes the id and can orphan incomplete tagged lines. [S2]
- If agents register docs themselves, owner tags are self-attested. A wrong tag can make the current Stop ignore the doc; untagged fail-closed does not protect against deliberate or buggy wrong-tagging. [S7]
- Automatic stale-owner cleanup is likely not worth it at this scale unless there is a reliable owner registry. Git cleans lockfiles on normal exit/signals, but that model is for a process-local lock object, not arbitrary work-doc ownership across Claude sessions. [S6]
- A better alternative for stronger fail-closed behavior is not “reclaim stale owners”; it is a stable work-owner token that survives `/clear` for the same terminal/work item, or a first-class per-session registry file under a session-owned directory. That avoids global blocking without making `/clear` silently drop enforcement. [S2][S11]

## Unverified

- I could not verify the current project’s actual hook code, registry update paths, or whether every writer can be forced through one `flock`.
- I could not verify the filesystem is local POSIX-style storage; NFS/CIFS would weaken both append and lock assumptions. [S3][S5]
- I could not verify whether `/clear` is intended to be an allowed escape hatch in this workflow or an ordinary continuation of the same work.
- I could not find a public Claude Code API for enumerating all live local sessions suitable for safe stale-owner reclamation.

## Sources

- [S1] Primary: Claude Code Hooks reference, no date, retrieved 2026-07-11. https://code.claude.com/docs/en/hooks
- [S2] Primary: Claude Code Environment variables, no date, retrieved 2026-07-11. https://code.claude.com/docs/en/env-vars
- [S3] Primary: Linux `open(2)`, Linux man-pages 6.18, 2026-02-08, retrieved 2026-07-11. https://man7.org/linux/man-pages/man2/open.2.html
- [S4] Primary: Linux `write(2)`, Linux man-pages 6.18, 2026-02-08, retrieved 2026-07-11. https://man7.org/linux/man-pages/man2/write.2.html
- [S5] Primary: Linux `flock(2)`, Linux man-pages 6.18, 2026-02-08, retrieved 2026-07-11. https://man7.org/linux/man-pages/man2/flock.2.html
- [S6] Primary: Git `api-lockfile`, last updated 2015-09-04, retrieved 2026-07-11. https://git-scm.com/docs/api-lockfile
- [S7] Primary: Claude Code Hooks security considerations, no date, retrieved 2026-07-11. https://code.claude.com/docs/en/hooks#security-considerations
- [S8] Primary: systemd daemon recommendations, no date, retrieved 2026-07-11. https://www.freedesktop.org/software/systemd/man/latest/daemon.html
- [S9] Primary: Linux `pidfd_open(2)`, Linux man-pages 6.18, 2026-02-08, retrieved 2026-07-11. https://man7.org/linux/man-pages/man2/pidfd_open.2.html
- [S10] Primary: Linux `kill(2)`, Linux man-pages 6.18, 2026-02-08, retrieved 2026-07-11. https://man7.org/linux/man-pages/man2/kill.2.html
- [S11] Primary: tmux manual, no date visible, retrieved 2026-07-11. https://man7.org/linux/man-pages/man1/tmux.1.html
- [S12] Primary: util-linux `flock(1)`, 2026-05-24, retrieved 2026-07-11. https://man7.org/linux/man-pages/man1/flock.1.html