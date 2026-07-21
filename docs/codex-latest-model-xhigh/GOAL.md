# GOAL — Agent calls default to full strength (Codex xhigh; review on gpt-5.6-sol; Claude ultracode)

## Goal (the one Why)
The maintainer's agents must run at their **user-directed strongest settings** by default
(xhigh — sol also exposes max/ultra; explicitly declined in the interview), without
per-call opt-ins: (a) every Codex invocation from Claude's skills runs at xhigh reasoning
— research pinned to gpt-5.5 (cost-efficient volume work), review pinned to the frontier
gpt-5.6-sol (the quality gate) — with the global `~/.codex/config.toml` supplying xhigh
as the machine default for any call that omits an effort (explicit flags out-rank it by
design; per-user scope); (b) Claude Code itself operates with
ultracode (multi-agent Workflow orchestration) on by default. Before this Goal,
codex-review silently ran the default model at effort **none** (its config.toml premise
was false), and ultracode was off unless typed per-prompt.

## Interview record (Phase 4)
- Q: sol supports max/ultra above xhigh — go higher? → **A: xhigh** (as originally asked).
- Q: global config.toml — effort only, or model too? → **A: effort only**; model is pinned
  per-skill via `-m` (interactive sessions keep the default model, saving cost).
- User directive (mid-turn): Claude Code should also run **ultracode by default** → M2.
- User directive (mid-turn): split the models — **research on gpt-5.5, review on
  gpt-5.6-sol** (research is the high-volume web pass; review is the quality gate worth 2×).
- Settled by prompt/convention (not asked): pin location = skill commands (repo rule:
  "pin model+effort; do not depend on config drift"); config.toml itself is untracked
  (gitignore hard-denies it), so the repo change is skills + tests only.

## Research summary (Phase 3)
Artifact: [research/latest-codex-model.md](research/latest-codex-model.md)
- Latest family is **GPT-5.6**; flagship slug **`gpt-5.6-sol`** (docs: strongest for
  complex coding, research, cybersecurity — fits the adversarial reviewer role).
- `xhigh` is valid for gpt-5.6 and for the CLI config key `model_reasoning_effort`.
- Strongest against-point: **cost** — sol is 2× gpt-5.5 by API-token pricing (Codex-credit
  units unverified; treat as an estimate — the research artifact carries an erratum: its
  S6 "credit rates" line actually cites the API pricing page), and OpenAI's
  migration guidance says benchmark rather than blind-pin. Accepted: user explicitly
  wants latest+xhigh; interactive sessions keep the cheaper default model.
- Availability risk (research flagged, then resolved live): CLI 0.143.0 rejected
  gpt-5.6-sol ("requires a newer version"); upgraded via brew to **0.144.0**, after which
  `codex debug models` lists gpt-5.6-sol with efforts low…xhigh,max,ultra.

## Milestones & tasks (Phase 5)
### M1-enforce — latest model + xhigh everywhere Claude calls Codex
- [x] **T01** global `~/.codex/config.toml`: `model_reasoning_effort = "xhigh"` (machine-default backstop; explicit flags out-rank by design) — all task gates ticked, `Consensus: resolved`
- [x] **T02** skills: pin models + xhigh (research → gpt-5.5, review → gpt-5.6-sol); tests enforce it — all task gates ticked, `Consensus: resolved` (6 review rounds)

### M2-ultracode — Claude Code defaults to ultracode
- [x] **T03** enable ultracode persistently (mechanism per claude-code-guide research delta) — repo layer + tests + task-level E2E done, `Consensus: resolved`; machine activation is the maintainer's manual step (see M2 E2E record)

## E2E records
### M1 E2E (all three captured live on this machine, CLI v0.144.0, exit 0 each)
- **Bare default, repo cwd** — `codex exec --ephemeral "Reply with exactly: ok"` run from
  the repo root (path redacted per public-safe rule): header `model: gpt-5.5 … reasoning
  effort: xhigh`, reply `ok`, EXIT=0 (sessions `019f48f9-b2fc-7403-b063-e657a177e236`,
  re-run `019f48ff-3479-7e32-8748-b7012d540f8b`) — the global config.toml default holds
  with project context present. Full sanitized transcript:
  [M1-enforce/01-global-config-xhigh/evidence-transcript.md](M1-enforce/01-global-config-xhigh/evidence-transcript.md).
- **Research pins** — documented codex-research flags (`--skip-git-repo-check --ephemeral
  -s read-only -C $SCRATCH -m gpt-5.5 -c model_reasoning_effort="xhigh"`), header probe:
  `model: gpt-5.5 … reasoning effort: xhigh`, EXIT=0 (session
  `019f48fb-8227-7cb1-836e-3a86e2090256`). (Header probe verifies the pins; the full
  research pass is the Phase-3 artifact itself.)
- **Review pins** — the exact documented codex-review command (assembler bundle → `codex
  exec --skip-git-repo-check -s read-only -C $SCRATCH -m gpt-5.6-sol -c
  model_reasoning_effort="xhigh"`): header `model: gpt-5.6-sol … reasoning effort: xhigh`
  (T01 round-2 session `019f48f6-d4ba-7652-ad49-6da35b1b0d7f`) — every live review round
  in this Goal is itself a capture of this command.

### M2 E2E (evidence captured; final check BLOCKED on maintainer activation)
- **Flag layer, verified live**: from a scratch cwd (repo-cwd probes inherit this Goal's
  own Stop gate and hang — root-caused in
  [M2-ultracode/03-ultracode-default/evidence-probe.md](M2-ultracode/03-ultracode-default/evidence-probe.md)),
  `command claude --effort ultracode -p "…is ultracode mode active…?"` → `Yes`, EXIT=0;
  the probe session's own context confirms *standing* ultracode mode.
- **Alias layer, verified transiently**: sourcing `claude/ultracode.zsh` in a clean
  `zsh -f` yields exactly `claude='claude --effort ultracode'` (ALIAS-OK); suite-enforced.
- **Blocked remainder (maintainer-manual by policy — AGENTS.md + permission classifier):**
  1. `./install.sh` (links `~/.claude/ultracode.zsh`), 2. add to `~/.zshrc`:
  `[ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"`,
  3. open a fresh terminal and confirm ultracode is on (`/effort` shows ultracode).
  Tick the M2 box only after step 3 is observed.

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E (enforce): bare `codex exec` (no flags, repo cwd) runs at xhigh via global config AND the documented skill commands run as pinned (research → gpt-5.5 @ xhigh, review → gpt-5.6-sol @ xhigh), verified together — all three captures live with session ids + sanitized transcript, see "M1 E2E" record above; both M1 tasks consensus-resolved
- [ ] M2 E2E (ultracode): ultracode confirmed active by default in a fresh session context
- [ ] GOAL E2E: one full end-to-end pass of the whole Goal, captured
