# Codex adversarial review — Round 002

## Review scope
Re-review

## GPT findings
[severity:high][security] The Round-1 nested-symlink repair landed only in the hook; the CLI still follows nested store and legacy symlinks, permitting reads, writes, and run capture creation outside the repository.
Sites: Primary: `dstack.ensure_store`; confirmed: `cmd_reg`, `cmd_reclaim`, `cmd_run_dir`, `cmd_migrate`, and `cmd_status`.
Evidence: [`ensure_store`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:90) checks only `.dstack` before following `active`, `runs`, `.gitignore`, and `version`; [`migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:297) follows the legacy file.
Verification: `active -> outside-dir` redirects registration files; `version -> readable-file` exposes its first line in an error; `.fullcycle-active -> readable-file` makes migration read and echo its contents.
Suggested direction: Reject symlinks and unexpected types at every store component and legacy path before any read, creation, or mutation.

[severity:high][technical correctness] The Stop hook still silently ignores malformed registry state: a regular-file `active` exits 0, hidden/non-regular entries are unenumerated or skipped, dangling record symlinks precede the symlink check, and malformed nonempty owners are treated as foreign.
Sites: Primary: `fullcycle-gate.sh` active-namespace check; confirmed: its record enumeration and schema/owner handling.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:102) allows any non-directory `active`; lines 106–132 glob only non-hidden files, test `-f` before `-L`, and never enforce the session grammar.
Verification: `{v:1,session:"bad/slash",doc:"docs/.../task.md"}` passes the implemented schema predicate and takes the foreign-owner skip for every valid session.
Suggested direction: Treat every non-transient namespace entry as blocking unless its type, filename hash, complete schema, session grammar, and canonical document identity all validate.

[severity:high][technical correctness] External-tool failures remain fail-open or destructive: failing-but-present `jq` emits no decision, failed Git root discovery exits 0, and unchecked CLI root/SHA results can redirect state to `/.dstack` or collapse migration keys.
Sites: Primary: `fullcycle-gate.sh.block`; confirmed: hook root resolution, `dstack` root resolution, `sha1`, and `cmd_migrate`.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:61) ignores `jq` status and lines 84–85 classify every `git rev-parse` failure as outside-repo; [`dstack`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:54) does not check physical-root or SHA output.
Verification: A failing `jq` produced exit 0 with zero output; a failing `git` produced the same from the full hook; an empty SHA makes migration’s destination equal `active/`, which line 356 treats as already present before archiving the legacy source.
Suggested direction: Validate both exit status and output shape at every dependency boundary, with static blocking fallback for the hook and hard failure for invalid roots or non-40-hex keys.

[severity:medium][technical correctness] Migration still archives legacy ownership without comparing a pre-existing record, so the exact Round-1 owner-loss counterexample remains.
Evidence: [`cmd_migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:355) treats any existing key as “already present” and proceeds to archive at line 368 without checking document, owner, schema, or type.
Verification: Legacy owner A plus an existing owner-B record for the same document leaves B untouched, reports success, and removes A’s only authoritative source.
Suggested direction: Preflight the complete plan and require exact record equality for every existing key before publishing or archiving anything.

[severity:medium][technical correctness] Lowercasing only the key does not canonicalize final-component case; the stored spelling still drives case-sensitive gate semantics and migration duplicate detection.
Sites: Primary: `dstack.canon`; confirmed: Stop-hook Goal classification and `cmd_migrate` duplicate detection.
Evidence: [`canon`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:130) preserves the caller’s basename, while [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:138) recognizes only exact `GOAL.md`.
Verification: On the declared APFS checkout, `docs/dstack-state-store/goal.md` resolves to the existing file, canonicalization retains that spelling, and the hook classifies it as a task.
Suggested direction: Resolve and store one authoritative final-component spelling, then use that same canonical identity in migration and gate classification.

[severity:medium][technical correctness] `reg`’s failed-`ln` branch still reads ownership and reports success without the per-key lock, leaving the original false-success race partially unfixed.
Sites: Primary: `cmd_reg`; confirmed: concurrent `cmd_reclaim` and `cmd_unreg`.
Evidence: [`cmd_reg`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:198) reads `session` at line 202 after losing publication, outside the locked existing-record branch.
Verification: Same-session B wins `ln`; reclaimer C locks and prepares replacement; A loses `ln`, reads B’s owner, C publishes, and A returns success although C now owns the document.
Suggested direction: Acquire the key lock and revalidate the complete record before every post-publication success report.

[severity:medium][UI & UX / DX] The PATH repair was not applied: installation creates an executable that the declared machine cannot invoke, while recovery messages and documentation still prescribe bare `dstack`.
Sites: Primary: `install.sh`; confirmed: `AGENTS.md` and all Stop-hook remediation messages.
Evidence: [`install.sh`](/Users/won/Desktop/Workspace/D-STACK/install.sh:31) links only `~/.claude/bin/dstack`; [`AGENTS.md`](/Users/won/Desktop/Workspace/D-STACK/AGENTS.md:84) and [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:100) use the bare name.
Verification: `~/.claude/bin/dstack` is executable, but `command -v dstack` returns no result in the current environment.
Suggested direction: Use the installed path consistently in every instruction and error, or install into a guaranteed PATH location.

[severity:medium][technical correctness] The string-sentinel deduplication can suppress a distinct registered Goal because accepted document paths may contain the `<doc>` delimiter sequence.
Evidence: [`fullcycle-gate.sh`](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:136) searches concatenated `<$doc>` strings, while `canon` does not reject angle brackets.
Verification: After `docs/outer><docs/target/GOAL.md`, the predicate falsely finds and skips distinct `docs/target/GOAL.md`; a completed first Goal can therefore hide an incomplete second one.
Suggested direction: Remove string-delimited deduplication and fail on duplicate validated document keys instead.

[severity:medium][technical correctness] `run-dir` still has a check-then-create race: two concurrent calls with the same session and label can both succeed and share one capture directory.
Evidence: [`cmd_run_dir`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:391) checks `-e` separately, then uses idempotent `mkdir -p`.
Verification: Both calls can observe absence; the first creates the directory and the second’s `mkdir -p` also returns success, mixing their bundles.
Suggested direction: Create parent directories first, then use a single plain `mkdir` on the leaf as the atomic claim.

[severity:low][UI & UX / DX] `status` can present an off-schema record as healthy even while the hook reports it unreadable.
Evidence: [`cmd_status`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:267) checks only for a nonempty `doc`; the hook validates version and field types.
Verification: A `v:999` record with `doc` and `session` is listed normally by `status` but blocks as unreadable in the hook.

[severity:low][technical correctness] `prune` can still report success after traversal failures.
Evidence: [`cmd_prune`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:404) suppresses `find` errors and pipelines into successful `wc` without `pipefail`.
Verification: An unreadable session directory is omitted from both counts, leaving captures intact while the command reports zero leftovers.

[severity:low][technical correctness] The archive non-clobber repair misses dangling symlinks.
Evidence: [`cmd_migrate`](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:366) tests only `-e` before `mv` with replacement semantics.
Verification: `-e .fullcycle-active.migrated` is false for a dangling symlink, so migration selects and overwrites that name.

[severity:low][UI & UX / DX] Help aliases still ignore extra arguments.

[severity:low][UI & UX / DX] Dot-prefixed run labels are accepted but omitted from `status` globbing.

[severity:low][technical correctness] `find -mtime +7` retains captures for nearly eight days despite the documented seven-day window.

Omitted-detail: 3 low

GPT verdict: reject — Round 2 leaves concrete gate-bypass, external-path-write, ownership-loss, dependency-failure, and concurrency blockers unresolved.

## Carried decisions
Round-1 decisions stand. Added in Round 2:

- **Sweep siblings, not instances.** Three Round-2 blockers existed only because a Round-1 fix
  landed in the hook and not the CLI. Before claiming a class fixed, grep for every site.
- Every store component is type-checked before use (`require_plain`), and the check runs BEFORE
  any `mkdir -p`.
- The hook treats a malformed namespace as blocking: non-directory `active`, hidden entries,
  non-key filenames, dangling symlinks, and owners violating the session grammar are all
  reported, never skipped.
- Dependency boundaries validate STATUS AND OUTPUT: `jq` emission, git's exit-128-versus-other,
  an absolute physical root, and a 40-hex digest.
- Migration compares an existing key for exact equality in the PREFLIGHT; conflicts block before
  anything is published or archived.
- `canon` stores the real on-disk spelling; the gate's Goal classification is case-insensitive.
- Dedupe on the record key, never on a string-delimited path set.
- Atomic claims everywhere: `ln` for records, plain `mkdir` for run directories.
- Accepted residuals unchanged: no fsync durability, Unicode normalisation on APFS, gitignored
  is not confidential.

Consensus: disagreed
