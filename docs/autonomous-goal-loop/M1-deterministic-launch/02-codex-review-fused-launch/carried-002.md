## Carried decisions — Round 002
- The retry fence is FAIL-CLOSED on BOTH launch records and mirrors `rm-run`'s invariant exactly.
  A missing or malformed pid is *live*, not quiescent, because `run` releases its claim on every
  pre-fork failure — so a claim with no child record means the fork may have happened while that
  pid was being written. The recipe and the deletion guard must never disagree about when a
  capture is finished.
- Frontmatter is part of the document. Three body sites agreed about `response-<NNN>.md` while the
  `description:` line still said the rebuttal goes in the numbered round file. A rule that lives in
  four places is wrong in whichever one was not edited.
- Consensus has FOUR dispositions, and the fourth is "accepted residual under a §4 closure".
  Without it the non-convergence rule was unsatisfiable: it *demands* closure with a concrete
  medium still open, while the consensus definition made every sealable value a lie. `resolved`
  means the loop resolved by measurement, and it is honest only because the defect reaches the
  user in the final report. A concrete HIGH still escalates before closing.
- Scope directives inside a task document are a defect wherever they appear, not only in the file
  that was reviewed for it first. Round 001 fixed this class in `03`'s task doc; the same sentence
  was still sitting in `02`'s.
- The bundle ratchet cannot hold at rounds 002 or 003 by construction. `assemble-review.sh` sends
  the two most recent rounds whole, so the first round in which anything compacts is **004**.
  Until then every round carries its whole predecessor plus a file that grew by that predecessor's
  fixes. Record the size and the violation with numbers; do not delete evidence to make the number
  fall, and do not read the early violations as a process failure.

Consensus: disagreed
