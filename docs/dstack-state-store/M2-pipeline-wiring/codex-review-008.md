# Codex adversarial review — Round 008

## Review scope
Re-review (Rounds 1-7 rejected; all findings accepted and fixed). Budget extended by the user — see GOAL.md Q5.

## GPT findings
[severity:high][technical correctness] The Round-7 assembler repair still does not bind review material to the committed worker identity; the launch never supplies `REVIEW_BASE`, and the assembler’s one-commit diff reads the working tree.
Sites: Primary: `claude/skills/codex-review/SKILL.md:185-197`; confirmed: `claude/skills/codex-review/assemble-review.sh:41-51,175`, `claude/skills/full-cycle/SKILL.md:174-179`.
Evidence: `REVIEW_BASE` occurs only inside the assembler, while `git diff <commit> -- <path>` compares that commit with the working tree, not another commit, as specified by the [Git documentation](https://git-scm.com/docs/git-diff).
Verification: A clean committed worker launched by the documented recipe defaults to `git diff HEAD` and contributes zero implementation bytes; even when manually set, the checkout probe produced 14,200 bytes for base-to-working-tree versus 12,024 for base-to-HEAD, so a post-check dirty edit can replace what the eventual merge actually contains.
Blocking: Yes — genuinely blocking; P9 can approve material that omits or differs from the commit later merged.
Suggested direction: Require explicit recorded base and head arguments, verify current HEAD and cleanliness at assembly time, and use a two-tree `base HEAD` diff for worker reviews.

[severity:medium][technical correctness] The Round-7 review-unit repair remains cyclic and retains task-scoped serialization, leaving milestone fan-out without a valid review/merge order and still permitting round-file races.
Sites: Primary: `claude/skills/full-cycle/SKILL.md:180-189`; confirmed: `claude/skills/codex-review/SKILL.md:356-370`, `claude/skills/full-cycle/SKILL.md:392-403`.
Evidence: Merge requires unit consensus, but every owned branch must merge before that unit’s round seals; Step 3 still says only reviews for the same task are serial.
Verification: For M2’s three tasks in one unit, sealing waits for merges that wait for sealing; interpreting Step 3 literally also lets two tasks select the same unused `codex-review-<NNN>.md`.
Blocking: Yes — genuinely blocking; the supported milestone-granularity worker path either deadlocks or clobbers its review record.
Suggested direction: Define one unit-level integration candidate, review its exact committed identity before final merge, and express serialization consistently per review unit.

[severity:low][security] The claimed fail-closed cleanup still cannot detect a mistyped or omitted capture label.
Sites: Primary: `claude/skills/codex-review/SKILL.md:110-121`; confirmed: `claude/bin/dstack:770-799`.
Evidence: Both deletion and verification operate solely on the manually repeated labels; `rm-run` intentionally returns success for a nonexistent label.
Verification: With `goal-unit-r001` present, supplying `goal-unit-r01` removes nothing, verifies that the same nonexistent typo is absent, and leaves the fresh plaintext bundle untouched by `prune`.
Blocking: No — this is non-blocking retention/privacy follow-up.
Suggested direction: Compare cleanup against an authoritative inventory of captures belonging to the closed unit rather than against the caller’s repeated list.

[severity:low][DX] The Round-7 size-accounting correction was not applied.
Sites: Primary: `docs/dstack-state-store/GOAL.md:134`; confirmed: M2 `task.md:96`, `06-inject-slim/task.md:30-33`.
Evidence: The records still claim +495 bytes, +7 lines, 9,165 bytes, and “net length stays flat.”
Verification: The checkout measures 8,670→9,304 bytes, 8,019→8,647 characters, and 163→171 lines: +634 bytes, +628 characters, and +8 lines.
Blocking: No — documentation-only accounting drift.

[severity:low][software structure] The durable M2 declarations still deny or omit executable surfaces added by their own review repairs.
Sites: Primary: M2 `task.md:29-33,86-103`; confirmed: `docs/dstack-state-store/GOAL.md:132`, `claude/skills/codex-review/assemble-review.sh`, `claude/bin/dstack`.
Evidence: The design record still says there is no API or sanitization path, its file list omits the assembler, and T04 still declares only `codex-review/SKILL.md`.
Verification: Round 7 changed the assembler’s gate semantics, while Round 5 added the validated destructive `rm-run` API; the Round-7 claim that the assembler was “named here” is false for the authoritative unit record.
Blocking: No — the implementation was supplied for review, but ownership and design history remain inaccurate.

[severity:low][technical correctness] The schema check still has no consolidated cleanup trap despite claiming one.
Evidence: `skill-schema.test.sh:68,102` allocates two directories, while lines 119-121 perform only normal-path final cleanup.
Verification: Interrupting after either `mktemp -d` leaves that directory behind.
Blocking: No — bounded temporary-directory leakage only.

[severity:low][technical correctness] Runner-file creation remains unchecked before the launcher reports success.
Evidence: `codex-review/SKILL.md:203-224` does not test the heredoc write before spawning `/bin/bash` and printing “launched.”
Verification: If the write fails or is partial, `Popen` can still succeed because `/bin/bash` itself starts; the child then fails and the monitor later reports VANISHED.
Blocking: No — it wastes an attempt but cannot seal a completed round.

[severity:low][technical correctness] The triage normalization converts `grep` read errors into “no blocking findings” — non-blocking because Step 2b still requires an independent output read.

Check execution: shell syntax checks and `git diff --check` completed; both pinned checks were attempted but could not complete because the enforced read-only sandbox denied their temporary files, which is not a repository finding.

Omitted-detail: 1 low

GPT verdict: reject — The review gate still does not consume the committed worker identity, and milestone-level fan-out remains cyclic with task-scoped round serialization.

## Carried decisions — Round 008
Rounds 1-7 decisions stand. Added in Round 8:

- **`git diff <commit> -- <path>` is commit-versus-WORKING-TREE.** A committed range needs two
  trees, a checked-out head, and a clean tree — all three, or the bundle describes something the
  merge will not carry.
- **A contract split across a tool and its caller is not a contract.** The assembler knowing about
  `REVIEW_BASE` while no recipe set it is the same as not having it.
- **Integrating and landing are different gates.** Gate the landing on review; gating the
  integration on review is how a cycle gets written.
- **Idempotent APIs need their input derived, not retyped.** Read the labels from the tool that
  knows them.
- **Do not record a number that keeps changing.** Record how to measure it, and date the figure.

Consensus: disagreed
