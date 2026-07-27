# Codex adversarial review — Round 001

## Review scope
Batch pass 1 (consolidated round; this unit's first) | `REVIEW_MODE=serial` | bundle 13860 bytes. Allowlist: the review-unit folder, `claude/skills/full-cycle/check-registration.sh`, `claude/skills/full-cycle/SKILL.md`.

## GPT findings

[severity:high][technical-correctness] Markdown context is parsed incorrectly: fenced examples can become declarations while real rows become inert, and granularity accepts fenced, negated, or ambiguous substrings.
Sites: Primary: declaration `awk`; confirmed: granularity `grep`/`case`.
Evidence: Fence state starts only after entering the section and recognizes only column-zero fences; granularity is the first global line containing either keyword.
Verification: The exact parser emitted only fenced `T99` instead of real `T01`; an indented fence emitted both, while `not task` classified as task and `task, not milestone` as milestone.
Suggested direction: Parse global fence state and require one canonical granularity declaration and one real milestones section in a single context-aware pass.

[severity:high][the-real-why] Review-unit identity is lossy: milestone mode compares only counts, while task mode discards malformed paths and collapses duplicate numeric prefixes.
Sites: Primary: milestone count branch; confirmed: task `want_ids`/`have_ids` pipelines.
Evidence: Milestones never have their IDs extracted; task `sed` emits nothing for nonconforming folders, and `uniq` removes duplicate identities without a cardinality check.
Verification: `{M1,M2}` versus `{M1,M9}` passed; one declared `T01` also passed with either `01-real` plus `misc`, or both `01-a` and `01-b`.
Suggested direction: Require every declaration and scaffold path to map bijectively to one canonical identity, rejecting malformed and duplicate identities before comparing sets.

[severity:high][technical-correctness] Legitimate false results are conflated with errors and other ownership states: unreadable or malformed units become “closed,” while foreign registrations become “absent.”
Sites: Primary: `unit_open` caller; confirmed: closed-unit negation and other-depth ownership branch.
Evidence: Any nonzero `awk` result enters the closed branch, and `check_owned` distinguishes only “owned here” from every other state.
Verification: A missing file returned 2 but was classified CLOSED; no gate section returned the same closed value; a foreign status line was accepted for both a closed unit and an other-depth unit.
Suggested direction: Use explicit tri-state classifiers for open/closed/invalid and absent/mine/foreign, mapping invalid execution to exit 2 and every foreign registration to a named block.

[severity:high][technical-correctness] Failures in the identity transformations are unchecked, so a failed comparator can produce empty differences and permit a false confirmation.
Sites: Primary: `missing`/`extra` `comm` assignments; confirmed: declaration, unit sorting, and ID-extraction assignments.
Evidence: The script uses only `set -u`; none of these command substitutions checks its producing command’s status.
Verification: With `comm` returning 7 for unequal IDs `{1}` and `{9}`, both result variables were empty and the existing emptiness checks selected PASS.
Suggested direction: Capture and validate every transformation’s status separately before consuming its output.

[severity:medium][DX] Numeric sorting violates `comm`’s requirement that both inputs use its lexical ordering, producing false missing and extra identities.
Evidence: Inputs are built with `sort -n`, then consumed directly by `comm`.
Verification: For wanted `{1,2,10}` and present `{1,3,10}`, the script falsely reported common identity `10` in both differences.
Suggested direction: Feed `comm` inputs sorted under the same fixed lexical locale, applying numeric ordering only when formatting diagnostics.

[severity:medium][technical-correctness] The “nothing else under the Goal’s tree” guarantee scans only the declared depth and one alternate depth, and only files named `task.md`.
Sites: Primary: `UNITS` scan; confirmed: `other_raw` scan.
Evidence: `other_depth` can only be 2 or 3, and the captured `dstack status` set is never checked for unexpected paths beneath the Goal.
Verification: For either granularity, the implemented selection inspected depths 2 and 3 but ignored registered documents at depths 1 and 4.
Suggested direction: Compare all session-owned registrations beneath the Goal against the complete allowed registration set.

[severity:low][DX] The success message claims every unit is owned by this session even when correctly closed units are intentionally unregistered.
Evidence: The final `printf` is unconditional after the closed-unit branch explicitly requires `! check_owned`.
Verification: The task document records closed `T01` as deregistered while also recording the “all owned by this session” confirmation.

[severity:low][DX] Accepted goal-directory spellings are not canonicalized before exact status matching.
Evidence: `G="${1%/}"` preserves `./` and absolute spellings, which are propagated into every ownership lookup.
Verification: Against canonical status path `docs/g/GOAL.md`, `docs/g` matched while `./docs/g` and `/repo/docs/g` were both reported unowned.

Omitted-detail: 0 low

GPT verdict: reject — multiple reproducible paths still allow the registration gate to confirm the wrong declared, scaffolded, or registered state.

## Carried decisions
- **Fences are tracked GLOBALLY from line one**, exactly as `check-parallel.sh` does. Starting fence
  tracking at the section inverts the whole file, and the fixture proves it rather than arguing it:
  with a fenced decomposition example sitting in an earlier section, the old parser read that
  block's `per milestone` granularity and its two fake task rows, and read none of the three real
  ones. The rewritten parser reads 3 rows and `task`.
- **Ids are zero-padded and sorted LEXICALLY under `LC_ALL=C`, because `sort -n` breaks `comm`.**
  Measured: declared {2,10,20} against scaffolded {10}, `comm -23` over `sort -n` inputs returned
  `2 10 20` — reporting a present id as missing. The padded form returns `002 020`, which is the
  truth. One collation now serves `sort` and `comm` both.
- **Duplicate ids are REPORTED, never collapsed.** `uniq` hid them, and hiding them also let a
  missing unit through: with 03 scaffolded twice and 02 absent, the deduped sets matched. The
  fixture now blocks with both reasons.
- **A false result and an error are different outcomes.** An unreadable unit doc used to read as
  "closed" and a foreign-owned registration as "absent" — the two states most likely to be broken
  became the two states nothing checked. They are now their own messages, and the foreign case says
  what it costs: the Stop hook SKIPS records owned by another session, so it looks registered and
  enforces nothing.
- **The exit-2 guarantee is backed by checking the transformations.** Every `find`, `sort`, `comm`
  and id-extraction status is tested, and a count-in/count-out guard fails loudly when the extractor
  reads fewer ids than the parser found rows — otherwise a silently shrinking `want` set makes every
  delta empty and the check passes by producing nothing.
- **`--depth` exists so the caller cannot register the wrong level.** Deriving the review-unit depth
  from GOAL.md is a deterministic transform, so the script does it once and P6's fence reads the
  answer instead of hard-coding a task-depth glob.
- **Honest scope, stated in the file.** The alternate-depth sweep looks at one other depth and only
  at files named `task.md`; a registered document elsewhere under the tree is outside what this
  proves.

Consensus: disagreed
