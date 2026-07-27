## Carried decisions — Round 003
Rounds 1-2 decisions stand. Added in Round 3:

- **A recipe in a skill must be runnable as written.** Define every variable it uses, in the
  block that uses it, and never reference one across a tool-call boundary.
- **Never validate a bundle by counting headers.** Skipped files emit headers too; grep for
  `(SKIPPED` and treat any skip as disqualifying.
- **Always-loaded instruction files are part of the change surface.** `claude/CLAUDE.md` drifted
  because it was in no `files` declaration. When behaviour changes, declare the documents that
  describe it, not just the code.
- **Propagate an accepted rule to every durable record that restates it**, or the docs and the
  skill disagree about what blocks.
- Age-based pruning never covers the run that just closed — delete it explicitly.
- Subagents inherit session reasoning effort by default; ultracode is a session mode, which is a
  different claim.

Consensus: disagreed
