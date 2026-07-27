# Codex adversarial review — Round 008

## Review scope
Batch pass 2 (consolidated round) | `REVIEW_MODE=serial` | bundle 52562 bytes (r7 43011). Allowlist unchanged: the review-unit folder, `claude/skills/codex-research/SKILL.md`. **§4 cap closure** — every finding fixed, residuals recorded, no concrete HIGH open.

## GPT findings

[severity:medium][technical correctness] The outer zsh wrapper traps only TERM/INT/HUP, so other wrapper-only signals can terminate the harness-visible process while `dstack` and the paid research process remain alive.
Sites: primary: `claude/skills/codex-research/SKILL.md` Step 2 trap block; confirmed: its “exactly two” signal-gap claim and `task.md` Deployment context.
Evidence: The recipe omits QUIT, PIPE, ALRM, USR1, and USR2 even though `dstack` handles them; a signal sent only to the wrapper is not propagated automatically to its child.
Verification: On the deployed `/bin/zsh`, a wrapper-only USR1 returned 158 while its child remained alive; this directly contradicts the claimed SIGKILL/SIGPROF-only gap.
Suggested direction: Enforce one invariant across the outer wrapper and `dstack`: the harness-visible wrapper must not terminate while its launched supervisor remains live.

[severity:medium][security] Root anchoring does not confine writes: a pre-existing symlink in `docs/<goal>/` or `research/` redirects the brief and generated artifact outside the repository.
Sites: primary: `claude/skills/codex-research/SKILL.md` Steps 1–2; confirmed: `mkdir -p`, `--stdin`, `-o`, and the task’s root-level artifact promise.
Evidence: Slug validation rejects path separators but does not reject symlink components; `mkdir -p` and subsequent file opens follow ancestor symlinks.
Verification: With `docs/victim -> /tmp/target`, `GOAL=victim` resolves both output paths below `/tmp/target`; `dstack` checks only whether the final stdin file itself is a symlink.
Suggested direction: Before either write, reject symlink path components and verify the physical research directory remains beneath the physical repository `docs` directory.

[severity:low][technical correctness] The caller’s reconstructed run identity still differs from `dstack` validation and ownership, causing scratch leaks for invalid session IDs and retaining a check-then-launch race.
Sites: primary: `claude/skills/codex-research/SKILL.md` session/run-directory checks; confirmed: `claude/bin/dstack` `require_sid`, `cmd_run_dir`, and `.launch` claim.
Evidence: The recipe accepts every non-empty session ID and uses `[ -e "$RUNDIR" ]`; `dstack` requires `[A-Za-z0-9_-]+` and relies on atomic directory claims.
Verification: `../cross-session` passed the recipe predicate but `dstack run` refused it with status 1; the full recipe would already have allocated scratch with no terminal record to authorize cleanup.
Suggested direction: Validate the exact launcher grammar and bind status handling to an attempt identity atomically acquired by the launcher.

[severity:low][technical correctness] The pinned zero-source gate counts malformed URL-shaped strings and URLs after the Sources section, allowing source-free output to suppress fallback.
Sites: primary: `claude/skills/codex-research/SKILL.md` Fallback; confirmed: `task.md` source-count claims.
Evidence: The host expression accepts `https://-`, retains Markdown `>`, and `sed '/^## Sources/,$p'` never stops at a later section.
Verification: The exact pipeline produced three “unique URLs” from `https://-`, `<https://example.com>`, and its bare form; a Sources section containing no citations followed by an Appendix URL counted 1.
Suggested direction: Bound extraction to the Sources section, normalize Markdown delimiters, and require a minimally valid host.

[severity:low][security] The task payload still embeds evaluator-disposition language that attempts to cap review and pre-resolve an unfixed defect.
Sites: primary: `task.md` “REOPENED after sealing”; confirmed: its “second and last round” and “Accepted as a stated limit” statements.
Evidence: Those statements prescribe review termination and acceptance rather than recording implementation behavior or evidence.
Verification: Ignoring those dispositions exposed both a reproducible source-gate failure and an unreported wrapper-signal gap.

Omitted-detail: 0 low

GPT verdict: reject — Unhandled wrapper signals can orphan a live paid run, and symlink ancestors can redirect research writes outside the repository.

## Carried decisions
- **The wrapper traps every signal `dstack` traps, and the "exactly two gaps" claim was wrong.**
  Measured: under zsh a wrapper-only USR1 exits 158 WITHOUT running the EXIT trap, leaking the
  scratch directory; bash runs it either way. The old three-signal set was the gap. What the fix
  does not buy is now stated rather than implied — a handler cannot cancel a foreground
  `dstack run`, so `codex exec` survives regardless; that residual is `dstack`'s, plus the standing
  rule that a capture with no terminal record must be checked for a live group before relaunching.
- **Root anchoring is not write confinement.** `mkdir -p` and every later open follow ancestor
  symlinks, so `docs/<goal>` pointing at /tmp/target sends both the brief and the `-o` artifact
  outside the repository while every path in the recipe still reads as repo-relative. `dstack` does
  not cover it — it checks only whether the `--stdin` file itself is a symlink. Symlinked ancestors
  are refused and the physical directory is confirmed under the physical repo `docs` before any
  write.
- **The session id is checked against `dstack`'s OWN grammar**, `[A-Za-z0-9_-]+`, not merely for
  non-emptiness. `../cross-session` passed the old predicate and `dstack run` then refused the
  launch — after scratch had been allocated with no terminal record to authorise cleaning it. The
  run-dir pre-check is labelled a pre-check: `dstack`'s `.launch` mkdir stays the atomic claim.
- **The zero-source gate is bounded, host-validated and delimiter-normalised.** Three ways a
  source-free artifact suppressed its own fallback: `sed '/^## Sources/,$p'` ran to end of file so an
  Appendix link counted; `https://-` counted as a source; `<https://example.com>` and its bare form
  counted twice. Fixed and measured — 22/12/7/5 unchanged on the four real artifacts, so no true
  positive was lost, while the reviewer's fixtures went 4 -> 1 and 1 -> 0.

Consensus: resolved
