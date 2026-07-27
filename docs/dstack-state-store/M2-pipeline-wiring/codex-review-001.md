# Codex adversarial review — Round 001

## Review scope
Adversarial review

## GPT findings
[severity:medium][technical correctness] The no-`cat` triage command cannot discover contract-compliant findings: it searches for headings, bold text, `Severity:`, or `Sites:`, while required findings begin with `[severity:…][axis]`; single-site findings disappear entirely.
Sites: primary: `claude/skills/codex-review/SKILL.md` Step 2; confirmed: `codex/skills/adversarial-review/SKILL.md` output contract.
Evidence: The prescribed regex is `^#{1,4} |^\*\*|Severity:|^Sites:`, but the contract mandates lowercase `[severity:high|medium|low][axis]`.
Verification: Passing a minimal contract-compliant finding through the exact command returned only its `Sites:` line, not the finding.
Suggested direction: Match `^\[severity:(high|medium|low)\]\[` directly and avoid any fixed cap that can hide later high/medium findings.

[severity:medium][technical correctness] The Round-4 rule converts a newly discovered concrete medium defect into a non-blocking follow-up solely because its code was visible earlier, permitting known blockers to ship.
Sites: primary: `claude/skills/codex-review/SKILL.md` “Unchanged-code rule”; confirmed: `codex/skills/adversarial-review/SKILL.md` re-review and consensus contracts.
Evidence: The new rule excludes every late finding except high severity or newly reachable code, while the reviewer contract requires continued full-scope review and resolution of concrete medium findings.
Verification: A medium path-injection defect first identified in Round 4 on unchanged code must be dismissed under the new rule despite remaining exploitable.
Suggested direction: Use the round budget for escalation or decomposition, but never change a concrete high/medium finding’s blocking status based on discovery time.

[severity:medium][technical correctness] The central background-handoff behavior remains unverified: the workflow now commands the model to end its turn while work is registered, but no evidence shows completion reliably re-invokes it.
Sites: primary: `docs/dstack-state-store/M2-pipeline-wiring/task.md`; confirmed: `claude/skills/full-cycle/SKILL.md`, `docs/dstack-state-store/research/dstack-state-store.md`.
Evidence: Milestone E2E is `pending`, all gate boxes are unchecked, and the research explicitly says the Stop-block/background-notification interaction was not verified.
Verification: The supplied bundle contains neither an E2E capture nor the gate implementation needed to disprove the failure path where the task completes on disk but the workflow never resumes.
Suggested direction: Capture a direct run covering background launch, turn termination, completion re-entry, output processing, and final gate behavior.

[severity:medium][technical correctness] UTC timestamps do not make migration names impossible to duplicate; the instruction leaves precision and collision handling undefined while asserting two streams “cannot” generate the same name.
Sites: primary: `claude/skills/full-cycle/SKILL.md` concurrent-stream guidance; confirmed: `docs/dstack-state-store/research/dstack-state-store.md` “Against the goal.”
Evidence: The supplied research itself states that timestamps only reduce collisions and do not solve migration naming or dependency ordering.
Verification: Two streams creating `20260727080000_add_users` within the same timestamp unit produce the identical path; in one worktree that permits clobbering, and across branches it recreates the merge collision.
Suggested direction: Specify a collision-resistant suffix or allocator, fail if the path already exists, and retain explicit dependency ordering.

[severity:medium][technical correctness] Review-unit registration remains ambiguous for milestone reviews because every executable example hardcodes a per-task directory even though this canonical unit lives at the milestone root.
Sites: primary: `claude/skills/full-cycle/SKILL.md` registry examples and directory schema; confirmed: `docs/dstack-state-store/M2-pipeline-wiring/task.md`, `claude/skills/codex-review/SKILL.md`.
Evidence: The example registers `docs/<goal>/<MN-milestone>/<NN-task>/task.md`, while the supplied canonical review document is `docs/dstack-state-store/M2-pipeline-wiring/task.md`.
Verification: Literal substitution cannot identify the current milestone document and can instead register individual task documents, leaving the milestone gate and review series unenforced.
Suggested direction: Use a single `<review-unit>/task.md` abstraction consistently and separately define whether subordinate task documents are also gated.

[severity:low][security] The documented lifecycle retains full review bundles without ever invoking the pruning command or defining its retention window.
Sites: primary: `claude/skills/codex-review/SKILL.md`; confirmed: `docs/dstack-state-store/M2-pipeline-wiring/task.md`.
Evidence: The workflow creates persistent files and only states that `dstack prune` can remove them; no phase runs it despite the milestone claiming captures are pruned.
Verification: Tracing every shown command creates `bundle.txt`, `out.txt`, and `err.txt` repeatedly with no deletion or pruning action.

[severity:low][technical correctness] Every review invocation leaks its `mktemp -d` scratch directory because removal of the old trap also removed scratch cleanup.
Evidence: Step 2 assigns `SCRATCH="$(mktemp -d)"` and has no subsequent trap or removal.
Verification: On success or handled failure, the shell exits without deleting the created directory.

[severity:low][DX] The ultracode note incorrectly treats `claude -p` itself as bypassing the alias; a command typed in an interactive zsh still receives alias expansion before Claude processes `-p`.
Evidence: `claude -p ...` is listed separately from script and CI launches among paths that allegedly never inherit the alias.
Verification: In an interactive shell where `alias claude='claude --effort ultracode'` is defined, the command token expands regardless of its later arguments.

[severity:low][DX] The legacy-cutover instructions say every mutating command exits 4 while simultaneously instructing the user to run the necessarily mutating `dstack migrate`.
Evidence: Both claims appear in the same “Cutover is fail-loud” paragraph.
Verification: Migration cannot carry legacy records into the new store without mutating state, so the stated universal rule makes its documented recovery command impossible unless an unstated exception exists.

Omitted-detail: 0 low

GPT verdict: reject — The milestone has unresolved medium blockers in review parsing, convergence policy, background handoff verification, migration naming, and milestone registration.

## Carried decisions
All Round-1 findings were accepted and fixed; none were rebutted. Standing decisions:

- **Discovery time never changes a finding's blocking status.** A concrete high or medium blocks
  whichever round surfaces it. Only non-concrete items may be aged out, and only from Round 4.
  The six-round budget escalates to the user; it never downgrades a defect.
- Review triage must match the contract's own `[severity:…][axis]` line format, and the
  high/medium query must never carry a fixed result cap.
- `<review-unit>` is a single abstraction: one folder holds the registered, gated, reviewed
  `task.md` and its review series. Subordinate task documents are records only.
- Migration filenames: timestamps reduce collisions, they do not remove them. Pin precision,
  refuse to overwrite an existing path, and keep ordering as a declared dependency.
- Open and deliberately deferred: the background-handoff E2E belongs to P11 (milestone E2E),
  after review by phase order. It is not evidence this round claims.
- Repo policy: no tests, no Red-Green-Refactor; gates rest on recorded direct-run evidence.

Consensus: disagreed
