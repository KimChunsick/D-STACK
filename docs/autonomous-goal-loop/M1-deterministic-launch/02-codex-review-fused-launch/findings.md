# Finding ledger — 02-codex-review-fused-launch

The loop closes when a round raises nothing both NEW to this ledger and CONCRETE.

| id | round | severity | class | summary | status |
|---|---|---|---|---|---|
| F001 | 001 | medium | a guarantee stated wider than it holds | the recovery rule promised teardown after any session death, so a retry after an untrappable kill would pay for a second concurrent round | fixed — guarantee scoped to catchable termination; retry gated on a `.launch/child` pid/group liveness check |
| F002 | 001 | medium | one instruction contradicting another | Step 3 both forbade and required the maintainer response in the round file | fixed — template, §2 and the sealing sentence now agree; the sentence names its own former text |
| F003 | 001 | medium | a check that names the wrong entity | the skip gate scanned the whole bundle for the marker shape, so content quoting a marker refused a valid bundle | fixed — per-allowlist-path fixed-string check; reviewer's counterexample reproduced against the old form and passes the new one |
| F004 | 001 | low | resource leak | every round leaked its `mktemp -d` scratch directory | fixed — `trap 'rm -rf "$SCRATCH"' EXIT`, same fix applied in `codex-research` |
| F005 | 002 | medium (security) | an evaluator directive inside the reviewed payload | this task's own Deployment context still said "Out of scope: …" after the same class was fixed in `03`'s | fixed — reworded as filing information; it now names its former text |
| F006 | 002 | medium | two guards disagreeing about the same invariant | the retry fence read one launch record and treated absence as quiescence, while `rm-run` reads both and treats absence as unknown | fixed — fence mirrors `rm-run`; 7-case direct run recorded, reviewer's counterexample reproduced against the old form |
| F007 | 002 | medium | the same rule stated in several places, fixed in only some | the `description:` frontmatter still routed rebuttals into `codex-review-<NNN>.md` after three body sites were fixed | fixed — all four sites now agree |
| F008 | 002 | medium | one instruction contradicting another | §4 demanded closure with a concrete medium open while Step 4's consensus definition left no sealable value for it | fixed — Step 4 names the fourth disposition (accepted residual under a §4 closure); §4 points at it by name; the suggested direction was NOT taken, see `response-002.md` |
| F009 | in-use, between 002 and 003 | medium | a check that names the wrong entity (F003's class, second instance) | the PATHLESS skip marker was still matched as a substring over the whole bundle, so the recipe refused every bundle containing itself — hit for real while assembling round 003 | fixed — whole-line match (`grep -qxF`); reproduced (substring: 1 match, a `+`-prefixed diff line quoting the recipe; whole-line: 0) |
| F010 | in-use, between 002 and 003 | medium | a rule that cannot be satisfied | §1's bundle ratchet demanded round N ≤ N-1 from round 002, but `FULL_ROUNDS=2` means round 004 is the first round in which anything compacts — three units reported an unavoidable violation | fixed — the ratchet binds from round 004; 002/003 record the number without it counting as a process failure. Also corrected my own wrong annotation in `codex-review-002.md` (it said compaction starts at 003) |

| F011 | 003 | medium | a channel that cannot carry what it must | a DISPROVED finding's measurement lived only in the never-bundled response, so the next round re-raised it | fixed — §2 routes a disproof and its number into `## Carried decisions`; the argument stays in the response |
| F012 | 003 | medium | two authorities disagreeing | `codex/skills/adversarial-review/SKILL.md` still says one file per invocation+rebuttal and lists three consensus dispositions | precedence stated (this file governs the pipeline's closure semantics); the Codex-side edit is a follow-up for its own unit. Suggested direction NOT taken — see `response-003.md` |
| F013 | 003 | medium | a claim adopted without measuring it | "any CATCHABLE termination" was wrong, and so was the reviewer's counter-claim that XCPU/XFSZ/VTALRM bypass cleanup | fixed — measured table in the file; bash 3.2.57 runs the EXIT trap on fatal signals, so the real gaps are `SIGKILL` and `SIGPROF` only |
| F014 | 003 | medium | the same rule stated in several places, fixed in only some | §1's round-004 ratchet qualification was not carried to the closing procedure | fixed — both sites qualified |
| F015 | 003 | medium (security) | an evaluator directive inside the reviewed payload (third instance) | the round-002 *repair* told the evaluator how to classify a named dependency | fixed — Deployment context is a changed-files line only |
| F016 | 003 | low | a resource leak that survives its fix | the `SCRATCH` trap was EXIT-only, and the fence runs under zsh, which does not fire EXIT on a fatal signal | fixed — `trap … EXIT INT TERM HUP`, verified firing in both shells |
| F017 | in-use, at 003 | medium | a rule whose measurement was undefined | §4's "blocking count" never said open-at-end vs raised-this-round; the two readings disagree and one closes healthy loops | fixed — the count is what remains OPEN after the round's fixes, and the test applies from round 004 |

| F018 | 004 | medium | RESTATEMENT of F012 | the consensus/contract override with `adversarial-review`, raised again with the gate regex as evidence | not reopened — §3 restatement; disposition unchanged (this file governs; the Codex-side edit is a follow-up for its own unit) |
| F019 | 004 | medium | a handler that suppresses instead of terminating | `trap 'rm -rf …' EXIT INT TERM HUP` let the shell continue and return 0, deleting a live codex's cwd | fixed — handlers disarm EXIT, clean once, exit with the signal status; measured rc=143 / one CLEAN in both shells |
| F020 | 004 | medium | a guard whose matcher is wider than the thing it guards (third instance) | the per-path skip gate matched the path anywhere on a line and `SKIPPED:` anywhere later, so ordinary prose refused a valid bundle | fixed — `awk` complete-marker-line match, all comparisons literal; reproduced old REFUSE / new PASS on prose, both still REFUSE a real skip |
| F021 | 004 | medium (DX) | a rule fixed in the contract and not in what the contract invokes | `full-cycle` was corrected at T04 R1 while this file still said "nothing else in that call" | fixed — identical invariant in both, this file names `waits.external` as the source |

| F022 | 005 | medium | RESTATEMENT of F012/F018 (third time) | the `adversarial-review` contract disagreement, raised again | not reopened — disposition unchanged; the inconsistency is REAL and stays named in the follow-ups until that file is edited |
| F023 | 005 | medium | a rule that licenses shipping your own regression | §3 exempted "a variant of an already-recorded class in code a fix just introduced" — contradicting the discovery-time rule two paragraphs above, and demonstrated live by F024 | fixed — a regression introduced by a fix ALWAYS reopens; the exemption now covers only a restatement about code that has not moved. (The disposition-4 half is a restatement, unchanged.) |
| F024 | 005 | medium | the matcher is not what I thought it was (FOURTH instance) | `awk -v` decodes backslash escapes, so a path containing `\t`/`\n` had its skip marker silently missed | fixed — marker passed via `ENVIRON`; measured `path\to`/`path\new` MISSED by `-v`, caught by `ENVIRON`, real bundle unaffected |
| F025 | 005 | medium (DX) | a guarantee never actually held | the signal handlers cancel nothing — both shells defer a pending trap until the foreground command returns, so a completed round can be reported 143 | stated as a limit rather than implemented: `<run-dir>/exit` is the round's status, handlers do not clean up, stop the process group to cancel. Forwarding NOT built — see `response-005.md` |
| F026 | 005 | low | fixed in one sibling and not the other | the printed signal probe has its `$$` expanded by the invoking shell | fixed — single-quoted program, signal name as an argument; corrected form reproduces the table |

| F027 | 006 | medium | a repair that broke the sentence it repaired | the non-reopening rule was left syntactically broken with a dangling "a variant of an already-recorded class in / or an objection" | fixed — §3 is now stated positively: exactly two things do not reopen |
| F028 | 006 | medium (security) | substitutable text reaching the shell as source | allowlist paths and the launch label were embedded as shell source, so an unquoted literal globbed (`*/task.md` expanded to three entries in both shells) and a label was parsed as syntax | fixed — every allowlist entry quoted, `LABEL` assigned then passed as `"$LABEL"` |
| F029 | 006 | medium | a directive addressed to someone told to ignore it | the "THIS file governs" override claimed precedence over the elected review contract | fixed in round 007 — the claim is withdrawn; these rules govern the ORCHESTRATOR, and the Codex-side inconsistency stays a named follow-up |
| F030 | 006 | low | a repair that leaks on the path it was built for | the conditional cleanup leaks `$SCRATCH` on pre-launch failure and after a deferred wrapper signal when `dstack` already published `exit` | **partly fixed, partly an accepted residual.** The signalled path is fixed — handlers leave the gated EXIT trap ARMED; measured both shells, exit present → cleaned, absent → kept. The PRE-LAUNCH leak is NOT fixed and was never going to be: if `dstack` dies before publishing `exit`, quiescence is unknown and keeping the directory is the deliberate choice. Round 008 was right that the ledger said "fixed" for both halves |
| F031 | 006 | low | a claim adopted without measuring it | `SIGPROF` was described as untrappable in two files; it is catchable and merely bypasses bash's implicit EXIT-trap firing | fixed — corrected in both files, with the measured table |

| F032 | 007 | medium | a variable read before it exists | `RUNDIR="$RD"` executed before `RD` was defined, in a separate tool call where the assembly step's `$RD` no longer existed — so the trap tested `[ -e "/exit" ]`, always false | fixed — `LABEL` first, `RD` reconstructed from it, trap armed after |
| F033 | 007 | medium | prose contradicting the rule it introduces | Step 2a opened by calling any nonzero notification a failed round, moments after Step 2 established that the wrapper's status is not authoritative | fixed — `<run-dir>/exit` is the verdict in the prose too; a missing `exit` file is also not a pass |
| F034 | 007 | medium | a documented invocation its own validator rejects | the older-round resend recipe publishes `REVIEW_FULL_ROUND_IDS="1 3"` while the assembler split on commas only, so the published form died FATAL | fixed in `assemble-review.sh` — splits on commas AND whitespace; every empty-field rejection preserved and verified case by case |
| F035 | 007 | medium | F029, still open | the "THIS file governs" override | fixed with F029 |
| F036 | 007 | medium | a rule with no legal transition out of a real state | there was no budget for a post-seal reopening past the round cap — which this Goal hit: two units sealed AT the cap were reopened by `post-seal-rule` | fixed — §4's cap counts rounds SINCE the reopening and resets smaller (2 per-task, 3 per-milestone); the non-convergence window restarts with it |

| F037 | 008 | medium | a fix that stopped failing closed | accepting whitespace as a separator let IFS absorb an empty field: `1, ` came back as a quiet request for round 1 alone and ` ` as no request, both silently REDUCING what the reviewer asked for | fixed — the whole grammar is validated BEFORE splitting, so a `case` over the trimmed string sees what the split cannot; verified end to end, every documented form rc=0 and every malformed one FATAL |
| F038 | 008 | medium | a budget that can strand its own unit | the reset epoch had no legal closure: round 007 sealed `disagreed`, the budget was declared spent, and cap closure requires a positive seal | fixed — the budget starts when the rule does (it cannot retroactively govern earlier rounds) and cannot expire on a `disagreed` round, because §4 closure is an ACTION the cap OBLIGES you to take, not a permission that runs out |
| F039 | 008 | medium | the contract split, sixth raising | the orchestrator files rebuttals separately and auto-disposes concrete mediums; the elected reviewer contract requires one immutable exchange and explicit user disposition, and the gate checks only the consensus token | recorded follow-up — `codex/skills/adversarial-review/SKILL.md` is outside this unit's declaration. The gate's self-attestation scope is documented, not a new defect |
| F040 | 008 | medium | the file inventory did not match the change | the task listed only `SKILL.md` although the fix modified `assemble-review.sh` | fixed — the inventory names both and declares the scope expansion, with the reason it could not be split |
| F041 | 008 | low | a ledger claiming more than the fix | F030 was marked fixed although its pre-launch leak was explicitly accepted and unchanged | fixed — F030 is now split into the fixed signalled path and the accepted pre-launch residual |
| F042 | 008 | (found while fixing) | three trapped signals was not enough | under zsh an untrapped fatal signal skips the EXIT trap, so a wrapper-only USR1 exited 158 and LEAKED the scratch dir | fixed — the wrapper traps the full `RUN_SIGNALS` set; measured old vs new in both shells |

## Non-blocking follow-ups (recorded, not carried into another round)

- **From F003 — `assemble-review.sh` needs a skip channel the payload cannot write.** It publishes
  skip status only inside the bundle it emits, so copied content can still impersonate a marker for
  one of your own allowlisted paths. The fix (a manifest on stderr, or a distinct exit status) is a
  change to that script, which is not in this task's declaration; the ratchet rule forbids growing
  an allowlist to absorb a finding. Follow-up for its own review unit.
- **From F012/F018 — `codex/skills/adversarial-review/SKILL.md` needs two edits.** It still says an
  invocation and its rebuttal are one immutable file, and still lists three consensus dispositions.
  Both were deliberately changed in `claude/skills/codex-review/SKILL.md`, which now states that it
  governs the pipeline's closure semantics. Until the Codex-side contract is updated a reviewer will
  keep filing the disagreement, correctly. That file is outside this unit's declaration; follow-up
  for its own review unit.
- **From F019 — `RUN_SIGNALS` in `claude/bin/dstack` does not name `PROF`,** and the launched pid is
  recorded just after the fork rather than atomically with it. Measured: bash 3.2.57 fires the EXIT
  trap on `ABRT`/`XCPU`/`XFSZ`/`VTALRM`, so those are covered; `SIGPROF` is not, and `SIGKILL` cannot
  be. `dstack` is T01's declaration; follow-up for its own review unit.

## Blocking count per round

§4's counter is the number of concrete blocking findings still OPEN at the END of the round, after
that round's fixes — not the number the round raised. Both are recorded, because the ambiguity
between them is F017 and this unit is where it surfaced.

| round | raised (new, concrete, blocking) | OPEN at end of round |
|---|---|---|
| 001 | 3 (F001–F003) | 0 |
| 002 | 4 (F005–F008) + 1 in-use (F009) | 0 |
| 003 | 5 (F011–F015) + 1 in-use (F017) | 0 |
| 004 | 3 (F019–F021) + 1 restatement (F018) | 0 |
| 005 | 3 (F023 §3-half, F024, F026) + F025 as a stated limit + 1 restatement (F022) | 0 |
| 006 | 3 concrete blocking (F027, F028, F029) + 2 low | 1 (F029 carried) |
| 007 | 4 new (F032, F033, F034, F036) + F035 = F029 recurring | 0 |
| 008 | 3 (F037, F038, F040) + F039 recorded follow-up + 2 low | 0 — **§4 cap closure** |

Raised is rising; open is flat at zero, because every concrete finding so far has been fixed or,
for F012, resolved by a recorded precedence plus a follow-up. Round 004 is the first round the §4
test applies to and the first in which the bundle compacts.

**Rounds 006 and 007 are the post-seal reopening**, and its budget is §4's reset one: 2 rounds for
a per-task unit, counted from the reopening rather than from 001. Round 007 is the second, so this
is the last round the reopening is entitled to. The rule it runs under did not exist when the
reopening happened — the absence of any legal transition here is what F036 raised, and fixing it is
what makes these two rounds accountable to something instead of to nothing.

## Closure

Sealed at round 005, the §4 round cap for a per-task unit. **Open concrete findings at close: 0.**

Raised per round: 3, 4, 5, 3, 5 — no decay. The ledger names the two structural causes. The skip
gate took FOUR attempts because each matcher was wider or narrower than I believed (bare substring →
anchored regex → two-grep substring → `awk -v` escape decoding → `ENVIRON`). And three separate
rounds found a rule fixed in this file but not in the sibling it invokes. Both are closed:
`waits.external` is now the single source for the shared rules, and §3 no longer exempts a
regression introduced by a fix.

**Genuinely outstanding, named rather than dressed up as agreement:**
`codex/skills/adversarial-review/SKILL.md` still contradicts this file on round-file shape and on
consensus dispositions. A reviewer will keep filing it, correctly, until that file is edited.
