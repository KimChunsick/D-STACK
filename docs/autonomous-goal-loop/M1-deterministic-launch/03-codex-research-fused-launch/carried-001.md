## Carried decisions — Round 001
- Teardown guarantees are stated only for CATCHABLE termination, in this file as in `codex-review`.
  The two skills state the same residual because they wrap the same launcher, and stating it once
  wider than it holds is the defect being fixed, not a wording preference.
- A "verified" claim names WHICH invocation was verified and WHEN. The Goal's P3 research round
  predates `dstack run`, so it verifies `codex exec` with stdin and `-o` and nothing about the
  wrapper. The fused block is now verified by its own run, recorded in `task.md`.
- `-s read-only` blocks MODEL-initiated mutation. `-o` is a CLI-managed write and is the one
  deliberate repository write the invocation makes; "never mutate the tree" was false as written.
- Every recipe that allocates `mktemp -d` removes it on EXIT. The detached launcher used to clean
  its own scratch dir; that cleanup vanished with the launcher and had to be re-stated in the recipe.
- A task document must not tell the reviewer what is in or out of scope. State what the change
  touched as filing information; scope comes from the review prompt alone.

Consensus: disagreed
