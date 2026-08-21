## Needed info

- The best-supported protocol pattern is not “same model writes a Socratic dialogue with itself”; it is **claim decomposition → independent verification questions → evidence/data/tool grounding → final synthesis → explicit unverified residue**. CoVe’s strongest variant avoided letting verification answers attend to the original answer, because otherwise the model can repeat its own hallucinations. [S4]
- SocREval is relevant but narrower than the maintainer’s target: it used Socratic-inspired prompt components for reference-free reasoning evaluation and improved GPT-4 correlation with human judgment from `0.40` to `0.58`, at under `2.1x` the baseline GPT-4 evaluation cost. It explicitly omitted Elenchus because its datasets already contained enough context. [S1]
- Published Socratic/question-decomposition methods report gains on reasoning tasks, but often by decomposing problems into subquestions rather than by adversarially testing claims. Socratic Questioning beat CoT/SC-CoT/ToT on MATH, MMLU Physics/Chemistry, LogiQA, GSM8K, StrategyQA, and selected GPT-4 runs, while costing about `9.22` calls and `34.15s` per 2-turn instance versus CoT’s `1` call and `3.35s`. [S2]
- Self-ask shows the cleanest analogy for data-grounded research: make follow-up questions explicit, then optionally route them to search. The paper reports that adding a search engine to self-ask further improved accuracy on compositional questions. [S3]
- For self-correction, the critical distinction is **intrinsic self-correction** versus **external feedback**. The ICLR 2024 paper says reasoning self-correction without external feedback can fail or degrade. The TACL 2024 survey says no prior work shows reliable general-task self-correction from prompted LLM feedback alone, while reliable external feedback and fine-tuning can help. [S6][S7]
- There is some evidence that structured intrinsic verification can help when the verification task is constrained: ProCo masks key conditions and verifies against reconstructed question conditions, reporting average gains over Self-Correct of `+6.8` EM on open-domain QA, `+14.1` accuracy on arithmetic, and `+9.6` on commonsense reasoning. [S8]
- Cross-examination prior art supports role separation, but not necessarily separate model weights. LM vs LM used examiner/examinee prompts, sometimes with the same LM in different roles, and reported detection of over `70%` of incorrect claims with precision over `80%`; majority variants improved F1 in reported tables. [S5]
- For “data verification” in a read-only research phase, the realistic contract should require a **data-check ledger**, not guaranteed computation: identify measurable claim, source dataset/API/table, date/version, denominator, unit, transformation, recomputed or source-quoted value, and whether executable reproduction is deferred to the orchestrator. BLADE, ScienceAgentBench, CORE, RDAB, and Microsoft’s Excel repair benchmark all treat executable outputs or execution-based checks as stronger than prose-only judging. [S13][S14][S15][S16][S17]
- A claim is “checkable with data” when it has measurable variables, scope, date/version, unit, denominator, and an available primary dataset or table. Claims about intent, taste, architecture tradeoff, or future behavior are usually not directly checkable and should be interrogated through assumptions, counterexamples, and precedent instead.

## Opposing views

- A Socratic layer can become verification theater if the same model invents both the claim and the challenge without new evidence. The strongest self-correction survey result is negative for general tasks with prompted LLM feedback alone. [S7]
- Yes/no verification questions are a weak design choice. CoVe found open verification questions outperformed yes/no formats, and the authors observed models tending to agree with yes/no factual premises whether right or wrong. [S4]
- More agents and more rounds are not automatically safer. A 2026 Scientific Reports paper reports that a single persuasive adversarial agent lowered group accuracy by `10–40%`, increased false consensus by over `30%`, and that more agents/rounds or simple prompt warnings did not reliably mitigate the attack. [S9]
- Data checks can create false confidence if the model chooses the wrong dataset, denominator, unit, transformation, or causal interpretation. BLADE reports best F1 only `44.8%` for open-ended data-driven science tasks; CORE reports best overall accuracy `45.93%` and hard-task accuracy `22.22%`; RDAB argues correctness alone misses statistical validity failures. [S13][S15][S16]
- The existing adversarial-research contract already demands both sides, citations, and unverified items. The marginal gain from adding scripted Socratic dialogue may be lower than adding a stricter **checkable-claims/data ledger** and moving high-risk verification to the adversarial review phase.

## For the goal

- Structured interrogation has measured value when it forces decomposition and independent checks: SocREval improved human-judgment correlation, Socratic Questioning improved benchmark accuracy, self-ask improved compositional QA and could route subquestions to search, CoVe reduced hallucinations, and LM vs LM improved factual-error detection. [S1][S2][S3][S4][S5]
- External grounding is the main reason the goal is sound. The TACL survey’s positive cases are reliable external feedback, large-scale fine-tuning, or tasks especially suited to self-correction; the maintainer’s pipeline can provide external feedback through live web sources, primary data, and orchestrator-run code. [S7]
- Data-verification benchmarks support the idea that prose research should not be trusted when a claim can be computed. ScienceAgentBench uses self-contained Python programs and expert-validated tasks from peer-reviewed publications; CORE evaluates agents by reproducing published paper results; Microsoft’s Excel repair benchmark uses execution-based metrics. [S14][S15][S17]
- A good contract addition is achievable as markdown: require each research artifact to include Socratic probes over definitions, assumptions, evidence, counterexamples, implications, and data interpretation; require a data ledger for checkable claims; require a “deferred executable checks” list for the orchestrator.

## Against the goal

- Do not require a long visible Socratic dialogue for every finding. The evidence supports **structured challenge and verification**, not theatrical transcript generation. Long dialogues increase cost and can hide rather than reduce errors. [S4][S7]
- Do not rely on same-context self-questioning as the main guardrail. CoVe’s factored setup and the self-correction literature both point toward independent context, external evidence, or executable feedback as the stronger mechanism. [S4][S7]
- A second cross-examiner invocation is likely higher-value for important goals, but it probably doubles cost and latency. Published cost signals are material: SocREval stayed under `2.1x`; Socratic Questioning 2-turn averaged `9.22` calls; CoVe notes extra token/computation cost. [S1][S2][S4]
- A simpler alternative may capture most value: keep one research invocation, add a mandatory checkable-claims ledger, and let the existing adversarial review phase or orchestrator run executable checks for claims whose data matters.
- Multi-agent debate can be actively harmful under persuasion, bad retrieval, or role-play drift. If the pipeline adds a second agent, the contract should make it an evidence auditor, not a debater trying to win. [S9]

## Unverified

- I could not verify a direct, controlled result showing that a **separate Codex invocation/context** outperforms a same-invocation Socratic self-check for delegated research artifacts specifically.
- I could not verify exact metric tables for the 2026 FC-MAD, GKMAD, or SEIMAD papers beyond publisher abstracts/search-visible metadata; their abstracts report improvements, but exact gains need full-text inspection. [S10][S11][S12]
- I could not verify whether the maintainer’s specific Codex CLI model, “GPT-5.5 xhigh,” has the same self-correction, debate, or data-analysis behavior as the models studied in the cited papers.
- I could not verify pipeline-specific cost/latency multipliers for the maintainer’s actual Phase 3 workflow; published multipliers are proxies, not measurements of this system.
- I could not verify that all primary datasets needed in future research rounds will be fetchable from the read-only Codex web environment; the contract should require deferral to orchestrator-run checks when data access or execution is blocked.

## Sources

- [S1] Primary. SocREval, ACL Anthology. URL: https://aclanthology.org/2024.findings-naacl.175/ Publication date: 2024-06. Retrieved: 2026-08-21.
- [S2] Primary. The Art of Socratic Questioning, ACL Anthology. URL: https://aclanthology.org/2023.emnlp-main.255/ Publication date: 2023-12. Retrieved: 2026-08-21.
- [S3] Primary. Measuring and Narrowing the Compositionality Gap in Language Models, ACL Anthology. URL: https://aclanthology.org/2023.findings-emnlp.378/ Publication date: 2023-12. Retrieved: 2026-08-21.
- [S4] Primary. Chain-of-Verification Reduces Hallucination in Large Language Models, arXiv. URL: https://arxiv.org/abs/2309.11495 Publication date: 2023-09-20; revised 2023-09-25. Retrieved: 2026-08-21.
- [S5] Primary. LM vs LM: Detecting Factual Errors via Cross Examination, arXiv. URL: https://arxiv.org/abs/2305.13281 Publication date: 2023-05-22. Retrieved: 2026-08-21.
- [S6] Primary. Large Language Models Cannot Self-Correct Reasoning Yet, ICLR 2024. URL: https://proceedings.iclr.cc/paper_files/paper/2024/hash/8b4add8b0aa8749d80a34ca5d941c355-Abstract-Conference.html Publication date: 2024. Retrieved: 2026-08-21.
- [S7] Primary. When Can LLMs Actually Correct Their Own Mistakes?, TACL. URL: https://aclanthology.org/2024.tacl-1.78/ Publication date: 2024-11. Retrieved: 2026-08-21.
- [S8] Primary. Large Language Models Can Self-Correct with Key Condition Verification, ACL Anthology. URL: https://aclanthology.org/2024.emnlp-main.714/ Publication date: 2024-11. Retrieved: 2026-08-21.
- [S9] Primary. When collaboration fails: persuasion driven adversarial influence in multi agent large language model debate, Scientific Reports. URL: https://www.nature.com/articles/s41598-026-42705-7 Publication date: 2026-04-08. Retrieved: 2026-08-21.
- [S10] Primary. Debating to verify: A robust and explainable multi-agent LLM system for fact-checking, ICT Express. URL: https://www.sciencedirect.com/science/article/pii/S2405959526000883 Publication date: 2026-05-29. Retrieved: 2026-08-21.
- [S11] Primary. Guided and knowledgeable multi-agent debate for fact verification, Expert Systems with Applications. URL: https://www.sciencedirect.com/science/article/abs/pii/S0957417425037194 Publication date: 2026-03-01. Retrieved: 2026-08-21.
- [S12] Primary. Socratic Elenchus-inspired multi-agent debate for mitigating hallucinations in large language models, Expert Systems with Applications. URL: https://www.sciencedirect.com/science/article/abs/pii/S0957417426011218 Publication date: 2026-07-15. Retrieved: 2026-08-21.
- [S13] Primary. BLADE benchmark project. URL: https://blade-bench.github.io/ Publication date: 2024. Retrieved: 2026-08-21.
- [S14] Primary. ScienceAgentBench repository. URL: https://github.com/OSU-NLP-Group/ScienceAgentBench Publication date: 2025; repository update noted 2026-04-30. Retrieved: 2026-08-21.
- [S15] Primary. CORE benchmark project. URL: https://crab.cs.princeton.edu/core-website/ Publication date: no date. Retrieved: 2026-08-21.
- [S16] Primary, unreviewed software benchmark. RealDataAgentBench repository. URL: https://github.com/patibandlavenkatamanideep/RealDataAgentBench Publication date: no date; repository citation year 2026. Retrieved: 2026-08-21.
- [S17] Primary. Benchmark Dataset Generation and Evaluation for Excel Formula Repair with LLMs, Microsoft Research. URL: https://www.microsoft.com/en-us/research/publication/benchmark-dataset-generation-and-evaluation-for-excel-formula-repair-with-llms/ Publication date: 2025-08. Retrieved: 2026-08-21.