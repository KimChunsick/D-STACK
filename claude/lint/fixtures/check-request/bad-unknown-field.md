---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: auto
review: on
codex_effort: high
e2e: cli
unit_tests: on
visual: none
korean_polish: on
reviewer_mood: cheerful
---
# Unknown frontmatter key

The key reviewer_mood is not in REQ_FIELDS, so nothing downstream would ever read it.

## Requirements

- [ ] **R01** a key outside the fixed set is refused by name — accept: the output names "field reviewer_mood"
