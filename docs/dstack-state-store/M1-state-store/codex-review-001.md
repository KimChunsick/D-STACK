# Codex adversarial review — Round 001

## Review scope
Adversarial review

## GPT findings
[severity:high][technical correctness] The Stop hook resolves state relative to its invocation CWD, while `dstack` resolves it at the Git root; entering a subdirectory therefore makes active work look absent and opens the gate.
Sites: Primary: `fullcycle-gate.sh`; confirmed: `dstack` repository-root resolution.
Evidence: [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:64) uses relative registry paths and exits at line 72; [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:49) uses `git rev-parse --show-toplevel`. Claude documents that hooks run in the current directory and that CWD can change during a session in the [hooks reference](https://code.claude.com/docs/en/hooks).
Verification: The real root store exists; from `docs/`, `{"stop_hook_active":false}` produced no block and exited 0.
Suggested direction: Resolve one absolute Git/project root before every gate lookup and use it for both registry and document paths.

[severity:high][security] Only the top `.dstack` path is checked for symlinks; nested state paths, records, and the legacy file can redirect reads or writes outside the repository.
Sites: Primary: `dstack.ensure_store`; confirmed: `cmd_status`, `cmd_migrate`, `cmd_run_dir`, and the Stop-hook record scan.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:76) checks only `STORE`, then lines 82–90 follow/create `active`, `runs`, `.gitignore`, and `version`; migration reads `LEGACY` without rejecting symlinks. [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:72) likewise follows active-directory and record symlinks.
Verification: A regular `.dstack` containing `active -> outside-dir` redirects registration writes; a dangling `version` symlink creates its external target; `.fullcycle-active -> sensitive-file` makes migration read and print that file.
Suggested direction: Reject symlinks and unexpected file types at every state-path component before access, and verify physical containment independently in the CLI and hook.

[severity:medium][technical correctness] Final-component case is not canonicalized on the declared case-insensitive APFS volume, so two sessions can claim the same physical document under distinct keys.
Sites: Primary: `dstack.canon`; confirmed: Stop-hook deduplication and `status`.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:105) physically resolves only the parent directory and preserves the caller’s basename spelling; [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:97) also treats those spellings as distinct.
Verification: `task.md` and `TASK.md` both resolve to the existing file on this machine, while their keys are `ce29b34f…` and `1334df13…`.
Suggested direction: Resolve the final component to one authoritative filesystem/Git spelling before hashing, storing, or comparing it.

[severity:medium][technical correctness] `reg` ignores the per-key lock during its existing-record/idempotence path, allowing concurrent release or reclaim to make it report ownership that no longer exists.
Sites: Primary: `cmd_reg`; confirmed: `cmd_unreg` and `cmd_reclaim`.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:149) reads and returns success without `klock`; lines 178–218 remove or replace that record while holding a lock that `reg` does not honor.
Verification: Reclaimer B can lock/read A’s record; A then re-registers and receives success; B’s pending `mv` replaces the record with owner B, silently releasing A.
Suggested direction: Every same-key state transition, including idempotent registration reads, must participate in the per-key serialization or use a generation-checked compare-and-swap.

[severity:medium][technical correctness] Gate records are neither read as one snapshot nor schema-validated, so malformed ownership can be skipped silently and concurrent reclaim can mix generations.
Sites: Primary: Stop-hook scan; confirmed: `dstack status`.
Evidence: [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:81) runs separate `jq` reads and validates neither `v`, session grammar, canonical document identity, nor filename hash before its foreign-owner skip.
Verification: `{v:999,session:"bad/slash",doc:"docs/.../task.md"}` is treated as foreign by every valid session rather than unreadable; a reclaim between the two reads can similarly supply an obsolete owner.
Suggested direction: Parse each record once and fail closed unless its complete tuple satisfies schema, owner, canonical path, and key invariants.

[severity:medium][technical correctness] Migration can claim success while discarding ownership or creating records the CLI can never address, contradicting its lossless-cutover contract.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:259) bypasses `canon`, and line 306 leaves any existing key untouched without comparing its document or owner before line 315 archives the legacy source.
Verification: Legacy owner A plus an existing owner-B record archives A silently; `A<TAB>docs/../AGENTS.md` passes migration and remains gate-enforced, but later `unreg`/`reclaim` canonicalization rejects it as outside `docs/`.
Suggested direction: Canonicalize and validate the entire migration plan, then require exact equality for pre-existing records before publishing or archiving anything.

[severity:medium][technical correctness] Missing or failing `jq` opens the supposedly fail-closed global gate.
Evidence: [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:62) has no dependency check; the legacy branch exits 0 after failed generation, while the final generator at line 214 exits nonzero. Claude documents that non-2 exits are non-blocking and JSON is processed only on exit 0 in the [hooks reference](https://code.claude.com/docs/en/hooks).
Verification: With `jq` absent, a legacy refusal emits no decision and exits 0; an active-work refusal exits 127, which Claude treats as a non-blocking hook error.
Suggested direction: Convert dependency and JSON-generation failures into an explicit blocking response or exit 2 with a safe static reason.

[severity:medium][UI & UX / DX] Installation creates `~/.claude/bin/dstack` but never makes it discoverable, while every user-facing instruction invokes bare `dstack`.
Sites: Primary: `install.sh`; confirmed: `AGENTS.md` and the Stop-hook cutover/error messages.
Evidence: [install.sh](/Users/won/Desktop/Workspace/D-STACK/install.sh:31) only creates the symlink; [AGENTS.md](/Users/won/Desktop/Workspace/D-STACK/AGENTS.md:84) and [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:68) prescribe an unqualified command.
Verification: On the declared maintainer machine, the installed executable exists but `command -v dstack` returns no result; the recorded verification tested only `~/.claude/bin/dstack --help`.
Suggested direction: Either install a guaranteed PATH entry or use the installed absolute path consistently.

[severity:low][security] `run-dir` accepts `.` and `..`, and repeated labels reuse an existing directory, defeating per-run isolation and bounded retention.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:321) permits dots and uses `mkdir -p`; pruning only visits directories exactly two levels below `runs/`.
Verification: Label `..` returns a path resolving to `runs/`, where captures are shared and never selected by the retention scan; the same label always returns the same directory.

[severity:low][UI & UX / DX] The final gate message still tells users to remove a line from `.fullcycle-active`, the retired mechanism.
Evidence: [fullcycle-gate.sh](/Users/won/Desktop/Workspace/D-STACK/claude/hooks/fullcycle-gate.sh:213) contains the obsolete escape-hatch instruction.
Verification: Any ordinary incomplete-work block emits that stale remediation instead of `dstack unreg`.

[severity:low][UI & UX / DX] `status` reports both an unreadable record and “(none)” when corruption is the only entry.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:229) does not increment `n` for unreadable records, then line 240 prints `(none)`.
Verification: One malformed record follows both output branches sequentially.

[severity:low][UI & UX / DX] Command arity and help handling are inconsistent: read-only commands ignore extra arguments, and `--help` fails outside a Git repository.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:42) performs environment/repository discovery before dispatch; lines 366–370 do not validate remaining arguments.
Verification: Control flow accepts `status junk` and never reaches `usage` outside a repository.

[severity:low][technical correctness] Trailing-newline document names are not preserved despite JSON being chosen to support newline-containing paths.
Evidence: [dstack](/Users/won/Desktop/Workspace/D-STACK/claude/bin/dstack:108) obtains basename and canonical output through command substitutions, which strip trailing newlines.
Verification: A final component ending in newline is changed before the existence check and cannot be registered under its real identity.

[severity:low][technical correctness] Migration overwrites an existing `.fullcycle-active.migrated` archive with `mv -f`.

[severity:low][technical correctness] `unreg` and `prune` can report success after suppressed filesystem failures.

[severity:low][technical correctness] `prune` mutates store contents without the cutover guard promised for every mutating command.

Omitted-detail: 3 low

GPT verdict: reject — the gate can be bypassed by ordinary CWD changes, and unresolved namespace, ownership, migration, and dependency failures violate the milestone’s fail-closed state guarantees.

## Carried decisions
All Round-1 findings were accepted and fixed; none were rebutted, so nothing is carried as an
open disagreement. Standing decisions relevant to later rounds:

- State is anchored at the git root by BOTH `dstack` and the Stop hook. Any future reader of
  `.dstack/` must resolve the root the same way; a CWD-relative read is a gate bypass.
- Keys are SHA-1 of the LOWERCASED canonical path. This is collision-conservative on
  case-sensitive volumes by decision, matching `check-parallel.sh`'s stance on file overlap.
- Blocking must never depend on an external tool being present. `block()` has a jq-free fallback.
- The global registry lock stays removed; per-key locks cover read-then-write operations only.
- Accepted residuals, recorded not overlooked: no fsync durability (bash cannot express it and
  this state is reconstructible from the work documents); Unicode-normalisation variants of a
  path on APFS; gitignored is not confidential (mode 700 and bounded retention are the mitigation).
- Repo policy: no tests, no Red-Green-Refactor. Gates are satisfied by recorded direct-run
  evidence.

Consensus: disagreed
