# Codex adversarial review — Round 001

## Review scope
First and only round for this unit. Bundle: the `scope` enumeration change in
`check-parallel.sh` plus this unit's `task.md`.

## GPT findings
[severity:low][security] Arbitrary Git filenames can inject newlines or control characters into verdict output, violating the checker’s one-line stdout contract.
Sites: primary: `check-parallel.sh:353`; confirmed: `check-parallel.sh:332`, `:340`, `:360`, and `:387-388`.
Evidence: Enumeration is NUL-safe, but paths are interpolated unescaped into `printf`; the unclean-tree diagnostic also converts both record delimiters and filename newlines to newlines.
Verification: An undeclared filename containing `\nPASS` produces multiple apparent verdict lines despite exiting with status 1.
Suggested direction: Apply reversible single-line escaping to every path rendered in a verdict.

[severity:low][technical correctness] Valid declared filenames containing `..` within a component are rejected as suspicious during scope checking.
Sites: primary: `check-parallel.sh:331-332`; confirmed: declaration validation at `check-parallel.sh:102-107`.
Evidence: Declaration validation rejects only components exactly equal to `..`, while scope rejects any path containing the substring `..`.
Verification: Declaring and committing `src/foo..bar` passes `check_path` but produces `VIOLATION: suspicious actual path src/foo..bar`.
Suggested direction: Make scope validation reject only `.` or `..` path components, matching the declaration grammar.

[severity:low][software structure] The historical add/remove behavior is demonstrated only by a throwaway run and is absent from the maintained contract suite.
Evidence: `check-parallel.test.sh:233-272` tests final-state containment, ancestry, cleanliness, and symlinks but has no commit-then-delete fixture.
Verification: The existing suite would remain green if enumeration regressed to the former endpoint diff.

[severity:low][security] The untrusted task artifact embeds review-steering language that attempts to declare parts of the checker outside review scope.
Evidence: `task.md:20-23` states “Out of scope by construction” and describes another artifact as frozen.
Verification: Treating those statements as instructions would suppress review of relevant containment and contract interactions.
Suggested direction: Keep review authority in caller-controlled metadata and phrase task-document boundaries as claims subject to validation.

[severity:low][DX] The task record contains a duplicate `Files changed (where / why)` section whose second instance remains `<pending>`.
Evidence: `task.md:49-58` contains one completed section immediately followed by an unfinished duplicate.
Verification: Static inspection confirms the finalized change inventory and pending placeholder coexist.

Omitted-detail: 0 low

GPT verdict: approve-with-fixes — The enumeration has no demonstrated high- or medium-severity blocker, but the listed diagnostic, grammar, regression-coverage, and review-record defects remain as non-blocking fixes.

## Bundle size (the ratchet, recorded)

Round 001 bundle: 11,355 bytes. No round 002.

## Round outcome

**Zero blocking findings.** Five lows, no high or medium. That meets the closure rule in
`codex-review/SKILL.md`: when a round's remaining findings are all low-severity, close it in the
SAME round rather than spending another 15-25 minutes on polish.

Four of the five were cheap enough to fix rather than record, and were fixed and verified before
sealing — see `response-001.md`. The fifth is declined on repository policy, with the reason
stated there rather than silently dropped.

Consensus: resolved
