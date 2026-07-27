# Codex adversarial review — Round 001

## Review scope
Adversarial review | `REVIEW_MODE=serial` | bundle 41621 bytes
Allowlist: the review-unit folder, `claude/skills/codex-review/SKILL.md`.

## GPT findings

[severity:medium][technical correctness] The recovery rule falsely guarantees teardown after any session death; an uncatchable supervisor death can leave Codex running, so retrying creates a duplicate paid round.
Sites: `SKILL.md:156-159`; confirmed: `SKILL.md:288-302`.
Evidence: `dstack:902-909,1108-1116` explicitly places Codex in a separate process group and acknowledges that SIGKILL bypasses cleanup and can orphan it.
Verification: Killing the supervisor with SIGKILL cannot execute its trap; Step 2a treats the missing `exit` record as failed and directs an immediate retry without checking `.launch/child`.
Suggested direction: Limit the guarantee to catchable teardown and require a child-liveness check before retrying a capture with no terminal record.

[severity:medium][technical correctness] Step 3 simultaneously forbids and requires storing the maintainer response in the round file, leaving the orchestrator two incompatible sealing procedures.
Sites: `SKILL.md:360-362`; confirmed: `SKILL.md:409-413`, `SKILL.md:498-503`.
Evidence: Lines 360-362 and 498-503 require a separate, never-bundled `response-<NNN>.md`, while lines 409-413 say each round file contains one maintainer response.
Verification: The assembler sends the two newest round files whole, so following the latter instruction reintroduces maintainer prose into subsequent bundles and can violate the size ratchet.

[severity:medium][technical correctness] The skip gate scans untrusted bundle contents as though every matching line were assembler metadata, so legitimate material can prevent any review from launching.
Evidence: `SKILL.md:217-218` greps the entire bundle, while `assemble-review.sh:139-144,254-258` copies task/review snapshots and untracked content verbatim.
Verification: A payload line `--- docs/example.md (SKIPPED: illustrative text) ---` matched `SKIP_RE` with status 0, causing the recipe’s refusal branch despite no skipped file.
Suggested direction: Publish skip status through a separate manifest or exit channel that reviewed content cannot impersonate.

[severity:low][right-sized technology] Non-blocking follow-up: every round leaks its `mktemp -d` scratch directory.
Evidence: `SKILL.md:226` creates `SCRATCH`; its only later uses pass it to Codex, while cleanup covers only `.dstack/runs`.
Verification: Repository search found no trap or removal for `SCRATCH`; the removed launcher previously cleaned its scratch directory on exit.

Omitted-detail: 0 low

GPT verdict: reject — three concrete instruction failures can duplicate a live review, reintroduce excluded response prose, or refuse a valid bundle.

## Carried decisions
- Teardown guarantees are stated only for CATCHABLE termination. `SIGKILL` can orphan the launched
  round, so before relaunching a capture that has no terminal record, check `.launch/child` for a
  live pid or group — otherwise the retry pays for two concurrent rounds.
- The round file holds findings, bundle size, `## Carried decisions` and the consensus line, and NO
  maintainer response. Every place that says otherwise is a defect: three sites disagreed at once.
- The skip check iterates the ALLOWLIST with a fixed-string match per path. Scanning the bundle for
  the marker shape refuses valid bundles, because the bundle carries documents that quote the
  marker — demonstrated twice now.
- The assembler publishes skip status only inside the bundle it emits, so content it copies can
  still impersonate a marker for one of your own allowlisted paths. Closing that needs a separate
  channel from `assemble-review.sh` and is a FOLLOW-UP for that file's own review unit.
- A quoted array expansion `"${ALLOW[@]}"` is safe in both bash and zsh; the no-variables rule was
  only ever about an unquoted scalar.

Consensus: disagreed
