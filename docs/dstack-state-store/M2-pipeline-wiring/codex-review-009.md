# Codex adversarial review — Round 009

## Review scope
Re-review (Rounds 1-8 rejected; all findings accepted and fixed). Budget extended by the user — see GOAL.md Q5.

## GPT findings
[severity:high][security] The assembler checks only the final pathname for symlinks, allowing an allowlisted snapshot beneath a symlinked parent to resolve outside the repository and be sent to Codex.
Sites: primary: `claude/skills/codex-review/assemble-review.sh:91-106`; confirmed: `emit_file` at lines 177-217.
Evidence: After `[ -L "$f" ]`, `-f`, `wc`, `grep`, and `cat` all follow parent-directory symlinks; no physical-root containment check exists.
Verification: A benign path such as `docs/unit/task.md` passes the leaf check when `docs/unit` is a symlink, after which the automatic task snapshot reads its external target.
Blocking: Yes — genuinely blocking; this is a concrete external-file and potential secret-exposure path.
Suggested direction: Resolve and validate every path beneath the physical repository root before any read, rejecting every symlinked component.

[severity:high][technical correctness] The Round 8 committed-identity repair remains optional: its environment assignment is commented out, while the executable assembler invocation supplies neither commit.
Sites: primary: `claude/skills/codex-review/SKILL.md:198-204`; confirmed: `assemble-review.sh:41-74,197-209`, `full-cycle/SKILL.md:174-190`.
Evidence: The runnable fence executes plain `bash "$AS" …`; with both variables absent, the assembler deliberately selects serial `git diff HEAD`.
Verification: On a clean integration checkout, that command emits zero implementation bytes and labels changed tracked files “no change,” without producing a `SKIPPED` marker that would stop launch.
Blocking: Yes — genuinely blocking; worker-fanout review can still approve a bundle containing none of the committed implementation.
Suggested direction: Give the assembler explicit serial and committed-range modes, making base and head mandatory arguments in the worker invocation.

[severity:medium][technical correctness] Unit integration was added without unit-level downstream contracts: landing still invokes a one-task scope checker, while assembly expects main-owned P8 documentation inside the clean integration checkout.
Sites: primary: `claude/skills/full-cycle/SKILL.md:124-131,174-190`; confirmed: lines 391-394 and `assemble-review.sh:58-70,296`.
Evidence: Checker scope accepts one task branch and declaration, but an integration head contains multiple tasks; workers never modify `docs/`, yet the assembler reads the relative unit `task.md` from its current checkout.
Verification: M2’s three-task integration cannot fit any single declaration; its recorded base contains neither M2 task document, so assembly gets an absent/stale snapshot, or adding the current document makes it undeclared integration content.
Blocking: Yes — genuinely blocking; milestone-granularity fanout still cannot complete a valid review-and-land sequence.
Suggested direction: Define a unit-scope check over the union of owned declarations and supply the orchestrator-owned document snapshot separately from the integration worktree.

[severity:low][security] The Round 8 cleanup repair still relies on manually selecting, stripping, and repeating labels from an inventory that has no review-unit ownership metadata.
Sites: primary: `claude/skills/codex-review/SKILL.md:110-125`; confirmed: `claude/bin/dstack:556-575,812-840`.
Evidence: `status` prints `session/label`, while `rm-run` accepts bare labels; the recipe asks the model to copy “this unit’s” labels and repeat the same manually derived list for verification.
Verification: Omitting an attempt leaves its plaintext capture untouched, and verification examines only the supplied labels, reproducing the Round 8 retention failure.
Blocking: No — retention/privacy follow-up only.
Suggested direction: Persist an authoritative unit-to-capture association and delete or verify from that inventory directly.

[severity:low][technical correctness] The claimed runner-write check accepts a nonempty partial `run.sh`.
Evidence: `SKILL.md:212-234` does not check the heredoc command’s status or publish atomically; it tests only `[ -s "$RD/run.sh" ]`.
Verification: Disk exhaustion or interruption after the first bytes leaves a nonempty truncated script; `Popen` succeeds in starting Bash and the parent falsely prints “launched.”
Blocking: No — it wastes an attempt but cannot seal a completed round.

[severity:low][software structure] The Round 8 declaration repair remains internally false.
Sites: primary: M2 `task.md:28-34,99-104`; confirmed: `04-review-io/task.md:11-22,51-53` and `GOAL.md:132`.
Evidence: M2 still denies any API or sanitization path and says the assembler appears in no milestone declaration, while T04 now declares it; the subordinate record likewise says neither executable file is declared and omits the assembler from “Files changed.”
Verification: Direct comparison of those records yields mutually incompatible ownership and design-history claims.
Blocking: No — documentation drift only.

[severity:low][technical correctness] The maintained schema check fails open when its first temporary-directory setup fails.
Evidence: `skill-schema.test.sh:68-86` uses only `set -u`, does not guard `mktemp` or `awk`, and treats an empty fence set as proof that no bare or unbound CLI calls exist.
Verification: Under the read-only sandbox, both setup commands failed yet the check printed `ok` for both per-fence invariants; without Ruby, the later YAML path is skipped and this can end successfully.
Blocking: No — check reliability follow-up, not a runtime defect.
Suggested direction: Guard every temporary allocation and extraction command and require at least one Bash fence before asserting success.

Omitted-detail: 0 low

GPT verdict: reject — The assembler can read outside the repository, worker reviews still omit their committed implementation, and the unit-integration path remains unsatisfiable.

## Carried decisions — Round 009
Rounds 1-8 decisions stand. Added in Round 9:

- **Leaf checks do not contain paths.** Anything that reads a file must resolve its PARENT and
  prove containment; `-L` on the last component tells you almost nothing.
- **A commented-out assignment is not a mode.** If a contract can be skipped by following the
  runnable line, it is not a contract — make it mandatory and let it fail loudly.
- **Parameterising a scope means the DOWNSTREAM contracts too.** Introducing a unit-level
  integration without a unit-level scope check and a document-supply rule just moves the
  unsatisfiable step later.

Consensus: disagreed
