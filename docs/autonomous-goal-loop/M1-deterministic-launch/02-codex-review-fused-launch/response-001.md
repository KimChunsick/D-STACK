# Maintainer response — Round 001

Not bundled into any review round.

## F001 [medium] the recovery rule overstated teardown — AGREED, fixed

Right, and the consequence the finding names is the expensive one: Step 2a treats a missing `exit`
record as a failed round and says retry, so an orphan left by an untrappable kill would have been
joined by a second paid round on the same bundle. The guarantee is now scoped to catchable
termination, and the retry path carries a liveness check on `.launch/child` (pid *and* group)
before relaunching, with `rm-run`'s matching refusal named as the reason the evidence stays put.

## F002 [medium] Step 3 forbade and required the maintainer response — AGREED, fixed

Correct: I fixed the template and §2 but left the sentence below the template saying each file
contains "one maintainer response". Three sites disagreed at once, which is worse than either
answer. The sentence now states the round file's contents explicitly and says where the response
lives, and it names its own former text so the contradiction is not silently re-introduced.

## F003 [medium] the skip gate scanned untrusted bundle content — AGREED, fixed

The sharpest of the three, because the previous repair had already been made once for the same
class and did not go far enough. The check now iterates the allowlist and does a FIXED-STRING match
per path, so ordinary content cannot impersonate a marker for a file this round did not name. The
allowlist became an array used by both the assembler call and the check, since two hand-kept copies
would drift; the no-variables rule was only ever about an unquoted scalar, and that is now stated
precisely rather than as a blanket ban.

Verified against real bundles, including a reproduction of the reviewer's exact counterexample:

```
1. genuinely good bundle                                  → accept
2. same bundle + '--- docs/example.md (SKIPPED: illustrative text) ---' appended
     OLD anchored check: REFUSES (false positive reproduced)
     NEW per-path check: accept
3. allowlist names a nonexistent file                     → REFUSE: 'does/not/exist.md' was skipped
4. allowlist names a symlink (secret-deny/symlink path)   → REFUSE: '<path>' was skipped
```

**Residual, recorded as a follow-up rather than fixed here.** The assembler publishes skip status
only inside the bundle it emits, so content it copies verbatim can still impersonate a marker for
one of your *own* allowlisted paths. The real fix is a channel the payload cannot write — a
manifest on stderr, or a distinct exit status from `assemble-review.sh`. That file is not in this
task's declaration, and the ratchet rule forbids growing an allowlist to absorb a finding, so it is
a follow-up for its own review unit.

## F004 [low] the scratch directory leaked — AGREED, fixed

`SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT`. The old detached launcher cleaned its own
scratch dir inside `run.sh`; that cleanup was lost with the launcher and not replaced. The same fix
went into `codex-research`, which had the identical pattern.

## Class-wide sweep (Step 0)

Class: *a guarantee stated wider than it holds*. Swept every claim this file makes about what
happens on failure — teardown (narrowed), the retry rule (now gated on liveness), "never mutate the
tree" (that one is in `codex-research`, fixed there in the same pass), and the skip check's
"the only honest signal" (narrowed to markers naming allowlisted paths). Also swept for the F002
class — one instruction contradicting another — across the round-file shape: template, §2, and the
sealing sentence now agree.
