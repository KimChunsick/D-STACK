# Codex adversarial review — Round 002

## Review scope
Re-review | `REVIEW_MODE=serial` | bundle 39716 bytes (round 001: 40922 — ratchet held)
Allowlist SHRANK: the research artifact was dropped; round 001 had already challenged its
assumptions. Remaining: the review-unit folder, `claude/bin/dstack`, `AGENTS.md`.

## GPT findings

[severity:high][technical correctness] The F001 fix still has catchable-signal gaps: teardown starts after child recording and ends before status publication, so TERM/HUP can orphan a live child or prevent the terminal record.
Sites: primary: `claude/bin/dstack:992-1003`; confirmed: `claude/bin/dstack:1008-1017`, `claude/bin/dstack:1065-1087`, `task.md:87-108`, `task.md:186-196`
Evidence: The child launches before the trap; a failed PID write exits via `die`, while `rm-run` treats a missing child record as safe once the supervisor dies. The trap is also cleared before `exit` is published.
Verification: Injecting TERM into the implemented launch→record→trap ordering reproduced a dead supervisor with its child still alive; the recorded test signals only after the child PID becomes observable and therefore misses this interval.
Suggested direction: Establish cleanup ownership before launch and retain it through terminal publication; until an explicit prelaunch state exists, treat a missing child record as unknown rather than safe to delete.

Omitted-detail: 0 low

GPT verdict: reject — A catchable supervisor signal can still recreate the invisible orphan and capture-loss failure that the change claims to eliminate.

Consensus: disagreed
