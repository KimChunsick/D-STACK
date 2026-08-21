# GOAL — Fold a fluent-Korean output guideline into the standing agent configs

## Goal (the one Why)
The user wants their AI assistants to produce clear, natural Korean — complete predicates,
restored particles, no translationese — across the surfaces they actually use (Claude Code
and Codex CLI). After an interview pivot, the instrument is NOT installing the third-party
`fluent-korean-not-coding` file: it is authoring our own guideline, adapted from
github.com/snflkd/fluent-korean (MIT), folded into the SSOT configs this repo already
manages — with two deliberate differences: the rules also apply to code comments, and the
register is 해요체 instead of the original's 합니다체.

## Interview record (Phase 4)
- Q: Which "Claude" surface? → A: **Claude Code CLI** (not the Claude web/desktop app).
- Q: Where on the "ChatGPT" side? → A: the user answered "AGENTS.md?" — i.e. **Codex CLI**;
  confirmed in a follow-up. The ChatGPT app is out of scope.
- Q: ChatGPT plan tier (custom-instruction length limits)? → A: paid — mooted by the pivot.
- Q: Claude Code — default `outputStyle` or install-only? → A: install-only — also mooted.
- **PIVOT** (user interruption at the repo-policy question, decisive): instead of
  installing the third-party file anywhere, author OUR OWN guideline into the SSOT repo:
  "그러면 저 레포를 참고해서 그냥 우리 저장소에 녹여줘. 대체적인 내용은 따라가게 하되,
  주석에도 사용되게끔 해주고 대신 ~해요 체로 해주게끔 해줘."
  - Follow the original's general content.
  - Widen scope: the rules apply to code comments too (the original explicitly excludes
    quotes, code, and code comments).
  - Register: 해요체 (polite informal) instead of the original's 합니다체.
- Recorded assumptions (careful-colleague readings, stated for review rather than asked):
  - 해요체 governs user-facing Korean AND Korean code comments the agents write; which
    LANGUAGE comments are written in still follows the target repository's conventions
    (standing language-boundary rule, unchanged).
  - Integration points: the '한국어 작성 규칙' section of `claude/CLAUDE.md` (global,
    loads into every Claude Code session on every machine via symlink) and a matching
    section in `codex/AGENTS.md` (loads into every Codex invocation). No output-style file
    is installed; the plugin is not used.
  - The guideline is rewritten in our own words, merged with the existing 한국어 작성 규칙
    rather than appended beside it, with a one-line provenance credit (snflkd/fluent-korean,
    MIT). The text is duplicated across the two files because Codex has no import
    mechanism; drift risk is accepted and noted here.

## Research summary (Phase 3)
Artifact: [research/output-style-application.md](research/output-style-application.md) — 27 unique sources.

Key findings:
- Claude Code custom output styles live in `~/.claude/output-styles/` (user) or
  `.claude/output-styles/` (project). Frontmatter keys: `name`, `description`,
  `keep-coding-instructions` (default `false`). Activation is via `/config` or the
  `outputStyle` settings key — the `/output-style` command was deprecated in v2.1.73 and
  removed in v2.1.91. A style takes effect only at session start (`/clear` or new session),
  and does NOT propagate to non-fork subagents.
- A style without `keep-coding-instructions: true` removes Claude Code's built-in
  software-engineering system-prompt section (change-scoping, verification guidance).
  `CLAUDE.md` content is separate, stays loaded regardless of output style, and the docs
  name it the better place for standing personal/project conventions.
- ChatGPT: custom instructions are account-wide — 1,500-char limit on Free/Go, 5,000 on
  paid tiers. Project instructions override global ones inside a project.
- The not-coding body is 2,396 characters without frontmatter (5,932 bytes UTF-8). The
  document itself warns against summarizing it.

Strongest against-the-goal point: instruction conflict and added token load are documented
failure modes (IHEval: sharp performance declines under conflicting instructions; Claude
Code docs warn conflicting instructions are followed arbitrarily). The user's existing
CLAUDE.md Korean rules overlap in intent, so a second overlapping rule set is the risk to
avoid.

Delta after the P4 pivot: the goal moved from "install the third-party artifact" to
"author our own adaptation inside the standing configs". The artifact's findings still
govern — S1 itself recommends CLAUDE.md-style standing context over output styles for
persistent conventions, and the conflict findings now argue for MERGING with the existing
한국어 작성 규칙 section rather than adding a second overlapping rule set. No research
re-run needed; no new questions opened.

Unverified: any controlled measurement that fluent-korean-style guidance improves output
quality (evidence is rationale + adoption, not a benchmark).

Security review: the full not-coding file was read verbatim before any use — pure Korean
writing guidance, no tool directives, no injection surface, no secrets. Since we now author
our own text, no third-party artifact enters the repo; a provenance credit line is kept.

## Milestones & tasks (Phase 5)

Review granularity: per task

### M1 — korean-style-guideline
- [x] **T01** fold-korean-guideline — author an own-written Korean-fluency guideline
  (content adapted from snflkd/fluent-korean; register switched to 해요체; scope widened to
  code comments) and fold it into the two standing agent configs so every Claude Code and
  Codex session loads it. deps: []; files: [claude/CLAUDE.md, codex/AGENTS.md]
- [x] **T02** third-party-notices — add a tracked THIRD-PARTY-NOTICES.md carrying the
  upstream MIT license text in full (review finding F2: a linked notice is not an included
  one), updating the .gitignore allowlist line and the secret-guard pinned negation list
  in the same change per repo golden rule. deps: []; files: [THIRD-PARTY-NOTICES.md,
  .gitignore, tests/secret-guard.sh]

## E2E evidence

M1 integration (2026-08-21, commit 06faeaa): the two units' work holds together in the
landed tree — `git show HEAD:claude/CLAUDE.md | grep -c THIRD-PARTY-NOTICES.md` → 1 and
the same for `codex/AGENTS.md` (T01's credit lines reference T02's file), with
`git ls-files THIRD-PARTY-NOTICES.md` confirming the referenced file is tracked. The two
fresh-session generation probes recorded in T01's task.md ran against this integrated
state (both agents produced complete-sentence 해요체 Korean).

Goal E2E (2026-08-21): one full pass across the Goal — `~/.claude/CLAUDE.md` and
`~/.codex/AGENTS.md` resolve by symlink to the repo files and each contains the guideline
(`grep -c "해요체를 쓴다"` → 1 in both live paths); `tests/secret-guard.sh` → PASS; and a
fresh headless Claude Code session plus a fresh Codex invocation each produced
complete-sentence 해요체 Korean (captures: `M1-korean-style-guideline/
01-fold-korean-guideline/e2e-claude.txt`, `…/e2e-codex.txt`).

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: the folded guideline demonstrably governs live output of both agents
- [x] GOAL E2E: one full end-to-end pass — both configs load the guideline through their
  symlinks and a fresh session of each agent produces 해요체, complete-sentence Korean
