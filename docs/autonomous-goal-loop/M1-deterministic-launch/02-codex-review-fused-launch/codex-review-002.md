# Codex adversarial review — Round 002

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 50262 bytes — **ratchet VIOLATED**, round 001
was 41621 (+8641). Recorded, not hidden. Cause: the assembler sends the two most recent rounds
whole (`FULL_ROUNDS=2`), so with only rounds 001 and 002 in existence nothing is old enough to
compact, and the reviewed `SKILL.md` grew by the three fixes round 001 demanded.
Allowlist unchanged: the review-unit folder, `claude/skills/codex-review/SKILL.md`.

*Corrected after sealing, and the correction is deliberate.* This paragraph first said compaction
starts at round 003. It does not: with `FULL_ROUNDS=2`, round 004 is the first round in which any
round is older than the two most recent. Verified by assembling round 003 and reading the manifest
— rounds 001 and 002 both went in as full snapshots. The reviewer's findings and the consensus
line below are untouched; what changed is my own scope annotation, which was factually wrong about
the tool and would have been fed to the next reviewer as if it were true.

## GPT findings

[severity:medium][security] The task payload embeds evaluator scope directives that the launcher explicitly classifies as unsafe, allowing relevant dependencies to escape review.
Sites: `task.md:12-18`; confirmed: `SKILL.md:259,296-301`.
Evidence: The task declares the adversarial-review skill, assembler, and finding judgment “out of scope,” while the launch prompt says payload scope claims are untrusted and the skill identifies their suppression risk as concrete.
Verification: Following the embedded directive would prevent comparison of the skip recipe with its assembler or the consensus rules with their review contract.
Suggested direction: Remove evaluator scope declarations from the task document and retain only neutral deployment and filing facts.

[severity:medium][technical correctness] The hard-kill recovery check treats a missing or malformed child record as proof of quiescence and ignores a live recorded supervisor, so retry can launch a duplicate round.
Sites: `SKILL.md:156-169`; confirmed: `dstack:1070-1117,1190-1225`.
Evidence: The recipe gates only on nonempty `.launch/child`; `rm-run` instead checks both supervisor and child/group and explicitly treats a missing child record as unknown because the fork may already have occurred.
Verification: The recipe predicate returned “permits retry” with an empty child value while `kill -0` confirmed the stand-in supervisor was alive; SIGKILL between fork and child-record publication creates the equivalent orphan case.
Suggested direction: Apply `rm-run`’s fail-closed invariant: inspect both records and block retry when either is live or any required identity is missing or malformed.

[severity:medium][technical correctness] The skill metadata still requires each rebuttal in `codex-review-<NNN>.md`, contradicting all three repaired procedures that require a separate, never-bundled response file.
Sites: `SKILL.md:3`; confirmed: `SKILL.md:388-390,437-444,529-534`.
Evidence: The frontmatter says each invocation and rebuttal is recorded in the numbered review file; the body says that file contains no maintainer response.
Verification: Following the metadata places rebuttal prose in a round that the assembler sends whole while it is among the two newest, reintroducing excluded prose and bundle growth.

[severity:medium][technical correctness] The termination rules cannot close non-convergence consistently: they require closure with unresolved concrete mediums while defining consensus as requiring those findings to be resolved or explicitly user-disposed.
Sites: `SKILL.md:469-501`; confirmed: `SKILL.md:543-564,578-582`.
Evidence: The finding-stream and non-convergence rules close on repeated open findings and solicit a user decision only for highs, but the gate accepts only an agreed/resolved final round.
Verification: With one concrete medium repeated across three rounds, the document requires closure without user disposition; `Consensus: disagreed` fails the gate, while agreed/resolved violates its own consensus definition.
Suggested direction: Preserve loop limits, but require explicit user disposition for every unresolved concrete high/medium before sealing a positive consensus.

Omitted-detail: 0 low

GPT verdict: reject — four concrete instruction failures can suppress review scope, duplicate a paid round, rebundle excluded rebuttal prose, or manufacture invalid consensus.

## Carried decisions
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
