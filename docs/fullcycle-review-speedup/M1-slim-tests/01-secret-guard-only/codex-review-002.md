# Codex adversarial review — Round 002

## Review scope
Re-review

## GPT findings
[severity:high][security] The sole retained control inspects pathnames, not file contents, so it deterministically permits credentials inside deliberately tracked configuration or documentation files.
Evidence: Section 4 pipes only `git ls-files` pathnames into a filename regex. For example, `claude/settings.json` is explicitly allowlisted, but adding a token value to that tracked file does not change its pathname and therefore passes every section. The supplied research recommends content-aware scanners as backstops, yet no such backstop or evidence of one is included. This contradicts the absolute claim that secrets "are never tracked" and the stated public-repository safety intent.
Suggested direction: At the staged-content boundary, add content-aware secret detection to `tests/secret-guard.sh` or an enforced CI/push control. If content scanning is intentionally excluded, narrow the documentation and record this concrete residual risk as an explicit user decision.
Illustrative example:
```text
tracked path: claude/settings.json
new content: {"api_token": "<synthetic test credential>"}
pathname scan: no match
current result: PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Stage synthetic credentials in each allowlisted text/configuration format and confirm the retained control rejects them without exposing fixture values in output; verify representative benign content remains accepted.

[severity:medium][security] The claimed class-wide sensitive-path fix still leaves the ignore probes, `.gitignore`, and tracked-tree scan inconsistent.
Evidence: The tracked-tree regex recognizes `.sqlite3-wal` and `.db3-wal`, but the displayed ignore rules provide `**/*.sqlite-*`, `**/*.sqlite3`, `**/*.db-*`, and `**/*.db3`; neither numbered-extension journal family is denied or probed. Conversely, the probe list declares extensionless `api_token` and `deploy_key_prod` sensitive, while section 4 does not recognize those basename families. A force-added `claude/skills/full-cycle/api_token` is outside the one hard-coded probe path and does not match the tracked-tree regex, so the guard can pass with that sensitive path in the index.
Suggested direction: Define sensitive filename families once and derive or validate both ignore coverage and tracked-index coverage from that definition. Add numbered-extension journal variants and every extensionless secret family to the invariant at the `.gitignore`/guard boundary.
Illustrative example:
```text
claude/skills/full-cycle/cache.sqlite3-wal -> missing ignore-family coverage
claude/skills/full-cycle/api_token         -> forced tracked; tracked regex misses
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: In a disposable clone, test `.sqlite3-wal`, `.sqlite3-shm`, `.db3-wal`, and `.db3-journal` as untracked files inside every wholesale-allowed subtree. Separately force-add `api_token` and `deploy_key_prod` at paths other than the hard-coded probes; every case must fail.

[severity:medium][security] The destructive-probe repair checks only the final pathname, allowing writes through symlinked ancestor directories and retaining a check-to-write truncation race.
Evidence: After `[ -e "$f" ] || [ -L "$f" ]`, the script evaluates `[ -d "$d" ]`, which follows directory symlinks, and then performs `: > "$f"`. If `claude/agents/nested` is a pre-existing symlink to an external directory and `inner-agent.md` is absent there, both final-path checks pass and the redirection creates the file outside the repository. A file appearing between the non-atomic check and redirection can also be truncated.
Suggested direction: Reject symlinks in every existing component from the repository root to each probe, and use atomic no-clobber creation. Prefer moving physical probes into a controlled disposable worktree so cleanup cannot cross repository boundaries.
Illustrative example:
```text
claude/agents/nested -> /outside/probe-dir
/outside/probe-dir/inner-agent.md absent
: > claude/agents/nested/inner-agent.md
result: external file created, then removed by cleanup
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Test symlinked `claude/agents` and `claude/agents/nested` directories pointing to external sentinel directories. Assert the guard fails before writing and that directory contents remain byte-for-byte unchanged; also race file creation against the probe loop and confirm no existing file is truncated.

[severity:medium][technical correctness] `git diff --quiet` does not reliably prove that the working-tree `.gitignore` equals the index when the index entry is marked assume-unchanged.
Evidence: Section 0 relies exclusively on `git diff --quiet -- .gitignore`. Git documents that the assume-unchanged bit permits Git to omit checking working-tree modifications ([Git update-index documentation](https://git-scm.com/docs/git-update-index)). With a dangerous `.gitignore` staged in the index, a safe working-tree copy, and that bit set—manually or through `core.ignorestat`—the divergence check can return success. The pin and `check-ignore` then inspect the safe working copy while the next commit records the dangerous index blob.
Suggested direction: Compare the actual index blob directly with the working-tree bytes, independent of cached stat/index flags. Optionally reject assume-unchanged and skip-worktree flags on policy files as unsupported local state.
Illustrative example:
```text
index blob:       !/claude/agents/rogue.md
working-tree:     pinned safe rules
index flag:       assume-unchanged
git diff --quiet: success
commit payload:   unsafe index blob
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Reproduce the staged-dangerous/worktree-safe scenario with both assume-unchanged and skip-worktree states, including `core.ignorestat=true`. The repaired guard must reject each case by comparing index and working-tree content directly.

GPT verdict: reject — The only retained safety control still has deterministic secret-detection gaps, incomplete policy-family coverage, an external-write path, and a staged-policy bypass.

## Maintainer response
1. **Disposed by explicit user decision (recorded), plus the documentation was narrowed
   as the finding's own direction offers.** Content-aware scanning was put to the user
   in the Phase 4 interview (Q3) with the research's recommendation attached; the user
   explicitly declined a gitleaks/trufflehog-style dependency and chose "one standalone
   name/trackability guard only." The overbroad absolute claims are now fixed: the
   guard header carries an explicit SCOPE paragraph (names/trackability only, contents
   invisible, GitHub public-repo secret scanning as the content-level backstop, "never
   paste secrets into tracked files"), README §Safety says "secret-named files …
   never trackable" and warns about contents, and AGENTS.md golden rule 1 carries the
   same warning. Residual recorded below as a carried decision.
2. **Agreed (family inconsistency) — fixed, both directions.** `.gitignore` deny list
   gained `**/*.sqlite3-*`, `**/*.db3-*`, and `**/*deploy_key*`; the probe battery
   gained the wholesale-subtree cases `claude/skills/full-cycle/{api_token,
   deploy_key_prod, cache.sqlite3-wal, cache.db3-wal}` (the wholesale skill dirs are
   the only surface where name-based denies are load-bearing — everywhere else the
   structural deny-all covers first); the tracked-tree regex gained `deploy_key` and
   `_token$`. Verified: all four names force-added in disposable clones each fail the
   guard; the clean tree still passes.
3. **Agreed (symlinked ancestors, race) — fixed with one recorded residual.** Every
   probe path is now walked component-by-component and any symlink at any component
   refuses the run before a single write; creation itself is `set -C` (noclobber), so
   a file appearing between check and write is refused, not truncated. Verified:
   `claude/agents/nested` symlinked to an external sentinel dir → guard fails, the
   external dir stays empty. The remaining window — a *directory component replaced by
   a symlink* in the instant between the walk and the redirection — is a same-machine,
   single-user race; recorded as an accepted residual rather than solved with a
   separate worktree, which would re-introduce the fixture machinery this task exists
   to retire.
4. **Agreed (assume-unchanged) — fixed.** The divergence check now compares the index
   blob to worktree bytes directly (`git show :.gitignore | cmp -s - .gitignore`),
   which no stat-cache flag can influence; it also fails when `.gitignore` is
   untracked. Verified: staged rogue + restored worktree + `--assume-unchanged` →
   guard fails.
Full battery: 19 sabotage scenarios (rounds 1+2) all behave; clean run passes with
zero residue. Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Content-level secret scanning is **excluded by user decision** (interview Q3;
  reaffirmed against R2-1): the guard is a name/trackability tripwire; GitHub
  public-repo secret scanning is the content backstop; docs now state this scope.
- Accepted residual: single-user TOCTOU window between component walk and noclobber
  create (R2-3); no separate-worktree fixture machinery.
- Prohibition (not enumeration) remains the nested-ignore invariant (R1).

Consensus: disagreed
