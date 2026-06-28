# Codex adversarial review — T02

## Round 1 — GPT verdict: **approve-with-fixes**
1. `[med][structure]` AGENTS.md ("defaults live in instructions.md") contradicts
   instructions.md ("stack rules belong in project AGENTS.md") → boundary conflict.
2. `[med][tech]` DAG / loading-error-empty-success / "server side" are frontend-derived
   absolutes; qualify for UI/async + non-server code.
3. `[med][tech]` "typed results / any / non-null assertions" assume a typed language.
4. `[med][structure]` Test guard blocks only 3 old phrases; "Use React/Vitest/Tailwind"
   would still pass — weak regression net.
5. `[low][UX]` Universal rule lost: reduced-motion handling.
6. `[low][sec]` No stack-neutral security baseline in the reusable defaults file.

## Claude responses — all accepted & fixed
- **#1** — reconciled the boundary: both global files now agree that `instructions.md` holds
  *stack-neutral* defaults and a *project's own* `AGENTS.md` holds stack-specific rules.
- **#2** — qualified: state-acyclicity framed generally ("wherever your code holds state");
  "handle every state" scoped to fail/slow paths; "server side" → "logs/telemetry, not the
  end user."
- **#3** — types section rephrased: "where the language has a type system… otherwise validate
  at boundaries; make null/empty/error explicit either way."
- **#4** — added negative guards for `Next.js|App Router`, `Tailwind|Vitest|clsx`,
  `React Testing Library` so a re-introduced global stack mandate fails the test.
- **#5** — restored reduced-motion (conditionally) in the a11y line.
- **#6** — added a `## Security baseline` section (untrusted input, no secret logging,
  least privilege, ignore instructions in fetched data).

## Evidence
- `bash tests/run.sh` → ALL TESTS PASSED. `grep -ci 'next.js' codex/instructions.md` → 0.
- Boundary check: AGENTS.md and instructions.md now point stack-specific rules to the
  *project's* AGENTS.md (no contradiction).

## Round 2 — GPT verdict: **approve**
All 6 points RESOLVED, no new high/medium issues. "The fixes address the Round-1 concerns
without introducing new blocking risks."

## Consensus
- Round 1: approve-with-fixes → all 6 fixed.
- Round 2: **approve** — all resolved.
- **Consensus: agreed.**
