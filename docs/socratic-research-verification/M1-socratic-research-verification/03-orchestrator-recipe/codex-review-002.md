# Codex adversarial review — Round 002

## Review scope
Re-review | serial | bundle 44768 bytes (round 001: 33796) | label socratic-research-verification-t03-r002a (first attempt r002 died signalled, exit 143, discarded per contract)

## GPT findings
[severity:high][security] The leaf-path repair still permits file clobbering: `-L` accepts hard-linked output paths, while Step 1 and Step 2a perform writes before any terminal-path guard.
Sites: Primary: `SKILL.md:119-120`; confirmed: `SKILL.md:49,175,289-291,336-341,359,373-378,434-449`.
Evidence: The `-o` guards reject only symbolic links; the recipe itself states that Codex writes through the destination, and the orchestrator-written artifacts lack equivalent pre-write protection.
Verification: A hard link is another directory entry for the same file, so an output hard-linked to `AGENTS.md` passes `-L` and a truncating open modifies the shared inode. (POSIX `link()`, `open()`.)
Suggested direction: Stage every generated artifact in scratch and atomically replace its destination after immediate ancestor and leaf validation, including Step 1, Step 2a/2c, and both fallbacks.

[severity:medium][technical correctness] Audit-input assembly remains fail-open: the brace group's status is ignored and its source guards do not require readability, allowing a partial bundle to reach the auditor.
Sites: Primary: `SKILL.md:336-347`; confirmed: `SKILL.md:352`.
Evidence: The fence uses `set -u`, not `set -e`; if the first `cat` fails and the second succeeds, the group returns success, while any nonzero group status is otherwise ignored before subsequent traps and `dstack`.
Verification: A regular, nonempty but unreadable research file passes `-f`/`-s`; its `cat` fails, the data-check `cat` succeeds, and the labeled bundle contains no research artifact.
Suggested direction: Require readable inputs and make every `printf`, `cat`, and bundle write failure abort before invoking `dstack`.

[severity:medium][technical correctness] Structural acceptance still trusts producer-declared H coverage and does not validate required F-items or data-check rows, so heading-complete artifacts can pass without any claim-level audit.
Sites: Primary: `SKILL.md:368-371`; confirmed: `SKILL.md:400-444`, `adversarial-research/SKILL.md:24-49`, `socratic-audit/SKILL.md:22-27,100-107`.
Evidence: Research acceptance checks headings and source count; audit acceptance checks headings plus rows only for H-items the research artifact chose to enumerate, despite both contracts requiring broader target coverage.
Verification: Put a measurable claim under `Needed info`, declare `Hypotheses`, ledger, and deferred checks as `none`, then return seven audit headings with empty F/data sections and summary; every stated trigger passes.
Suggested direction: Independently compare research claims against H/ledger/deferred coverage, then compare every H-item, F-item, and data-check identifier against the audit body and summary.

[severity:medium][technical correctness] The F4 repair lets the orchestrator assign final noncritical verdicts from post-audit evidence, so those results never receive the audit contract's required data-reading probes.
Sites: Primary: `SKILL.md:373-381`; confirmed: `SKILL.md:393-396`, `socratic-audit/SKILL.md:49-51,60-75`.
Evidence: A delta audit runs only for decision-critical changes; otherwise a `superseded:` line is sufficient and Step 3 reports that orchestrator-selected verdict.
Verification: A new check using the wrong denominator can appear to refute an H-item; the orchestrator records `refuted` and P5 consumes it without any fresh auditor checking the dataset, unit, denominator, or transformation.
Suggested direction: Re-audit every completed check that changes a verdict; use decision-criticality only to decide whether Phase 4 is required.

Omitted-detail: 0 low

GPT verdict: reject — The leaf guards still allow destructive output aliasing, while shallow acceptance and post-audit reconciliation can silently produce unaudited final verdicts.

## Carried decisions
- F1–F5 (round 001): fixes verified by this round except where sharpened below; F1's
  repair is extended by F6, F3's by F8, F4's is replaced by F9's rule.
- F6 (high, hard links pass `-L`; orchestrator writes unguarded): ACCEPTED — guards
  extended to refuse any aliased leaf (regular file, not a symlink, link count 1, via
  POSIX `find -prune -links 1`) for inputs and output targets in BOTH fences; a prose
  rule now covers every orchestrator-written artifact (Step 1 brief, Step 2a record,
  fallback artifacts): destination must be absent or a plain unaliased file before
  writing. Consistent with the recipe's stated mistake-tripwire model (`cp -al` clones
  make hard-linked working trees a real mistake, not only an adversary's act).
- F7 (medium, fail-open assembly): ACCEPTED — inputs must also be readable (`-r`), the
  concatenation is an `&&` chain, and a nonzero group status refuses the launch.
- F8 (medium, producer-declared coverage): ACCEPTED with a boundary — the structural test
  now also requires every declared ledger row / deferred check to be reconciled somewhere
  in the audit, treats an empty `## Audit of findings` over a claim-bearing artifact as
  broken, and the research-side triggers treat all-`none` blocks over measurable claims
  as missing sections. The boundary: claim-level semantic coverage IS the auditor's
  contract (F-item pass); the orchestrator's checks are a structural backstop read by the
  orchestrating model, not a mechanical re-derivation of the audit.
- F9 (medium, unaudited verdict changes): ACCEPTED — any verdict-CHANGING check outcome
  re-enters the auditor (Step 2b re-run under the next label with the appended results);
  the `superseded:` line records the delta audit's verdict, never the orchestrator's own;
  decision-criticality decides only Phase 4 re-entry. GOAL.md's interview-record
  assumption is amended with this review revision.
- Standing context: no-new-tests repo policy (direct-run evidence in task.md); install.sh
  is untouched by this unit.

Consensus: disagreed
