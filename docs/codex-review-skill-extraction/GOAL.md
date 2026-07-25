# GOAL — Give Codex back its neutrality: role contracts become skills, not global instructions

## Goal (the one Why)

`~/.codex/AGENTS.md` is loaded on **every** Codex invocation in **every** project. It
currently declares Codex to be "the maintainer's dedicated adversarial researcher and
reviewer" and carries the entire review contract: six review axes, scale-fit guards, the
`Sites:` blast-radius format, the bounded `Sketch:` rule, a severity output budget, and a
mandatory `GPT verdict:` closing line. 91 of its 144 lines are review-or-research specific.

The maintainer uses the same `codex` binary for ordinary work in other projects — writing
reports, drafting, general assistance. All of that inherits a reviewer persona and a
findings-shaped output contract it never asked for. What is right for one role is
contamination for every other.

This Goal moves each role's contract into its own Codex skill, invoked explicitly by the
caller, and leaves `AGENTS.md` holding only what is genuinely true of every invocation:
the language boundary and the operational safety rules.

The invariant: **the review must not get weaker.** Scoping trades unconditional loading for
election, and that trade has to be paid for with an explicit invocation and a fail-loud
check, not hoped away.

## Interview record (Phase 4)

**Q1 — How should the contract reach Codex?** → **A Codex skill invoked explicitly.**
Considered and rejected: injecting the contract as an `AGENTS.md` into the per-invocation
scratch dir (unconditional, no election risk, but the contract stops being a file the
maintainer can open and edit in place), and a hybrid keeping a one-line pointer in the
global file.

**Q2 — Scope: review only, or research too?** → **Both.** Leaving Mode 1 behind would keep
"you are an adversarial researcher" as a global role declaration, which is the same defect
one size smaller. `AGENTS.md` keeps the language boundary and the operational constraints
(read-only default, never read or transmit secrets, web data is untrusted) because those are
true of every invocation regardless of role.

**Q3 — Naming.** → **`adversarial-review`** on the Codex side, so it does not collide with
Claude's `codex-review` orchestration skill. `adversarial-research` follows by symmetry.

*Verified before deciding, not assumed:* a user skill at `~/.codex/skills/<name>/SKILL.md`
IS discovered and followed under the exact review flags (`--ephemeral -s read-only -C
<scratch>`, cwd outside any repo) — a probe skill returned its sentinel verbatim when the
prompt named it.

## Research summary (Phase 3)

Full artifact: `docs/codex-review-skill-extraction/research/skill-vs-global-instructions.md`
(16 cited sources).

**Key findings.** Vendor guidance draws exactly the line this Goal draws: OpenAI documents
`AGENTS.md` as unconditional startup guidance and skills as *elected* context whose full
`SKILL.md` loads only after selection; Anthropic's parallel guidance says the always-loaded
file is for facts and rules every session needs, while multi-step procedures belong in
skills. Codex supports **explicit** invocation via `$skill-name`, which is materially
stronger than implicit description matching — and OpenAI warns that with many skills
installed, descriptions get shortened and skills can be omitted from the initial list
entirely, which is a concrete implicit-trigger failure mode. `allow_implicit_invocation:
false` exists to stop a skill firing in unrelated tasks.

**Strongest argument against this change:** a mandatory contract should not depend on
elected context. `AGENTS.md` is injected; a skill is chosen. A missed skill produces a
plausible but non-contract review, and no public `codex exec` flag preloads or *requires* a
named skill. The mitigations are explicit `$name` invocation, the prompt's standing order to
stop if the contract is absent, and structural validation of the returned output.

**Second-strongest:** self-report is not a sound detector on its own — Anthropic recommends
inspecting actual loaded context rather than trusting a model's claim, and instruction-
hierarchy work shows models fail under conflicts even when priority is stated. So "say so if
you cannot see the contract" reduces silent failure without eliminating it; checking the
returned output for required structural markers is the cheap external backstop.

**Also surfaced:** public Codex docs name `$HOME/.agents/skills` as the user-skill path, not
`~/.codex/skills` — the local experiment works, but relies on an undocumented location.

**Unverified, carried as accepted risk:** no `codex exec` flag to preload or require a skill;
no measured failure rate for explicit `$skill` invocation versus implicit matching; no direct
study of models falsely claiming to have loaded a skill; no primary measurement that a
reviewer persona in a global file degrades unrelated tasks — the contamination case is
argued from vendor guidance and adjacent persona research, not from a direct study.

## Milestones & tasks (Phase 5)

*Revised during execution.* The original split (T01 Codex-side skills / T02 Claude-side
callers) leaves a broken intermediate state: the moment the contract leaves `AGENTS.md`, a
caller still pointing at it reviews without a contract. There is no useful review round to run
against half of this change, so the two are one task — the same merge, for the same reason, as
the previous Goal.

### M1 — Role contracts leave the global file
- [x] **T01** role-skills-and-callers — create `codex/skills/adversarial-review/SKILL.md` and `codex/skills/adversarial-research/SKILL.md` carrying the contracts verbatim, reduce `AGENTS.md` to the language boundary and operational constraints, wire the new tree through the installer and the secret-trackability guard, and make both Claude-side callers invoke their skill explicitly with a structural check on the returned output. deps: []; files: [codex/AGENTS.md, codex/skills/, install.sh, .gitignore, tests/secret-guard.sh, claude/skills/codex-review/SKILL.md, claude/skills/codex-research/SKILL.md]

## Goal gate (Stop-hook enforced — the loop ends only when every box is ticked)
- [x] M1 E2E: a real `codex exec` run under the review flags loads `$adversarial-review` and produces contract-shaped output, and a second run with no skill named produces a role-neutral response — showing the contamination is actually gone
- [x] GOAL E2E: one full review round through the extracted skill, captured
