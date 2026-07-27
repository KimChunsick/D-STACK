# Maintainer response — Round 002

Deliberately OUTSIDE the reviewed corpus: prose about what was fixed is not evidence,
the diff is, and re-bundling this text every round is what made the review eat its own
output (see codex-review SKILL.md, 'The bundle ratchets DOWN').

Four mediums and two lows. Five accepted and fixed; one low is a correction TO me that I have
verified and adopted, and it changed what the document says rather than merely how.

**[medium] Assembly was not a checked precondition.** Confirmed and fixed. Without `set -e` the
shell walks past a failed assembler straight into a review of an empty file, and `codex exec`
exits 0 on it — so the status check the document showed proves nothing. The recipe now guards
`run-dir` and the assembler with `|| exit 1` and, before launching, counts the bundle's `--- `
entries against an expected minimum. This is not theoretical: it is the guard that stopped the
Round-2 relaunch of this very unit from shipping an empty bundle.

**[medium] Backgrounding loses `RD`/`OUT`, and a failed label is unretryable.** Confirmed and
fixed, and I hit both. A shell variable does not survive between tool calls, so the triage step
referencing `"$OUT"` was unusable from a later turn; the document now gives the durable path,
`.dstack/runs/$CLAUDE_CODE_SESSION_ID/<label>/out.txt`, and says explicitly never to call
`run-dir` again to recover it. On the retry point: labels are per-*attempt*, not per-round —
`…-r2`, then `…-r2a`. The allocator refusing a used label is the correct behaviour (it is what
now prevents two attempts writing into one directory), so the fix is to name attempts, not to
soften the allocator.

**[medium] Bare `dstack` in the pause and handoff paths.** Confirmed. My Round-1 fix was
instance-wise: I corrected the P6 code block and stopped, leaving `waits.user-input`, the
concurrent-stream guidance, the milestone handoff, and the cutover line all prescribing a
command that resolves to "command not found" in the setup the same document describes. Swept
class-wide this time — a grep for every backticked `dstack <verb>` across both skills found five
sites, all corrected, and the re-sweep is clean.

**[medium] "Refuse if the path exists" is check-then-write.** Confirmed and fixed, and the
finding is sharper than it looks: this is the same defect Round 1 caught in `dstack reg`, where
a `rename()` publish let two sessions each believe they owned one document, and I did not carry
the lesson across to the migration-naming advice I wrote in the same session. The guidance now
requires atomic exclusive creation (`set -o noclobber`, `O_CREAT|O_EXCL`, `ln`) with `EEXIST`
handled by regenerating, and says out loud that testing-then-writing loses the race.

**[low] The file-list warning reversed Bash behaviour.** Correct, and I checked rather than
conceded. In bash an unquoted `$FILES` does split into three arguments; what I observed was one
argument, because **this harness runs commands under zsh**, where unquoted parameter expansion
does not word-split. Verified directly: `set -- $F` yields 1 argument under zsh and 3 under
`bash -c`. The document now states that, so the advice (pass literal arguments; count the bundle
entries) is justified by the real mechanism instead of a wrong claim about bash.

**[low] "No tests" versus editing `skill-schema.test.sh`.** Confirmed as a genuine ambiguity for
an instruction-following reader. The policy bans Red-Green-Refactor cycles and adding new test
files; it explicitly requires running the two checks that already exist. Updating an assertion
that pinned a mechanism this milestone removed is maintenance of an existing check, not
authorship of a new one — the set did not grow, two assertions were replaced. Stated plainly in
the milestone document. Sharpening the same distinction in `AGENTS.md` itself is recorded as a
follow-up rather than done now, because that file sits inside M1's currently open review bundle
and editing it would void that round.

**Regression caught by my own fixes, and fixed in turn.** Making `run-dir` allocate a unique
directory (a Round-1 low in M1) broke the pattern of resolving one label twice to get one path:
the second call silently returned `<label>.2`, and the relaunched rounds read a bundle that was
not there. No false review was recorded — the runs produced no output at all — but the failure
was silent, which is the part that mattered. `run-dir` now refuses a used label loudly instead
of allocating a variant.
