# Codex adversarial review — Round 004

## Review scope
Re-review (Rounds 1-3 rejected; all findings accepted and fixed)

## GPT findings
[severity:high][technical correctness] The dependency-failure repair remains fail-open: Git status 128 is treated as proof of “outside a repository,” jq extraction statuses are ignored, and digest failures are masked by `cut`.

Sites: Primary: `fullcycle-gate.sh` repository discovery; confirmed: `stop_hook_active`, record field extraction, hook key derivation, `dstack.sha1`, and `read_record`.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:97) exits 0 for every status 128; lines 88 and 173–194 ignore or mask dependency status; [`dstack`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:103) likewise observes only the pipeline’s final `cut`.
Verification: Inside the checkout, `GIT_DIR=/definitely/missing-dstack-gitdir git rev-parse --is-inside-work-tree` returned 128, which the hook classifies as safe; a jq emitting `y` then exiting nonzero still opens line 88, and a digest tool emitting 40 hex before failure is accepted.
Suggested direction: Validate status and semantic output separately at every dependency boundary, and permit “outside repository” only after independently proving no repository metadata exists.

[severity:high][technical correctness] Migration still lacks a quiescent snapshot, allowing a concurrent legacy writer to be archived without its claim ever reaching the new registry.

Evidence: [`cmd_migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:455) finishes reading the live legacy file, publishes the captured plan at line 524, then moves the current file wholesale at line 543 without locking, identity comparison, or quiescence enforcement.
Verification: If an older tab appends a claim after the reader reaches EOF but before `mv`, the appended line moves into `.migrated`, no active record is created for it, and the hook sees no legacy file—silently releasing that owner.
Suggested direction: Establish an immutable migration snapshot and an enforceable quiescent-cutover protocol that detects or refuses every write occurring after snapshot acquisition.

[severity:high][security] The “complete record invariant” accepts lexical `docs/` paths that traverse `..` or symlinked parents, so the global hook can read outside the repository and accept records no CLI command can address.

Sites: Primary: Stop-hook record scan and `section`; confirmed: `dstack.read_record`, `cmd_status`, `cmd_unreg`, and `cmd_reclaim`.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:196) checks only the `docs/*` prefix and final component; [`read_record`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:268) repeats that partial predicate instead of applying `canon`, and both validators also omit the advertised `ts` field.
Verification: A self-hashed current-owner record for `docs/../../outside/GOAL.md` passes the prefix and final-file tests before `section` opens the external file; `docs/x/../real/GOAL.md` is shown healthy by `status`, while `unreg` canonicalizes it to a different key and cannot release it.
Suggested direction: Require every stored document to equal its independently derived physical, repository-relative, printable-ASCII canonical identity before ownership filtering or file access.

[severity:medium][security] The Round-3 dynamic-child symlink repair remains partial: run status traverses session symlinks, while dangling active-record symlinks are reported as absent by mutation commands.

Sites: Primary: `cmd_status`; confirmed: `cmd_unreg`, `cmd_reclaim`, and `cmd_reg`’s failed-publication branch.
Evidence: [`cmd_status`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:431) expands `$RUNS/*/*` and tests `-d`, both of which follow a symlinked session directory; lines 352 and 378 use only `-e`, and line 339 opens a record before `assert_record`.
Verification: `runs/session -> outside-directory` makes `status` enumerate external child names; a dangling active-record symlink makes `unreg` and `reclaim` return “not registered” while the hook continues blocking on that same entry.
Suggested direction: Enumerate every dynamic namespace child without traversal, and treat `-e || -L` as occupied before routing all record access through `read_record`.

[severity:medium][technical correctness] The schema marker is not authoritative for the two read paths: the hook and `status` ignore it, while mutating commands refuse it.

Sites: Primary: Stop-hook store discovery; confirmed: `cmd_status` and `ensure_store`.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:113) checks only `.dstack`, `active`, and the legacy path; [`cmd_status`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:397) omits `version`, whereas `ensure_store` enforces it at line 170.
Verification: With `version` set to `2` and an empty `active/`, the hook exits 0 and `status` reports no documents, while every mutation dies; additionally, command substitution makes `1` followed by extra blank lines pass the claimed whole-file comparison.
Suggested direction: Make existence, plain-file type, and exact supported version a shared prerequisite for every store reader.

[severity:medium][technical correctness] Removing the angle-bracket refusal exposed the still-delimiter-based migration dedupe, which rejects two distinct valid documents as duplicates.

Evidence: [`cmd_migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:490) searches a concatenated `<doc>` string, while [`canon`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:209) now accepts printable angle brackets.
Verification: After recording `docs/outer><docs/target/GOAL.md`, the exact predicate falsely classifies distinct `docs/target/GOAL.md` as already seen and blocks migration.
Suggested direction: Deduplicate migration entries by their validated 40-hex keys, never by a path-delimited string.

[severity:medium][UI & UX / DX] The PATH repair remains incomplete inside the CLI’s recovery messages, so ownership and cutover failures still prescribe commands that do not resolve in the declared installation.

Sites: Primary: `cmd_reg`; confirmed: `cmd_status`, `cmd_run_dir`, and `usage`.
Evidence: [`dstack`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:320) recommends bare `dstack reclaim`; line 402 recommends bare `dstack migrate`, despite [`AGENTS.md`](/Users/won/Desktop/Workspace/D-STACK/AGENTS.md:75) stating the directory is never added to PATH.
Verification: `command -v dstack` returned no result in the reviewed environment.
Suggested direction: Render every executable recovery command with `$HOME/.claude/bin/dstack` and shell-safe document quoting.

[severity:low][technical correctness] Lock release failures are suppressed, so a successful ownership mutation can leave a stale lock while reporting success.

Evidence: [`kunlock`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:301) ignores `rmdir` failure and immediately clears every cleanup trap.
Verification: Any nonempty or otherwise unremovable lock directory survives, after which later operations time out despite the original command claiming completion.

[severity:low][technical correctness] Empty-legacy cleanup reports removal without checking whether removal succeeded.

Evidence: [`cmd_migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:453) executes `rm -f` and unconditionally prints “removed.”
Verification: An immutable or permission-denied empty legacy file produces the success message while remaining present.

[severity:low][UI & UX / DX] A `run-dir` permission failure permanently consumes the requested label.

Evidence: [`cmd_run_dir`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:580) claims the leaf before line 581 performs the fallible chmod, with no cleanup on failure.
Verification: After chmod fails, the command exits unsuccessfully but a retry reports the label as already taken.

[severity:low][software structure] The Round-1 evaluator-control separation remains unfixed: the untrusted review unit still embeds directives declaring what is reviewed and what is out of scope.

Evidence: The supplied task snapshot says “This document is the review unit,” directs the reviewer to read subordinate documents, and declares exclusions “out of scope by construction,” despite the design disposition saying future briefs would separate settled context from reviewer instructions.
Verification: Those directives are still present in the Round-4 payload and could suppress the concurrent-cutover defect above if followed as evaluator commands.
Suggested direction: Keep evaluator instructions outside the reviewed artifact and express settled decisions only as labelled historical context.

Omitted-detail: 0 low

GPT verdict: reject — fatal Git failures can still open the gate, migration can silently discard a concurrent legacy claim, and malformed records can escape repository path identity.

## Carried decisions — Round 004
Rounds 1-3 decisions stand. Added in Round 4:

- **A tool's exit status is not a pipeline's exit status.** Never let a digest, a git query or a
  jq read reach a consumer without its own status checked.
- **"The tool said no" is not proof of absence.** Status 128 from git means "no repository OR a
  broken one"; prove which before choosing the fail-open branch.
- **Path identity is canonical equality, never a prefix plus an existence test.** A stored path
  must equal what the writer's own canonicaliser derives from it.
- **A file other processes append to cannot be locked, but it can be digested.** Detect the race
  and refuse the destructive step; never assume quiescence.
- **Dedupe on fixed-width keys, never on a delimited concatenation of caller-supplied strings.**
- **Evaluator instructions never live inside the artifact under review.** Work docs describe how
  work is filed; the prompt decides scope.
- Accepted residuals unchanged: no fsync durability, gitignored is not confidential, a ticked box
  is self-attested, `kill -0` cannot prove process identity.

Consensus: disagreed
