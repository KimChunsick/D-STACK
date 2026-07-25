## Needed info

- P1: Real review guidance already treats scale/context fit as reviewable. Google asks whether design is “appropriate for your system” and explicitly names over-engineering as excess generality or future functionality; Microsoft asks reviewers to verify task goals, scope, unnecessary functionality, parallel-programming risks, security, and bigger-picture effects; Azure Well-Architected says to define functional/nonfunctional requirements, understand risk tolerance, and avoid both over- and under-engineering. [S1][S2][S3][S4]

- P1: The defensible encoding is a declared, versioned `deployment_context` / `project_profile` block in the task bundle, not only a static reviewer-prompt clause. Prior art is closer to PR/design templates: goals, non-goals, stack, performance/scalability concerns, data growth, usage pattern, RTO/RPO, cost constraints, and operationalization. This keeps context auditable per task and prevents a hidden global prompt from overriding evidence. [S2][S5]

- P1: Over-engineering can be a reportable defect class, but severity should require evidence. High only when the extra machinery creates a concrete correctness/security/operability failure; medium when it adds current maintenance risk or blocks the task’s real Why; low when it is speculative complexity or optional simplification. Google and Microsoft both support severity labels/nits and code-health tradeoffs rather than perfectionism. [S1][S2]

- P1: The reviewer still needs a “context is not a waiver” invariant: local-only does not suppress concrete concurrency, injection, path traversal, secret-handling, data-loss, corruption, or supply-chain findings. Google specifically calls out edge cases, concurrency, security-qualified reviewers, and reviewing whole-system context. [S1]

- P2: Blast-radius reporting has guideline support but limited direct LLM evidence. Google tells reviewers to look beyond diff hunks and consider whole-system code health; Microsoft says adjacent out-of-scope issues should become separate tasks rather than block the PR; Metabase explicitly asks reviewers to point out PR implications for code the PR does not touch. [S1][S2][S6]

- P2: A safe finding format is `Primary site`, `Confirmed sibling sites`, and `Suspected follow-up`. Only “confirmed” sites should block the current round; “suspected” sites should be filed as follow-up unless the current diff already proves the same invariant is violated. This matches the scope discipline in Microsoft’s reviewer guidance and reduces hallucinated ripple claims. [S2]

- P2: A commonize/extract-helper suggestion should require evidence that the duplicated sites share the same invariant, will change together, and are in current scope. Refactoring is defined as behavior-preserving restructuring, but wrong abstractions can be worse than duplication; “rule of three” is a heuristic, not a law, but it captures the need for multiple examples before abstraction. [S14][S15][S16]

- P2: The current “never include code examples or patches” rule is stricter than major review guidance. Google says reviewers are not responsible for detailed design/code, but sometimes direct instructions, suggestions, or code are useful; Microsoft allows examples and line-level suggestion features. If changed, a bounded sketch should be “shape only,” no complete patch, no imports, no full helper body, and only when the invariant is otherwise ambiguous. [S1][S2]

- P3: Public OpenAI model docs now list `gpt-5.6-sol` / `gpt-5.6` with 1.05M context window and 128K max output; Codex non-interactive docs say piped stdin becomes additional context when a prompt argument is also provided, `codex exec` defaults to read-only sandbox, and JSONL output exposes events. [S17][S18]

- P3: The public Codex docs I found do not document a `codex exec` stdin byte cap, exact assembled-bundle truncation behavior, or whether overflow fails before API submission, silently truncates, or triggers compaction. The CLI help observed locally confirms stdin handling but not overflow semantics. [S18]

- P3: OpenAI’s Responses compaction docs support compaction as a first-class method for long-running loops: it reduces context size while preserving state, can run when a threshold is crossed, returns a compacted item, and lets callers drop earlier items after the most recent compaction item in stateless chaining. [S19]

## Opposing views

- Against P1 context-scaling: Telling a reviewer “small/local-only” can bias it toward dismissing real issues. This is not directly measured for deployment-context prompts, but LLM code-review studies show prompt framing can shift error profiles: richer explanation/fix prompts increased false rejection of correct code, and LLM security-awareness studies show models often fail to warn about vulnerabilities unless the task framing supports it. [S20][S21]

- Against P1 over-engineering-as-finding: A deliberately hardened repo can be falsely attacked as “too much” if the model treats simplicity as an aesthetic preference. Azure explicitly warns against both over- and under-engineering and requires risk tolerance/business objectives; that cuts both ways: hardening may be appropriate even for local software if the protected asset or failure mode justifies it. [S4]

- Against P2 blast-radius reporting: It invites scope creep and hallucinated call sites. CR-Bench reports a tradeoff where pushing agents to find more issues increases spurious findings; static-analysis literature shows alert volume and false positives require triage/prioritization because full manual audit exceeds budget. [S22][S23]

- Against P2 commonization: The fastest way to “fix all siblings” can be premature abstraction. Sandi Metz’s wrong-abstraction argument is especially relevant: once callers diverge, shared condition-laden helpers become harder to understand and easier to break. [S14]

- Against P2 code sketches: Snippets can anchor the implementer on the reviewer’s design. The best code-review-specific evidence is mixed: existing review comments can positively prime reviewers to find similar bugs without reducing other bug detection, but AI-generated comments have low acceptance rates and often need shortening/editing. GitHub’s Copilot responsible-use docs also warn about overreliance and inaccurate generated code. [S11][S12][S13]

- Against P3 compaction: Summaries are lossy. OpenAI compaction preserves state in an opaque item, but it is not human-interpretable; for an immutable audit trail, that is not a substitute for the sealed files. Dropping full older rounds can reopen accepted risks or lose the exact rebuttal/evidence chain unless carried decisions are explicit and machine-checkable. [S19]

- Against P3 output caps: Hard caps can hide high-severity findings if the reviewer wastes slots on easier low/medium items. CR-Bench’s recall/noise frontier suggests the cap should be severity-aware and require omitted-count disclosure, not just “top 5 comments.” [S22]

## For the goal

- P1 is sound: Add a deployment-context field and an explicit “right-sized technology” axis. This is consistent with Google/Microsoft/Azure review doctrine and should reduce infrastructure fantasy findings while preserving real bug review through a “context is not a waiver” clause. [S1][S2][S4]

- P1 is achievable: Review output already has severity and evidence lines. Add one line per relevant finding: `Context fit: local-only / single-user / no network service / data criticality / justified hardening?` and require the reviewer to cite either the task profile or code evidence. [S1][S2]

- P2 is sound if bounded: Requiring sibling sites can reduce “next round finds same bug elsewhere” when the sibling is mechanically verifiable by `rg`, call graph, imports, route table, shared schema, or repeated validation pattern. Guidelines support whole-system context and out-of-diff implications. [S1][S6]

- P2 sketch allowance is defensible only as a narrow exception: `Sketch:` max about 3-6 lines of pseudocode or structural shape, no working patch, no replacement code block, and only when `Suggested direction` cannot name the invariant clearly. This follows Google’s “sometimes code is helpful” without making the reviewer the designer. [S1][S2]

- P3 is sound: Full prior-round refeeding is not required for audit integrity if sealed files remain immutable on disk and the model receives a compact, explicit “carried decisions” ledger. OpenAI’s compaction guidance supports preserving state with fewer tokens while keeping long-running workflows coherent. [S19][S24]

- P3 output bounding is achievable: Keep unlimited high-severity reporting in principle, cap lows aggressively, consolidate by root cause, require `Omitted: N low / M unverified candidates`, and require a second focused round if high/medium candidates exceed the cap. Static-analysis triage literature supports prioritization because raw alert volume overwhelms review capacity. [S23][S25]

## Against the goal

- P1 could weaken adversarial value if “local-only” becomes a permission to ignore robustness. The reviewer should still report concrete data loss, file clobbering, command injection, unsafe parsing, TOCTOU/race bugs in actual concurrent paths, secret exposure, and supply-chain risk. [S1][S21]

- P1 could create false “over-engineered” findings against intentional hardening. The fix is to require a counterfactual: “what concrete current requirement is made harder by this complexity?” If the reviewer cannot show cost, confusion, dead code, or wrong deployment assumption, it should be low or omitted. [S1][S4][S20]

- P2 could increase rounds if every finding becomes a mini-audit of the whole repo. The blast-radius line should be restricted to allowlisted files plus cheaply verifiable sibling references; unverified siblings should be labeled non-blocking. [S2][S22]

- P2 commonization could violate “Simplicity First / Surgical Changes.” The reviewer should recommend extraction only when the same fix must be applied in multiple confirmed sites and the abstraction boundary is already present or obviously local. Otherwise, say “apply same invariant at B/C” rather than “create shared helper.” [S14][S15]

- P2 sketches may worsen independent verification. Given Copilot/security studies showing generated code can contain weaknesses and GitHub’s own overreliance warnings, a sketch should not be copy-pastable production code. [S13][S26]

- P3 compaction can silently lose provenance. The right unit is not a rewritten old round; it is a new immutable “input manifest” or “review-state capsule” that references sealed round files by number/hash and carries only current decisions, open findings, accepted risks, out-of-scope items, and claimed fixes to verify. This preserves auditability while bounding input. [S19][S24]

- P3 caps can lower recall. CR-Bench suggests recall and noise trade off; the safer cap trims explanation length and low-severity tail first, not the existence of high/medium findings. [S22]

## Unverified

- I could not verify any empirical study specifically testing whether telling an LLM reviewer “this is small/local-only” suppresses legitimate security or correctness findings. The risk is inferred from broader prompt-framing, overcorrection, and security-awareness evidence. [S20][S21]

- I could not verify direct evidence that LLM “blast-radius reporting” reduces review iteration count. Existing evidence supports the practice conceptually, but the LLM-specific iteration-count effect appears unmeasured. [S1][S2][S6][S22]

- I could not verify that instructing an LLM reviewer to flag over-engineering causes false over-engineering findings against deliberate hardening. This remains a plausible failure mode, not a measured one.

- I could not verify public Codex CLI documentation for exact stdin maximum size, exact total assembled-bundle limit, or exact overflow behavior. Public model docs list the model context window, and Codex docs describe stdin handling, but not CLI-level truncation/fail semantics. [S17][S18]

- Local observation only: `codex-cli 0.145.0` help confirmed `codex exec` reads stdin when prompt is absent or `-`, and appends piped stdin as a `<stdin>` block when a prompt is also provided. The bundled local model catalog reported `gpt-5.6-sol` metadata including a 272K `context_window` and 1,000,000 `max_context_window`, but I did not find a public URL documenting those exact CLI-internal fields. Treat this as non-public implementation metadata, not a stable contract.

- I could not verify measured evidence that output caps mainly trim low-value noise without dropping high-severity findings. The best evidence is indirect: static-analysis prioritization, CR-Bench recall/noise tradeoffs, and code-review usefulness studies. [S22][S23][S25]

## Sources

- [S1] Google Engineering Practices, “What to look for in a code review” and related reviewer guidance. Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://google.github.io/eng-practices/review/reviewer/looking-for.html

- [S2] Microsoft Engineering Fundamentals Playbook, “Reviewer Guidance.” Primary. Publication date: last update 2024-08-22. Retrieved: 2026-07-26. URL: https://microsoft.github.io/code-with-engineering-playbook/code-reviews/process-guidance/reviewer-guidance/

- [S3] Microsoft Engineering Fundamentals Playbook, “Author Guidance.” Primary. Publication date: last update 2024-08-22. Retrieved: 2026-07-26. URL: https://microsoft.github.io/code-with-engineering-playbook/code-reviews/process-guidance/author-guidance/

- [S4] Microsoft Azure Well-Architected Framework, “Architecture strategies for designing for simplicity and efficiency.” Primary. Publication date: 2023-12-01. Retrieved: 2026-07-26. URL: https://learn.microsoft.com/en-us/azure/well-architected/reliability/simplify

- [S5] Microsoft Engineering Fundamentals Playbook, “Template: Milestone / Epic Design Review.” Primary. Publication date: last update not separately shown on opened section. Retrieved: 2026-07-26. URL: https://microsoft.github.io/code-with-engineering-playbook/design/design-reviews/recipes/templates/milestone-epic-design-review/

- [S6] Metabase Developer Guide, “Code reviews.” Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://www.metabase.com/docs/latest/developers-guide/code-reviews

- [S11] Spadini, Calikli, Bacchelli, “Primers or reminders? The effects of existing review comments on code review.” Primary. Publication date: 2020; page last updated 2025-10-03. Retrieved: 2026-07-26. URL: https://research.chalmers.se/publication/524536

- [S12] Olewicki et al., “Impact of LLM-based Review Comment Generation in Practice.” Primary. Publication date: 2024-11-11 preprint; Mozilla page 2026-01-16. Retrieved: 2026-07-26. URL: https://www.mozillafoundation.org/en/research/library/impact-of-llm-based-review-comment-generation-in-practice-a-mixed-open-closed-source-user-study/

- [S13] GitHub Docs, “Application card: GitHub Copilot inline suggestions.” Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://docs.github.com/en/enterprise-cloud@latest/copilot/responsible-use/inline-suggestions

- [S14] Sandi Metz, “The Wrong Abstraction.” Primary opinion/prior art. Publication date: 2016-01-20. Retrieved: 2026-07-26. URL: https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction

- [S15] Martin Fowler, “Refactoring.” Primary/prior art. Publication date: no date. Retrieved: 2026-07-26. URL: https://refactoring.com/

- [S16] HandWiki, “Rule of three (computer programming).” Secondary. Publication date: no date. Retrieved: 2026-07-26. URL: https://handwiki.org/wiki/Rule_of_three_%28computer_programming%29

- [S17] OpenAI API Docs, “Models.” Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://developers.openai.com/api/docs/models

- [S18] OpenAI / ChatGPT Learn, “Non-interactive mode.” Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://learn.chatgpt.com/docs/non-interactive-mode

- [S19] OpenAI API Docs, “Compaction.” Primary. Publication date: no date. Retrieved: 2026-07-26. URL: https://developers.openai.com/api/docs/guides/compaction

- [S20] Jin and Chen, “Are LLMs reliable code reviewers? systematic overcorrection in requirement conformance judgement.” Primary. Publication date: 2026-06-26. Retrieved: 2026-07-26. URL: https://link.springer.com/article/10.1007/s10515-026-00638-5

- [S21] Sajadi et al., “Do LLMs Consider Security? An Empirical Study on Responses to Programming Questions.” Primary. Publication date: 2025-04-11. Retrieved: 2026-07-26. URL: https://link.springer.com/article/10.1007/s10664-025-10658-6

- [S22] Pereira et al., “CR-Bench: Evaluating the Real-World Utility of AI Code Review Agents.” Primary preprint. Publication date: 2026-03-10. Retrieved: 2026-07-26. URL: https://arxiv.org/abs/2603.11078

- [S23] CMU SEI, “Prioritizing Alerts from Multiple Static Analysis Tools, Using Classification Models.” Primary. Publication date: 2018-08-14. Retrieved: 2026-07-26. URL: https://www.sei.cmu.edu/library/prioritizing-alerts-from-multiple-static-analysis-tools-using-classification-models/

- [S24] OpenAI, “From model to agent: Equipping the Responses API with a computer environment.” Primary. Publication date: 2026-03-18. Retrieved: 2026-07-26. URL: https://openai.com/index/equip-responses-api-computer-environment/

- [S25] Imtiaz, Murphy, Williams, “How Do Developers Act on Static Analysis Alerts? An Empirical Study of Coverity Usage.” Primary. Publication date: 2019-10. Retrieved: 2026-07-26. URL: https://www.microsoft.com/en-us/research/publication/how-do-developers-act-on-static-analysis-alerts-an-empirical-study-of-coverity-usage/

- [S26] Fu et al., “Security Weaknesses of Copilot-Generated Code in GitHub Projects: An Empirical Study.” Primary preprint accepted to TOSEM. Publication date: arXiv v4 2025-02-06. Retrieved: 2026-07-26. URL: https://arxiv.org/abs/2310.02059

- [S27] Liu et al., “Lost in the Middle: How Language Models Use Long Contexts.” Primary. Publication date: 2024. Retrieved: 2026-07-26. URL: https://aclanthology.org/2024.tacl-1.9/

- [S28] Hsieh et al., “RULER: What’s the Real Context Size of Your Long-Context Language Models?” Primary preprint. Publication date: 2024-04-09. Retrieved: 2026-07-26. URL: https://arxiv.org/abs/2404.06654