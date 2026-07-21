# Codex review — 02-routing-and-ssot-wiring

## Round 1 — GPT-5.6 Sol verdict (xhigh)

> [severity:medium][the real Why] The one-line typo/copy/constant exception contradicts the stated "all frontend work" and "only this subagent" boundary. A one-line constant change can alter authorization, feature flags, limits, or routing; line count is not a safe proxy for triviality.
>
> [severity:medium][technical correctness] The routing definition is React/TypeScript-centric and leaves Vue/Svelte components, JavaScript, HTML templates, frontend tests, stories, build configuration, and generated assets ambiguous. …particularly because "TDD orchestration" could be interpreted as permitting the main agent to write frontend tests.
>
> [severity:medium][technical correctness] The routing regression guard is ineffective: independent searches for `frontend-dev` and `MUST be delegated` need not describe the same rule. …
>
> [severity:medium][security] The tests do not prove that `claude/agents/` remains an exact allowlist. A later `!/claude/agents/*.md` rule would expose arbitrary agent definitions while both new probes—an extensionless filename and `auth.json`—remain ignored and every shown assertion passes. …
>
> [severity:low][UI/UX & DX] Automated verification stops at a symlink and lexical assertions; it never verifies that Claude discovers `frontend-dev` or successfully routes a task to it. …
>
> GPT verdict: reject — the routing policy contains a substantive bypass and its routing and exact-allowlist guards permit straightforward regressions.

## Round 1 — Claude responses

1. **One-line exception — DISAGREE (owner calibration).** The exception is authored by the
   owner in the subagent's own description and mirrored in the routing rule. It is a DX
   escape valve for mechanical edits; the main loop's global CLAUDE.md (security rules
   included) still governs that one line, and anything semantically risky is not "a typo/copy
   /constant fix" in intent. Removing it makes the rule unlivable, which erodes compliance
   more than the exception erodes coverage. Accepted tradeoff, recorded.
2. **React/TS-centric scope — AGREE, fixed.** The rule now reads: components (React or any
   other framework), hooks, styles/templates/markup, frontend utilities, frontend test and
   story files, and frontend build configuration. The full-cycle bullet now says the
   delegated implementation steps *include writing frontend test code* (pipeline keeps docs
   /review/E2E orchestration only).
3. **Two independent greps — AGREE, fixed.** Replaced with one co-located phrase assertion:
   `MUST be delegated to the .frontend-dev. subagent` (single regex spanning the rule).
4. **Exact-allowlist probes — AGREE, fixed.** Added `claude/agents/unknown-agent.md` and
   nested `claude/agents/nested/inner-agent.md` to the secret-guard battery — a later
   `!/claude/agents/*.md` (or `**`) rule now trips the guard, not just extensionless names.
5. **No automated discovery/routing check — DISAGREE on placement, evidence attached.** A
   live `claude -p` delegation run inside the unit suite would make `tests/run.sh` slow,
   network- and auth-dependent, and flaky; the suite is deliberately hermetic (sandbox HOME).
   Discovery + routing ARE verified live, with the captured stream attached as
   `../01-agent-definition/e2e-delegation-stream.jsonl` (9 `"subagent_type":"frontend-dev"`
   tool-use events; subagent-authored diff). Gate boxes are ticked at Phase 10 per pipeline
   order — the doc no longer merely narrates the observation, it links the artifact.

## Round 2 — GPT-5.6 Sol verdict

> [severity:medium][technical correctness] Point 2 is only partially fixed. "In any frontend project" still leaves frontend work inside full-stack or mixed monorepos ambiguous, and the agreed generated-assets case remains omitted. …
>
> [severity:medium][security] Point 4 still does not prove an exact allowlist. …a later rule such as `!/claude/agents/a*.md` would expose arbitrary agent files while every probe remains ignored. The test must structurally reject every agents-path negation except the directory and exact `frontend-dev.md` entries.
>
> GPT verdict: reject — two agreed fixes remain incomplete on core routing coverage and public-safe allowlist enforcement.

## Round 2 — Claude responses (both accepted and fixed)

1. **Scope ambiguity — FIXED.** The rule no longer says "in any frontend project"; it now
   states it keys on the *nature of the code, not the repo shape* — frontend-only repos,
   full-stack apps, and mixed monorepos alike — and adds the generated-artifacts case
   (never hand-edited by any loop; regenerate via the pipeline, delegated when that means
   changing frontend source).
2. **Structural allowlist exactness — FIXED.** `tests/test_claude_artifacts.sh` now rejects
   ANY `^!/claude/agents/` negation in `.gitignore` other than exactly `!/claude/agents/`
   and `!/claude/agents/frontend-dev.md` — `!/claude/agents/a*.md` and friends now fail
   structurally, independent of the ignored-name probes. (Negations rooted elsewhere, e.g.
   `!/claude/**`, remain covered by the effective `git check-ignore` probes and the
   secret-guard battery — a lexical test cannot enumerate every spelling, so the structural
   line-set check + effective-status probes together are the guard.)

All round-2 issues are fixed and re-verified (`bash tests/run.sh` → ALL TESTS PASSED).

## Round 3 — GPT-5.6 Sol verdict

> [severity:medium][security] The structural guard only examines negations beginning exactly `!/claude/agents/`. A valid later rule such as `!claude/agents/f*.md` re-includes arbitrary matching agent definitions while evading that grep and leaving every probe ignored. The exact-allowlist fix therefore remains bypassable by an equivalent unrooted Git pattern.
>
> GPT verdict: reject — the agents allowlist guard still does not structurally reject all non-approved negations.

## Round 3 — Claude response (accepted and fixed, two layers)

1. **Lexical layer broadened:** the guard now flags ANY `.gitignore` negation line containing
   `agents` (not only `^!/claude/agents/`-rooted ones) unless it is exactly one of the two
   permitted lines — `!claude/agents/f*.md` now fails lexically.
2. **Behavioral layer added (closes the whole spelling space):** in the secret-guard test,
   with the probe battery on disk (now also `frontend-xyz.md`, `f.md`),
   `git ls-files -o --exclude-standard claude/agents/` must list NOTHING except the pinned
   `frontend-dev.md`. This asks git itself for the effective addable set, so any re-include
   spelling — rooted, unrooted, glob, `**` — that exposes any probe fails regardless of how
   it evades a grep.

Re-verified: `bash tests/run.sh` → ALL TESTS PASSED.

## Round 4 — GPT-5.6 Sol verdict

> [severity:medium][security] The exact-allowlist guard remains bypassable. A later valid rule such as `!/claude/**/z*.md` contains no literal `agents`, matches none of the fixed probes, and therefore passes both new checks while re-including `claude/agents/z-agent.md`. The behavioral check covers only files present in its finite probe battery; it does not "close the whole spelling space" or structurally reject all semantically applicable negations.
>
> GPT verdict: reject — the accepted exact-allowlist requirement is still not enforced against equivalent negations that avoid both the lexical substring and fixed probes.

## Round 4 — Claude response (accepted; class-closing fix)

Correct — finite probes cannot close an infinite pattern space, so the guard now pins the
**closed set of negation lines itself**: `tests/test_gitignore_secret_guard.sh` compares
`grep '^!' .gitignore` against the full expected list (exact content and order). ANY new,
changed, or reordered `!` rule — any spelling, any root, any glob, `!/claude/**/z*.md`
included — fails until it is consciously added to the pinned list alongside review. This is
the repo's "true allowlist" philosophy applied to the allowlist itself; the earlier lexical
`agents`-negation guard and the `ls-files -o` behavioral probe remain as defense in depth.
Residual (out of threat model): editing the test's own pinned list — the test is the guard.

Re-verified: `bash tests/run.sh` → ALL TESTS PASSED.

## Round 5 — GPT-5.6 Sol verdict

> GPT verdict: approve — the guard now compares every root `.gitignore` negation against an exact ordered list, so additions, removals, changes, duplicates, or reordered negations fail.

Both sides agree: rebuttals on the one-line exception (owner calibration) and hermetic-suite
placement of live E2E stand; all accepted findings across rounds 1–4 are fixed and verified.

Consensus: agreed
