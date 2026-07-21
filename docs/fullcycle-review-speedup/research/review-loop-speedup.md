## Needed info

- The practical target is not “make the reviewer weaker”; it is to reduce avoidable serial discovery. The relevant failure pattern is adjacent defects in the same class. Evidence from PSP-style personal reviews supports a pre-external-review pass when it is structured, measured, and based on recurring personal defect classes: Kemerer & Paulk found review rate significantly affected defect-removal effectiveness in PSP data, and Vallespir & Nichols report personal reviews removing 60%+ of defects before unit test. Sources: S1, S2.

- A mandatory builder self-review should be scoped as a defect-class sweep, not a generic checklist. Prior inspection literature distinguishes checklist-based reading from stronger procedural/perspective/usage-based reading. Checklists help focus attention, but studies also show mixed or negative evidence for checklist benefit when the checklist is generic or not tied to actual defect data. Sources: S3, S4, S5, S6.

- Dropping “suggested direction + small illustrative example” is not clearly supported. Human code-review guidance and empirical comment research favor actionable, explained feedback, but not full reviewer-owned design. Google’s code-review guide explicitly recommends balancing problem statements with direct guidance; Widyasari et al. found code-review comments often contain suggestions without explanations and studied explanation types as a way to reduce back-and-forth. Sources: S7, S8.

- LLM-agent evidence cuts both ways. Google’s ML-suggested edit system reports that authors applied ML-suggested edits for 7.5% of all reviewer comments at Google scale, suggesting fix suggestions can save time when quality-filtered. OpenAI’s CriticGPT result suggests AI-assisted critique can improve human review quality. But CR-Bench and “Are LLMs Reliable Code Reviewers?” show that more exhaustive/fix-oriented prompting can lower signal-to-noise or increase false rejections. Sources: S9, S10, S11, S12.

- Upfront design review is most useful when the task has architecture, API, persistence, cross-cutting security, partitioning, idempotency, or invariants that cannot be cheaply inferred from a final diff. Classic inspection and architecture-evaluation literature supports earlier defect discovery for design-level issues; modern code-review data shows design discussions are rare in ordinary code review, which argues for a separate design checkpoint only when design risk is present. Sources: S3, S13, S14, S15.

- For the public config repo meta-test deletion: `.gitignore` is only a prevention layer for untracked files; Git’s own docs say gitignore specifies intentionally untracked files and has sharp pattern-negation constraints. It does not verify future edits or protect files already tracked. GitHub public secret scanning and push protection are valuable backstops, but have documented pattern, size, legacy-token, pair-detection, and bypass limitations. Sources: S16, S17, S18, S19.

- Lighter mitigations exist: keep a tiny targeted guard that runs only when `.gitignore`, installer, hook, skill pins, or symlink logic changes; add Gitleaks or TruffleHog in pre-commit/CI; enable GitHub push protection and secret scanning; rely on GitHub user-level push protection for public pushes only as a backstop, not as the only control. Sources: S18, S20, S21, S22.

- Closure rules are directionally sound: low-severity items should not force new rounds, repeated concerns should cite prior disposition, and loop termination should be explicit. Current agent-pattern guidance warns that critic/generator loops need clear termination conditions to avoid runaway cost; CR-Bench shows aggressive review can trade recall for noise. Sources: S11, S23.

## Opposing views

- Against mandatory self-review: self-review can reinforce the builder’s own blind spots. LLM self-correction literature is especially skeptical when there is no reliable external feedback: Kamoi et al. found little evidence that prompted LLM self-correction works broadly except in tasks suited to self-correction or with external feedback; Huang et al. found LLMs can degrade after intrinsic self-correction. Sources: S24, S25.

- Against checklists: Hatton’s industrial analysis of 308 inspections found no evidence that checklists significantly improved inspections; individual variation dominated. Thelin et al. found usage-based reading more effective for critical faults than checklist-based reading. So a recurring checklist can ossify into ritual unless updated from actual reviewer findings. Sources: S4, S5.

- Against removing reviewer guidance: comments that include code suggestions may be more useful. The 2026 replication-package description for “Go Home Copilot, You’re Drunk” reports inline code suggestion as the strongest predictor of usefulness, with rules/examples/benefit explanations modestly increasing adoption. Google’s ML edit suggestions also show measurable adoption. Sources: S26, S9.

- Against keeping reviewer guidance: prescriptive repair requests can anchor the builder and add latency. The 2026 overcorrection paper reports that prompts requiring explanations and proposed corrections can increase LLM misjudgment rates. CR-Bench reports Reflexion-style review improved recall but reduced signal integrity versus single-shot review in its setup. Sources: S12, S11.

- Against upfront design review: if most defects are Unicode, boundary, test-isolation, sanitization-path, or idempotency edge cases, a design meeting may not expose them. Zanaty et al. found design-related comments are rare in code review, and CR-Bench notes some defects require context or execution not fully present in a PR diff. Sources: S13, S11.

- Against deleting meta tests: secret exposure is high-impact even in a single-maintainer public config repo, and GitHub reported over 1 million leaked secrets detected in public repositories in the first eight weeks of 2024. GitHub protections are helpful but intentionally incomplete. Sources: S19, S18.

- Against keeping the full meta suite unchanged: if tests run on every small config edit and are slow/noisy, they can create maintenance drag. The better opposing position is not “delete all guards”; it is “replace broad always-on tests with narrow invariant checks and secret scanning.” Sources: S20, S21, S22.

## For the goal

- Intervention 1 is the strongest. A mandatory builder pass using recent defect classes maps directly to the observed failure mode: one finding exposing adjacent instances. PSP and inspection evidence supports personal review before later validation, especially when the checklist is derived from actual defect history. Sources: S1, S2, S6.

- The self-review pass should require artifacts: “defect class swept,” “files/paths checked,” “tests/probes added or updated,” and “why this class cannot recur in adjacent scope.” This is an inference from PSP’s measured/checklist-based personal review model and the loop’s known failure pattern. Sources: S1, S2.

- Intervention 2 should be modified, not fully adopted. Keep severity, evidence, and verification mandatory. Make suggested direction optional and forbid large fix recipes. This preserves actionable review while reducing anchoring and reviewer latency. Sources: S7, S8, S11, S12.

- Intervention 3 should be conditional. Add a short design-review gate only when the task touches cross-cutting invariants, API contracts, auth/security boundaries, persistence/logging consistency, cursor/idempotency semantics, partitioning, rendering boundaries, or multi-path sanitization. Do not require it for small localized implementation tasks. Sources: S3, S13, S14, S15.

- Intervention 4 should not be “delete all meta tests because tests are slow.” A sound version is “shrink and target the suite.” Keep the secret-trackability invariant and run it only on relevant files; use Gitleaks/TruffleHog/GitHub secret scanning as backstops. Sources: S16, S18, S20, S21.

- The existing closure rules are aligned with cost control: classify low-severity work as follow-up, close approve-with-fixes when only non-blockers remain, and avoid re-litigating disposed findings. This directly counters loop-thrash and matches agent-loop advice to define exit conditions. Sources: S23, S11.

## Against the goal

- There is no direct evidence that these exact interventions reduce rounds in a two-frontier-model adversarial code-review loop with fixed reviewer model/reasoning effort. Most evidence is from human review, PSP, classic inspections, or early LLM-agent studies. Generalization is plausible but unverified.

- Structured self-review can reduce first-round defects but may not reduce adversarial rounds if the reviewer’s value is precisely finding what the builder’s representation missed. Without external or executable feedback, LLM self-correction is unreliable. Sources: S24, S25.

- Removing suggested direction could increase ambiguity and cause more rebuttal/fix rounds, especially for findings involving non-obvious invariants. The evidence on useful review comments favors clarity, actionability, and sometimes examples; fully terse findings may save reviewer minutes while costing builder/reviewer rounds. Sources: S7, S8, S26.

- Keeping suggested direction also has real risk: LLM reviewers may over-prescribe and create false constraints. The overcorrection paper is directly relevant because it says explanation/repair requirements can worsen false rejection. Sources: S12.

- Upfront design review can become ceremony if the dominant defects are implementation-level. The design-review gate needs a trigger list and a timebox, otherwise it moves latency earlier without reducing total rounds.

- Dropping the meta suite has asymmetric downside. A single accidental secret commit to a public repo can require rotation, history cleanup, and incident response. GitHub protections reduce risk but are incomplete by design and can be bypassed or miss unsupported/legacy/large-push cases. Sources: S18, S19.

## Unverified

- I could not verify any controlled study measuring “number of external adversarial LLM review rounds” before/after mandatory builder self-review.

- I could not verify data specific to GPT-5.6 Sol at fixed xhigh reasoning effort.

- I could not verify whether the maintainer’s current meta suite is actually slow enough to dominate change latency; that requires local timing data.

- I could not verify the exact false-positive/false-negative behavior of GitHub secret scanning for the maintainer’s specific secret file names and nested-unknown probes.

- I could not verify whether “Go Home Copilot, You’re Drunk” has a peer-reviewed paper beyond the 2026 Zenodo/metadata description found.

## Sources

S1. Primary. Kemerer & Paulk, “The Impact of Design and Code Reviews on Software Quality,” IEEE TSE, July 2009. Retrieved 2026-07-22. https://www.researchgate.net/publication/260648247_The_Impact_of_Design_and_Code_Reviews_on_Software_Quality_An_Empirical_Study_Based_on_PSP_Data

S2. Primary. Vallespir & Nichols, “Quality is Free, Personal Reviews Improve Software Quality at No Cost,” March 2016. Retrieved 2026-07-22. https://asq.org/quality-resources/articles/quality-is-free-personal-reviews-improve-software-quality-at-no-cost?id=5c8dc2072927459fa0f694a26349ffb5

S3. Primary classic. Fagan, “Design and Code Inspections to Reduce Errors in Program Development,” 1976. Retrieved 2026-07-22. https://www.ifsq.org/work-fagan-1976.html

S4. Primary. Hatton, “Testing the Value of Checklists in Code Inspections,” August 2008. Retrieved 2026-07-22. https://www.researchgate.net/publication/3249530_Testing_the_Value_of_Checklists_in_Code_Inspections

S5. Primary. Thelin, Runeson, Wohlin, “An experimental comparison of usage-based and checklist-based reading,” 2003. Retrieved 2026-07-22. https://portal.research.lu.se/en/publications/an-experimental-comparison-of-usage-based-and-checklist-based-rea/

S6. Primary. Laitenberger et al., “An Internally Replicated Quasi-Experimental Comparison of Checklist and Perspective-based Reading of Code Documents,” 1999. Retrieved 2026-07-22. https://publica.fraunhofer.de/entities/publication/eb2a71d4-2bfc-43c8-a5bf-8a03f643c016

S7. Primary industry guidance. Google Engineering Practices, “How to write code review comments,” no date. Retrieved 2026-07-22. https://google.github.io/eng-practices/review/reviewer/comments.html

S8. Primary. Widyasari et al., “Explaining Explanations: An Empirical Study of Explanations in Code Reviews,” 2025-07-01. Retrieved 2026-07-22. https://research.monash.edu/en/publications/explaining-explanations-an-empirical-study-of-explanations-in-cod/

S9. Primary. Google Research, “Resolving Code Review Comments with Machine Learning,” 2024. Retrieved 2026-07-22. https://research.google/pubs/resolving-code-review-comments-with-machine-learning/

S10. Primary. OpenAI, “Finding GPT-4’s mistakes with GPT-4,” 2024-06-27. Retrieved 2026-07-22. https://openai.com/index/finding-gpt4s-mistakes-with-gpt-4/

S11. Primary preprint. Pereira et al., “CR-Bench,” submitted 2026-03-10. Retrieved 2026-07-22. https://arxiv.org/abs/2603.11078

S12. Primary preprint. Jin & Chen, “Are LLMs Reliable Code Reviewers?” submitted 2026-02-28. Retrieved 2026-07-22. https://arxiv.org/abs/2603.00539

S13. Primary. Zanaty et al., “An Empirical Study of Design Discussions in Code Review,” 2018. Retrieved 2026-07-22. https://rebels.cs.uwaterloo.ca/confpaper/2018/10/10/an-empirical-study-of-design-discussions-in-code-review.html

S14. Secondary systematic review. Qureshi et al., “Evidence in software architecture,” 2013-04-14. Retrieved 2026-07-22. https://doi.org/10.1145/2460999.2461014

S15. Secondary review. “Lightweight Software Architecture Evaluation for Industry,” 2022. Retrieved 2026-07-22. https://www.mdpi.com/1424-8220/22/3/1252

S16. Primary docs. Git, “gitignore Documentation,” last updated 2026-06-29. Retrieved 2026-07-22. https://git-scm.com/docs/gitignore

S17. Primary docs. GitHub Docs, “Push protection,” no date. Retrieved 2026-07-22. https://docs.github.com/en/code-security/concepts/secret-security/push-protection

S18. Primary docs. GitHub Docs, “Secret scanning detection scope,” no date. Retrieved 2026-07-22. https://docs.github.com/en/code-security/reference/secret-security/secret-scanning-scope

S19. Primary blog. GitHub, “Keeping secrets out of public repositories,” 2024-02-29. Retrieved 2026-07-22. https://github.blog/news-insights/product-news/keeping-secrets-out-of-public-repositories/

S20. Primary docs/repo. Gitleaks README, no date. Retrieved 2026-07-22. https://github.com/gitleaks/gitleaks

S21. Primary docs. TruffleHog, “Scanning in CI,” no date. Retrieved 2026-07-22. https://trufflesecurity.com/docs/scanning-in-ci

S22. Primary docs. pre-commit documentation, no date. Retrieved 2026-07-22. https://pre-commit.com/index.html

S23. Primary cloud architecture guidance. Google Cloud, “Choose a design pattern for your agentic AI system,” 2026-06. Retrieved 2026-07-22. https://docs.cloud.google.com/architecture/choose-design-pattern-agentic-ai-system

S24. Secondary critical survey. Kamoi et al., “When Can LLMs Actually Correct Their Own Mistakes?” TACL 2024. Retrieved 2026-07-22. https://aclanthology.org/2024.tacl-1.78/

S25. Primary. Huang et al., “Large Language Models Cannot Self-Correct Reasoning Yet,” ICLR 2024. Retrieved 2026-07-22. https://proceedings.iclr.cc/paper_files/paper/2024/hash/8b4add8b0aa8749d80a34ca5d941c355-Abstract-Conference.html

S26. Primary dataset/metadata. Zenodo/NLM listing for “Go Home Copilot, You’re Drunk,” March 2026. Retrieved 2026-07-22. https://doi.org/10.5281/zenodo.19251296

S27. Primary preprint. Zhong et al., “From Human-Centric to Agentic Code Review,” submitted 2026-07-14. Retrieved 2026-07-22. https://arxiv.org/abs/2607.13196