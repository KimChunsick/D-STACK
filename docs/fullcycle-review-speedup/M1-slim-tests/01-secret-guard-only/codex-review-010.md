# Codex adversarial review — Round 010

## Review scope
Re-review

## GPT findings
[severity:low][technical correctness] The local-exclude check misclassifies whitespace-prefixed patterns as comments, allowing an untracked protected name to be hidden from the addable scan.
Evidence: The guard permits lines matching `^[[:space:]]*(#|$)`, but Git treats a line as a comment only when `#` is its first character; leading spaces are significant. Therefore ` #PASSWORD` is an active `$GIT_COMMON_DIR/info/exclude` pattern that passes the guard's emptiness check. Because `git ls-files --exclude-standard` includes `info/exclude`, a file with that name is omitted from section 5 even though its component matches `COMP_RE`; on a case-sensitive clone without that local rule, the same pathname is addable. [Git's pattern rules](https://git-scm.com/docs/gitignore#_pattern_format), [git-ls-files `--exclude-standard`](https://git-scm.com/docs/git-ls-files#Documentation/git-ls-files.txt---exclude-standard).
Suggested direction: At the `info/exclude` boundary, permit only genuinely blank lines and lines whose first byte is `#`; do not allow whitespace before the comment marker.
Illustrative example:
```text
info/exclude bytes: " #PASSWORD\n"
current check:       treated as comment
Git interpretation:  active pattern
section 5:           protected file hidden
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a case-sensitive disposable clone, add ` #PASSWORD` to `info/exclude`, create the identically named file, and confirm `git check-ignore -v` identifies the local rule while the current guard passes. The repaired guard must reject the local rule before scanning.

[severity:low][UI/UX & DX] The whole-content hash check preempts the readable negation-set diagnostic during the normal allowlist-edit workflow.
Evidence: Section 3 exits immediately whenever `.gitignore` differs from `GITIGNORE_SHA_PIN`; section 3b therefore cannot display its negation diff unless the developer first updates the hash pin. A newly added `!` rule initially produces only the generic hash-drift message, contrary to the stated purpose of retaining the negation pin as the readable diagnostic layer.
Suggested direction: Evaluate the negation-set comparison before the whole-file hash, or collect both failures and report the negation diff first when applicable. Preserve the hash failure for non-negation changes.
Illustrative example:
```text
change: add !/claude/agents/rogue.md
before: generic SHA drift -> exit
after:  closed-set negation diff -> exit
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Add a rogue negation without updating either pin and require the failure to show the sanitized negation diff; separately remove an unprobed deny rule and require the whole-content hash failure.

GPT verdict: approve-with-fixes — No concrete high/medium blocker remains, but the local-exclude parser and diagnostic ordering warrant non-blocking correction.

## Maintainer response
Zero high/medium findings remain; the two lows are recorded as NON-BLOCKING
follow-ups (per the loop's closure rule, a round left with only recorded lows closes
now rather than opening another round):

- **Follow-up 1 (info/exclude comment parsing):** tighten the emptiness check to
  git's actual semantics — a comment only when `#` is the FIRST byte; whitespace-
  prefixed lines are active patterns and must be rejected. One-line change to the
  `grep -qvE` pattern (`'^(#|$)'`-style with no leading-whitespace tolerance),
  verified by the reviewer's ` #PASSWORD` fixture. Both sides note this is
  fail-open only in combination with a deliberately crafted local exclude on this
  machine; other machines are unaffected (the rule does not travel).
- **Follow-up 2 (diagnostic ordering):** evaluate the negation-set pin before the
  whole-content SHA pin so the common allowlist-edit workflow gets the readable
  diff first; keep the SHA failure for non-negation drift. Pure reordering of two
  existing checks.

Both are recorded here and in the task doc; neither weakens a blocking guarantee
established in rounds 1–9 (the whitespace-comment case still requires a local
info/exclude edit that round-3's check already rejects in its ordinary spelling).

## Carried decisions
- Non-blocking follow-ups 1–2 above (owner: maintainer; to be made in a future
  reviewed change together with a GITIGNORE_SHA_PIN update if `.gitignore` moves).
- All dispositions from rounds 1–9 stand: content scanning excluded by user
  decision; single-user TOCTOU; per-machine addable verdict; case-insensitive
  nested-ignore prohibition; global runtime-dir families; component-level COMP_RE;
  double-pinned `.gitignore`; strict key-material denial incl. `.pub`.

Consensus: agreed
