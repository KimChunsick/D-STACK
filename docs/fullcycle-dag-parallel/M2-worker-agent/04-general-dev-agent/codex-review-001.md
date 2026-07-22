# Codex adversarial review — Round 001

## Review scope
Adversarial review (GPT-5.6 Sol, xhigh; bundle: claude/agents/general-dev.md,
.gitignore, install.sh, tests/secret-guard.sh, delta research artifact).

## GPT findings
[severity:medium][security] The custom worker loses Claude Code's baseline safety prompt while inheriting unrestricted internal, nested-agent, and MCP capabilities.
Evidence: The frontmatter specifies neither `tools` nor `disallowedTools`, while the body lacks secret-handling, destructive-action, external-side-effect, and untrusted-content boundaries. It also requires reporting "actual output/results," creating a leakage path when commands expose credentials. Current Claude Code documentation confirms that custom subagents receive their own prompt instead of the full default prompt and inherit all available tools—including MCP and `Agent`—when unrestricted. [Claude Code subagent documentation](https://code.claude.com/docs/en/sub-agents)
Verification: Under documented loader semantics, invoking this definition in a session with connector tools exposes those tools to the worker; it can also spawn a differently defined nested agent that does not automatically inherit this worker's scope contract. This directly contradicts the research's own conclusion that enforceable restrictions belong in metadata.
Suggested direction: Establish a least-privilege tool boundary in the agent definition and enforce immutable safety and file-scope invariants at the permission or hook layer.

[severity:medium][technical correctness] The agent does not actually request worktree isolation, so supported direct or automatic parallel invocation runs in the parent checkout.
Evidence: The frontmatter omits `isolation: worktree`; the prose merely says "Inside a worktree." Claude Code starts such a subagent in the main conversation's current working directory unless isolation is configured. [Claude Code subagent documentation](https://code.claude.com/docs/en/sub-agents)
Verification: This behavior follows deterministically from the supplied frontmatter. No supplied orchestrator change establishes an alternative per-worker checkout invariant, so two fan-out workers can share one checkout and collide despite the task's stated isolation intent.
Suggested direction: Enforce per-worker worktree isolation in the agent metadata or in the invocation boundary, then demonstrate it through the pending parallel E2E capture.

[severity:medium][the real Why] The precedence contract makes the supposedly unconditional ownership and scope rules overridable by every delegation brief.
Evidence: The definition ranks the brief above repository conventions and the agent definition and says the higher layer wins. A brief declaring components, hooks, styles, frontend tests, `docs/`, or pipeline files therefore instructs this worker to proceed even though the task says frontend ownership is unconditional and the definition otherwise forbids those areas.
Verification: An explicit `@general-dev` invocation with a frontend file in the declared scope produces a direct logical conflict whose resolution is already specified as "the brief wins"; no routing or runtime safeguard in the supplied changes rejects the misdelegation.
Suggested direction: Separate task-specific precedence from immutable worker ownership, safety, and orchestration boundaries.

GPT verdict: reject — concrete medium-severity capability, isolation, and precedence failures prevent this agent from safely satisfying the conditional parallel-worker intent.

## Maintainer response
Per the user's per-goal directive, fixes landed during implementation; this round seals
as disagreed pending independent re-verification in round 002 (consolidated).
1. Agreed, fixed in metadata + body. Frontmatter now carries a least-privilege
   `tools:` allowlist (Bash, Read, Edit, Write, Glob, Grep, TodoWrite) — no `Agent`
   (no nested-agent laundering), no MCP/connector tools, no publishing tools. The body
   gained secret-redaction ("never reproduce secrets/credentials into reports, code,
   or logs; redact credential-shaped values from quoted output") and a
   content-as-data anti-injection rule. Hook/permission-layer enforcement beyond
   metadata is recorded as a non-blocking follow-up (deterministic deny rules live in
   settings/hooks, outside this task's declared files).
2. Rebuttal with adopted alternative. The pipeline's design consult (T01, recorded)
   chose EXPLICIT orchestrator-owned worktree lifecycles — recorded base commit,
   deterministic branch naming, topological merge, cleanup-after-deregistration.
   Frontmatter `isolation: worktree` would have the harness create a second, unmanaged
   worktree outside that bookkeeping (unrecorded base, auto-cleanup semantics),
   breaking scope computation (base..HEAD) and merge ordering. Enforcement therefore
   sits at the invocation boundary (SKILL.md worktree-lifecycle), and the agent's
   immutable boundaries now REFUSE the failure mode the finding names: when the brief
   names a task branch, the worker verifies it is on that branch before writing and
   STOPs on mismatch — it never falls back to the main checkout for parallel work.
   The parallel E2E demonstration is deferred to this goal's E2E phase as suggested.
3. Agreed, fixed. A `<boundaries>` block (immutable: no frontend code, no
   registry/docs/undeclared paths, branch verification, secret redaction,
   content-as-data) now sits ABOVE precedence; precedence applies only within it, and
   a brief conflicting with a boundary is a STOP condition (fail closed), not a
   precedence decision.

## Carried decisions
- Hook/permission-layer deterministic enforcement for worker tool bounds: non-blocking
  follow-up, outside this task's declared files.
- Worktree isolation is enforced at the invocation boundary by design (consult-backed);
  the agent-side guard is the branch-verification STOP rule.
- All fixes await independent re-verification in round 002.

Consensus: disagreed
