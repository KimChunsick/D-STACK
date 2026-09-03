---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: auto
review: on
codex_effort: ultra
e2e: cli
unit_tests: on
visual: none
korean_polish: on
---
# Value outside the enum

codex_effort has three legal values and "ultra" is not one of them.

## Requirements

- [ ] **R01** a value outside the enum is refused by field name — accept: the output names "field codex_effort"
