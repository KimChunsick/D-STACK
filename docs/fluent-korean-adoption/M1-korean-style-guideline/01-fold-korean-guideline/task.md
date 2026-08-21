# 01-fold-korean-guideline

## Intent / Why
Fold an own-authored Korean-fluency guideline into the two standing agent configs
(`claude/CLAUDE.md`, `codex/AGENTS.md`) so that every Claude Code session and every Codex
invocation writes clear, natural Korean — complete predicates, restored particles and
endings, no translationese — in 해요체, including Korean code comments. Content is adapted
from snflkd/fluent-korean (MIT) by explicit user direction; the third-party file itself is
NOT installed and never enters this public repo.

## Deployment context
Personal agent configuration on the maintainer's machines. `claude/CLAUDE.md` is symlinked
to `~/.claude/CLAUDE.md` (loads into every Claude Code session, all projects);
`codex/AGENTS.md` is symlinked to `~/.codex/AGENTS.md` (loads into every Codex invocation,
including this pipeline's English research/review rounds — the guideline therefore scopes
itself to Korean-writing situations only). Public repo: no secrets, no third-party
artifacts. Out of scope by construction: the Claude web/desktop app, the ChatGPT app,
output-style files, and any change to which language a target repo's comments use.

## Design consult
Skipped — no trigger (config-document edit: no architecture, API, persistence, or
rendering boundary involved).

## What was done (what / why)
Rewrote the '## 한국어 작성 규칙' section of `claude/CLAUDE.md`, folding the substance of
fluent-korean into the user's existing rules instead of adding a second overlapping rule
set (the research's instruction-conflict finding drove the merge-not-append decision).
Every pre-existing rule was kept verbatim (Slack-tone comments, no transliteration jargon,
translationese replacement table, no em dash, no header/bullet spam). New rules folded in,
rewritten in our own words with the original's examples where they clarify:
- 해요체 for user-facing Korean and Korean code comments (user's register choice; the
  original uses 합니다체), with target-repo conventions still winning for product copy.
- No omission of meaningful sentence constituents; unroll chained '~의'.
- Sentences end with a predicate and closing ending (headers/list items exempt); keep
  particles and endings, attach them to concrete vocabulary to make relations explicit.
- No figurative substitutes for ordinary nouns/verbs; established idioms stay.
- Do not mirror the user's tone; the rules hold regardless.
- Proper nouns / technical terms: established translation or transliteration first,
  original form otherwise (merged into the existing transliteration rule).
- Scope guard kept from the original: the rules govern HOW Korean is written, never
  translate foreign text INTO Korean.
Two deliberate deviations from the original, both by explicit user direction: the rules
APPLY to code comments (the original excludes them), and the register is 해요체. A
one-line provenance credit (snflkd/fluent-korean, MIT) is included; the third-party file
itself is not installed and not committed.

Added a '## Korean output style' section to `codex/AGENTS.md` between 'Language boundary'
and 'Operational constraints': a short English lead-in (scope: how Korean is written,
never which language to use) plus the same Korean rule block, kept byte-identical with
the CLAUDE.md section — the lead-in instructs editing both together (Codex has no import
mechanism, so duplication with a stated lockstep rule was chosen over new plumbing).

## Files changed (where / why)
- `claude/CLAUDE.md` — merged the fluent-korean adaptation into the existing 한국어 작성
  규칙 section; loads into every Claude Code session via the `~/.claude/CLAUDE.md` symlink.
- `codex/AGENTS.md` — added the Korean output style section; loads into every Codex
  invocation via the `~/.codex/AGENTS.md` symlink. English rounds are unaffected by
  construction (the rules bind only Korean-writing situations).

## Direct verification (repo policy: no TDD)
Recorded from actual runs (2026-08-21):
- `bash tests/secret-guard.sh` → `✓ PASS: secret guard`
- `./install.sh --dry-run` → `= up to date: .claude/CLAUDE.md`, `= up to date: .codex/AGENTS.md`
  (idempotent; no new link entries needed — both files were already mapped)
- Korean rule block extracted from both files and diffed → `IDENTICAL` (byte-identical)
- `grep -c 해요체 ~/.claude/CLAUDE.md ~/.codex/AGENTS.md` → `1` / `1` (symlinks resolve to
  the edited content)

## Pre-review defect-class self-sweep (codex-review Step 0)
Classes checked against this repo's own defect history, across both changed files:
- Third-party provenance in a public repo: the folded text is a rewrite with a one-line
  MIT credit; a few short example phrases are carried from the original where they clarify
  a rule. The maintainer explicitly directed this fold ("저 레포를 참고해서 그냥 우리
  저장소에 녹여줘"), which is the provenance go-ahead the review precondition requires;
  the full source file was read verbatim beforehand (inert style guidance, no directives).
- Duplicated-text drift: the Korean block was extracted from both files and diffed —
  byte-identical; the AGENTS.md lead-in states the lockstep rule ("edit both together").
- Instruction conflict with standing rules: 해요체 is consistent with the existing
  Slack-tone comment rule and the Korean-to-user boundary; the scope guard (rules govern
  HOW Korean is written, never which language) prevents collision with the English-only
  pipeline rounds; the original's comment-exclusion was deliberately inverted by user
  direction and is recorded in GOAL.md.
- Hook-parsed surfaces: neither changed file is a hook-parsed work doc; gate headings and
  the registry were untouched.
- Secret trackability: `tests/secret-guard.sh` green; no new tracked paths (both files
  were already allowlisted in .gitignore and mapped in install.sh).

## E2E verification
Post-merge (commit 06faeaa), 2026-08-21. Both agents were started FRESH so the changed
config loads at session start, from a scratch cwd outside this repo:
- Claude Code headless (`claude -p "레이스 컨디션이 뭔지 세 문장 정도로 설명해줘"`) —
  full output captured in [e2e-claude.txt](e2e-claude.txt). The reply is complete-sentence
  해요체 with particles intact ("…문제예요", "…식이에요", "…막아요"); no telegraphic
  endings, no em dash.
- Codex CLI (`codex exec … -m gpt-5.5 --ephemeral "데드락이 뭔지 세 문장 정도로
  설명해줘"`, loads `~/.codex/AGENTS.md` globally) — full output captured in
  [e2e-codex.txt](e2e-codex.txt). Same register: "…상태예요", "…멈춰요", "…예방해요".
This also closes review follow-up F4 (live-generation verification).

## Gate status
- [x] Verification: behavior confirmed by direct run (repo policy: no TDD)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
- [x] E2E capture verified
