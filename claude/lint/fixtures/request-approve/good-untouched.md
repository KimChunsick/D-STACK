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
---
# Untouched after approval

Nothing edits the file between dstack request approve and dstack check request, so the hash still matches.

## Requirements

- [ ] **R01** the approval hash matches an unedited file — accept: check request prints "approved: yes" and exits 0
