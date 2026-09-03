---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: off
codex_effort: high
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# a request whose second row is covered by a task but proven by nothing

<!-- selftest-tasks: R01 R02 -->
<!-- selftest-evidence: R01 -->

- [ ] **R01** the command prints what it counted — accept: stdout carries "checked N"
- [ ] **R02** the command refuses bad input — accept: exit code 1 with the reason on stderr
