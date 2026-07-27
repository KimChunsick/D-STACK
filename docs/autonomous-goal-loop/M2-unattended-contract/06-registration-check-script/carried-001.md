## Carried decisions — Round 001
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
