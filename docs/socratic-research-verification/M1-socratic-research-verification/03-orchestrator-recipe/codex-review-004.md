# Codex adversarial review — Round 004

## Review scope
Re-review | serial | bundle 65132 bytes (round 003: 56391) | label socratic-research-verification-t03-r004

## GPT findings
Re-review verification: F12's unconditional auditor return, F13's attempt-specific non-overwriting output, and F14's complete brief guard are present. F15 closes the recorded assembly-failure path. All three Bash fences pass `bash -n`, and the recorded ATTEMPT results match the current guard.

[severity:high][security] F10 remains open: authorizing a public input does not make active content safe, and the recipe neither forbids executing fetched code nor confines it from user files and network egress.
Sites: Primary: `claude/skills/codex-research/SKILL.md:296-308`; confirmed: `codex/skills/adversarial-research/SKILL.md:51-60`.
Evidence: The rules permit public sources and orchestrator-authored computations but impose only non-mutation and a scratch working directory; neither contract prohibits executing, importing, sourcing, or installing content obtained from an authorized source.
Verification: A deferred specification can identify a public repository's verifier as its input and request its computation; running that verifier can read credentials and transmit them without modifying the filesystem, while satisfying every written authorization and recording rule.
Suggested direction: Treat fetched material as inert data, forbid execution/import/source/install/macros, and require a scratch-only, credential-free, egress-restricted sandbox for any explicitly approved exception.

[severity:medium][technical correctness] F11 remains open at the verdict boundary: acceptance requires summary rows for H-items only, so substantively audited F-items may disappear before Phase 4 and P5.
Sites: Primary: `claude/skills/codex-research/SKILL.md:410-426`; confirmed: `claude/skills/codex-research/SKILL.md:450-460`, `codex/skills/socratic-audit/SKILL.md:20-27,98-107`.
Evidence: F coverage is checked only for substantive audit-body content, while the explicit verdict-summary predicate covers H-items; Step 3 then consumes summary counts and named weakened/refuted claims.
Verification: An audit can refute F1 substantively in `Audit of findings` but omit F1 from `Verdict summary`; all stated acceptance predicates pass, and the decision-critical refutation can be omitted from GOAL.md.
Suggested direction: Require exactly one verdict-summary row, including unresolved checks, for every independently derived H-item and F-item.

[severity:low][technical correctness] F15 closes assembly failures, but scratch still leaks when `dstack` refuses before establishing or publishing a run.
Sites: Primary: `claude/skills/codex-research/SKILL.md:392-396`; confirmed: `claude/bin/dstack:1021-1068`.
Evidence: After assembly, cleanup occurs only when `$RUNDIR/exit` exists; `dstack` has several refusal paths before its launch claim and terminal record.
Verification: If another invocation claims the label between the recipe's precheck and `dstack`'s atomic `mkdir`, `dstack` exits without `exit`, and the outer trap leaves its unused scratch directory behind.
Suggested direction: Remove scratch when no launch claim exists or when a terminal record exists, preserving it only for a genuinely launched nonterminal run.

[severity:low][the real Why] The research artifact used to motivate the design remains unaudited pre-contract evidence because it lacks all three new semantic blocks.
Sites: Primary: `docs/socratic-research-verification/research/socratic-and-data-verification.md:1-44`; confirmed: `claude/skills/codex-research/SKILL.md:8-14`.
Evidence: The artifact contains measurable numerical claims but has no `Hypotheses`, `Data-check ledger`, or `Deferred executable checks` sections.
Verification: The recipe's current research acceptance rule would reject this artifact as missing required sections, although the introduction cites it as the design foundation.
Suggested direction: Record regeneration and audit of this artifact as a non-blocking follow-up before citing it more strongly than the current "evidence-informed" qualification.

Omitted-detail: 0 low

GPT verdict: reject — Public-source deferred checks can still execute attacker-controlled code with host access, and decision-relevant F-item verdicts can still vanish from the accepted summary.

## Carried decisions
- F12, F13, F14, F15: verified CLOSED by this round (fences `bash -n` clean, ATTEMPT
  guard matches recorded probes).
- F16 (high, F10 sharpened — fetched content executable): ACCEPTED — Step 2a now
  declares fetched material INERT DATA: never execute, import, `source`, install, or
  evaluate logic obtained from any source however public; authorization makes an input
  readable, not runnable; computations are always orchestrator-authored; scratch runs
  carry no credentials in the environment.
- F17 (medium, F-items vanish at the verdict boundary): ACCEPTED — the structural test
  now requires exactly one verdict-summary row (verdict, grounds, unresolved checks) for
  every H-item the artifact enumerates AND every F-item the audit examines; mirrored in
  the fallback trigger.
- F18 (low, scratch preserved on pre-launch dstack refusal): ACCEPTED — both fences'
  launch-time trap removes scratch when no launch claim exists OR a terminal record
  exists, preserving it only for a launched, nonterminal run. Residual stated: a claim
  another attempt owns cannot be attributed from this shell, so that rare race preserves
  scratch fail-closed rather than deleting a possibly-live cwd.
- F19 (low, pre-contract research artifact): ACCEPTED AS RECORDED FOLLOW-UP —
  regenerating and auditing this Goal's own research artifact through the new pipeline
  is recorded in task.md as non-blocking; the intro already carries the
  evidence-informed qualification from F5.
- Standing context: no-new-tests repo policy; install.sh untouched by this unit.

Consensus: disagreed
