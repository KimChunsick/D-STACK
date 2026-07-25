# Codex adversarial review — Round 004

## Review scope

Re-review. First round in which compaction actually ran on this task's own history (round 001
compacted, 002-003 full). Bundle 71,609 bytes.

## GPT findings

[severity:medium][technical correctness] The delimiter-count predicate still accepts a carried-decisions heading hidden inside live Markdown, causing silent omission of sealed review history.
Sites: primary: `assemble-review.sh` `emit_round_compact`; confirmed: backtick/tilde parity checks, HTML-comment count check, unhandled raw-HTML blocks, `task.md` section 5 and E2E check 5; suspected: none.
Evidence: Fence-like content and stray comment closers are counted without regard to parser state, while containers such as `<pre>` are not counted at all.
Verification: Inside an open ```text fence, a ``` line that is not a closing fence followed by the heading produces b=2 and check 0; a `-->`, then `<!--`, then the heading produces o=c=1 and check 0. Both compact from the false heading and discard preceding MUST-SURVIVE content.
Suggested direction: Compact only when the candidate is proven to be a top-level heading by a semantics-aware boundary or an unambiguous machine-generated marker; otherwise emit the full round.

Omitted-detail: 0 low

GPT verdict: reject — the compaction path still has reproducible counterexamples that silently remove substantive sealed-round context.

## Maintainer response

Accepted. Both counterexamples reproduce. This is the fourth consecutive round in which a
derived-from-Markdown rule lost to a construct its author had not modelled, and that pattern
is the finding, not the two specific bugs.

**Why counting was never going to work.** Order-insensitive counts cannot see state: a ```
line *inside* an open ```text fence makes the backtick count even, and `-->` before `<!--`
balances while leaving a comment open. Both are unfixable by counting harder. The alternative
the reviewer names first — a semantics-aware boundary — means writing a CommonMark block
parser inside a bash assembler, which is precisely the over-engineering the right-sized
technology axis exists to reject for a local single-user pipeline.

**Took the reviewer's second option: an unambiguous machine-generated marker, as a separate
file.** Sealing a round now also writes `carried-<NNN>.md` containing that round's carried
decisions through its consensus line, and the assembler emits that companion for older rounds
instead of parsing anything. A file whose entire content *is* the carried state cannot be
impersonated by its own contents, so all five decoy shapes from rounds 001-004 become moot
rather than individually defended. The companion's name stays outside the `codex-review*.md`
namespace the assembler validates. `emit_round_compact` is now nine lines with no `awk`, and
`CARRIED_RE` is gone.

**Measurement before rejecting the "no delimiters allowed" shortcut.** I checked whether a
blunt rule — refuse to compact any round with a fence or comment before the heading — would
have worked: of 32 real sealed rounds carrying a decisions section, only 12 are
delimiter-free. It would have disabled compaction for the 10-round task that motivated this
work, so it was not a viable simplification.

Verification: this task's bundle is 25,987 bytes with round 001 replaced by its 908-byte
`carried-001.md` and rounds 002-003 full. The legacy 10-round task, which predates companions,
falls back to full snapshots for all eight older rounds and reports `no carried-NNN.md
companion` on each — a visible, correct degradation rather than a silent loss. Companions are
trackable in git (not caught by the allowlist `.gitignore`), and `bash tests/secret-guard.sh`
is green.

**Honest scope note.** Compaction now only benefits rounds sealed from here on. The 88KB →
38KB figure measured earlier on the legacy task no longer applies to that task, because it has
no companions; the mechanism is correct instead of retroactive, and the task doc says so.

## Carried decisions

- Compaction reads a **companion file, never the round's Markdown**. Four rounds killed four
  successive derived rules (fenced heading, HTML-commented heading, `~~~`-inside-``` plus a
  line that closes and opens a comment, and a ``` line inside an open ```text fence). Do not
  reintroduce derivation in any form, including "just one more delimiter check".
- Sealing a round means writing two files: `codex-review-<NNN>.md` and `carried-<NNN>.md`.
  A missing companion costs bundle size only — the round is sent whole. A *wrong* companion
  misleads every later round, so restate the complete live decision set in each round rather
  than only the delta.
- The companion name must stay outside the `codex-review*.md` namespace the assembler
  validates for contiguity.
- The check fails toward emitting the whole round. Any future change must keep that direction:
  sending too much is a cost, dropping real carried state is a defect.
- The budget bounds elaboration only. Every low actually found is reported, in full or as a
  one-line title.
- `MAX_BUNDLE` = 524288, derived from the smallest documented window (`context_window` 272000
  for `gpt-5.6-sol`). Changing it requires citing a window.
- Changing the contract means changing every surface in the same edit: `codex/AGENTS.md`,
  `claude/skills/codex-review/SKILL.md`, `claude/skills/codex-review/assemble-review.sh`,
  `claude/hooks/fullcycle-inject.sh`, and the task doc.
- Accepted, unchanged: no public documentation of a `codex exec` stdin byte cap or its
  overflow semantics was found; the context figures come from local CLI metadata and the
  public model spec, and the budget is a runaway detector rather than a proven ceiling.

Consensus: disagreed
