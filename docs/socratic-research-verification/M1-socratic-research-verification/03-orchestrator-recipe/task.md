# 03-orchestrator-recipe

## Intent / Why
Wire the verification layer into the pipeline the orchestrator actually follows:
`claude/skills/codex-research/SKILL.md`. Pass-1 prompt gains the new required sections
(hypotheses, data-check ledger, deferred executable checks); a new step has the
orchestrator run the feasible deferred checks locally under untrusted-data discipline and
record them; a new step launches the audit invocation in the hardened dstack shape
(label `<goal>-research-audit`, artifact `<topic>.audit.md`); fallback rules mirror the
research fallback (nonzero after one retry, or missing sections → orchestrator performs
the examination itself, recorded); and the GOAL.md summary step now records per-claim
verdicts, with a refuted decision-critical premise routing back to P4.

## Deployment context
Markdown recipe in `claude/skills/codex-research/SKILL.md`, symlinked into
`~/.claude/skills/`, read by the orchestrating model at every P3. Public repo; no
secrets. Depends on T01 (research contract sections) and T02 (audit skill). Out of scope:
changes to full-cycle SKILL.md (P3's phase text already delegates to codex-research).

## Design consult
Skipped — no trigger (instruction-document edit; invocation shape reuses the existing
hardened recipe verbatim).

## What was done (what / why)
Rewired `claude/skills/codex-research/SKILL.md` into the three-pass shape:
- Title/description/intro now state the flow (research → orchestrator-run deferred data
  checks → fresh-context Socratic audit) and why the audit runs in a context that did not
  write the claims (the Goal's cited research).
- Step 2's prompt now instructs the researcher to apply the contract's research-mode
  blocks and pins NINE output sections (the six legacy ones plus `## Hypotheses`,
  `## Data-check ledger`, `## Deferred executable checks`). The `--output-schema` bullet
  warns that a schema must carry the three blocks.
- New Step 2a: the orchestrator reads the deferred-check list as untrusted declarative
  specs, AUTHORS its own non-mutating commands (never pasting artifact text), runs them
  from scratch dirs with no secrets, and records spec/command/output/reading into
  `<topic>.data-checks.md` — written even when the list is `none`, because an absent file
  is indistinguishable from a skipped step.
- New Step 2b: the audit invocation in the hardened dstack shape — label
  `<goal>-research-audit`, same slug/root/symlink/session-id/label guards re-derived in
  the fence (no cross-fence variables), preconditions on both input files, stdin built as
  a labeled concatenation of artifact + data-checks record, `-o <topic>.audit.md`,
  GPT-5.5 xhigh, `$socratic-audit` prompt with untrusted-data framing.
- New Step 2c: exit-file discipline for the audit; new-deferred-checks handling (run,
  append; delta audit only when decision-critical); refuted/weakened decision-critical
  premise routes to Phase 4; verdicts and unresolved checks feed P5 with `unverifiable`
  premises recorded as assumptions.
- Step 3 now writes per-claim verdict counts, every refuted/weakened claim by name, and
  unresolved checks into GOAL.md, linking all three artifacts.
- Fallback extended: audit failure (triggers below) → the orchestrator performs the
  Socratic examination itself, writes the same verdict shape into `<topic>.audit.md`
  marked orchestrator-performed on the first line, and records the degraded mode
  (weakened fresh-context property stated honestly) in GOAL.md.

Round-001 review fixes (all five findings accepted):
- F1 (high): Step 2b gained leaf guards — both inputs must be regular, non-symlink,
  non-empty; the `-o` audit target must not be a symlink; the stdin concatenation moved
  into `$SCRATCH` so the fence never opens a predictable repo path for writing. Class
  sweep: Step 2's fence gained the same leaf guard (brief + `-o` artifact).
- F2 (high): the research fallback now replaces only the researcher — the fallback
  artifact carries the same nine sections and Steps 2a–2c run unchanged on it; Phase 3
  without a data-checks record and an audit verdict summary is explicitly unfinished.
- F3 (medium): audit structural acceptance now requires all seven pinned sections AND a
  verdict-summary row per enumerated H-item, in Step 2c and in the fallback trigger.
- F4 (medium): every completed new-check outcome is reconciled — a contradicting outcome
  gets a `superseded:` line in `<topic>.data-checks.md` (audit artifact untouched);
  decision-criticality gates only the delta-audit escalation; Step 3 reports reconciled
  verdict counts.
- F5 (low): the intro now says evidence-informed and names the research's own Unverified
  residue, assigning mechanism-specific evidence to the Goal's E2E rounds.

Round-002 review fixes (all four findings accepted):
- F6 (high): the leaf guards now refuse ALIASED paths, not only symlinks — an existing
  leaf must be a regular file with link count 1 (POSIX `find -prune -links 1`), applied
  to Step 2b's inputs and `-o` target and Step 2's brief/artifact leaves; a prose rule
  extends the same discipline to every orchestrator-written artifact (Step 1 brief,
  Step 2a record, fallback artifacts).
- F7 (medium): Step 2b's inputs must also be readable, the stdin concatenation is an
  `&&` chain, and a nonzero assembly status refuses the launch.
- F8 (medium): the audit structural test now also requires every declared ledger row /
  deferred check to be reconciled somewhere, treats an empty `## Audit of findings` over
  a claim-bearing artifact as broken, and the research triggers treat all-`none` blocks
  over measurable claims as missing sections — stated as the orchestrating model's
  reading obligation (structural backstop; claim-level coverage stays the auditor's
  contract).
- F9 (medium): any verdict-CHANGING check outcome re-enters the auditor (Step 2b re-run
  under the next label with the appended results); the `superseded:` line records the
  delta audit's verdict, never an orchestrator-assigned one; decision-criticality decides
  only Phase 4 re-entry. GOAL.md's interview assumption carries the amendment.

Round-003 review fixes (all six findings accepted):
- F10 (high): Step 2a gained input AUTHORIZATION — legitimate inputs are public,
  internet-addressable sources and scratch-derived files from them; a spec naming a
  local path outside scratch, a private/internal service, or anything credentialed is
  `not-run (unauthorized input)` — and BOUNDED recording (derived value/comparison plus
  the few justifying lines, never wholesale contents; the record rides into Step 2b's
  stdin).
- F11 (medium): coverage reading deepened — a checkable H beside `ledger: none` (or a
  `deferred` row pointing at no list entry) is the finer-grain defect; audit acceptance
  requires SUBSTANTIVE per-target coverage with expected sets derived from the research
  artifact itself; unresolved-column mentions count only for checks that could not run;
  token-F over a finding-rich artifact is breakage.
- F12 (medium): the orchestrator never classifies confirm/change — EVERY executed
  new-check result returns to the auditor via a delta audit; bounded termination (a
  third round of new checks marks affected claims `unverifiable (unstable check set)`).
- F13 (medium): ATTEMPT parameter suffixes both the audit label and the `-o` artifact
  (`<topic>.audit-2.md`); the fence refuses an existing artifact outright; the fallback
  writes the NEXT attempt's name; predecessors stay on disk. GOAL.md artifact-name
  assumption amended.
- F14 (medium): the brief is now a full INPUT test (regular, non-symlink, readable,
  non-empty, unaliased) before anything is allocated.
- F15 (low): Step 2b arms an unconditional scratch cleanup at `mktemp` and swaps to the
  exit-record-gated trap at launch, so an assembly failure no longer leaks scratch.

Round-004 review fixes (F16–F18 fixed; F19 recorded as follow-up):
- F16 (high): Step 2a declares fetched material INERT DATA — never execute, import,
  `source`, install, or evaluate logic obtained from any source however public;
  authorization makes an input readable, not runnable; computations are always
  orchestrator-authored; scratch runs carry no credentials in the environment.
- F17 (medium): the audit structural test requires exactly one verdict-summary row
  (verdict, grounds, unresolved checks) for every enumerated H-item AND every F-item the
  audit examines; mirrored in the fallback trigger — an F refutation can no longer live
  only in the audit body and vanish before Step 3/P5.
- F18 (low): both fences' launch-time trap now cleans scratch when the run is proven
  over (`exit` published) OR was never launched (no `.launch` claim), preserving only a
  launched nonterminal run; the unattributable-foreign-claim race preserves fail-closed
  (stated in the fence comment).

## Recorded follow-ups (review loop)
- F19 (low, round 004): this Goal's own research artifact predates the T01 contract and
  lacks the three semantic blocks; regenerating and auditing it through the new pipeline
  is non-blocking follow-up work. The recipe's intro already carries the
  evidence-informed qualification (F5), so the citation strength matches the artifact's
  status.
- F20 (low, round 005): the `--output-schema` bullet contradicted the Markdown-only
  downstream flow (Step 2a and the fallback gate on literal headings). Fixed at the
  round-005 seal — the bullet now forbids the option for this flow — but the loop closed
  at the per-task 5-round cap under codex-review §4, so this fix was NOT re-verified by
  a further reviewer round. Evidence: codex-review-005.md.

## Pre-review defect-class self-sweep (codex-review Step 0)
- Fence-runnability class (this file's own history): Step 2b re-derives every value it
  uses — no variable crosses a fence; `bash -n` on the extracted block passes; the block
  refuses to run without the research artifact and the data-checks record.
- Injection-handoff class (sibling T01 finding, swept here): Step 2a bars pasting or
  lightly editing command text out of the artifact and bars mutation/secrets; the audit
  prompt frames all stdin as untrusted with in-payload directives reportable.
- Silent-skip class: the data-checks file exists in every path (`none` included); Step 2b
  refuses without it; the audit has its own fallback trigger; the closing rule now reads
  "neither the research nor its audit".
- Loader-substitution hazard: the skill loader replaces dollar-digit tokens in this file;
  all added fences avoid positional parameters (the one pre-existing `kill -"$1"` example
  in Step 2's measurement block is untouched).
- Label-reuse class: the audit uses its own label family with documented retry suffixes.
- Leaf-aliasing class (round 001 F1, widened by round 002 F6): ancestors were checked but
  terminal paths were not, and `-L` alone misses hard links; swept BOTH fences — Step 2b
  (inputs regular/non-symlink/readable/non-empty/link-count-1, `-o` target same-or-absent,
  stdin assembled under `$SCRATCH`) and Step 2 (brief + `-o` artifact, same test) — plus a
  prose rule for every orchestrator-written artifact. dstack additionally refuses a
  symlinked stdin file at launch.
- Silent-degradation class (round 001 F2/F3/F4): swept every acceptance/branch point —
  research fallback resumes 2a–2c, audit acceptance requires full structure, every check
  outcome reconciles into the reported verdicts.

## Files changed (where / why)
- `claude/skills/codex-research/SKILL.md` — the three-pass rewiring above; read by the
  orchestrating model at every P3 through `~/.claude/skills/codex-research` (symlinked
  dir).

## Direct verification (repo policy: no TDD)
Recorded from actual runs (2026-08-21):
- Section sequence via `grep -n "^## "` → Step 1, 2, 2a, 2b, 2c, 3, Fallback in order.
- `readlink ~/.claude/skills/codex-research` → resolves into this repo; the live file
  carries the new steps.
- `bash -n` on the extracted Step 2b fence (slugs substituted) → `SYNTAX OK`.
- The nine-section prompt list present exactly once.
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard`.
Re-run after the round-001 fixes: all three extracted bash fences (Step 2, Step 2b,
source counter) pass `bash -n`; section sequence unchanged; `audit-input` appears only
inside the Step 2b fence (write + `--stdin`, both under `$SCRATCH`); guard still green.
Re-run after the round-002 fixes (functional probes, recorded outputs):
- all three fences pass `bash -n` again;
- the aliased-leaf guard expression REFUSES a hard-linked file, a symlink, a directory,
  and a file whose twin link exists, while PASSING an absent path and the same file once
  its extra link is removed;
- the `&&`-chained assembly REFUSES when the first input is unreadable, and the old
  plain-group form was demonstrated to pass in that case (the reported fail-open defect).
Re-run after the round-003 fixes (functional probes, recorded outputs):
- all three fences pass `bash -n` again;
- the ATTEMPT case guard accepts `''`/`-2`/`-99` and refuses `x`/`-abc`/`--2`;
- an empty brief is REFUSED by the new brief guard while a non-empty one passes;
- a bash probe of the trap ordering shows the unconditional mktemp-time trap removing
  the scratch directory when assembly fails (rc=1, directory gone).
Re-run after the round-004 fixes (functional probes, recorded outputs):
- all three fences pass `bash -n` again;
- the launch-time trap condition probed over its three states: no-launch/no-exit →
  CLEAN, launch/no-exit → PRESERVE, launch+exit → CLEAN;
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard`.

## E2E verification
NOT COMPLETED — halted by user decision (2026-08-22). The post-merge behavioral E2E (one
real research round through the new three-pass recipe, doubling as the M1 E2E) was
launched via the installed fence (label `socratic-research-verification-research-2`,
brief `research/e2e-node-lts.brief.txt`); the user directed a stop mid-round before the
artifact existed (capture exit=143, clean teardown, no partial artifact). What IS
verified: the review loop's direct-run evidence above (fence syntax, guard probes,
symlink liveness) and rounds 001–005 closing `Consensus: resolved`. What is NOT: no
research round has yet exercised Steps 2a–2c end to end. The next real Goal's P3 runs
this pipeline unconditionally, which is where the behavioral confirmation will land.

## Gate status
- [x] Verification: direct-run checks recorded above (section sequence, live symlink,
  `bash -n` on the extracted fence, prompt pin count, guard); behavioral confirmation of
  the full three-pass flow is the M1 E2E research round (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [ ] E2E capture verified
