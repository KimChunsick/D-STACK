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
# Minimal valid request

Three rows, every frontmatter field in its enum, no pending markers and no open questions.

## Requirements

- [ ] **R01** dstack check request accepts a well formed file — accept: exit code 0 and a counts line naming rows 3
- [ ] **R02** every frontmatter key is in the fixed set — accept: the fields line reads "checked 11, bad 0"
- [ ] **R03** rows are numbered in increasing order — accept: no "does not increase" line in the output
