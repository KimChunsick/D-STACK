# Codex adversarial review — Round 009

## Review scope
Re-review

## GPT findings
[severity:medium][technical correctness] The guard can approve removal of hard-deny rules because its `.gitignore` probe table is not synchronized with `COMP_RE`.
Evidence: `.gitignore` declares `**/*password*`, and `COMP_RE` recognizes `password`, but the `leaks` array contains no password-named probe. Scenario V force-adds `PASSWORD-list.txt`, testing only the indexed matcher—not whether the `.gitignore` rule exists. In a clean clone, deleting `**/*password*` from both the index and worktree leaves the negation pin unchanged and provides no actual pathname for sections 4–5 to inspect, so the guard can pass although `PASSWORD-list.txt` is now trackable. Other newly added rules, including several `.DS_Store`, key, P12/PFX, secret, and database suffix variants, likewise lack one-to-one ignore probes. This contradicts the retained guard’s purpose of detecting policy drift during configuration changes.
Suggested direction: At the `.gitignore`/section-1 boundary, either pin the complete security deny block or maintain a declarative rule-to-probe table covering every protected pattern. The invariant should be that removing or narrowing any hard-deny rule fails even when no corresponding file currently exists.
Illustrative example:
```text
change:  remove **/*password*
leaks:   contains no password path
tree:    contains no password path
guard:   PASS
probe:   PASSWORD-list.txt is now addable
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In separate disposable clones, remove each security deny rule while keeping index and worktree bytes identical and stage the change. The repaired guard must fail every mutation; additionally assert that each representative probe is ignored before mutation and addable afterward.

[severity:low][security] The diagnostic hardening still emits untrusted `.gitignore` bytes directly.
Evidence: The negation-pin failure interpolates raw `diff` output into `fail`. A rogue negation containing ESC, BEL, carriage-return, or other terminal-control bytes therefore reaches stderr without the `%q` escaping used for discovered pathnames. `fail` also uses `echo`, whose escape interpretation can vary with Bash options.
Suggested direction: Encode `.gitignore` differences before rendering them and implement `fail` with `printf '%s\n'`. Keep raw bytes only for comparison, never terminal output.
Illustrative example:
```text
.gitignore: !/rogue<ESC><terminal-control>
diff output -> fail -> raw control bytes on stderr
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Trigger negation-pin failures containing ESC, BEL, carriage return, newline, and invalid bytes. Capture stderr and assert that controls appear only as escaped representations, including when Bash is invoked with non-default echo behavior.

GPT verdict: reject — The sole retained control still permits a reproducible hard-deny policy regression to pass undetected.

## Maintainer response
1. **Agreed (policy-drift blindness for probe-less rules) — fixed with a
   whole-content pin.** Section 3 now pins the ENTIRE `.gitignore` by SHA-256:
   any byte change — including removing a hard-deny rule for which no pathname
   exists anywhere — fails until `GITIGNORE_SHA_PIN` is deliberately updated in the
   same reviewed commit. The negation line-pin remains as the readable layer for
   the common allowlist-edit case, and the name-based probe battery (which needs no
   files on disk) gained a `password-list.txt` probe. Verified (scenario AA):
   removing the unprobed `**/*.pfx.*` rule (no file, no probe, index==worktree)
   fails with the SHA message; removing `**/*password*` fails as well (battery or
   pin, whichever trips first).
2. **Agreed (diagnostic bytes) — fixed.** `fail`/`pass` now use `printf '%s\n'`
   (no `echo` escape variance), and the negation-pin diff is passed through a
   control-byte scrub (`tr -c '[:print:]\n\t' '?'`) before rendering; comparison
   still uses raw bytes.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- `.gitignore` is double-pinned (SHA-256 whole-content + negation line set); every
  policy edit must update the pin in the same commit — deliberate friction on the
  security boundary, accepted.
- All prior dispositions unchanged.

Consensus: disagreed
