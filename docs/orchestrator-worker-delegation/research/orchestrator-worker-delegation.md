## Needed info

- Current Claude Code docs say non-fork subagents start in a fresh, isolated context: they do not see the parent conversation history, invoked skills, or files already read; they receive the delegation task, CLAUDE.md hierarchy, startup git status, and any preloaded skills. This makes “complete declaration” necessary but not sufficient: the handoff must serialize the relevant discoveries and constraints. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- Claude Code docs also say resumed subagents retain full prior conversation history, including prior tool calls and reasoning; subagent transcripts are unaffected by main-conversation compaction and persist within the same session, with default cleanup after 30 days. Explore and Plan are one-shot and cannot be resumed. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- Claude Code’s own delegation guidance is conditional: use the main conversation for frequent back-and-forth, iterative refinement, shared context across planning/implementation/testing, quick targeted changes, or latency-sensitive work; use subagents for verbose, self-contained work that can return a summary. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- Worktree isolation is documented as a way to prevent parallel session edits from colliding, but it carries setup costs: a worktree is a fresh checkout, gitignored files need explicit copying via `.worktreeinclude`, and subagent worktrees branch from the default branch unless configured to use current `HEAD`. [Primary: https://code.claude.com/docs/en/worktrees, no date, retrieved 2026-07-27]

- Anthropic’s production multi-agent research system found multi-agent architectures strongest for breadth-first tasks with many independent directions and information exceeding one context window; it also says multi-agent systems used about 15x chat tokens and that most coding tasks have fewer truly parallelizable parts than research. [Primary: https://www.anthropic.com/engineering/multi-agent-research-system, 2025-06-13, retrieved 2026-07-27]

- Anthropic’s general agent guidance says start with the simplest solution, add complexity only when it demonstrably improves outcomes, use workflows for predictable tasks, and reserve agents for cases needing model-driven flexibility. [Primary: https://www.anthropic.com/engineering/building-effective-agents, 2024-12-19, retrieved 2026-07-27]

- Anthropic’s 2026 workflow guidance maps task shape directly to orchestration: sequential for dependencies, parallel only for independent simultaneous subtasks, evaluator-optimizer for measurable iterative refinement, and single-agent/no workflow when that already meets the bar. [Primary: https://claude.com/blog/common-workflow-patterns-for-ai-agents-and-when-to-use-them, 2026-03-05, retrieved 2026-07-27]

## Opposing views

- Opposing view: “Complete declaration is enough, because resumed workers keep context warm.” Counter-argument: current Claude Code docs support resume warmth, but only after the initial handoff; non-fork workers still start without the parent’s accumulated context, so the first delegation brief becomes a critical lossy compression boundary. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- Opposing view: “Delegating serial tasks preserves the orchestrator context and is worth the total token cost.” Counter-argument: Anthropic’s measured production data says multi-agent systems can burn tokens fast, and AgentDropout’s ACL 2025 results show redundant agents/communications are a real enough problem that pruning them reduced prompt tokens by 21.6% and completion tokens by 18.4%. [Primary: https://www.anthropic.com/engineering/multi-agent-research-system, 2025-06-13, retrieved 2026-07-27; Primary: https://aclanthology.org/2025.acl-long.1170/, 2025-07, retrieved 2026-07-27]

- Opposing view: “Implementation should be delegated because coding agents are verifiable.” Counter-argument: verifiability helps, but Anthropic still says human review is crucial for broader system requirements, and its eval guidance recommends deterministic graders where possible rather than treating agent routing as inherently reliable. [Primary: https://www.anthropic.com/engineering/building-effective-agents, 2024-12-19, retrieved 2026-07-27; Primary: https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents, 2026-01-09, retrieved 2026-07-27]

- Opposing view: “A manager/orchestrator model is standard practice.” Counter-argument: OpenAI’s guide recommends maximizing a single agent first and splitting only when complicated instructions or tool selection failures justify multiple agents; it also says deterministic solutions may suffice when the workflow does not clearly resist rule-based automation. [Primary: https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/, no date, retrieved 2026-07-27]

## For the goal

- The proposed attribution rule is technically aligned with Claude Code’s resume semantics: if a finding’s fix lies wholly inside one worker’s file declaration, `SendMessage` can return to the same subagent with its prior tool history intact, avoiding a cold second handoff. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- Delegating implementation can reduce parent-context pollution in exactly the way Claude Code subagents are designed to do: verbose exploration, test logs, and file contents stay in the worker context while only a summary returns. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- MASAI is direct prior art for modular software-engineering agents: it used sub-agents with well-defined objectives/strategies, gathered repo information from different sources, avoided long trajectories with extraneous context, and reported 28.33% on SWE-bench Lite. [Primary: https://arxiv.org/abs/2406.11638, submitted 2024-06-17, retrieved 2026-07-27]

- The “complete declaration” gate is directionally supported by eval/task-design guidance: well-specified tasks, stable environments, clear graders, and explicit expected outcomes are repeatedly identified as prerequisites for reliable coding-agent evaluation. [Primary: https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents, 2026-01-09, retrieved 2026-07-27]

- Worktrees remain justified for this pipeline if containment depends on a clean tree and committed `base..HEAD` range, because Claude Code documents worktrees as isolated file/branch spaces and subagent worktrees as a built-in isolation mode. [Primary: https://code.claude.com/docs/en/worktrees, no date, retrieved 2026-07-27]

## Against the goal

- No reviewed source supports delegating all implementation work solely because declarations are complete; current guidance supports conditional delegation based on task shape, context sharing, cost, and measurable improvement. [Primary: https://claude.com/blog/common-workflow-patterns-for-ai-agents-and-when-to-use-them, 2026-03-05, retrieved 2026-07-27; Primary: https://www.anthropic.com/engineering/building-effective-agents, 2024-12-19, retrieved 2026-07-27]

- Iterative review-fix loops are a poor fit for blanket worker delegation when findings depend on cumulative reasoning across rounds or across declarations; Anthropic explicitly warns against parallel workflows when agents need cumulative context or must build on each other’s work. [Primary: https://claude.com/blog/common-workflow-patterns-for-ai-agents-and-when-to-use-them, 2026-03-05, retrieved 2026-07-27]

- Agentless is strong counter-evidence against “more agentic implementation is necessarily better”: it deliberately avoided autonomous planning/tool use and reported 32.00% on SWE-bench Lite at $0.70 per task, outperforming existing open-source software agents at the time. [Primary: https://arxiv.org/abs/2407.01489, submitted 2024-07-01, revised 2024-10-29, retrieved 2026-07-27]

- MAST found that multi-agent performance gains are often minimal and identified 14 failure modes across system design, inter-agent misalignment, and task verification, based on 1600+ traces across 7 frameworks. That maps directly onto risks from extra handoffs, file-boundary assumptions, and review attribution. [Primary: https://arxiv.org/abs/2503.13657, submitted 2025-03-17, revised 2025-10-26, retrieved 2026-07-27]

- For a one-maintainer/local-terminal deployment, serial worktree delegation may reduce parent context but increase wall-clock friction: fresh checkout setup, ignored-file copying, dependency installs, shared ports/test databases, cleanup, and branch integration are concrete local costs not erased by the absence of parallelism. [Primary: https://code.claude.com/docs/en/worktrees, no date, retrieved 2026-07-27]

- If the orchestrator becomes mostly a router over already-declared dependencies and ownership, that conflicts with current agent-building guidance: deterministic/rule-based solutions should handle what they can, and expensive models should be reserved for ambiguity, judgment, synthesis, and cases where simpler workflows fail. [Primary: https://www.anthropic.com/engineering/building-effective-agents, 2024-12-19, retrieved 2026-07-27; Primary: https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/, no date, retrieved 2026-07-27]

## Unverified

- I could not independently verify the requester’s measured 205,400 implementation-message tokens, 37,000 fixed overhead tokens, 563k auto-compact window, or 530k compaction trigger from public sources.

- I could not verify public evidence specifically comparing “orchestrator implements serial tasks directly” versus “orchestrator delegates all serial tasks to Claude Code subagents in git worktrees” for a one-maintainer pipeline.

- I could not verify that Claude Code subagents can be resumed after `/clear`; current docs say subagent transcripts persist within the same session and are unaffected by compaction, but also say the `SendMessage` name/ID check is scoped to the current conversation and resets on `/clear`. [Primary: https://code.claude.com/docs/en/sub-agents, no date, retrieved 2026-07-27]

- I could not verify public evidence that adversarial code-review fix loops specifically improve when routed back to the original implementation worker; the closest evidence is general evaluator-optimizer guidance and Claude Code subagent resume semantics.

- I could not verify environment-specific costs for this pipeline’s worktrees, such as dependency reinstall time, port contention, or test database contention; those need local measurement.

## Sources

- https://code.claude.com/docs/en/sub-agents — primary, no date, retrieved 2026-07-27.
- https://code.claude.com/docs/en/worktrees — primary, no date, retrieved 2026-07-27.
- https://claude.com/blog/using-claude-code-session-management-and-1m-context — primary, no date visible in retrieved page, retrieved 2026-07-27.
- https://www.anthropic.com/engineering/multi-agent-research-system — primary, published 2025-06-13, retrieved 2026-07-27.
- https://www.anthropic.com/engineering/building-effective-agents — primary, published 2024-12-19, retrieved 2026-07-27.
- https://claude.com/blog/common-workflow-patterns-for-ai-agents-and-when-to-use-them — primary, published 2026-03-05, retrieved 2026-07-27.
- https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents — primary, published 2026-01-09, retrieved 2026-07-27.
- https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/ — primary, no date, retrieved 2026-07-27.
- https://arxiv.org/abs/2406.11638 — primary, submitted 2024-06-17, retrieved 2026-07-27.
- https://arxiv.org/abs/2407.01489 — primary, submitted 2024-07-01, revised 2024-10-29, retrieved 2026-07-27.
- https://arxiv.org/abs/2503.13657 — primary, submitted 2025-03-17, revised 2025-10-26, retrieved 2026-07-27.
- https://aclanthology.org/2025.acl-long.1170/ — primary, published 2025-07, retrieved 2026-07-27.
- https://swe-bench-live.github.io/ — primary, no page date; includes June 2026 update note, retrieved 2026-07-27.