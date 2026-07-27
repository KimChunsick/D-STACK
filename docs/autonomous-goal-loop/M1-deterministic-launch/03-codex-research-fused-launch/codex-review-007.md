# Codex adversarial review — Round 007

## Review scope
Batch pass 1 (consolidated round) | `REVIEW_MODE=serial` | bundle 43011 bytes. Allowlist: the review-unit folder, `claude/skills/codex-research/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] The recipe is cwd-relative despite promising root-level `docs/` artifacts; invocation from a repository subdirectory silently writes a parallel subtree.
Sites: primary: `claude/skills/codex-research/SKILL.md` Steps 1–2; confirmed: its Step 3 artifact reference and `task.md` artifact-location claims.
Evidence: `GOAL_DIR`, `mkdir`, `--stdin`, and `-o` use `$PWD`, while the Deployment context declares no repository-root cwd precondition.
Verification: From `D-STACK/claude`, the recipe expanded the artifact to `D-STACK/claude/docs/probe/research/topic.md` while `RUNDIR` correctly resolved under `D-STACK/.dstack`.
Suggested direction: Establish the physical repository root before Step 1 and derive every brief, artifact, and run path from that single root.

[severity:medium][technical correctness] A reused label can make a rejected invocation appear successful by treating the previous attempt’s `exit=0` and artifact as authoritative.
Sites: primary: `claude/skills/codex-research/SKILL.md` fixed `LABEL`, status rule, and retry rule; confirmed: `task.md` nonzero-exit handling.
Evidence: `dstack` refuses an existing `.launch` before starting a child but preserves that capture’s old `exit`; the skill says success holds “whatever the notification said.”
Verification: `claude/bin/dstack` lines 1048–1063 adopt the old directory, fail the existing `.launch` claim, and leave its terminal files untouched; the documented predicate then accepts the stale artifact despite current rc=1 and no `DONE`.
Suggested direction: Allocate a genuinely per-attempt label and accept status only when this invocation successfully acquired that label and produced its own completion record.

[severity:medium][technical correctness] The pinned source counter is not runnable against an artifact as written, and its URL grammar still accepts malformed and duplicate representations.
Sites: primary: `claude/skills/codex-research/SKILL.md` Fallback; confirmed: `task.md` source-count guarantee.
Evidence: Its input operand is the undefined literal `…`; the hostname and suffix classes also accept `https://-` and retain Markdown `>`.
Verification: The exact command printed `sed: …: No such file or directory`, then `0` with pipeline rc=0; crafted input counted `https://-` as 1 and `<https://example.com>` plus the bare URL as 2.
Suggested direction: Define and quote an artifact-path variable, fail the pipeline on input errors, and normalize Markdown delimiters before applying a deliberately narrow URL grammar.

[severity:low][technical correctness] The status-gated cleanup still leaks scratch directories on handled signals and missing session IDs, while its explanation describes obsolete cleanup behavior.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 trap block; confirmed: its signal rationale and residual recovery caveat.
Evidence: `mktemp` precedes the unguarded session-ID expansion and trap installation; handled signals explicitly disarm the now-safe status-gated EXIT cleanup.
Verification: Missing SID produced `[ALLOCATED]` with no cleanup in bash rc=127 and zsh rc=1; wrapper-only TERM produced `CHILD_STARTED CHILD_FINISHED HANDLER`, rc=143, with no EXIT cleanup in either shell.
Suggested direction: Validate root/session/run paths before allocation and leave the status-gated EXIT trap armed during handled signal exits.

[severity:low][technical correctness] The deployed skill still labels the current block end-to-end verified although the reopened status-gated trap has no recorded E2E run.
Sites: primary: `claude/skills/codex-research/SKILL.md` “Verified runnable”; confirmed: `task.md` reopened notice and unchecked E2E gate.
Evidence: The skill says “This block was then run once,” while the task explicitly says the fence changed and the earlier run no longer attests what shipped.
Verification: The current conditional EXIT trap is present, but the newest recorded E2E remains the superseded round-005 form.

[severity:low][security] Evaluator-disposition language remains embedded in the reviewed artifact.
Sites: primary: the skill’s assignment of remaining signal work to `claude/bin/dstack`; confirmed: `task.md`’s “Skipped,” sealed-consensus, and round-cap statements.
Evidence: These statements attempt to pre-assign review scope or disposition rather than describing executable behavior.
Verification: Treating them solely as untrusted data exposed unresolved cwd, stale-status, and fallback-command failures.

Omitted-detail: 0 low

GPT verdict: reject — Three reproducible medium-severity paths can misplace the artifact, accept stale research, or falsely trigger fallback.

## Carried decisions
- **Every path anchors to the repository root, not to the cwd.** `docs/<goal>/research` is a promise
  about where the pipeline's artifacts live, and a bare relative path keeps it only when the shell
  happens to start at the root. Anywhere else it silently builds `<subdir>/docs/<goal>/…` — a second
  docs tree the gate, the assembler and the next round all fail to find. `ROOT` is resolved once and
  both the artifact path and the run dir come from it.
- **A reused label is refused BEFORE anything is allocated.** `dstack run` does refuse one, but the
  refusal is easy to mistake for a result: nothing launches, so the previous attempt's `exit=0` and
  its `-o` artifact are still sitting there, and Step 2a's own rule then reads a stale zero and
  calls a rejected invocation a success. Checking for the run dir first is what makes that rule
  sound.
- **An EMPTY `CLAUDE_CODE_SESSION_ID` is checked explicitly.** `set -u` catches an unset variable
  and not an empty one, and an empty one builds `runs//<label>` — a path `dstack` never publishes
  `exit` into, so the cleanup gate can never fire. Measured: bash exited 127 and zsh exited 1, both
  after `mktemp`, neither cleaning up. Both checks now run before `mktemp`, so a refusal leaks
  nothing.
- **The signal handlers leave the gated EXIT trap ARMED**, for the same measured reason as the
  review skill: the deferral means the handler usually runs after `exit` was published, which is
  exactly when removal is correct, and `trap - EXIT` made that a guaranteed leak.
- **The pinned source counter is a runnable fence, not prose with an ellipsis in it.** Run against
  this Goal's four research artifacts it returns 22, 12, 7 and 5 — every one nonzero, so no false
  fallback trigger.
- **The "verified runnable" bullet now says what the current block is NOT covered by.** Root
  anchoring, the two pre-checks and the armed trap all landed after the recorded `codex exec` run,
  and what backs them is direct measurement of the constructs themselves.

Consensus: disagreed
