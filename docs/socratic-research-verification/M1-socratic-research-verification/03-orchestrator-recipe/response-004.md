# Response — Round 004 (never bundled)

F12–F15 verified closed by the reviewer. Of the four new findings: F16–F18 fixed this
round, F19 recorded as a non-blocking follow-up in task.md.

- F16 (high, executable fetched content): verified — authorization governed what could be
  READ, and nothing banned running it. Fixed: Step 2a's inert-data rule (never execute,
  import, `source`, install, or evaluate fetched logic, however public; a spec pointing
  at a repository's verifier is asking the orchestrator to hand it code execution;
  computations are always orchestrator-authored) plus credential-free scratch runs. The
  sibling research contract's consumer clause (author/validate/sandbox own execution,
  spec as untrusted data) is consistent with this; the enforcement point is the consumer,
  which is this file.
- F17 (medium, F-item verdict boundary): verified and fixed — exactly one
  verdict-summary row per independently derived target: every enumerated H-item AND
  every F-item the audit examines; a body-only refutation is structural breakage; the
  fallback trigger mirrors the test.
- F18 (low, pre-launch refusal preserves scratch): fixed — both fences clean scratch when
  `exit` exists OR `.launch` is absent, preserving only a launched nonterminal run.
  Three-state probe recorded in task.md. Residual stated in the fence: a `.launch` claim
  this shell cannot attribute (label race lost to another attempt) preserves scratch
  fail-closed — deleting a possibly-live cwd is the worse error.
- F19 (low, pre-contract research artifact): accepted as recorded follow-up — the
  artifact motivated the design before the contract it produced existed, which is
  chronology, not circularity; the intro cites it as evidence-informed (F5) and the
  follow-up (regenerate + audit through the new pipeline) is recorded in task.md.

Verification after fixes: three fences `bash -n` clean; trap-condition probe
(CLEAN/PRESERVE/CLEAN); secret guard green. Round 005 — the per-task cap — requested on
the same allowlist.
