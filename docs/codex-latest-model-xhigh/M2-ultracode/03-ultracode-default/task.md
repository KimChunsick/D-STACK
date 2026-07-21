# 03-ultracode-default

## Intent / Why
Claude Code should operate with ultracode (xhigh reasoning + standing multi-agent Workflow
orchestration) on by default, without typing `ultracode` per prompt.

Research delta (official docs, retrieved 2026-07-10):
- Ultracode is **session-scoped upstream by design**: the persisted `effortLevel` setting
  and `CLAUDE_CODE_EFFORT_LEVEL` explicitly do not accept `ultracode`
  (https://code.claude.com/docs/en/model-config — "`max` and `ultracode` are session-only
  and are not accepted here").
- A bare `"ultracode": true` in `settings.json` is **silently ignored** — open bug
  anthropics/claude-code#64817 (labels: bug, area:core; created 2026-06-02, no fix).
- The documented ways to start enabled: `/effort ultracode` (per session),
  `claude --effort ultracode` (requires ≥ v2.1.203; installed: v2.1.205), or
  `--settings '{"ultracode": true}'`.
- What the flag does, per the same doc: "run `/effort ultracode`… **`--effort` flag**:
  launch with `claude --effort ultracode`, which starts the session at `xhigh` effort
  with ultracode on"; ultracode itself "sends `xhigh` to the model and additionally has
  Claude orchestrate dynamic workflows for substantive tasks."
Durability therefore requires baking the flag into the launch. A zsh alias is the
**chosen** mechanism (simplest that covers this maintainer's actual workflow: interactive
zsh launches), not the only conceivable one — wrapper scripts/shell functions could do the
same. Known scope limits, accepted: IDE/GUI launches, other shells, subprocess spawns, and
`command claude` don't get the flag.

## Design decision (conflict surfaced, not blended)
An earlier stub of this task planned a bare `alias` line directly in `~/.zshrc`
(machine-local, untracked). Rejected in favor of a **repo-tracked fragment**:
`claude/ultracode.zsh` in the SSOT, linked by `install.sh` to `~/.claude/ultracode.zsh`,
with a one-line machine-local `source` hook in `~/.zshrc`. Reasons: (a) this repo exists
precisely so authored agent config lives in-repo, not as untracked machine state; (b) an
untracked alias cannot be regression-tested — the fragment is (see TDD below); (c) the T01
review round explicitly faulted "declare it machine-local and untracked" as a false
dichotomy where in-repo carriage is possible. Only the `source` hook stays machine-local
(zshrc is not an agent artifact and is outside install.sh's map).

## What was done (what / why)
- TDD Red: added assertions to `tests/test_claude_artifacts.sh` — `claude/ultracode.zsh`
  must exist, contain the exact executable line `alias claude='claude --effort ultracode'`,
  be `!`-allowlisted in `.gitignore`, and have an `install.sh` MAP row. Captured failing
  run: `✗ FAIL: missing or empty: claude/ultracode.zsh`.
- Green: created `claude/ultracode.zsh` (alias + header documenting the session-only
  upstream limitation, the #64817 bug, the ≥ 2.1.203 requirement, the `source` hook line,
  and the `command claude` escape hatch); added `!/claude/ultracode.zsh` to `.gitignore`;
  added `claude/ultracode.zsh|.claude/ultracode.zsh|link` to `install.sh`. Full suite:
  ALL TESTS PASSED (incl. secret-scan guard and install.sh tests).
- Review round 1 hardening: the lexical assertions gained behavioral teeth — the test now
  SOURCES the fragment under `zsh -f` and compares the *effective* alias exactly (catches
  `unalias`/override/syntax breakage), and asks git itself via `git check-ignore` whether
  the artifact is effectively tracked-eligible (catches a later overriding ignore rule).
  Fragment header rewritten: "only durable opt-in" softened to name the alternatives
  (`--settings`, wrapper function), and the alias's full blast radius stated (wraps every
  interactive invocation, shadows pre-existing aliases, inert where no session starts).
- Refactor: none needed (one artifact + three one-line registrations).

## Trust boundary (review round 1, surfaced)
Sourcing a repo-symlinked fragment at shell startup means a compromised repo checkout
becomes shell-startup code execution. Analysis: this repo ALREADY crosses that boundary —
its hooks (`fullcycle-*.sh`), `statusline-command.sh`, and skills execute inside every
Claude session via the same symlink mechanism; the shell fragment widens exposure from
agent-runtime to shell-startup, but the trust model is unchanged: single-maintainer repo,
every change lands via this reviewed pipeline, and the suite pins the fragment's exact
executable content. The lower-blast-radius alternative (copy mode, like Gemini's entries)
was considered and declined: a stale copy that silently diverges from the SSOT is this
repo's core failure mode. Accepted residual.

Also explicitly accepted (round 2): alias-expanded invocations beyond a session launch
(subcommands, print-mode, calls already carrying `--effort`) are not individually
verified — the flag is inert where no session starts, and interactive-TTY automation to
exercise each case is out of proportion; `command claude` bypasses the alias entirely.

## Files changed (where / why)
- `claude/ultracode.zsh` — the authored artifact (new)
- `tests/test_claude_artifacts.sh` — Red-first assertions pinning the exact alias line,
  gitignore allowlisting, and install.sh row; round-1 additions: behavioral `zsh -f`
  source + effective-alias equality, `git check-ignore` effective-status check, broadened
  `-m` occurrence guard (attached `-mVALUE`/`-m=VALUE` forms) and compact `-c ?model=`
  ban (the latter two touch T02's section of the shared file; raised by this task's
  review, fixed here, Red-demonstrated)
- `.gitignore` — allowlist the new artifact (deny-all layout requires naming it)
- `install.sh` — MAP row linking it to `~/.claude/ultracode.zsh`
- `~/.zshrc` — NOT changed by the agent (see Activation)
- `evidence-probe.md`, `evidence-launcher.md` (this folder) — task-produced E2E evidence
  (probe-side and launcher-side captures); named in this task's review bundle (round-2
  reviewer flagged their absence from the earlier bundle)

## Activation (maintainer-manual, by policy)
Two machine-local steps remain, and they are deliberately the maintainer's:
1. `./install.sh` — AGENTS.md: "Running against your real home is a deliberate, manual
   step." (The agent's attempt was also denied by the permission classifier as
   unauthorized persistence — the denial and the repo policy agree.)
2. Add to `~/.zshrc`: `[ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"`
Until both are done, the M2 E2E Goal-gate box must stay unchecked.

## E2E verification
Layered, everything the agent can verify without persisting machine state:
- Alias layer (transient `zsh -f`, nothing persisted): sourcing the repo fragment yields
  exactly `claude='claude --effort ultracode'` → `ALIAS-OK`.
- Version precondition: `claude --version` → 2.1.205 (≥ 2.1.203 required for the flag).
- Flag layer — **verified live** (after an earlier false start, kept for the record in
  [evidence-probe.md](evidence-probe.md)): the first probes ran with the repo as cwd, so
  the probe session inherited this Goal's own Stop gate and hung — a probe-design defect,
  not a flag defect. The probe file's prescribed fix (scratch cwd, mirroring the codex
  skills' `-C "$SCRATCH"` isolation) works: `cd "$(mktemp -d)" && command claude --effort
  ultracode -p "Answer with one word, yes or no: is ultracode mode active in this
  session?"` → `Yes`, **EXIT=0** — launcher-side capture (external observer, with the
  self-attestation limits stated) in [evidence-launcher.md](evidence-launcher.md).
  evidence-probe.md additionally records, from inside a probe session, that the launch
  flag manifests as *standing* ultracode mode in the session's system context
  ("Ultracode is on … Workflow-tool orchestration enabled"), not per-prompt keyword
  detection. Doc cross-reference: code.claude.com/docs/en/model-config
  (retrieved 2026-07-10).

## Gate status
- [x] TDD: Red→Green→Refactor complete (Red `✗ FAIL: missing or empty: claude/ultracode.zsh` captured; Green = full suite; refactor n/a)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus (rounds 1+1b in parallel, round 2
  re-examined all points: 6 withdrawn, sustained ones fixed or explicitly accepted as the
  reviewer prescribed — see codex-review.md, `Consensus: resolved`)
- [x] E2E capture verified — task-level layers: behavioral alias check (`zsh -f` sourced,
  effective alias byte-equal), version precondition 2.1.205, live flag probe from scratch
  cwd (`Yes`, EXIT=0; probe- and launcher-side captures in evidence-probe.md /
  evidence-launcher.md). The post-activation fresh-session check is the M2 Goal-gate box,
  which stays unchecked until the maintainer activates.
