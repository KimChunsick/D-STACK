## Carried decisions — Round 002
Round-1 decisions still standing (discovery time never changes blocking status; the six-round
budget escalates rather than downgrades; triage matches the contract's `[severity:…][axis]`
format with no cap on the high/medium query; `<review-unit>` is a single abstraction). Added in
Round 2:

- **Assembly is a precondition, not a step.** Guard `run-dir` and the assembler with `|| exit 1`
  and check the bundle's entry count before launching. A review of an empty bundle exits 0.
- **This harness runs zsh, not bash.** Unquoted parameter expansion does NOT word-split here.
  Pass file lists as literal arguments, never through a variable.
- **Run labels are per-attempt.** The allocator refuses a used label by design; retry with a new
  suffix. The durable path is `.dstack/runs/$CLAUDE_CODE_SESSION_ID/<label>/out.txt` — never
  call `run-dir` again to recover it.
- **Atomic exclusive creation, never test-then-write**, wherever two streams can generate the
  same path. Same lesson as `dstack reg`'s `ln` publish.
- **The CLI is always invoked by absolute path**; nothing puts `~/.claude/bin` on `PATH`.
- Open follow-up: sharpen the no-tests-versus-pinned-checks distinction in `AGENTS.md`, deferred
  while that file is inside M1's open review bundle.

Consensus: disagreed
