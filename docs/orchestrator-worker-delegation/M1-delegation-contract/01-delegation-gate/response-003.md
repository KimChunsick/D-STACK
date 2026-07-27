# Maintainer response — Round 003

Outside the reviewed corpus. Five findings, all accepted.

**[high] "The change defines no runnable worker mechanism."** The sharpest and most useful finding
of this unit's loop, because it did not just reject the position — it named the mechanism I had
failed to find. Round 002 left the contract saying binding was UNCONFIRMED and serial was the
answer, and the finding's real complaint is that this is a STATIC instruction no end-to-end run
could ever lift. An unfalsifiable "not yet" is worse than a wrong procedure.

The mechanism is the `WorktreeCreate` hook, and it holds up. Verified in the installed client
2.1.220 rather than taken on the finding's word: the hook emits the worktree path and the platform
adopts it for the subagent, logging `Created hook-based worktree at:`, and it screens that path for
dot segments and for symlinks below the checkout root before using it. `WorktreeRemove` owns
teardown, which is exactly what retention-until-review-closure needs. Both are configured in
`settings.json`. The contract now names this as the mechanism, keeps the worker's identity report
as a tripwire rather than a binding, and replaces the blanket prohibition with an evidence
statement: not yet run, first real fan-out confirms it and records the result, a failure falls back
to serial FOR THAT TASK rather than making serial permanent.

**[high] The `.worktreeinclude` rule still let one entry resolve to several files.** Correct, and
my Round-001 repair was aimed at the wrong thing. I required "no pathspec metacharacter", which
assumes I know the syntax — and I do not: the client is documented as gitignore syntax, where a
bare basename matches at every depth, while the installed binary passes entries to
`git ls-files --others --ignored --exclude-standard --directory`. Under either reading, a
metacharacter-free `config.json` can also select `cache/config.json` holding a live token, whose
NAME appears on no deny list. The rule now checks the RESOLVED set against the single anchored path
the entry was meant to name, which does not depend on knowing which reading is right. And the
`WorktreeCreate` mechanism above sidesteps the manifest entirely, since a hook-created worktree
does not use it — the hook copies a named list of generated or attested secret-free fixtures.

**[high] Merge-resolution paths might not be covered.** Accurate about `git diff-tree`'s default
and not applicable to what T04 shipped, which passes `-m`. Verified rather than asserted, on a
branch whose merge resolution adds a file present in neither parent: the default enumeration is
empty, `-m` reports `evil-from-merge.txt`, and the checker returns
`VIOLATION: evil-from-merge.txt is not in T01 declaration`. The evidence is recorded in THIS unit's
`task.md`, not T04's — T04's review is sealed, and editing a file inside a sealed bundle would
reopen it.

Two of this round's three highs were stale in the same way Round 002's was: the fixed file is
T04's and the bundle carries only this unit's. That is the ratchet's cost showing up as review
noise, and it is a real trade rather than a bug — a bundle that grew to carry every adjacent file
is how the previous Goal reached round ten.

**[low] `keep-in-the-orchestrator` ended on the fragment "no worker may".** Mine, a truncated
sentence from the Round-002 edit. This is the second dangling fragment a round has caught, which
says something about editing dense YAML by string replacement rather than about the design.

**[low] `POSITIVE ISOLATION BENEFIT` had no observable threshold.** Fair — "predictably verbose or
materially multi-step" is operator inference in a document with no runtime, and the finding's own
example (a three-file mechanical rename) can be argued either way. Replaced with something readable
off the declaration and the task doc: more than one declared file, OR a verification run of its own
whose output the orchestrator would otherwise carry. A single declared file with no verification
run is a quick targeted change. Ties go to the orchestrator, like every other doubt in this gate.
