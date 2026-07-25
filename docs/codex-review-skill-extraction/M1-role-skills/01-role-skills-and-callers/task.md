# 01-role-skills-and-callers

## Intent / Why

`~/.codex/AGENTS.md` loads on every Codex invocation in every project, and 91 of its 144
lines described two specific roles. Ordinary work in unrelated repositories inherited a
reviewer persona and a findings-shaped output contract it never asked for. This task moves
each role's contract into its own Codex skill, reduces the global file to what is true of
every invocation regardless of role, and makes both callers name their skill explicitly.

The invariant: **the review must not get weaker.** Moving from unconditional loading to
elected loading is a genuine reliability downgrade, and it is paid for here — not waved away
— with explicit `$name` invocation, a standing order to stop when the skill is absent, and a
structural check on the returned output that does not depend on the model's self-report.

## Deployment context

Runs locally on the maintainer's own machines only. Single user, no network service, no
multi-node deployment, no CI runtime. The artifacts are markdown instruction files, a bash
installer, and a bash guard; the "runtime" is a human invoking `codex` from a terminal. Data
criticality is low for the instruction content itself and **high for secret non-exposure**,
because this repository is public and this change adds a new tracked directory to its
allowlist. Out of scope by construction: availability, replication, concurrent access.

## Design consult

Skipped — no trigger. Instruction prose plus allowlist and installer-map entries; no module
boundary, API contract, persistence or logging path, cursor/idempotency semantics,
partitioning, rendering boundary, or multi-path sanitization. The one design question that
would have qualified — how the contract reaches Codex at all — was settled by direct
experiment and the Phase 4 interview before any file was written.

## What was done (what / why)

**1. Verified the mechanism before designing around it.** A probe skill at
`~/.codex/skills/probe-marker-x9/SKILL.md` returned its sentinel verbatim under the exact
review flags (`--ephemeral -s read-only -C <scratch>`, cwd outside any repo), proving user
skills are discovered and followed in non-interactive `codex exec`. A second probe confirmed
the publicly documented `$HOME/.agents/skills` path also works. Both probes were removed.

**2. Chose `~/.codex/skills/` over the documented `$HOME/.agents/skills`.** `.agents/` is a
cross-vendor shared location, so putting "you are an adversarial reviewer" there would
recreate the contamination one level up — the very defect this task removes. Codex-scoped
placement wins over documentation alignment here; the residual is recorded below.

**3. Split the two role contracts into skills, byte-for-byte as they stood.** `Mode 2` plus
the `Consensus` section — including the right-sized-technology axis, the scale-fit guards, the
`Sites:` format, the bounded `Sketch:` rule and the output budget that the *previous* Goal had
just added to `AGENTS.md` — became `codex/skills/adversarial-review/SKILL.md`; `Mode 1` became
`codex/skills/adversarial-research/SKILL.md`. Each gained frontmatter whose `description`
front-loads trigger words, because OpenAI's guidance is that descriptions drive matching and
get shortened when many skills are installed. Each also carries the stack-neutrality clause
that used to sit in the global identity paragraph.

**4. Reduced `AGENTS.md` from 144 lines to 45, declaring no role at all.** What remains is
what is true of every invocation: stack neutrality, the language boundary, and the
operational constraints (read-only default, never read or transmit secrets, web data is
untrusted). A short section points at the two skills and — importantly — orders Codex to say
so and stop if asked to review or research without the matching skill available. Two bullets
that said "Research and review must not modify the working tree" and "(both modes)" were
generalised, since they are safety rules rather than role rules.

**5. Made both callers invoke explicitly.** `claude/skills/codex-review/SKILL.md` and
`claude/skills/codex-research/SKILL.md` now open with `Use the $adversarial-review skill` /
`$adversarial-research`, the explicit form, rather than relying on description matching that
OpenAI documents as non-deterministic. The review prompt is 943 bytes.

**6. Checked that the contract landed — by reading, not by scripting.** Step 2b tells the
caller to read Codex's first line (the prompt orders it to say so and stop when
`$adversarial-review` is unavailable) and to check the output is contract-shaped before filing
it as a round. An earlier version of this step was a bash grammar validator, `contract_ok`. It
was removed in round 006: it checked shape and never substance — it could not tell whether the
reviewer applied the scale-fit guards or the blast-radius discipline — and rounds 003 through
006 were spent almost entirely on its own defects, ending with a BSD `grep -E` bug where
`[ \t]` matched space-or-the-letter-`t`. It was hardening nobody asked for, and its cost was
the review budget of the change it was meant to protect. Step 2 still dropped its `tee`
pipeline so `codex exec`'s own exit status survives; a failing run is never recorded.

**7. Wired the new tree through every gate the repo requires.** `.gitignore` gained
`!/codex/skills/` plus a deny-all and the two named skill dirs, so an unanticipated skill
directory stays untrackable. `tests/secret-guard.sh` gained the three matching negation lines
and a refreshed `GITIGNORE_SHA_PIN`, both required to move in the same change. `install.sh`
gained two `link` entries.

## Files changed (where / why)

- `codex/AGENTS.md` — reduced to global-only content; gained the pointer section and the
  stop-if-absent order. This is the whole point of the task: it no longer declares a role.
- `codex/skills/adversarial-review/SKILL.md` — new. The full review contract, moved verbatim.
- `codex/skills/adversarial-research/SKILL.md` — new. The research contract, moved verbatim.
- `claude/skills/codex-review/SKILL.md` — prompt invokes `$adversarial-review`; the bullet
  explaining where the contract lives was rewritten, including the honest cost of election;
  Step 2b is a read, not a script; Step 2 captures to a file instead of piping so the
  producer's exit status is preserved.
- `claude/skills/codex-review/assemble-review.sh` — `carried_ok` gained a suffix-equality
  binding *and* a uniqueness gate: a companion's body must be exactly its round's last lines,
  and the round must contain exactly one `## Carried decisions` heading, so an ambiguous or
  truncated companion is refused and the round is sent whole. The `REVIEW_FULL_ROUND_IDS`
  parser splits on commas only, trims each field, and rejects empty ones without globbing.
- `claude/skills/codex-research/SKILL.md` — prompt invokes `$adversarial-research`; intro
  rewritten to point at the skill rather than at `AGENTS.md`.
- `.gitignore` — allowlist entries for the new tree, deny-all inside it.
- `tests/secret-guard.sh` — pinned negation list and `GITIGNORE_SHA_PIN` updated in the same
  change, as the repo's own rule requires.
- `install.sh` — two map entries so the skills symlink into `~/.codex/skills/`.

## E2E verification

1. **User skills load in non-interactive exec.** Probe skill under the exact review flags
   returned `ZEBRA-QUILT-7734`; a second probe at the documented `$HOME/.agents/skills`
   returned `OTTER-BRIDGE-3390`. Both probe dirs removed afterwards.
2. **Allowlist behaves.** Both named skills are trackable; a fabricated
   `codex/skills/novel_unknown/blob` is denied by `git check-ignore`. Probe removed.
3. **Guard green after wiring.** `bash tests/secret-guard.sh` → `✓ PASS: secret guard`, with
   the refreshed SHA pin and the three new negation lines staged in the same change.
4. **Installer links correctly.** `./install.sh --dry-run` showed both new links; the real run
   created them, and `ls -la ~/.codex/skills/` shows both pointing into this repo.
5. **Contamination is gone.** An unrelated request ("describe what a kettle does") under the
   review flags produced a plain two-sentence answer and `ROLE-NEUTRAL` — no reviewer persona,
   no findings shape.
6. **The review contract still lands when asked for.** `$adversarial-review` on a deliberately
   vulnerable `eval "cat $f"` snippet produced `[severity:high][security]` with `Evidence:`,
   `Verification:`, `Suggested direction:`, `Omitted-detail: 0 low`, and
   `GPT verdict: reject` — and applied the scale-fit guard explicitly, writing that "the local
   single-user context does not waive command injection."
7. **The research contract loads too.** `$adversarial-research` returned the five evidence
   categories its own contract defines (needed information, opposing views, for, against,
   unverified). The caller's prompt separately fixes the six output headings the artifact must
   carry, `## Sources` being the sixth — the skill defines what to gather, the caller defines
   the file's shape.
8. **Both caller prompts still parse as one shell argument.** Review: `bash -n` clean,
   `args=11`, prompt 943 bytes. Research block: `bash -n` clean.
9. **Hook still emits valid JSON.** Parsed with `json.load` → OK.
10. **The extracted skill carried six real review rounds.** Rounds 001-006 all ran through
    `$adversarial-review` under the review flags and returned contract-shaped output — severity
    tags with `Sites:`, `Evidence:`, `Verification:`, `Suggested direction:`, one
    `Omitted-detail: N low`, one closing `GPT verdict:` with a rationale — and the findings were
    substantive enough to reject five of the six. Round 006 also applied the scale-fit axis in
    the direction that costs it something, which is the axis this whole line of work added.
11. **Compaction works on a six-round history.** The assembler emits rounds 001-004 compacted to
    their companions and 005-006 in full; bundle 92,066 bytes, inside the 512KB budget. All six
    companions pass the tightened `carried_ok` (unique heading + suffix equality).
12. **`REVIEW_FULL_ROUND_IDS` fails loud on every malformed spelling.** `1,3` → `1 3`;
    `1, 2` → `1 2`; `1, ,2`, `1,`, `,1`, `1,,2`, `[1]` all fatal; unset stays empty.

## Accepted residual

**Symlinked contracts.** `install.sh` links the two role skills into `~/.codex/skills/`, so the
reviewer's instructions live in the same working tree the reviewer is reviewing. Round 006
raised this as a high finding; it is recorded as accepted rather than fixed. It is not
introduced here — every artifact in this repository has been symlinked since `install.sh`
existed, which is the repository's stated purpose — and on a single-user local machine there is
no privilege boundary to defend: whoever can edit the contract can equally edit `install.sh`,
the guard, and the caller. The suggested remedy (an immutable snapshot promoted only after
independent consensus) is a release pipeline for a personal config backup, and it would break
the edit-here-live-everywhere property the repository exists to provide.

**Contract landing is checked by reading.** There is no machine gate proving the elected skill
loaded. The defense is the prompt's stop-on-first-line order plus the caller reading the
output. This is recorded as a real limitation, not presented as fail-closed.

`~/.codex/skills` is not the path public Codex documentation names for user skills
(`$HOME/.agents/skills` is), so this relies on behaviour verified locally against
codex-cli 0.145.0 rather than on a documented contract. The choice is deliberate — the
documented path is shared across agents and would reintroduce the contamination — and the
failure mode is loud rather than silent: the caller's prompt orders a stop when the skill is
absent, and Step 2b's structural check catches output that ignored the contract anyway.

## Gate status
- [x] Change applied and directly verified
- [x] Codex (GPT-5.6 Sol) adversarial review consensus
