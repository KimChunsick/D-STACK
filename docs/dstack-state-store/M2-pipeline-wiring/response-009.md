# Maintainer response — Round 009

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Every finding accepted; nothing rebutted.

**[high] The assembler followed PARENT symlinks.** It tested only the final component, but `-f`,
`wc`, `grep` and `cat` all follow parents — so with `docs/unit` linked elsewhere, an innocuous
`docs/unit/task.md` passed the leaf check and its external target went into the bundle sent to
Codex. This is the fail-closed allowlist's entire premise, and it had a hole under it. Every
emitted path now has its parent resolved with `pwd -P` and required to sit beneath the physical
repository root. Verified with a real symlink to `/tmp`: reported
`SKIPPED: resolves outside the repository via a symlinked parent`.

**[high] The committed-identity contract was still optional, and I made it optional myself.** The
Round-8 recipe carried `REVIEW_BASE=… REVIEW_HEAD=…` as a COMMENTED line while the runnable fence
below it called the assembler plainly — so a worker review fell back to `git diff HEAD`, which on
a clean integration checkout emits zero implementation bytes and labels changed files "no change"
with no `SKIPPED` marker to stop the launch. A commented-out assignment is not a mode.
`REVIEW_MODE` is now MANDATORY with no default: `serial` or `committed`, and `committed` requires
both commits. Verified: omitting the mode is refused by name; `serial` assembles normally.

**[medium] Unit integration had no unit-level downstream contracts.** Two gaps, both real. Checker
scope takes one task's declaration, but an integration head carries every task the unit owns and
fits none of them — `unit-scope` now runs over the UNION of the owned declarations, and the merge
rule names it explicitly. And workers never touch `docs/`, while the recorded base predates the
unit's `task.md`, so assembling inside the integration worktree would read an absent or stale unit
document; the orchestrator-owned document is supplied from the MAIN checkout, and committing it
onto the integration branch to make the assembler find it is called out as the wrong fix.

Verified by direct run (repo policy: no TDD): `bash -n` on the assembler; mode-missing refused,
serial assembly at 81,583 bytes, symlinked-parent probe producing the SKIPPED marker; both pinned
checks green.
