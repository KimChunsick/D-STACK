# Codex adversarial review — Round 006

## Review scope
Re-review

## GPT findings

Round 005 verification: the exact duplicate-heading, indented-header, malformed-axis, bare-verdict, and glob-expansion counterexamples are rejected. The findings below use distinct failure paths.

[severity:high][security] The installed role contracts are live symlinks into the untrusted working tree, allowing a change under review to rewrite its reviewer instructions before review.
Sites: primary: `install.sh` role-skill links; confirmed: both `codex/skills/*/SKILL.md` files and both Claude caller skills
Evidence: Both new installer mappings use `link`; live `readlink` resolves `~/.codex/skills/adversarial-{review,research}` directly into the D-STACK working tree.
Verification: A changed review skill can mandate a rubber stamp retaining only `Omitted-detail: 0 low` and a valid approval verdict; that output passes every predicate, including the intentionally valid zero-finding AWK path.
Suggested direction: Load review-time contracts from an immutable last-approved snapshot outside the reviewed tree, promoting changed contracts only after independent consensus.

[severity:medium][technical correctness] `contract_ok` still accepts malformed verdict and empty support-label grammar, so its fail-closed claim remains false.
Evidence: BSD `grep -E` interprets `[ \t]` as space-or-`t`, while the AWK branches accept `Evidence:` and `Verification:` without nonblank payloads.
Verification: The live predicate accepted `GPT verdict: rejectthis-has-no-separator`, rejected a literal-tab separator, and returned status 0 for a finding whose Evidence and Verification values were empty.
Suggested direction: Use portable POSIX blank classes consistently and require nonblank payloads for every mandatory labelled line.

[severity:low][DX] `REVIEW_FULL_ROUND_IDS` silently normalizes a whitespace-only comma field instead of failing as documented.
Evidence: The raw empty-field guard recognizes adjacent commas but not a comma field containing spaces; subsequent word splitting discards that field.
Verification: `REVIEW_FULL_ROUND_IDS='1, ,2'` produced `raw_gate=accept` and parsed IDs `1 2`.

Omitted-detail: 0 low

GPT verdict: reject — the live-symlink trust inversion permits the reviewed change to control its reviewer, and the independent structural gate still accepts malformed output.

## Maintainer response

**H1 — symlinked role contracts (disagreed; recorded as accepted, pre-existing design).**
The mechanism is real and described correctly, but it is neither introduced by this change nor
a defect at this scale.

*Not introduced here.* `install.sh` has symlinked **every** artifact in this repository into the
live agent dirs since it existed — that is the repository's stated purpose ("the live agent dirs
hold symlinks pointing back into this repo, created by `install.sh`", `AGENTS.md`). The two new
`link` entries follow the existing map exactly. If live symlinks are a trust inversion, the
inversion predates this task by every other skill, hook and instruction file the maintainer runs.

*No privilege boundary exists to protect.* The threat model the finding needs is a writer who can
modify the working tree but should not control the review. On a single-user local machine there is
no such actor: anyone who can edit `codex/skills/adversarial-review/SKILL.md` can equally edit
`install.sh`, `tests/secret-guard.sh`, the Claude-side caller, and the maintainer's shell profile.
The reviewer contract is not the weakest link in that scenario; it is one of many equivalent ones.

*Right-sized technology.* The suggested direction — an immutable last-approved snapshot outside the
tree, promoted only after independent consensus — is a release-and-attestation pipeline. For a
personal agent-config backup with one human author, that is a stack far beyond what the project
needs, and it would break the property the repository exists to provide: edit here, and the change
is live everywhere. The scale-fit guard cuts both ways and this is the over-engineering side.

Recorded as an accepted risk, not waved away: the reviewer's instructions are as mutable as
everything else the maintainer runs, and the defense is the maintainer being the only writer.

**M1 — `contract_ok` grammar (agreed; resolved by removing the code).** Both new bypasses are
real, and the BSD detail is the decisive one: `grep -E` has no C escapes, so `[ \t]` matched
space-or-the-letter-t and a literal tab was rejected while `GPT verdict: rejectthis-has-no-separator`
passed. That is a portability bug I would have needed yet another round to find.

The response is to delete `contract_ok` rather than fix it. It was never part of the requested
change; it was hardening I added for the case where the elected skill fails to load. Rounds 003
through 006 were spent almost entirely on its own defects — a shape checker that, by its own
recorded admission, could never tell whether the reviewer applied the scale-fit guards or the
blast-radius discipline. Step 2b is now a read: the prompt already orders Codex to say so on its
first line and stop when `$adversarial-review` is unavailable, and the caller checks the first
line and the output shape by eye. Less code, no false claim of a fail-closed gate, and the review
surface this task created is gone.

**L1 — whitespace-only comma field (agreed, fixed).** The glob guard saw adjacent commas but not
a field of spaces, and word splitting then discarded it. The parser now splits on commas only,
trims each field, and rejects an empty one; a trailing comma is caught before the loop because
IFS splitting drops a trailing empty field.

Verified: `1,3` → `1 3`; `1, 2` → `1 2` (spacing still tolerated); `1, ,2`, `1,`, `,1`, `1,,2` and
`[1]` all fatal; empty unset stays empty. `bash -n` clean; `bash tests/secret-guard.sh` → `✓ PASS`.

## Carried decisions
- **Symlinked contracts are an accepted risk, closed.** Single-user local machine, no privilege
  boundary between the tree and the reviewer, and the SSOT model requires live links. Do not
  reopen this as a new finding; do not build a snapshot-and-promote pipeline for it.
- **Do not re-add an output grammar validator.** It checks shape, never substance, and it cost
  four review rounds on its own bugs. If the elected skill fails to load, the prompt's
  stop-on-first-line order plus reading the output is the whole defense, and that is recorded as
  the accepted limitation rather than dressed up as a gate.
- Compaction stays gated on a *unique* `## Carried decisions` heading: fail-open on bundle size,
  fail-closed on content. Do not "improve" it into picking the last or outermost heading.
- `~/.codex/skills` remains an undocumented user-skill path (documented one is
  `$HOME/.agents/skills`), chosen because that path is cross-vendor shared and would recreate the
  contamination one level up. Verified locally against codex-cli 0.145.0.

Consensus: resolved
