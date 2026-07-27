# Maintainer response — Round 001

Outside the reviewed corpus by design. All five findings accepted; one accepted with its stated
mechanism corrected, because the fix is identical either way and the record should be right.

**[high] The `.worktreeinclude` allowlist was not enforceable as written.** I wrote "list
individual non-secret fixtures; never a directory", which assumed entries are literal filenames.
They are not. Correction to the finding's mechanism: the entries are not gitignore patterns
matched at every depth — the installed client passes them to
`git ls-files --others --ignored --exclude-standard --directory` as PATHSPECS. That distinction
does not rescue the rule. A pathspec still supports wildcards, so one entry can expand to many
files, and gitignored is exactly where `.env`, tokens and service-account keys live. The rule is
now three conditions: exact repository-relative paths to regular files, no pathspec metacharacter
and no directory entry, and each RESOLVED source checked against the secret deny list immediately
before a worker is spawned. Resolve-then-check is the ordering that matters; checking the manifest
text rather than its expansion is precisely how one entry smuggles the rest.

**[medium] The contract verified a worktree the worker was never bound to.** The sharpest finding
of the round. `worktree-lifecycle.create` mandates `git worktree add` and confirms the base, and
then `per-task` only put the path in the brief — a request, not a binding. Spawn with platform
worktree isolation and the platform creates a SECOND, different checkout, leaving the verified one
unused; spawn without it and the subagent starts in the parent working directory. Either way the
whole base-identity requirement I had just added was decorative. Now: one creation mechanism, and
before its first write the worker reports `pwd -P`, `git rev-parse --git-common-dir`,
`--abbrev-ref HEAD` and `HEAD`, which the orchestrator checks against the record. A mismatch voids
the delegation rather than re-pointing it, because work done in the wrong tree was done against
the wrong base.

**[medium] Eligibility had no benefit threshold.** "Correct one typo in `src/x.ts`" satisfied
every predicate and would have paid agent startup, worktree creation, bootstrap, commit,
verification, fan-in and a retained checkout to save a few hundred tokens. Added as a fourth
`delegate-when` condition: a positive isolation benefit, meaning predictably verbose or materially
multi-step work. Eligibility is necessary, not sufficient. This also aligns the rule with the
platform's own guidance to keep quick targeted changes in the main conversation, which the Goal's
research had quoted and I had not applied to my own gate.

**[low] The ticked verification box was unsupported.** Correct, and it is the failure this
repository's own policy names: the row said "behavior confirmed by direct run" while the recorded
commands parse YAML and exercise a checker this task did not modify. This task changes a routing
rule in an instruction document; there is no runtime to drive. The row now says what was actually
run — document invariants, pinned schema check, checker behaviour unchanged — and the section
opens by stating what it cannot claim. The routing rule is exercised at the unit E2E against
crafted declarations.

**[low] The task record had two `Files changed` sections, one still `<pending>`.** An editing
mistake of mine, not a design issue. Removed.

**Not changed, and why.** Nothing was rebutted this round. The corrected `.worktreeinclude`
mechanism is recorded above rather than argued, because it does not change what the rule has to
say.
