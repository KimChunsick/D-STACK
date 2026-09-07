// Required normative clauses and known contradictory forms, not a prose semantic model.
pub const REFERENCE: &str =
    "Follow runtime.md's coordinator boundary, compact receipt and interruptible wait protocol.";
pub const RECEIPT: &str = "Compact receipt: location/HEAD; R outcomes; changed files/commit; commands/exits; artifact paths; blockers/skips. Raw logs stay in artifacts.";
pub const WORKERS: &[&str] = &[
    "claude/templates/prompts/worker.md",
    "claude/agents/general-dev.md",
    "claude/agents/frontend-dev.md",
    "claude/agents/e2e-runner.md",
];
pub const REFERENCES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "claude/CLAUDE.md",
    "codex/AGENTS.md",
    "claude/skills/dstack-workflow/SKILL.md",
    "claude/skills/dstack-develop/SKILL.md",
    "claude/skills/dstack-verify/SKILL.md",
    "claude/skills/dstack-quick/SKILL.md",
    "claude/skills/unit-test/SKILL.md",
    "claude/skills/codex-review/SKILL.md",
];
pub const RUNTIME: &[(&str, &str)] = &[
    ("main boundary", "Main boundary: main owns user conversation, dispatch, compact receipts and CLI state only."),
    ("worker boundary", "Worker boundary: fresh bounded workers own investigation, implementation, tests, failure diagnosis, fixes and heavy evidence inspection."),
    ("fresh retry boundary", "Retry boundary: a failure, missing receipt or capacity/tool refusal remains pending or blocked; use a new worker with a bounded handoff, never main takeover or stale context reuse."),
    ("compact receipt", "Receipt: location/HEAD; R outcomes; changed files/commit; commands/exits; artifact paths; blockers/skips. Raw logs stay in artifacts."),
    ("interruptible wait", "Wait protocol: keep active run/Plan/worker IDs; use the host's interruptible native completion wait and handle input before resuming the same wait."),
    ("question handling", "Question: answer then resume the same wait with the same active IDs."),
    ("addition handling", "Addition: main records req/decision changes through dstack, approves authorized scope and adjusts only affected work."),
    ("conflict handling", "Conflict: safe-stop affected workers, preserve busy-plan guards and confirm they stopped before replacement; if no supported CLI transition exists, leave pending/blocked."),
    ("stop handling", "Stop: stop affected workers and report their actual state; stopped is not done."),
    ("unsupported-host handling", "Unsupported: if interruptible wait/input is unavailable, disclose that limit and use supported completion events; do not promise concurrent input."),
    ("launch/completion integrity", "No duplicate launch, blanket restart, manual JSON reset or false completion to free input."),
    ("host-native tools", "Host tools: Claude Agent completion events; Codex wait_agent (when available), with interruptible bounded waits using the actual tool schema."),
    ("test ownership", "Main checks receipt metadata only; workers execute repository tests and lint, diagnose failures and inspect large artifacts."),
    ("independent review ordering", "Independent review/research/audit use fresh saved-sub sessions via dstack mode exec; seal the independent Plan review before dstack plan done."),
    ("validation limits", "Validation limits: deterministic document/fixture checks are not live host UI or concurrent-input evidence and cannot prove arbitrary prose semantics."),
    ("GSD provenance", "GSD provenance: retained secondary attribution; original checkout/revision unavailable, no fresh primary-source inspection claimed."),
    ("GSD source paths", "GSD sources: docs/explanation/the-phase-loop.md and skills/gsd-execute-phase/SKILL.md; quotations are retained in dstack-develop §11."),
 ];

// Deliberately bounded positive instruction patterns. Negated policy sentences such as
// "Never let main implement" do not match; arbitrary rewording is outside this lint's reach.
pub const FORBIDDEN: &[(&str, &str)] = &[
    (
        "table-only return omits compact receipt",
        r"(?i)(?:^|[.!?] )return only the table",
    ),
    (
        "raw-log return",
        r"(?i)(?:^|[.!?] )(?:return|send|paste) (?:the |full )?raw logs to main",
    ),
    (
        "question restart",
        r"(?i)question: relaunch the worker after answering",
    ),
    (
        "addition blanket restart",
        r"(?i)addition: restart every plan without recording the request",
    ),
    (
        "stop treated as done",
        r"(?i)stop: mark the plan done to stop execution",
    ),
    (
        "forbidden main work",
        r"(?i)(?:^|[.!?] )(?:(?:the )?main(?: session)?) (?:may |must )?(?:directly )?(?:implement|edit|runs? (?:the required )?repository tests|inspects? raw logs)",
    ),
    (
        "failure takeover",
        r"(?i)(?:if|when) (?:delegation|a worker) fails,? (?:the )?main (?:takes? over|implements?|fixes?)",
    ),
    (
        "stale worker reuse",
        r"(?i)(?:^|[.!?] )(?:resume|reuse|continue in) the same (?:worker )?context (?:to retry|after failure)",
    ),
    (
        "direct-edit exception",
        r"(?i)(?:one obvious edit stays in the main loop|only exception is a one-line|exception: a one-line typo|open case and run it yourself|one-line typo fix \| no pipeline)",
    ),
    (
        "duplicate launch",
        r"(?i)(?:^|[.!?] )launch a duplicate worker",
    ),
    (
        "false completion",
        r"(?i)(?:^|[.!?] )report completion to accept user input",
    ),
    (
        "premature Plan completion",
        r"(?i)(?:^|[.!?] )run dstack plan done before independent review",
    ),
    (
        "manual state reset",
        r"(?i)(?:^|[.!?] |conflict: )reset plan\.json by hand",
    ),
    (
        "unsupported input promise",
        r"(?i)(?:^|[.!?] |unsupported: )concurrent input is guaranteed on every host",
    ),
];
