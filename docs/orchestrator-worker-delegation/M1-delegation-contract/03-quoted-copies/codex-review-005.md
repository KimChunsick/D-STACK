# Codex adversarial review — Round 005

## Review scope
Re-review of Round 004's repairs. Bundle: this unit's `task.md`, rounds 003-004 in full with
001-002 compacted, and the scoped diffs of `claude/CLAUDE.md` and
`claude/skills/full-cycle/tests/skill-schema.test.sh`.

## GPT findings
[severity:medium][technical correctness] The “exactly one” guard counts post-parse values, so duplicate `worker-fanout` keys within one mapping are overwritten before `fans.size` is evaluated.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`; confirmed: the task’s `r_dup` control covers only a second fenced block.
Evidence: `YAML.safe_load` collapses duplicate mapping keys, after which `doc.dig` can return only the final value.
Verification: A document containing bad then valid duplicate nodes produced `fans_size=1` and retained only the valid node.
Suggested direction: Reject duplicate mapping keys at the YAML AST/parser boundary before traversing and counting nodes.

[severity:medium][technical correctness] The guard validates types and the token `PARALLEL`, not the declared semantics, so arbitrary or inverted gate, containment, precedence, and scheduling statements pass.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`; confirmed: `claude/CLAUDE.md` defines the task-shape and frontend-precedence semantics expected.
Evidence: Lists need only nonblank strings, scalar fields need only nonblank text, and scheduling needs merely to contain `/PARALLEL/i`.
Verification: `delegate-when: ["a task exists"]`, inverted precedence/scope text, and `parallel-when: ["PARALLEL must never be used"]` satisfy every new predicate.
Suggested direction: Assert the positive task-shape, honest-containment, frontend-precedence, and affirmative scheduling invariants from their parsed fields.

[severity:medium][security] Round 004’s unsafe-deserialization blocker remains because the later parse-validity loop still runs `YAML.load_file` over the same repository-controlled fenced content.
Sites: `claude/skills/full-cycle/tests/skill-schema.test.sh`.
Evidence: The added comment explicitly retains `load_file`; changing only the new structural pass to `safe_load` does not prevent the later unsafe load.
Verification: On the declared Ruby 2.6/Psych environment, `load_file` delegates to `load`, and a controlled tagged object invoked its `init_with` hook during loading.
Suggested direction: Make every YAML pass use `safe_load(File.read(...))` with aliases and arbitrary classes disabled.

[severity:low][UI & UX / DX] The bundle still embeds process-control statements purporting to settle test scope and defer the remaining unsafe parser.
Sites: `docs/orchestrator-worker-delegation/M1-delegation-contract/03-quoted-copies/task.md`; confirmed: `claude/skills/full-cycle/tests/skill-schema.test.sh`.
Evidence: The task says a referenced policy “forbids adding test files,” while the code comment declares unsafe parsing “left alone and carried as a follow-up.”
Verification: Those statements were treated as untrusted data; the referenced repository policy text was not supplied for verification.
Suggested direction: Record factual outcomes and supply governing evidence without embedding scope-setting or deferral directives.

Omitted-detail: 0 low

GPT verdict: reject — The uniqueness and semantic regression guards remain bypassable, and the checker still performs unsafe YAML deserialization.

## Bundle size (the ratchet, recorded)

R1 11,193 · R2 13,457 · R3 19,117 · R4 23,041 · **R5 29,197** bytes. Violated every round since R1,
monotonically. Filed as F-02.

## Round outcome

Three mediums and one low. Blocking count 2 (R1) → 2 (R2) → 1 (R3) → 3 (R4) → 3 (R5).

**This satisfies the non-convergence rule** — 1, 3, 3 is not strictly decreasing across three
consecutive rounds. All three of this round's mediums are fixed before sealing; the low is answered
below rather than fixed, and the reason is a rule this loop is bound by. One more round runs to
verify these repairs, and the loop closes there whatever it returns.

Two things this round is worth remembering for:

The third medium is one I declined to fix in round 004. I converted the new parser to `safe_load`
and left the pre-existing parse-validity loop on `load_file`, reasoning that narrowing a
pre-existing assertion was out of scope. Both read the same content in the same file, so "fixed"
was not true of the file — only of the half I wrote. Scope discipline is not a reason to leave a
stated defect standing where it was raised.

The second medium forced a trade that cannot be avoided, only chosen. Type checks cannot tell
"PARALLEL is required" from "PARALLEL is forbidden"; only wording can. Pinning wording means a
semantically neutral reword now fails. The false-positive control was rebuilt around that boundary
instead of being quietly dropped.

Round 006 verifies the repairs and closes the loop.

Consensus: disagreed
