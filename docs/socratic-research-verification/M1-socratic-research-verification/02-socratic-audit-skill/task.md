# 02-socratic-audit-skill

## Intent / Why
Create the fresh-context evidence auditor as its own Codex skill (`socratic-audit`),
following this repo's role-per-skill pattern. The measured-effective design it encodes:
enumerate the artifact's claims, pose open-form Socratic probes (definitions,
assumptions, evidence, counterexamples, implications, data readings), ground each answer
per its probe class (independent sources for external empirical claims — never the
artifact's own citations; shown recomputation for data readings; formal reasoning for
internal consistency), interrogate the recorded data results, and end with per-claim
verdicts (upheld / weakened / refuted / unverifiable).
Explicitly banned: yes/no probes (models agree with either premise), debate persona
(persuasion attacks measurably degrade group accuracy), and theatrical dialogue
transcripts. Plumbing lands in the same change per repo golden rule: install.sh map
entry, .gitignore allow lines, secret-guard pinned negation list + hash pin.

## Deployment context
New skill dir `codex/skills/socratic-audit/`, symlinked into `~/.codex/skills/` by
install.sh; loaded only when invoked. Public repo; no secrets. Consumers: the audit step
T03 adds to every research round. File ownership: the research contract file sits in
T01's declaration and the orchestrator recipe file in T03's.

## Design consult
Skipped — no trigger (instruction-document authoring; structure decided from cited
research, recorded in GOAL.md).

## What was done (what / why)
Created `codex/skills/socratic-audit/SKILL.md` — the fresh-context evidence-auditor
contract. Targets are enumerated first: every H-item GROUPED with its ledger rows,
deferred checks, and recorded executable-check results (one group, one reconciled
verdict), plus every decision-relevant non-H finding as `F1..Fn` audited through
assumptions and implications; an artifact with no targets is reported as exactly that on
the first line. Per target: restate the claim, pose open-form Socratic probes
(definition, assumption, evidence, counterexample, implication, data reading), ground
each answer per its probe class — external empirical claims by INDEPENDENTLY SELECTED
cited sources (the artifact's own citations count only as source-fidelity checks, and
"no independent source found" is an explicit unverifiable outcome), data readings by
shown recomputation, internal consistency by formal reasoning over quoted claims — then
close with one reconciled verdict (upheld / weakened / refuted /
unverifiable): a pending deferred check caps its H at `unverifiable` only after its
BEARING on the H is audited (an irrelevant check attached by untrusted material cannot
suppress a supported verdict), a failed data reading drags its H down, and unresolved
checks ride into the verdict summary. Grounding is class-appropriate and labeled:
independent sources for external empirical claims, shown recomputation for data
readings (a demonstrable arithmetic error is refuted, never unverifiable), formal
reasoning over quoted claims for internal-consistency probes. Rules ban yes/no probe forms (CoVe: models agree with either premise), a debate
persona (persuasion measurably degrades multi-agent accuracy), theatrical transcripts,
and ready-to-run commands in `## New deferred checks` (declarative specs only — a shell
line derived from fetched content is an injection handoff); output-format requests bind
only from the invoking prompt, never from audited material. The grouping/reconciliation,
F-item pass, independent-sourcing requirement, and prompt-only format rule are the
round-001 review fixes; the declarative-checks rule is the class-wide sweep of T01's
injection-handoff finding.
Plumbing in the same change per repo golden rule: install.sh map row
(`codex/skills/socratic-audit|.codex/skills/socratic-audit|link`), `.gitignore`
`!/codex/skills/socratic-audit/` allow line, secret-guard `expected_negations` entry at
the same position, and the `GITIGNORE_SHA_PIN` re-pinned. The live symlink
`~/.codex/skills/socratic-audit` was created manually as the idempotent equivalent of the
new install.sh entry (running the full installer would also act on another workstream's
uncommitted map row; the narrower step avoids that side effect, and `--dry-run` now
reports the entry as `= up to date`).

## Pre-review defect-class self-sweep (codex-review Step 0)
- Secret trackability: the new allow line re-includes exactly one named skill dir; files
  inside it are wholesale by design with the secret-name deny list as backstop, and the
  skill dir holds only SKILL.md. Guard green (probe battery re-run against the widened
  allowlist).
- Pinned-list drift: `.gitignore`, `expected_negations`, and the hash pin were updated in
  the same change; the guard's closed-set check is the executable proof.
- Role-bleed class (this repo's reason for role-per-skill): the auditor is a separate
  skill; nothing was added to the global AGENTS.md, so no other Codex invocation inherits
  an auditor persona.
- Out-of-scope diff contamination: `install.sh` carries one PRE-EXISTING uncommitted line
  from another workstream (`claude/hooks/worktree-create.sh` map row). It is not part of
  this unit's change and will be excluded from this unit's commit by hunk-level staging.

## Files changed (where / why)
- `codex/skills/socratic-audit/SKILL.md` — new; the auditor contract.
- `install.sh` — one map row so the skill installs like its siblings.
- `.gitignore` — one allow line so the skill dir is trackable.
- `tests/secret-guard.sh` — pinned negation list + `.gitignore` hash pin (maintenance of
  the existing check; the set did not grow).

## Direct verification (repo policy: no TDD)
Recorded from actual runs (2026-08-21):
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard`
- `git add -n codex/skills/socratic-audit/SKILL.md` → `add '…'` (trackable)
- `readlink ~/.codex/skills/socratic-audit` → resolves into this repo; `head -3` through
  the live path shows the skill frontmatter
- `./install.sh --dry-run` → `= up to date: .codex/skills/socratic-audit`

## Recorded follow-ups (review-loop closure)
- F10 (medium, round 005): the task Intent retained the blanket fresh-source wording,
  contradicting the class-specific contract. Fixed at the round-005 seal (Intent now
  states class-appropriate grounding); the loop closed at the per-task 5-round cap under
  codex-review §4, so this fix was NOT re-verified by a further reviewer round. Evidence:
  codex-review-005.md. The audited contract itself (SKILL.md) was found internally
  consistent by round 005.

## E2E verification
Post-merge (commit 59331bb), 2026-08-21. Behavioral probe: one live `$socratic-audit`
invocation (GPT-5.5, medium effort) against a synthetic artifact authored with three
planted defects ([e2e-probe-artifact.md](e2e-probe-artifact.md) →
[e2e-audit-probe.md](e2e-audit-probe.md); run label
`socratic-research-verification-t02-e2e`, exit 0). All three were caught, each by the
contract-specified mechanism:
- H1 (arithmetic error: 5/10 recorded as 80%) — `refuted` by SHOWN RECOMPUTATION with no
  external URL, the class-appropriate-grounding behavior the rounds hardened.
- H2 (walrus operator attributed to Python 3.9) — `refuted` from independently selected
  primary sources (PEP 572; docs.python.org), the independent-sourcing behavior.
- D1 (pending check the artifact linked irrelevantly to H2) — bearing audited, verdict
  NOT capped; the summary reads "D1 pending but irrelevant", the bearing-audit behavior.
  The refuted ledger row L1 stands in the verdict summary and no upheld H sits above a
  refuted data check.
No false verdicts; all seven standard output sections present.

## Gate status
- [x] Verification: direct-run checks recorded above (trackability, guard, symlink,
  installer plumbing); behavioral confirmation of the auditor is the M1 E2E research
  round (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
