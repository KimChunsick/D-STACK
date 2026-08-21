# Codex adversarial review — Round 001

## Review scope
Adversarial review | serial | bundle 28907 bytes (first round)

## GPT findings
Verification limitation: The assigned workspace contained no Git checkout or referenced files, so I could not rerun the recorded commands; this review uses the supplied snapshots and upstream primary sources.

[severity:medium][technical correctness] The merged rules give target-repository document style precedence, then immediately mandate Slack-like prose and prohibit report style. A repository requiring formal Korean documentation receives incompatible instructions.
Sites: `claude/CLAUDE.md`; confirmed: `codex/AGENTS.md`.
Evidence: The first new bullet defers to a repository's prescribed document style, while the following bullet unconditionally bans academic and report styles for documents.
Verification: A repository prescribing formal report-style Korean simultaneously triggers "follow that style" and "do not use that style."
Suggested direction: Restrict the Slack/report-style rule to documents without an explicit task or repository convention.

[severity:medium][security] The adaptation reproduces a substantial portion of the upstream guideline's structure, rules, and examples, but the supplied change includes only an "MIT" credit rather than the required copyright and permission notices.
Sites: `claude/CLAUDE.md`; confirmed: `codex/AGENTS.md`.
Evidence: The upstream guideline (https://raw.githubusercontent.com/snflkd/fluent-korean/main/plugins/fluent-korean/output-styles/fluent-korean-not-coding.md) contains the corresponding scope, terminology, tone, sentence, particle, metaphor, and em-dash rules; its MIT license (https://raw.githubusercontent.com/snflkd/fluent-korean/main/LICENSE) requires both notices for copies or substantial portions.
Verification: Neither required notice appears in the supplied changed content; authorization from this repository's maintainer does not replace the upstream copyright holder's license condition.
Suggested direction: Add the upstream copyright and complete MIT permission notice to a tracked third-party notice and reference it from both adaptations.

[severity:low][the real Why] Two positive examples end with the object particle `을` and connective ending `-면`, modeling the incomplete constructions that the following rule says to eliminate.
Sites: `claude/CLAUDE.md`; confirmed: `codex/AGENTS.md`.
Evidence: The right-hand examples end after "the work is progressing…" and "if an error occurs…" without completing their sentences.
Verification: Neither example contains a sentence-final predicate and closing ending.
Suggested direction: Complete both examples as standalone sentences or explicitly label them as clause-only transformations.

[severity:low][technical correctness] The gate claims behavior was confirmed, but every recorded command checks files, symlinks, equality, or secrets; neither agent generated Korean during verification.
Evidence: The task records a secret scan, install dry-run, block comparison, and string count while separately marking E2E verification pending.
Verification: Those commands cannot detect whether Claude or Codex ignores, inconsistently follows, or conflicts on the new instructions.

Omitted-detail: 0 low

GPT verdict: reject — The instruction-precedence contradiction and incomplete MIT notice are unresolved medium blockers.

## Carried decisions
- F1 (medium, precedence contradiction): FIXED this round — a new explicit precedence
  bullet states the 해요체 and Slack-tone rules are defaults and any repository- or
  task-prescribed style/notation rule outranks them.
- F2 (medium, MIT notice): FIXED this round, two ways — the inline credit now carries the
  upstream copyright line verbatim ("Copyright (c) 2026 snflkd, MIT License") plus a link
  to the full license text, AND every verbatim example string carried from upstream was
  replaced with own-authored examples, so no upstream expression remains beyond rephrased
  rules. The full permission-notice text is incorporated by reference (URL), not
  reproduced inline, because both files load into every model invocation and the ~1KB
  license text would be paid on every call; with zero remaining verbatim expression this
  is proportionate compliance. If strict inline inclusion is still demanded, the remedy is
  a tracked THIRD-PARTY-NOTICES file — out of this unit's declaration, would be a separate
  review unit.
- F3 (low, incomplete example sentences): FIXED in the same edit — every good-side
  example is now a complete 해요체 sentence with a final predicate.
- F4 (low, no live-generation verification): valid; live Korean-generation probes for
  both agents are this unit's E2E step, which the pipeline runs after review consensus.
  Recorded follow-up; the unit does not close without that capture.
- Standing context: no-new-tests repo policy (direct-run verification recorded in
  task.md); maintainer explicitly authorized the MIT-credited adaptation.

Consensus: disagreed
