# GOAL — consolidate full-cycle runtime state into `.dstack/` and stop the pipeline from blocking itself

## Goal (the one Why)

The full-cycle pipeline currently sabotages its own sessions. Its runtime state is a set of
ad-hoc dotfiles at each target repo's root (`.fullcycle-active`, a `mkdir` lock directory, and
a stranded `.fullcycle-active.tmp` left by an interrupted deregistration), and its `Stop` hook
blocks turn-end unconditionally. Because a blocked turn can never end, the harness path that
re-invokes the agent when a background command finishes is unreachable, so waiting on a 15-25
minute Codex review round degenerates into repeated one-line status turns, each re-sending the
whole conversation context, until Claude Code's 8-block override fires and the gate is bypassed
anyway.

This Goal gives the pipeline one legible, gitignored per-repo state directory (`.dstack/`) owned
by a small `dstack` CLI instead of bash snippets embedded in skill Markdown, teaches the `Stop`
hook the documented `stop_hook_active` continuation field so a wait costs one blocked turn
instead of eight, keeps long external runs' input and output out of the main context and on
disk where a failed round can be inspected, and trims the per-prompt instruction injection that
duplicates already-loaded files. The point is that the gate should protect the work without
burning the session that does it.

## Interview record (Phase 4)

Four decisions, all settled by the user on 2026-07-27. Every answer took the recommended option.

**Q1 — How far to change the `Stop` hook for long external waits?**
*Answer: `stop_hook_active` only; no wait ticket.* The hook reads the documented
`stop_hook_active` field and returns success while a continuation is already in flight, which
collapses the spin from eight blocked turns to one. The originally planned PID wait ticket was
dropped: research showed PID liveness is a weak primitive (PIDs recycle, `kill(pid,0)` checks
existence and permission rather than identity, SIGKILL strands the ticket), and the platform
field achieves most of the benefit with none of that surface.

**Q2 — How should `.dstack/` be excluded from git?**
*Answer: a self-isolating `.dstack/.gitignore` containing `*`.* This is pytest's
`.pytest_cache/.gitignore` pattern. It never touches a target repository's tracked `.gitignore`,
survives re-clone, and produces no diff in a team repo. The user's original instruction was to
append to the repo's `.gitignore`; research surfaced this strictly better option and the user
switched to it.

**Q3 — Cross-session path claim lock (a `PreToolUse` hook)?**
*Answer: leave it out of this Goal.* `PreToolUse` only observes tool calls, so a migration CLI
or generator writing files from inside `Bash` is invisible to it — it would miss the exact case
that motivated the request. Replaced by documented guidance: timestamp+slug migration filenames
and a stated rule for when a long-lived stream deserves its own worktree. Revisit only if the
collision recurs after that guidance is in place.

**Q4 — Goal scope?**
*Answer: finish the tooling inside D-STACK.* Build the CLI, hooks, skill changes, and a
`dstack migrate` command, and verify them here. Other repositories already carrying a legacy
`.fullcycle-active` (for example `portfolio-events-dashboard`) are migrated later by running
`dstack migrate` there; no file outside this repository is touched by this Goal.

**Q5 — the review budget, asked at Phase 9 on 2026-07-27 when M2 reached round 6 and M1 round 5
without converging.** *Answer: extend the budget and keep going; and from the NEXT Goal on, use
per-task review units rather than per-milestone.* Every round had produced concrete, reproducible
findings rather than reviewer nitpicking, so stopping would have shipped known defects; nothing
was downgraded to fit the budget. The second half is the structural read: this Goal reviews at
milestone granularity, so each round's bundle carries 6-11 files, and a wide surface is what keeps
letting one fix expose an adjacent defect. The fix belongs in P5-decompose, not in the review loop.

**Carried from the pre-Goal discussion (not re-asked).** Registry records move from lines in one
file to one file per record under `.dstack/active/`, which makes one document have exactly one
owning session (the old format allowed two sessions to hold lines for the same document). The
user accepted this semantic change explicitly. Review happens at **milestone** granularity, not
per task, and this repository writes no tests and runs no Red-Green-Refactor cycle (see
`AGENTS.md`).

## Research summary (Phase 3)

Artifact: `docs/dstack-state-store/research/dstack-state-store.md` (23 cited sources, all
primary except two vendor/secondary pages). Brief:
`docs/dstack-state-store/research/dstack-state-store.brief.txt`.

**Findings that changed the design.**

- `stop_hook_active` exists and is documented on the Claude Code hooks *guide* page (it is
  absent from the hooks *reference* field list, which is why an earlier check missed it), and it
  is present in the installed binary 2.1.220 alongside `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`. Claude
  Code overrides a `Stop` hook after eight consecutive blocks without progress. The planned
  hand-rolled block counter is therefore unnecessary.
- pytest writes a `.gitignore` containing `*` inside its own cache directory. This removes the
  need to edit any target repository's tracked `.gitignore`, and git's own documentation says
  local workflow artifacts do not belong in a committed ignore file anyway.
- A `PreToolUse` hook cannot see writes performed inside `Bash`, so a claim lock would not catch
  migration-CLI collisions.
- "Directory of files needs no lock" is only true across *distinct* keys. `open(O_CREAT|O_EXCL)`
  makes name creation atomic but not content writes, so each record must be published by writing
  a same-directory temp file and `rename()`-ing it. POSIX also says directory iteration is not a
  snapshot, so any scan must tolerate entries vanishing mid-read.
- Prompt-cache economics: a stable injected block that is already cached costs cache-read rate
  (0.1x base input), so trimming it saves less than raw character count suggests. The context
  window it occupies is still real.

**Strongest opposing point.** "Use the existing platform mechanisms first." Claude Code already
ships background Bash with completion notifications, a `Stop` block cap, and `stop_hook_active`;
a bespoke wait protocol risks duplicating platform behaviour. This argument won Q1 outright and
narrowed the Goal.

**Second opposing point.** For a single maintainer with a few terminal tabs, one flat file plus
one tested lock is easier to reason about than a directory-backed record store, which needs
list/open race handling and per-record atomic publication. Accepted as a real cost; the deciding
factor is that the flat file's read-modify-write is exactly what stranded the `.tmp` and the lock
directory in the first place.

**Unverified, carried as risk.** No primary source confirms the specific interaction claim that a
blocked `Stop` prevents background-task re-invocation; the docs confirm each half separately. No
benchmark exists for per-write `PreToolUse` latency (moot now that Q3 dropped it). APFS atomic
`rename` claims are limited to same-filesystem local paths and were not tested on network or
cloud-synced folders.

## Milestones & tasks (Phase 5)

Review is per milestone. Each milestone folder carries the registered review-unit document and
its `codex-review-<NNN>.md` series; the task folders under it are documentation only and are not
registered. That document is named `task.md` because it *is* this Goal's unit of review, and
because both the Stop hook's gate schema and `assemble-review.sh` bind to that name — renaming
the document was the surgical choice against modifying two reviewed tools.

### M1 — state store and the gate that reads it

T01 is deliberately first and dependency-free: until the gate hook honours `stop_hook_active`,
every turn of this very Goal pays the eight-block spin the Goal exists to remove. The hook is
symlinked into the live agent directory, so the fix applies to the session building it.

- [ ] **T01** spin-fix — make `fullcycle-gate.sh` parse `stop_hook_active` from its stdin JSON and exit success while it is true, so a gate block is delivered once per turn-end attempt instead of up to eight times. Record in the HONEST SCOPE comment that this is a deliberate, documented weakening: the gate still states the incomplete work, but it no longer forces the model to keep producing turns. deps: []; files: [claude/hooks/fullcycle-gate.sh]
- [ ] **T02** dstack-cli — create `claude/bin/dstack` (`reg`, `unreg`, `status`, `reclaim`, `migrate`) owning the `.dstack/` layout: `.dstack/.gitignore` holding `*`, one record file per registration under `.dstack/active/` named by the sha1 of its doc path with the session id and doc path as content, and `.dstack/runs/<session>/` for external-run capture. Records publish via same-directory temp + rename; no lock directory. Wire it into `install.sh`, the `.gitignore` allowlist, the secret-guard pinned list, and `AGENTS.md`. deps: []; files: [claude/bin/, install.sh, .gitignore, tests/secret-guard.sh, AGENTS.md]
- [ ] **T03** gate-state — point `fullcycle-gate.sh` at `.dstack/active/` and make it fail loud with a `dstack migrate` instruction when a non-empty legacy `.fullcycle-active` is present, rather than silently reading both. Scans tolerate entries vanishing mid-read; an unreadable record is reported, never skipped. Also narrows the secret guard's blanket nested-`.gitignore` refusal, which `.dstack/.gitignore` would otherwise trip: discovered here, when the store was first created inside this repository. deps: [T01, T02]; files: [claude/hooks/fullcycle-gate.sh, tests/secret-guard.sh]

### M2 — pipeline wiring and prompt trim

- [ ] **T04** review-io — in `codex-review`, run the round as a background command with its bundle and output under `.dstack/runs/`, stop echoing the reviewer's full stdout into the main context (read the verdict line and finding headings from the file instead), and bound the loop honestly: discovery time never changes a concrete finding's blocking status (only non-concrete items may age out), and a six-round budget escalates to the user in Korean instead of silently grinding or downgrading. Its own review rounds forced two additions to the gate's enforcement point, so the assembler is declared here rather than left as an undeclared edit. deps: [T02]; files: [claude/skills/codex-review/SKILL.md, claude/skills/codex-review/assemble-review.sh]
- [ ] **T05** skill-wiring — in `full-cycle`, replace the embedded registry bash with `dstack` calls, update the byte-frozen hook-contract block for the new registry location, document the milestone-boundary session handoff (`dstack reclaim` after a `/clear`), and record the migration-filename and worktree guidance that replaced the dropped claim lock. Also refreshes the skill's own schema test, whose pinned assertions still demanded the `.fullcycle-active` registry and its `mkdir` lock — both removed here by design, so the test was asserting a mechanism that no longer exists. deps: [T02, T03]; files: [claude/skills/full-cycle/SKILL.md, claude/skills/full-cycle/tests/skill-schema.test.sh]
- [ ] **T06** inject-slim — cut the `UserPromptSubmit` injection down to the trigger sentence, since everything else it says is already carried verbatim by the always-loaded `claude/CLAUDE.md`, and record next to the ultracode alias that a non-interactive launch never receives it. `claude/CLAUDE.md` IS edited here, mostly by REPLACING stale text rather than adding to it — it grows, and by how much is measured per round with `git show HEAD:claude/CLAUDE.md | wc -c` against `wc -c < claude/CLAUDE.md` — at Round 8 that is 8,670 -> 9,304 bytes and 163 -> 171 lines. Its §0 still described the removed `.fullcycle-active` registry and still said the gate blocks the turn from ending, and the injection cut leans on §0 being the accurate copy. Growth is held to what accuracy requires, since that file loads every session; a fixed number is not recorded because later review rounds keep editing it, and a stale figure reads as a false claim. deps: []; files: [claude/hooks/fullcycle-inject.sh, claude/ultracode.zsh, claude/CLAUDE.md]

## Goal E2E (Phase 12) — one full pass, captured

Run on 2026-07-27 against a throwaway repository, so nothing here leans on state this Goal's own
session had already built.

`./install.sh --dry-run` → `linked=0 copied=0 backed-up=0 up-to-date=18 skipped=0`; every declared
entry, including `.claude/bin/dstack`, resolves to the live agent dir.

The pass itself, in one sequence: register a `GOAL.md` and a `task.md` → `.dstack/` materialises
with `.gitignore` (exactly `*`, 2 bytes), `version`, `active/<sha1>` per document, and `runs/` →
`git add -A` then `git status --porcelain` shows **0** `.dstack` paths, so the store isolates
itself without touching the target repository's tracked ignore file → `dstack run-dir probe-round`
creates the capture directory and `status` lists it under stored captures → the Stop hook blocks
naming the unchecked document, and the same stdin with `"stop_hook_active": true` exits 0 silently
→ `dstack rm-run probe-round` removes the capture through the CLI (not a check-then-delete race in
a calling shell) → both documents deregister and `status` reports `(none)`.

That is the Goal's own claim end to end: one legible gitignored state directory, owned by one
CLI, with a gate that states incomplete work once per turn instead of forcing eight.

**The ambient half is this session.** Twenty detached Codex rounds ran across M1 and M2 with
their bundles and outputs under `.dstack/runs/`, none of them echoed into the conversation; the
trimmed injection ran on every prompt. The evidence for both is recorded in the milestone
documents rather than repeated here.

**Not covered, stated plainly.** `shellcheck` is not installed on this machine and was not run.
No probe exercises two concurrent sessions racing for one lock — the reasoning is argued in code
comments, not demonstrated. Worker fan-out is not exercised anywhere in this Goal; it is carried
as a follow-up in `M2-pipeline-wiring/findings.md`.

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)

- [x] M1 E2E: `dstack` CLI and the reworked gate hook verified together against a real registration, a `stop_hook_active` continuation, and a legacy-file migration
- [x] M2 E2E: full-cycle and codex-review skills driven end to end against the new state store, with the trimmed injection active
- [x] GOAL E2E: one full pass of the whole Goal, captured
