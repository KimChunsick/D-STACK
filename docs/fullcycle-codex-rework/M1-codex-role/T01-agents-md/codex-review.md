# Codex adversarial review — T01

> Dogfooding the M3/G2 convention: Codex's review lives in its own file in the task
> folder; Claude's rebuttals are appended here; consensus is tracked at the bottom.

## Round 1 — GPT verdict: **reject**

Codex (GPT-5.5, xhigh) raised these (paraphrased):

1. `[high][tech]` Submitted diff omits the deliverable `codex/AGENTS.md`.
2. `[high][Why]` install.sh only symlinks; nothing proves Codex actually *loads* `~/.codex/AGENTS.md`.
3. `[high][Why]` `codex/instructions.md` is still Next.js-specific → live Codex not stack-neutral; contradicts AGENTS.md.
4. `[med][sec]` AGENTS.md doesn't encode read-only + never-pipe-secrets; security left to operator memory.
5. `[med][tech]` Test greps loose substrings; a hollow file could pass.
6. `[med][tech]` git-trackable check proves only "not ignored in working tree," not staged/committed/reproducible.
7. `[med][research]` T01 doesn't capture exact command/model/sandbox of the web.run probe → future drift.
8. `[med][UX/DX]` "Use the live web tool / prefer many sources" globally → slow/noisy for local-only reviews.
9. `[med][structure]` Hardcoding "Codex, GPT-5.5" → drift; file can't enforce model/effort.
10. `[low][UX/DX]` Task doc claims tests passed, but they failed in Codex's sandbox.
11. `[low][tech]` Installer integration test doesn't assert AGENTS.md is linked.
12. `[low][Why]` Gate boxes unchecked despite claiming complete.

## Claude responses

**Accepted & fixed:**
- **#2 (the load-path linchpin) — verified empirically, not assumed.** Wrote a sentinel
  instruction into `~/.codex/AGENTS.md` and ran `codex exec`; it obeyed, replying
  `[ROLE-LOADED-8821] Hello.` → **Codex does load `$CODEX_HOME/AGENTS.md`.** The mechanism
  is real. (Evidence below.)
- **#4** — added an `## Operational constraints` section to AGENTS.md: read-only by default,
  never read/transmit secret files, web data untrusted.
- **#5** — strengthened `tests/test_codex_artifacts.sh` to assert load-bearing markers
  (`against`, `untrusted`, `read-only`, `secret`, verdict contract), not loose words.
- **#8** — scoped web use to *research mode*; review mode works from provided material;
  added a proportionality rule ("do not web-search the trivially known or purely local").
- **#9** — softened the identity line: the model is config-derived (`~/.codex/config.toml`
  is the source of truth), reducing false self-identification / drift.
- **#11** — `tests/test_install_sh.sh` now asserts `~/.codex/AGENTS.md` links to the repo.
- **#3** — real, but resolved by **T02** (same milestone M1), which trims `instructions.md`
  to stack-neutral. M1 is not "done" until T02; T01's gate is for T01's scope only.
- **#1 / process** — correct: `git diff` omits *untracked* new files, so the review input
  lacked `codex/AGENTS.md`'s text (Codex still read it from disk and cited its lines). The
  fix — assemble review material to include untracked files — belongs to **T05** (codex-review
  skill) and is logged there. Round 2 below includes the file content explicitly.

**Rebutted (disagree, with evidence):**
- **#10** — the failure was Codex's *own* `sandbox: read-only`: `test_install_sh.sh` and the
  gitignore probes legitimately write temp dirs, which a read-only sandbox forbids. In the
  real environment `bash tests/run.sh` → `ALL TESTS PASSED` (run twice). The tests are not
  defective; they require a writable tmp, which is normal.
- **#12** — unchecked gates are the *correct* mid-flight state. They are ticked only after
  this review reaches consensus **and** E2E is captured — which is exactly the mechanical-gate
  discipline GOAL.md prescribes. Not a defect.
- **#6 (partial)** — the trackability check is correctly scoped for a deny-all repo's
  working-tree guard; "staged/committed/reproducible-from-patch" is really #1 (review
  material), now addressed. Proving "committed" in a dev-time guard would be wrong.
- **#7 (partial)** — `research/codex-capability.md` already records the exact command and
  the tool list; added note: the premise is CLI-version-dependent and should be re-probed if
  `codex` major-version changes.

## Evidence
- Load-path proof: `~/.codex/AGENTS.md` sentinel → `codex exec` replied `[ROLE-LOADED-8821] Hello.`
- `bash tests/run.sh` → `ALL TESTS PASSED` (after fixes).
- `./install.sh --dry-run` → backs up empty `~/.codex/AGENTS.md`, then `+ linked: .codex/AGENTS.md → codex/AGENTS.md`.

## Round 2 — GPT verdict: **approve-with-fixes**
Codex re-assessed all 12 points: **11 RESOLVED, 1 STILL-OPEN (#3)** — and #3 is explicitly
"T02 may fix it," i.e. the very next task in M1. Codex conceded its rebutted points (#6
trackability scope, #10 sandbox-failure, #12 mid-flight gates) rather than moving goalposts.
No new high/medium issues. Rationale: "T01's own identity file and wiring are acceptable;
M1 cannot be complete until T02 removes the contradictory global Next.js instructions."

## Consensus
- Round 1: reject → fixes applied + rebuttals recorded.
- Round 2: approve-with-fixes; 11/12 resolved; #3 is owned by **T02** (same milestone).
- **Consensus: resolved** for T01's scope. (M1 closes when T02 lands; M1 E2E will re-verify.)
