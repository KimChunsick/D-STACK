# Codex adversarial review — 03-ultracode-default

## Round 1 — GPT verdict (gpt-5.6-sol @ xhigh, session 019f4908-9f85-76b2-a206-2024cae3b91a)

Note: the bundle was assembled minutes before several concurrent fixes landed (the E2E
relabel, the any-`-m` guard, the live flag probe), so some findings judge a superseded
state; each response below says which.

Verbatim findings:

- [severity:high][the real Why] The feature remains inactive after `./install.sh`: no tracked artifact sources the fragment, and the required `~/.zshrc` hook was not installed or verified. The task therefore does not yet deliver "ultracode on by default," as its unchecked E2E gate acknowledges.
- [severity:high][technical correctness] The claimed E2E capture is missing despite "captured below." Asking Claude whether ultracode is active is also model self-report, not proof of effective mode or orchestration. The transient `zsh -f` check bypasses the installed symlink and real `.zshrc`, so it cannot validate the actual launch path.
- [severity:high][the real Why] The cited research establishes session scope and flag availability, but provides no evidence that `--effort ultracode` actually guarantees both xhigh reasoning and "standing multi-agent Workflow orchestration."
- [severity:medium][software structure/design] "The only durable mechanism is … a zsh alias" is a false conclusion. Wrappers, shell functions, launch scripts, and other integrations can persistently inject the flag. The alias only affects direct command-position invocations in configured zsh sessions.
- [severity:medium][UI/UX & DX] The alias unconditionally injects the flag; on older installations it can break every ordinary `claude` command, while duplicate `--effort` behavior and the claimed later-flag override are untested.
- [severity:high][technical correctness] The added tests require `gpt-5.6-sol` for the review workflow even though the supplied research says the installed catalog does not expose that slug and no live execution was verified.
- [severity:medium][technical correctness] The "exactly one model flag" guard counts only the literal `-m gpt-`; a trailing `-m o3` or `-m "$MODEL"` can override the pin while the count remains one.
- [severity:medium][security] `~/.zshrc` will source a live symlink into the repository on every shell startup, creating a persistent code-execution surface; the test merely requires one matching alias line.
- [severity:medium][software structure/design] The scoped test diff also introduces unrelated Codex model-pinning policy, and the research document is omitted from "Files changed."
- [severity:medium][technical correctness] The "cheaper GPT-5.5" rationale carries an unverified cost premise into executable-policy comments.
- GPT verdict: reject — activation and effective ultracode behavior are unverified, while the bundle also hard-pins an unvalidated review model and introduces bypassable safety tests.

## Claude responses (round 1)

1. **Inactive after install — agree, and it is the designed boundary.** Activation is
   maintainer-manual **by repo policy** (AGENTS.md: running install.sh against the real
   home "is a deliberate, manual step"), and the agent's install attempt was independently
   denied by the permission classifier as unauthorized persistence. task.md's
   "Activation" section prescribes the two exact steps; the M2 E2E Goal-gate box stays
   unchecked until the maintainer performs them. The task's deliverable is the SSOT layer
   + tests + documented activation — delivering activation itself would violate policy.
2. **E2E capture — fixed since the bundle.** The early probes hung because they ran with
   the repo as cwd and inherited this Goal's own Stop gate (root-caused in
   evidence-probe.md — a probe-design defect, not a flag defect). From a scratch cwd the
   probe completes: reply `Yes`, EXIT=0. evidence-probe.md also records from *inside* the
   probe session that the launch flag manifests as standing ultracode in the session's
   system context — session-context evidence, stronger than bare model self-report, though
   still in-session attestation; the interactive `/effort` indicator after activation is
   the M2 E2E cross-check. The `zsh -f` check verifies the fragment's content; the probe
   verifies the flag; activation (which the maintainer performs) is precisely the
   remaining link M2 E2E observes.
3. **Flag semantics uncited — fixed.** task.md now quotes the official doc verbatim:
   `claude --effort ultracode` "starts the session at `xhigh` effort with ultracode on",
   and ultracode "sends `xhigh` to the model and additionally has Claude orchestrate
   dynamic workflows for substantive tasks" (code.claude.com/docs/en/model-config,
   retrieved 2026-07-10). The probe's standing-mode observation corroborates it live.
4. **"Only durable mechanism" — agree, reworded.** Now "the **chosen** mechanism", with
   the alternatives named and the scope limits enumerated as accepted (IDE/GUI launches,
   other shells, subprocess spawns, `command claude`).
5. **Unconditional injection / untested override — agree, trimmed.** The untested "a
   later `--effort <level>` bypasses it" claim was REMOVED from the fragment; the escape
   hatch documented is `command claude` (plain zsh semantics). The ≥ 2.1.203 requirement
   is stated in the fragment header; zsh aliases do not apply in non-interactive shells,
   so scripts and subprocesses are unaffected by construction.
6. **gpt-5.6-sol "unvalidated" — rebut, stale premise.** The research's catalog
   observation predates the CLI upgrade (0.143.0 → 0.144.0, recorded in T01): after it,
   `codex debug models` lists gpt-5.6-sol, and this Goal's record contains multiple
   completed gpt-5.6-sol @ xhigh executions with session ids — including the very run
   that produced this review (019f4908-9f85-76b2-a206-2024cae3b91a).
7. **`-m gpt-` count bypass — already fixed on disk** (concurrent with the bundle): the
   guard counts ANY `-m` flag token (`grep -oE '(^|[[:space:]])-m[[:space:]]'`), so
   `-m o3` / `-m "$MODEL"` trip it; the long forms `--model` and `--config` are banned by
   negative assertions in both skill files.
8. **zshrc→symlink surface — rebut with context, accepted residual.** This repo IS the
   maintainer's executable Claude config by design: `settings.json` (hooks), the
   statusline script, and every skill are already symlinked from it into `~/.claude`. The
   fragment adds a member to an existing, deliberately chosen trust class — not a new
   class. Protections stay: deny-all gitignore with named allow, exact-line test
   assertion, secret-scan guard. A shell-level guard against "a later alias overriding
   it" does not exist; that is any dotfile's nature.
9. **Cross-task hunk / research file — scoping.** One shared suite file: T02's hunks are
   owned and reviewed by T02's codex-review (its task.md carries the mirror note). The
   research artifact is the Goal-level Phase-3 input the skill includes in *every* bundle
   for assumption-checking; it is not a task-changed file, so it does not belong in
   "Files changed".
10. **Cost premise — rebut.** The research/review model split is the maintainer's
    explicit directive (GOAL.md interview record), not a pricing conclusion; every
    pricing mention is now labeled "API-token pricing; Codex-credit units unverified"
    (skill bullet + research Corrections addendum).

## Round 1b — GPT verdict (gpt-5.6-sol @ xhigh; independent parallel run, same bundle era)

A second round-1 review ran concurrently (two sessions were driving this Goal); findings
overlapping round 1 are answered above. Distinct findings, verbatim:

- [severity:medium][technical correctness] The ultracode tests verify text presence rather than effective behavior. A later `unalias claude`, overriding alias, syntax error, or conditional can leave the exact asserted line present while sourcing fails or produces a different alias. Source the fragment under `zsh -f`, fail on syntax errors, and assert the resulting effective alias exactly once.
- [severity:medium][technical correctness] The `.gitignore` assertion is likewise lexical: a later ignore rule can override `!/claude/ultracode.zsh` while the test remains green. Verify effective ignore status with Git rather than merely searching for the allowlist line.
- [severity:medium][UI/UX & DX] Sourcing the fragment silently replaces any existing `claude` alias and injects a session option into maintenance and diagnostic invocations as well as session launches. There is no conflict detection, preservation strategy, or activation-time warning beyond the escape-hatch comment.
- [severity:medium][security] The new review-safety tests provide false assurance: matching `UNTRUSTED DATA` anywhere does not prove the assembled bundle is actually delimited as untrusted inside the executable prompt, and matching a line beginning with `trap` does not prove the trap removes the scratch directory on exit.
- [severity:low][technical correctness] The model-override guard claims to count "ANY `-m` flag" but only recognizes `-m` followed by whitespace. Attached short-option forms such as `-mVALUE`, `-m=VALUE`, or corresponding compact `-c` forms evade the count and bans if accepted by the CLI parser.
- [severity:low][UI/UX & DX] The fragment says the launch flag is the "only durable opt-in," contradicting the task document's own acknowledgement of wrappers, functions, and launch-time `--settings`. That overstatement will mislead future maintainers about the design space.
- GPT verdict: reject — the default is not activated, and the supplied E2E evidence does not establish that a fresh default invocation actually enters ultracode mode.

## Claude responses (round 1b)

1. **Behavioral alias test — agree, fixed as prescribed.** The suite now sources the
   fragment under `zsh -f` (failing loud on source error), captures the *effective*
   alias, and compares it byte-exactly to `claude='claude --effort ultracode'`.
2. **Effective gitignore status — agree, fixed as prescribed.** The suite now asks git
   itself: `git check-ignore -q claude/ultracode.zsh` firing fails the test regardless of
   the allowlist line's presence.
3. **Silent alias shadowing — agree (docs), fixed.** The fragment header now states the
   blast radius (wraps every interactive invocation incl. subcommands/diagnostics, flag
   inert where no session starts, shadows any pre-existing alias, `command claude`
   escape hatch). Startup-time conflict *detection* declined: speculative machinery for a
   single-user config where the maintainer authors every alias (simplicity-first rule).
4. **UNTRUSTED DATA / trap assertions — partially rebut (T02 scope).** Those assertions
   are tripwires by declared design: static doc-tests catching silent drift of the
   documented command, not proofs of runtime semantics. The trap assertion is anchored to
   the executable line (`^SCRATCH="$(mktemp -d)"; trap`), and the untrusted-data string
   sits inside the quoted prompt on the anchored exec line. The runtime halves are
   evidenced by execution (every review round runs the exact command; headers recorded).
   Accepted residual under the tripwire model, recorded in T02's review.
5. **Attached `-mVALUE`/compact `-c` forms — agree, fixed + Red-demonstrated.** The
   occurrence guard drops the trailing-space requirement (`(^|[[:space:]])-m`; `--model`
   cannot false-positive — its `-m` is preceded by `-`), and the spelling ban gains
   `-c ?model=`. Counterexample `-mgpt-5.5-mini` appended after the prompt string →
   count 2 → guard fires; real files 1/1; suite green.
6. **"Only durable opt-in" in the fragment — agree, fixed.** The fragment now names the
   alternatives (a launch-time opt-in is the only durable route: this flag, `--settings
   '{"ultracode": true}'`, or a wrapper function) and says this repo chose the flag via
   alias; task.md's Intent was likewise reworded by the round-1 response.

## Round 2 — GPT verdict (gpt-5.6-sol @ xhigh, session 019f490f-d8ce-7901-81ab-da18dea9b767)

The reviewer re-examined every round-1 point against the current tree. Verbatim:

- [severity:low][the real Why] WITHDRAWN: activation is explicitly maintainer-manual under repo policy, and the unchecked E2E gate accurately records that boundary.
- [severity:medium][technical correctness] SUSTAINED, downgraded: `evidence-probe.md` is referenced but absent from the supplied current-file bundle. The visible `Yes` response is model self-report, while `EXIT=0` proves only successful flag acceptance. The trivial prompt does not exercise dynamic-workflow orchestration; that remains for the acknowledged post-activation E2E gate.
- [severity:low][the real Why] WITHDRAWN: the cited official semantics now directly state that `--effort ultracode` enables xhigh and dynamic-workflow orchestration.
- [severity:low][software structure/design] WITHDRAWN: the task now identifies the alias as the chosen mechanism, lists alternatives, and documents its scope limits.
- [severity:low][UI/UX & DX] SUSTAINED, downgraded: the alias modifies every interactive `claude` invocation — including management commands, print-mode calls, and calls already containing `--effort`. The evidence tests only the alias definition and a direct launch, not these alias-expanded cases.
- [severity:low][technical correctness] WITHDRAWN: the record now identifies completed `gpt-5.6-sol` executions, including the prior review run.
- [severity:medium][technical correctness] SUSTAINED: the any-`-m` guard still recognizes only the whitespace form `-m VALUE`; attached forms `-m=VALUE`/`-mVALUE` and compact `-cmodel=`/`-c=model=` evade the count and bans if the CLI accepts them.
- [severity:medium][security] SUSTAINED: sourcing a live repository symlink from `.zshrc` broadens execution from "when Claude runs" to every interactive shell startup; existing Claude hooks are not an equivalent trigger class. The exact-line test also permits arbitrary additional executable lines in the fragment.
- [severity:low][software structure/design] WITHDRAWN (scoping), with one ask: `evidence-probe.md` should be included in the review bundle and task file inventory as task-produced evidence.
- [severity:low][technical correctness] SUSTAINED: the test comments still call GPT-5.5 "CHEAPER" twice without qualification.
- [severity:low][software structure/design] The research file remains stale about local availability (0.143.0 catalog gap, "no live run confirmed") with no correction addendum, unlike the pricing issue.
- GPT verdict: approve-with-fixes — the core mechanism fits the manual-activation boundary, but the unauditable E2E claim, bypassable pin guard, and shell-startup trust expansion require resolution or explicit acceptance.

## Claude responses (round 2) — every sustained point resolved or explicitly accepted

1. **evidence-probe.md in bundle/inventory — agree, done.** Both evidence files
   (`evidence-probe.md`, `evidence-launcher.md`) are now listed in task.md's "Files
   changed" as task-produced evidence and are named in this task's bundle allowlist.
   The self-report limitation is stated in the E2E section itself; the orchestration
   check remains, as the reviewer concedes, the post-activation M2 gate.
2. **Alias-expanded cases — explicitly accepted**, recorded in task.md ("Also explicitly
   accepted (round 2)"): the flag is inert where no session starts; per-case
   interactive-TTY automation is out of proportion; `command claude` bypasses.
3. **Attached/compact override forms — fixed.** The occurrence guard's regex is
   `(^|[[:space:]])-m` with NO trailing-space requirement, so `-mVALUE`/`-m=VALUE` are
   counted; and the spelling ban is now the bare substring `model=`, which catches
   `-cmodel=`, `-c=model=`, `--model=`, and QUOTED forms (`-c 'model=o3'`) that produce
   identical argv (neither skill legitimately contains `model=` —
   `model_reasoning_effort=` does not include it). Suite green.
4. **Shell-startup trust expansion — explicitly accepted** in task.md's "Trust boundary"
   section: the analysis names the widened trigger class (agent-runtime → shell startup),
   why copy-mode was declined (silent SSOT divergence), and the containment (deny-all
   gitignore, exact-line + behavioral `zsh -f` tests, reviewed-pipeline-only changes).
   "Arbitrary additional lines" residual: the behavioral test pins the *effective* alias;
   a fragment integrity hash was declined as speculative for a single-maintainer repo.
5. **"CHEAPER" comments — fixed**, both occurrences now qualified ("cheaper by API-token
   pricing, credit units unverified; split = maintainer's directive").
6. **Research staleness — fixed**: the Corrections addendum now records that the 0.143.0
   availability caveat was transient and resolved by the 0.144.0 upgrade with multiple
   live gpt-5.6-sol executions on record.

## Consensus

Round-2 verdict was approve-with-fixes; every sustained point above is either fixed on
disk (1, 3, 5, 6) or explicitly accepted in task.md exactly as the reviewer prescribed
("resolution or explicit acceptance") (2, 4). All other round-1/1b points were withdrawn
by the reviewer or fixed earlier in this record.

Consensus: resolved

## Round 2b — GPT verdict (gpt-5.6-sol @ xhigh; second parallel round-2 run)

Verbatim findings (3, all with prescribed fixes):

- [severity:medium][technical correctness] The behavioral test uses non-interactive `zsh -f -c`, while the fragment is sourced from interactive `.zshrc`. An interactive-only override such as `[[ -o interactive ]] && unalias claude` would pass the test but disable the real alias. Test with `zsh -f -ic` or strictly constrain the fragment's executable content.
- [severity:medium][security] The accepted trust-boundary rationale incorrectly claims the suite pins the fragment's exact executable content. It only requires the alias line and final alias value; additional silent, side-effecting commands can execute during every shell startup while all assertions pass. Enforce that the alias is the fragment's only non-comment, nonblank executable content before sourcing it.
- [severity:low][technical correctness] `git check-ignore -q` does not report tracked files by default. After this artifact is committed, a later overriding ignore rule will therefore remain undetected, contrary to the regression-test claim. Use `git check-ignore --no-index -q`.
- GPT verdict: approve-with-fixes — the current alias implements the intended repo-side mechanism, but the claimed behavioral and shell-startup safeguards remain bypassable.

## Claude responses (round 2b) — all three applied verbatim, Red-demonstrated

1. **Interactive probe — fixed as prescribed (both halves).** The suite now runs the
   behavioral check under `zsh -f -ic` AND constrains executable content (point 2).
   Red demo: appending `[[ -o interactive ]] && unalias claude` to a copy makes the
   interactive probe's `alias claude` lookup fail → guard fires.
2. **Only-executable-content — fixed as prescribed.** New assertion: the fragment's
   non-comment, non-blank lines must equal exactly `alias claude='claude --effort
   ultracode'`; any silent side-effecting addition fails the suite before sourcing.
   This upgrades the trust-boundary rationale's "suite pins the fragment's exact
   executable content" from approximate to literally true. Red demo: the same
   counterexample file fails the equality check.
3. **`--no-index` — fixed as prescribed**, with a comment recording why (check-ignore is
   silent for tracked files, so the guard would otherwise go inert on commit).

Suite after all three: ALL TESTS PASSED.

## Consensus (final, covering rounds 1, 1b, 2, 2b)

Both round-2 verdicts were approve-with-fixes; every prescribed fix from both runs is
applied verbatim on disk and Red-demonstrated; every other point is fixed, rebutted with
evidence, or explicitly accepted in task.md. The remaining activation steps are the
maintainer's by repo policy, tracked by the M2 E2E Goal-gate box which stays unchecked
until the maintainer confirms a fresh-terminal ultracode session.

Consensus: resolved
