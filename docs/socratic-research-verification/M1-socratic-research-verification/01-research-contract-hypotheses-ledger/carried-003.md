## Carried decisions — Round 003
- F1-F8: fixes stand; round 003 verified F8 (reachability) effective and raised one
  residual of F7 plus one doc-staleness low.
- F11 (medium, partial-schema silent drop): FIXED this round — a format lacking an
  encoding for ANY of the three blocks (none or only some) yields an incomplete
  artifact: the carried blocks are encoded and each missing one is flagged as the
  caller's defect; the format rule now says "flagging any block the shape cannot
  encode".
- F12 (low, stale task record): FIXED this round — the task doc now describes the three
  added sections + amended rule and the per-shape encoding accurately.
- F9 stands as accepted-residual (immutable P3 record; contract supersedes).
- Standing context: no-new-tests repo policy; caller file (pinned section list) is
  declared task T03's work.

Consensus: disagreed
