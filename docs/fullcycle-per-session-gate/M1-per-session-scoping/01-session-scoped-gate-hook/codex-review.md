# Codex adversarial review — 01-session-scoped-gate-hook

Reviewer: GPT-5.6 Sol (xhigh). Material: assemble-review.sh allowlist bundle
(fullcycle-gate.sh, test_fullcycle_gate_hook.sh, SKILL.md, CLAUDE.md, GOAL.md, research artifact).

## Round 1 — GPT verdict: reject

### F1 [high][technical] Unlocked registry mutation can lose an *active* registration — "stray line only" claim is false
**Agree (partially) → fixed + honestly re-scoped.** The claim in the research summary that the worst
case is "a recoverable stray line" understated it: a concurrent `unreg` rewrite can drop a
simultaneous `reg` append, losing an active registration (fail-open). Corrected:
- Registration is now an idempotent atomic append (`reg`, `>>` whole-line, atomic on local APFS);
  removal is an atomic tmp+rename (`unreg`) — SKILL.md.
- The residual (remove-vs-append lost update) is stated honestly in GOAL.md + SKILL.md caveats, not
  as "stray line only". **Not locked**, and I push back on the demand to lock: macOS ships no
  `flock(1)`; a `mkdir`-lock in a prose recipe introduces a *worse* failure (stale-lock deadlock on
  crash) and the independent research + Simplicity-First both reject a lock at this scale (a few
  human-paced tabs). Recovery is idempotent re-`reg` or the escape hatch. Accepted, documented.

### F2 [high][real Why] A non-live/typo/stale/`/clear` owner is skipped by everyone → claimed fail-closed violation
**Disagree — this is the user's explicit interview decision, not a regression.** Fail-closed in this
design covers *unattributable* lines: no tag, empty owner, empty `$CLAUDE_CODE_SESSION_ID` — all
enforced by every session (tests C25/C26/C30). A line tagged with an id no live session holds
(typo / stale / `/clear`) is, by construction, an **orphan**, and the Phase-4 interview explicitly
chose "just leave orphans — they block nobody." Conflating "orphan owned by a dead session" with
"unattributable" would re-introduce the exact cross-blocking this Goal removes. The distinction is
now spelled out in the hook comment, GOAL.md, and SKILL.md so it can't be read as an accident.

### F3 [medium][technical] Duplicate registration double-counts → false one-Goal trip
**Agree → fixed at the enforcement point.** The hook now dedupes the in-scope doc set
(`seen`/`<$doc>` membership), so a doc registered twice — by any writer, or a race — counts once.
`reg` is also idempotent (`grep -qxF || >>`). New test C31 covers a doubly-registered `GOAL.md`
not tripping the one-Goal rule.

### F4 [medium][structure] Mutation is ad-hoc prose, no common helper enforcing the contract
**Agree in spirit → addressed proportionately.** SKILL.md now ships `reg`/`unreg` shell helpers
(idempotent insert, exact-match delete, atomic rename) as the single documented mutation path,
rather than scattered `printf`/`grep` one-liners. A standalone tracked helper *script* was
considered and rejected as over-scope (new file + install.sh entry + gitignore allowlist + test)
for a registry edited seconds apart; the hook-side dedupe is the real robustness guarantee.

### F5 [medium][DX] `grep -v "$ID"$'\t'path` removal is an unanchored, unescaped regex
**Agree → fixed.** Removal is now `grep -vxF -- "$line"` (full-line `-x`, fixed-string `-F`), so
regex metacharacters and partial matches can't remove an unintended entry.

### F6 [medium][real Why] Legacy untagged lines keep cross-blocking; no migration path; research "cleaned via next registration" is false
**Agree → fixed.** SKILL.md adds an explicit migration note: an untagged line stays globally
enforced until removed — appending a tagged line does not convert it; migrate once by clearing
`.fullcycle-active` (or `unreg` the untagged paths) before re-registering tagged. The GOAL.md
summary no longer claims the next registration cleans orphans.

### F7 [medium][DX] Block message "Only THIS session's registered docs are enforced" is false for untagged/empty-id
**Agree → fixed.** Reworded: "Enforced for this session: its own owner-tagged docs plus any
untagged/unknown-id lines; another session's tagged docs are not shown here."

### F8 [medium][technical] Tests don't cover empty owner, duplicate registration, isolation "both directions"
**Agree → fixed (partly), rebut (partly).** Added C30 (empty-owner fail-closed) and C31 (duplicate
registration deduped). On "C24 isn't both-directions": C24 asserts *both* that session A is NOT
blocked by B's incomplete doc AND that session B IS blocked by its own incomplete doc — that is the
isolation contract in both directions (A unaffected by B; B still self-gated). `/clear`-orphan and
concurrent-mutation are writer-side/lifecycle behaviors documented as accepted; they are not hook
logic and are asserted by the E2E (Phase 10), not unit cases.

### F9 [high][real Why] E2E is a placeholder; Goal-gate boxes unchecked; no concurrent-tab evidence
**Disagree on ordering, agree to capture.** Per the full-cycle pipeline, Codex review is Phase 9 and
E2E capture is Phase 10 — the E2E placeholder at review time is the defined order, not a defect. The
environment supplying a matching id is already verified (`$CLAUDE_CODE_SESSION_ID=9e03f814…` visible
in a Bash subprocess). The two-session concurrent-tab E2E will be captured in Phase 10 and the
Goal-gate M1/GOAL boxes ticked only then.

### F10 [low][security] Owner tags have no authenticity — any writer can forge a foreign owner
**Agree, already owned.** The hook's HONEST SCOPE states the tag is self-attested (like a checkbox),
not a security boundary, and forging a tag is no new *malicious* bypass because deleting the line
(the escape hatch) already suppresses enforcement. Threat model is a self-attesting agent, not an
external attacker. No change beyond the existing wording.

## Round 2 — GPT verdict: reject (F1 + F6 still open; F2/F3/F4/F5/F7/F8/F9/F10 accepted resolved)

### F1 [high] concurrent remove-vs-append race — now fixed structurally (Round 3)
**Fixed.** Took Codex's suggested "guarded `mkdir` lock with cleanup." `reg`/`unreg` now acquire a
portable `mkdir .fullcycle-active.lock` (macOS-safe, no `flock`) that serializes the read-modify-
write, so an `unreg` rewrite can no longer drop a simultaneous `reg` append. `_lock` bounds its wait
(~5s) and returns non-zero rather than hanging; `_unlock` is `rmdir`. Residual is only a stranded
lock dir after a *hard* kill (SIGKILL/power loss) mid-mutation — one-line recovery
`rm -rf .fullcycle-active.lock`, documented. Verified end-to-end (idempotent reg, exact unreg, lock
auto-cleaned).

### F6 [medium] migration command didn't work — now fixed (Round 3)
**Fixed.** Correct: `unreg` builds `<current-id><TAB><path>` and cannot match an untagged line — the
old "unreg the untagged paths" guidance was wrong. Replaced with a one-time, quiescent migration
that keeps only tagged (TAB-bearing) lines:
`t=$(mktemp); grep -F "$T" .fullcycle-active > "$t" || true; mv "$t" .fullcycle-active`, explicitly
gated on "no other tab registering." Verified: drops an untagged legacy line, keeps the tagged one.

## Round 3 — GPT verdict: reject (F1/F6 confirmed resolved; one NEW [medium] on helper error-handling)

### F11 [medium] helpers mask mutation failure / clobber on read error / INT·TERM strand lock — fixed (Round 4)
**Agree → fixed.** All three points valid:
- **Masked status:** `reg`/`unreg` now capture the mutation's `rc` and `return $rc` — `_unlock`'s
  success no longer stands in for the append/`mv`. The call sites warn on failure
  (`reg … || echo "WARN: … UNGATED" >&2`) so a lost registration is loud, not silent.
- **Clobber on read error:** dropped `grep … || true`. `unreg` now inspects grep's status and treats
  only `≥2` as a real error — on which it `rm`s the temp and returns 1 **without** `mv`, so a read
  error can never replace the registry with empty/partial content. `mktemp` failure aborts too.
- **Signal cleanup:** `_lock` sets `trap '_unlock' EXIT INT TERM` (cleared in `_unlock`), so an
  ordinary Ctrl-C / SIGTERM mid-mutation releases the lock. Only an uncatchable SIGKILL / power loss
  can now strand it — which is exactly what caveat (b) documents.

Verified end-to-end: normal reg/unreg rc=0 with no duplication and no stale lock; an unwritable
registry makes `reg` return rc=1 (loud) and still releases the lock.

## Round 4 — GPT verdict: reject (F11 signal path: trap unlocks but bash resumes → mutate-after-unlock)

### F11b [medium] signal handler must abort, not just clean up — fixed (Round 5)
**Agree → fixed.** Correct bash semantics: a signal trap runs then *resumes* the interrupted code,
so `trap '_unlock' … INT` would release the lock and then let the function keep mutating unlocked.
Split the traps: `trap '_unlock' EXIT` (cleanup only, for normal/`set -e` exit) but
`trap '_unlock; exit 130' INT` and `trap '_unlock; exit 143' TERM` — a signal now unlocks **and
aborts** with the conventional 128+signum code, so no mutation runs after the lock is released.
Verified in a child shell: `_lock; kill -INT $$; <mutation>` exits 130, the mutation does **not**
run, and the lock is released.

## Claude response summary
Fixed: F1 (honest re-scope + atomic recipes), F3 (hook dedupe + idempotent reg), F4 (reg/unreg
helpers), F5 (exact-match removal), F6 (migration note), F7 (accurate block message), F8 (C30/C31).
Rebutted with rationale: F2 (accepted orphan semantics per interview vs unattributable fail-closed),
F9 (pipeline phase ordering; env verified), F10 (already owned as non-boundary).
All test cases (C1–C31) green; full `tests/run.sh` green. Rounds 3–5 resolved every remaining hold
(F1 lock, F6 migration, F11/F11b helper error+signal handling) with real, verified fixes.

## Round 5 — GPT verdict: approve
"F11b is resolved. INT and TERM now release the lock and immediately terminate with conventional
status codes, so Bash cannot resume mutation after unlocking. The EXIT trap remains cleanup-only and
is cleared by `_unlock`. GPT verdict: approve."

Outcome: fixed — F1, F3, F4, F5, F6, F7, F8, F11, F11b. Rebutted-and-accepted — F2 (orphan vs
unattributable is a deliberate interview decision), F9 (E2E is Phase 10 by pipeline order), F10 (tag
is self-attested, not a security boundary, already owned). No open items.

Consensus: resolved
