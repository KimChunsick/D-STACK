# Codex adversarial review — T03 (codex-research skill)

## Round 1 — GPT verdict: **reject**
1. `[high][sec]` Brief pasted inside a double-quoted shell arg → shell injection / metachar breakage.
2. `[high][sec]` No `--ephemeral` → briefs/outputs persist in Codex session history.
3. `[high][UX]` Fallback references a `deep-research` skill not present in this repo → may dead-end.
4. `[med][tech]` No `-o/--output-last-message` → artifact capture is manual, not reproducible.
5. `[med][sec]` No `-C` isolation → runs in repo context, can read beyond the brief.
6. `[med][tech]` "xhigh automatic" not pinned → silent drift if config.toml changes.
7. `[med][Why]` full-cycle Phase 3 still conditional/`deep-research` until T04 wires this in.
8. `[med][UX]` Fallback triggers undefined (no retry/exit-code/min-source/malformed rules).
9. `[med][structure]` Test only checks keyword presence; would pass nonfunctional prose.
10. `[low][Why]` URL+date ambiguity (publication vs retrieval vs none).
11. `[low][structure]` T03 diff shows codex/AGENTS.md lines (T01 scope).

## Claude responses

**Accepted & fixed (the invocation was genuinely unsafe — good catches):**
- **#1** — brief now written to a **file** and piped via **stdin**; only static instructions
  are in the prompt arg. No brief content can break quoting or expand in the shell.
- **#2** — added `--ephemeral`.
- **#3** — fallback no longer hard-depends on a repo-local skill: "use the host's
  `deep-research` skill **if present, otherwise your own web search/fetch**." Self-contained.
- **#4** — added `-o "$.../$TOPIC.md"` (verified `-o/--output-last-message` exists in
  `codex exec --help`) for reproducible capture.
- **#5** — added `-C "$(mktemp -d)"` scratch root so pure web research reads no repo files.
- **#6** — pinned `-m gpt-5.5 -c model_reasoning_effort="xhigh"` in the command.
- **#8** — concrete fallback triggers: non-zero exit after one retry, OR empty/missing-sections/
  zero-sources output.
- **#9** — strengthened the test to assert the load-bearing flags (stdin, ephemeral, read-only,
  `-o`, pinned effort, concrete fallback trigger), not just keywords.
- **#10** — prompt now asks for URL + publication date (or 'no date') + retrieval date.

**Scoped / rebutted:**
- **#7** — correct and **owned by T04** (next task, same milestone M2): this skill is inert
  until Phase 3 is rewritten to call it. T03's scope is the skill itself.
- **#11** — not real scope bleed: the working tree has uncommitted M1 changes, so `git diff`
  shows T01's `.gitignore` lines as context. T03's own `.gitignore` edit is only the
  `codex-research` allow line. (Per-task commits would isolate this; the repo isn't being
  committed mid-work.)

## Round 2 — GPT verdict: **reject** (10/11 resolved; 1 real bug found)
All Round-1 points RESOLVED except **#9**: the documented command block had **inline comments
after the `\` line-continuation** (`--ephemeral \   # …`), which breaks the command if copied
verbatim. My earlier E2E passed only because I ran a single-line variant — the *documented*
block was not runnable. Codex was right; my keyword test could not catch it.

### Fix + proof
- Removed all inline comments from the continued command; moved them to a bullet list below.
- Added a test guard forbidding the `\␠#` pattern (`tests/test_claude_artifacts.sh`).
- **Extracted the exact bash block from SKILL.md and executed it verbatim** (placeholders
  substituted) → exit 0, produced all 6 sections. Functional invocation proven, not asserted.

## Round 3 — GPT verdict: **approve**
"#9 is RESOLVED. The current block has no continuation-breaking inline comments, the guard
forbids the `\␠#` failure mode, and the verbatim extraction/execution proof closes the
keyword-only objection. I see no remaining blocker."

## Evidence (raw codex transcripts removed — they embed machine paths; public repo)
- Two real `codex exec` research runs produced all 6 sections with cited, dated sources
  (e.g. GNU Stow manual 2024-09-08, Microsoft symlink docs, yadm 2025-03-18). The "against"
  evidence (symlink+repo becomes inadequate for machine-specific variation; Windows symlink
  caveats) is real and mirrors why this repo uses copy-mode for Gemini.
- Verbatim-block extraction run → exit 0, all 6 sections. `bash tests/run.sh` → ALL PASSED.

## Consensus
- Round 1: reject → fixes + rebuttals. Round 2: reject (found the copy-paste bug) → fixed +
  proven. Round 3: **approve**.
- **Consensus: agreed.**
