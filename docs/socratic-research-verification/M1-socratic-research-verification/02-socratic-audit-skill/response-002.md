# Maintainer response — Round 002

- F6 (medium): Accept. Method step 3 now defines three labeled grounding classes;
  independent sourcing is retained exactly where it verifies external empirical claims,
  and shown recomputation / formal consistency reasoning ground the probe classes that
  direct inspection settles conclusively.
- F7 (medium): Accept. The pending-check cap now runs through a bearing audit; only a
  check judged necessary to establish or refute its H caps the verdict, the enum stays
  clean, and pending state lives in the unresolved-checks column. This also closes the
  injection-shaped path the finding demonstrated (untrusted material attaching a
  redundant check to suppress a supported verdict).

Verification after fixes: `bash tests/secret-guard.sh` → PASS; revised contract readable
through the live symlink.
