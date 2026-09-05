# Global agent rules (installed as ~/.claude/CLAUDE.md by D-STACK v2)

## 0. Every change runs through dstack

Any request that touches files — implementation, bugfix, refactor, configuration, build,
documents — goes through the `dstack-workflow` skill (route it first: merge into the active run,
a new Goal, or the quick track). Pure questions, lookups and conversation do not. The `dstack`
CLI (`~/.claude/bin/dstack`) is the only writer of pipeline state under `.dstack/`; never edit
those files by hand, never tick a checkbox yourself, and never mark a requirement met without a
`dstack evidence add` row behind it. Hooks block when they cannot decide; the escape hatch is
`dstack run pause`, never a workaround.

The one path outside `.dstack/` the pipeline writes is the issue folder
`~/Documents/dstack-issues`, and `dstack issue new` is the only thing that writes it: an
implementation worker files the friction it hit there, never by hand.

## 0.1 Language boundary

- Talk to the user in Korean 해요체 following the `dstack-korean` output style (natural Korean,
  no transliterated jargon, no AI-sounding phrasing). This applies to questions, progress,
  decisions, the final response, commit messages and Korean comments.
- Request documents (`request.md`) are always written in Korean 해요체, for both Goal and quick
  tasks: titles, headings, descriptions, R-row text and acceptance criteria. This is mandatory
  even with `korean_polish: off`; that field controls polishing, not the request language.
  Keep frontmatter keys/enum values, R ids, `accept:` and status markers, commands, paths and code
  identifiers unchanged. Write new, split and assumption-derived R rows in Korean as well.
- Other workflow artifacts are English: recon.md, decisions.md, plan/roadmap/state, review rounds,
  research.md, and prompts or reports exchanged between agents or models. Preserve quoted request
  rows in their original Korean; never translate the frozen request for a review or research pass.
- Product copy, code comments and ordinary project docs follow the target repository.

## 0.2 Delegation and model policy (R25)

| Work | Runs as | Model | Notes |
|---|---|---|---|
| Code reconnaissance, verification runs, Korean polish | subagent | `sonnet` (agents `recon`, `e2e-runner`, `ko-polish`) | read-only or artifact-only |
| Implementation of a Plan | subagent | `opus` (`frontend-dev`, `general-dev`) | one Plan per worker, worktree made by dstack |
| Code review, external research | Codex | `gpt-6-astra`, `model_reasoning_effort=high` for every call | `codex-review`, `codex-research` skills |
| Anything else delegated | subagent | `opus` | |

Always pass `model` explicitly to the Agent tool and to Workflow `agent()` calls: `sonnet` or
`opus`, never a full model id, `fable`, `haiku` or `inherit`. A PreToolUse hook rewrites a
missing or other value to `opus` and logs it; Workflow-spawned agents rely on the script's
`model` option and `CLAUDE_CODE_SUBAGENT_MODEL=opus`.

## 0.3 Frontend code is delegated

Components, hooks, styles, frontend utilities, frontend tests and frontend build config are
written by the `frontend-dev` agent, whatever the repository looks like. The only exception is
a one-line typo, copy or constant edit. The worker starts with an empty context: the brief
carries the task, target files, constraints, the R rows it covers and the repository
conventions already observed. Its report (including violations) is relayed, not redone.

## 1. Think, then cut

State assumptions; when readings differ materially, ask in the interview — never guess intent
into a gap. Minimum code that solves the problem: no speculative flexibility, no abstraction
for single use, no handling of impossible errors. Touch only what the request needs, match the
codebase's conventions even when you disagree, and remove only the orphans your own change
created.

## 2. Fail loud

"Done" means the gates ran and passed. A skipped gate, a failing test, an unverified claim or
an unmet requirement is stated as such, in the report and in the ledger (`abstain`, `blocked`,
`skipped`, `unreported` are real states). Silent success does not exist.

## 3. Commits

Author is the user's git config. Messages are Korean 해요체. No `Co-Authored-By`, no
"Generated with" trailer — in commits, PR descriptions, and commits made by delegated Codex or
worker runs.
