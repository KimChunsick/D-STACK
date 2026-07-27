# Maintainer response — Round 003

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium][technical correctness] PARALLEL is still permitted under `delegate-when`, and
`honest-scope` is still global text.** Accepted, fixed, and the reproduction is exact — six PASS
results including the one that is false. `delegate-when` and `requires` both gate delegation, so
rejecting the line from one of them describes the list I happened to name rather than the invariant.
The check now iterates the gating set `%w[delegate-when requires]`, names which list carried the
violation, and resolves `honest-scope` and `frontend-takes-precedence` from the parsed node as
non-empty strings instead of through a global `has`. `frontend-takes-precedence` was added because
round 002's repair to `CLAUDE.md` section 0 quotes it; a summary pinned to prose that nothing pins
is the same blind spot one level up.

Re-controlled with five cases now: the live line moved into `requires` and into `delegate-when`
(both fail, each naming its list), the older phrasing under `parallel-when` (passes), and
`requires:` and `honest-scope:` each gutted to comments (both fail with `parsed as NilClass`).
Recorded in `task.md`.
