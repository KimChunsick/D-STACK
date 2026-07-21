# 01-installer-zshrc-hook

## Intent / Why
Ultracode-by-default silently broke: the alias fragment (`claude/ultracode.zsh` →
`~/.claude/ultracode.zsh`) survives, but its `source` hook lived unmanaged in `~/.zshrc`
and vanished when the user rewrote that file (2026-07-18). Research (claude-code-guide,
2.1.216) confirms no settings/env persistence route exists — `effortLevel` rejects
`ultracode`, #64817 closed as not planned — so the launch wrapper stays the only durable
mechanism, and the single point of failure is the unmanaged hook line. Fix: `install.sh`
gains an idempotent, dry-run-aware step that ensures the source hook exists in
`~/.zshrc`, making a zshrc rewrite recoverable by re-running the installer (the repo's
standard remedy). This reverses the earlier "zshrc is outside install.sh's map" decision
with new evidence: that design already caused one silent regression.

## Design consult (Phase 7 pre-step)
Skipped — trigger list does not apply: a single idempotent append step in an existing
installer; no new architecture, API contract, persistence, idempotency-*semantics* (the
step is idempotent but introduces no new cursor/dedup semantics), partitioning, or
sanitization surface. (Recorded before the M2 rule lands, applying it proactively.)

## What was done (what / why)
- **TDD Red (transient probe — the meta suite is retired; probe lives in scratchpad,
  not committed):** four behavioral assertions against a fake `$HOME`: (a) fresh
  install appends the exact source-hook line once; (b) re-run keeps it at exactly one
  (idempotent); (c) `--dry-run` leaves zshrc untouched; (d) absent `~/.claude` ⇒ no
  hook, no zshrc created. Captured failing run against unmodified install.sh:
  `✗ FAIL: expected hook once after install, got count=0` (exit 1).
- **Green:** added a "zshrc hook" step after the MAP loop in `install.sh`: exact-line
  `grep -qxF` idempotence check; appends the hook line under a
  `# D-STACK: ultracode-by-default (hook managed by install.sh)` marker comment;
  dry-run-aware; skips (counted in `skipped`) when `~/.claude` is absent, mirroring
  the map's agent-dir rule; counters/note style match the existing script. All four
  probes pass; `tests/secret-guard.sh` stays green.
- **Header sync:** `claude/ultracode.zsh` header no longer presents the source hook as
  a manual step — it names install.sh as the hook's manager and records why (the
  manual hook was silently lost in a 2026-07 zshrc rewrite, killing the default).
- **Refactor:** none needed beyond style-matching the existing note/counter idiom.
- **Review R1 fixes:** semantic effectiveness check added to install.sh (post-step,
  non-dry-run: resolves `alias claude` in interactive zsh, prints a loud WARNING when
  the effective command is not the ultracode wrapper — covers later overrides,
  inactive conditionals, ineffective legacy hooks); `CLAUDE_CODE_EFFORT_LEVEL`
  precedence caveat documented in the fragment header; probe battery extended to 7
  assertions (semantic zsh resolution + warning-on-override + no-warning-when-clean).
- Known residuals (carried from the prior ultracode task, unchanged): alias covers
  interactive zsh launches only (IDE/GUI/`command claude` bypass); `ZDOTDIR` users
  (not this machine) would need the hook in `$ZDOTDIR/.zshrc`.

## Files changed (where / why)
- `install.sh` — new idempotent, dry-run-aware "ensure zshrc source hook" step (the fix)
- `claude/ultracode.zsh` — header: hook is installer-managed now, with the why

## E2E verification
Verification is fully reproducible from this record alone (script is parameterized: pass the repo root).

**Probe battery — 24 assertions against fake `$HOME`s under owned temp roots, all passing.** Covers install mechanics; pty NON-EXECUTING metadata verification (exact alias text, aliases option, wrapper-function detection, global-alias hijack detection incl. of the verifier source itself — with a PATH-fake executable proving nothing runs); legacy startup shapes (early `exit 0`, `exec true`, non-TTY `exit`/`return`); env nonce forge; joined argument; compound alias; KSH_ARRAYS; bounded timeout + UNVERIFIED; orphan containment; strict signal-cancellation; hostile TMPDIR.
Probe script (exact content):

```bash
#!/usr/bin/env bash
# Verification probe for M3-T01 (transient — the meta suite is retired by user decision).
# Usage: bash m3-zshrc-hook-test.sh [repo-root]
# Why this behavior matters: the ultracode default died once because the ~/.zshrc source
# hook was unmanaged. install.sh must (a) append the hook when absent, (b) stay
# idempotent, (c) leave zshrc untouched in --dry-run, (d) skip without ~/.claude — and
# its semantic effectiveness check must (e-i) verify the OUTCOME, (j-n) stay bounded,
# not be fooled by early-exiting startup, and leave no orphan processes.
# Cleanup removes exactly the owned roots this run created (EXIT trap; never dirname).
set -euo pipefail
REPO="${1:-$(git rev-parse --show-toplevel 2>/dev/null || true)}"
[ -n "$REPO" ] && [ -f "$REPO/install.sh" ] || { echo "usage: $0 <repo-root>" >&2; exit 2; }
HOOK='[ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"'
fail() { echo "  ✗ FAIL: $*" >&2; exit 1; }
pass() { echo "  ✓ PASS: $*"; }

ROOTS=()
cleanup() { local r; for r in "${ROOTS[@]:-}"; do [ -n "$r" ] && rm -rf -- "$r"; done; }
trap cleanup EXIT
mkfake() { FAKE_DIR="$(mktemp -d)"; ROOTS+=("$FAKE_DIR"); mkdir -p "$FAKE_DIR/.claude"; : > "$FAKE_DIR/.zshrc"; }

# (a) fresh install appends the hook
mkfake; FAKE="$FAKE_DIR"; mkdir -p "$FAKE/.codex"
HOME="$FAKE" bash "$REPO/install.sh" >/dev/null
n="$(grep -cxF "$HOOK" "$FAKE/.zshrc" || true)"
[ "$n" = 1 ] || fail "expected hook once after install, got count=$n"
pass "hook appended on fresh install"

# (b) re-run stays idempotent
HOME="$FAKE" bash "$REPO/install.sh" >/dev/null
n="$(grep -cxF "$HOOK" "$FAKE/.zshrc" || true)"
[ "$n" = 1 ] || fail "expected hook once after re-run, got count=$n"
pass "idempotent re-run"

# (c) dry-run never touches zshrc
mkfake; FAKE2="$FAKE_DIR"
HOME="$FAKE2" bash "$REPO/install.sh" --dry-run >/dev/null
[ ! -s "$FAKE2/.zshrc" ] || fail "dry-run modified zshrc"
pass "dry-run leaves zshrc untouched"

# (d) no ~/.claude → no hook, no zshrc creation
FAKE3="$(mktemp -d)"; ROOTS+=("$FAKE3")
HOME="$FAKE3" bash "$REPO/install.sh" >/dev/null
[ ! -e "$FAKE3/.zshrc" ] || fail "hook written despite absent ~/.claude"
pass "skips when ~/.claude absent"

# (e) semantic effectiveness: interactive zsh against the fake HOME resolves the alias
out="$(HOME="$FAKE" zsh -ic 'alias claude' 2>/dev/null || true)"
case "$out" in *"--effort ultracode"*) pass "interactive zsh resolves ultracode alias" ;;
  *) fail "alias not effective in interactive zsh: '$out'" ;; esac

# (f) no warning when hook effective; warning when a LATER override alias wins
clean_out="$(HOME="$FAKE" bash "$REPO/install.sh" 2>&1)"
case "$clean_out" in *WARNING*) fail "unexpected warning on effective hook" ;; *) pass "no warning when hook effective" ;; esac
printf "alias claude='claude --effort high'\n" >> "$FAKE/.zshrc"
warn_out="$(HOME="$FAKE" bash "$REPO/install.sh" 2>&1)"
case "$warn_out" in *"does not effectively"*) pass "override after hook triggers warning" ;;
  *) fail "override alias did not trigger warning" ;; esac

# (g) `unsetopt aliases` after the hook → warning (alias table intact, expansion dead)
mkfake; FAKE_G="$FAKE_DIR"
HOME="$FAKE_G" bash "$REPO/install.sh" >/dev/null
printf 'unsetopt aliases\n' >> "$FAKE_G/.zshrc"
out_g="$(HOME="$FAKE_G" bash "$REPO/install.sh" 2>&1)"
case "$out_g" in *"does not effectively"*) pass "unsetopt aliases triggers warning" ;;
  *) fail "unsetopt aliases did not trigger warning" ;; esac

# (h) replacement alias whose body merely CONTAINS the substring → warning
mkfake; FAKE_H="$FAKE_DIR"
HOME="$FAKE_H" bash "$REPO/install.sh" >/dev/null
printf "alias claude='printf \"not claude: --effort ultracode\"'\n" >> "$FAKE_H/.zshrc"
out_h="$(HOME="$FAKE_H" bash "$REPO/install.sh" 2>&1)"
case "$out_h" in *"does not effectively"*) pass "substring-fake alias triggers warning" ;;
  *) fail "substring-fake alias did not trigger warning" ;; esac

# (i) noisy startup + real override → warning still fires (nonce channel, not stdout)
mkfake; FAKE_I="$FAKE_DIR"
HOME="$FAKE_I" bash "$REPO/install.sh" >/dev/null
printf 'echo "diag: --effort ultracode ready"\nalias claude="claude --effort high"\n' >> "$FAKE_I/.zshrc"
out_i="$(HOME="$FAKE_I" bash "$REPO/install.sh" 2>&1)"
case "$out_i" in *"does not effectively"*) pass "noisy startup cannot mask an override" ;;
  *) fail "noisy startup masked the override" ;; esac

# (j) blocking startup: installer finishes bounded and reports UNVERIFIED
mkfake; FAKE_J="$FAKE_DIR"
HOME="$FAKE_J" bash "$REPO/install.sh" >/dev/null
printf 'sleep 30\n' >> "$FAKE_J/.zshrc"
start=$(date +%s)
out_j="$(HOME="$FAKE_J" bash "$REPO/install.sh" 2>&1)"
dur=$(( $(date +%s) - start ))
[ "$dur" -le 20 ] || fail "installer blocked ${dur}s on a blocking zshrc"
case "$out_j" in *UNVERIFIED*) pass "blocking startup bounded (${dur}s) and reported UNVERIFIED" ;;
  *) fail "blocking startup not reported as UNVERIFIED" ;; esac

# (k) R3 finding 2 — `.zshrc` ends with plain `exit 0`: exit status lies, nonce cannot
mkfake; FAKE_K="$FAKE_DIR"
HOME="$FAKE_K" bash "$REPO/install.sh" >/dev/null
printf 'exit 0\n' >> "$FAKE_K/.zshrc"
out_k="$(HOME="$FAKE_K" bash "$REPO/install.sh" 2>&1)"
case "$out_k" in *"does not effectively"*) pass "early exit 0 startup triggers warning" ;;
  *) fail "early exit 0 startup did not trigger warning" ;; esac

# (l) R3 finding 2 — `exec true` replaces the shell before the predicate runs
mkfake; FAKE_L="$FAKE_DIR"
HOME="$FAKE_L" bash "$REPO/install.sh" >/dev/null
printf 'exec true\n' >> "$FAKE_L/.zshrc"
out_l="$(HOME="$FAKE_L" bash "$REPO/install.sh" 2>&1)"
case "$out_l" in *"does not effectively"*) pass "exec-true startup triggers warning" ;;
  *) fail "exec-true startup did not trigger warning" ;; esac

# (m) R3 finding 2 — the reviewer's exact legacy shape: non-TTY early exit hiding a
# later override; probe must NOT report silent success
mkfake; FAKE_M="$FAKE_DIR"
HOME="$FAKE_M" bash "$REPO/install.sh" >/dev/null
printf '[[ -t 0 ]] || exit 0\nalias claude="claude --effort high"\n' >> "$FAKE_M/.zshrc"
out_m="$(HOME="$FAKE_M" bash "$REPO/install.sh" 2>&1)"
case "$out_m" in *"does not effectively"*) pass "non-TTY early-exit + override triggers warning" ;;
  *) fail "non-TTY early-exit + override was silently accepted" ;; esac

# (n) R3 finding 3 — no orphans: a run-unique blocking child must be gone after the
# bounded check returns. Assert-only: NEVER a global pkill (it could hit an unrelated
# process); on failure the orphan is reported with its PID for manual handling.
mkfake; FAKE_N="$FAKE_DIR"
HOME="$FAKE_N" bash "$REPO/install.sh" >/dev/null
CANARY_SEC=$((31000 + $$ % 900))
printf 'sleep %s\n' "$CANARY_SEC" >> "$FAKE_N/.zshrc"
out_n="$(HOME="$FAKE_N" bash "$REPO/install.sh" 2>&1)"
case "$out_n" in *UNVERIFIED*) : ;; *) fail "blocking canary run not reported UNVERIFIED" ;; esac
sleep 1
orphans="$(pgrep -f "sleep $CANARY_SEC" || true)"
[ -z "$orphans" ] || fail "orphaned startup child survived the timeout kill (pid(s): $orphans — kill manually)"
pass "no orphan processes after timeout (tree kill)"

# (o) R4 finding 1 — the reviewer's `return`-based legacy shape: a detached probe
# would take the non-TTY branch and bless the hook; the pty probe must take the
# terminal path, see the override, and warn
mkfake; FAKE_O="$FAKE_DIR"
HOME="$FAKE_O" bash "$REPO/install.sh" >/dev/null
printf '[[ -t 0 ]] || return\nalias claude="claude --effort high"\n' >> "$FAKE_O/.zshrc"
out_o="$(HOME="$FAKE_O" bash "$REPO/install.sh" 2>&1)"
case "$out_o" in *"does not effectively"*) pass "non-TTY return + override triggers warning (pty path)" ;;
  *) fail "non-TTY return + override was silently accepted: $out_o" ;; esac

# (p) R5 finding 1 — the reviewer's env-based forge: a verifier-aware startup that
# echoes $DSTACK_VERIFY_NONCE and exits. With the nonce out of the environment the
# forge branch never fires, the override applies, and the warning must appear.
mkfake; FAKE_P="$FAKE_DIR"
HOME="$FAKE_P" bash "$REPO/install.sh" >/dev/null
cat >> "$FAKE_P/.zshrc" <<'ZRC'
if [[ -n ${DSTACK_VERIFY_NONCE-} ]]; then
  print -r -- "$DSTACK_VERIFY_NONCE" > "$DSTACK_VERIFY_OUT"
  exit 0
fi
alias claude='claude --effort high'
ZRC
out_p="$(HOME="$FAKE_P" bash "$REPO/install.sh" 2>&1)"
case "$out_p" in *"does not effectively"*) pass "env-based nonce forge no longer works" ;;
  *) fail "env-based nonce forge silently succeeded: $out_p" ;; esac

# (q) R5 finding 2 — cancellation windows: TERM delivered at varying instants after
# launch must always yield signal exit + zero verifier survivors (unique-token kill
# covers even the pre-registration window)
mkfake; FAKE_Q="$FAKE_DIR"
HOME="$FAKE_Q" bash "$REPO/install.sh" >/dev/null
CANARY_Q=$((33000 + $$ % 900))
printf 'sleep %s\n' "$CANARY_Q" >> "$FAKE_Q/.zshrc"
for delay in 0.05 0.3 2; do
  QOUT="$FAKE_Q/term-$delay.out"
  HOME="$FAKE_Q" bash "$REPO/install.sh" > "$QOUT" 2>&1 &
  qpid=$!
  sleep "$delay"
  kill -0 "$qpid" 2>/dev/null || fail "installer already gone before TERM at ${delay}s (probe did not exercise the window)"
  kill -TERM "$qpid" 2>/dev/null || true
  qrc=0; wait "$qpid" 2>/dev/null || qrc=$?
  sleep 0.5
  orph="$(pgrep -f "sleep $CANARY_Q" || true)"
  [ -z "$orph" ] || fail "verifier survivors after TERM at ${delay}s (pid(s): $orph — kill manually)"
  [ "$qrc" -eq 143 ] || fail "TERM at ${delay}s must yield status 143, got $qrc (swallowed cancellation?)"
  grep -q "Summary:" "$QOUT" && fail "TERM at ${delay}s still printed the success summary" || true
done
pass "TERM at 0.05/0.3/2s: status 143, no summary, no verifier survivors"

# (r) R6 finding 2 — global alias rewrites the expansion while the alias table
# stays byte-identical; the expansion-capture predicate must warn
mkfake; FAKE_R="$FAKE_DIR"
HOME="$FAKE_R" bash "$REPO/install.sh" >/dev/null
printf "alias -g ultracode=high\n" >> "$FAKE_R/.zshrc"
out_r="$(HOME="$FAKE_R" bash "$REPO/install.sh" 2>&1)"
case "$out_r" in *"does not effectively"*) pass "global-alias rewrite triggers warning" ;;
  *) fail "global-alias rewrite silently accepted: $out_r" ;; esac

# (s) R7 finding 1 — a REAL pre-existing wrapper FUNCTION that rewrites args: the
# verifier must not silently replace it; it must report unverifiable/ineffective
mkfake; FAKE_S="$FAKE_DIR"
HOME="$FAKE_S" bash "$REPO/install.sh" >/dev/null
cat >> "$FAKE_S/.zshrc" <<'ZRC'
function claude { command claude --effort high; }
ZRC
out_s="$(HOME="$FAKE_S" bash "$REPO/install.sh" 2>&1)"
case "$out_s" in *"does not effectively"*) pass "pre-existing wrapper function triggers warning" ;;
  *) fail "pre-existing wrapper function silently accepted: $out_s" ;; esac

# (u) R7 finding 1 — one JOINED argument must not equal two separate ones
mkfake; FAKE_U="$FAKE_DIR"
HOME="$FAKE_U" bash "$REPO/install.sh" >/dev/null
printf 'alias claude=%s\n' "'claude \"--effort ultracode\"'" >> "$FAKE_U/.zshrc"
out_u="$(HOME="$FAKE_U" bash "$REPO/install.sh" 2>&1)"
case "$out_u" in *"does not effectively"*) pass "joined single-argument alias triggers warning" ;;
  *) fail "joined single-argument alias silently accepted: $out_u" ;; esac

# (t) R6 finding 1 — hostile-but-valid TMPDIR (spaces/brackets): check must stay
# bounded, produce no stray files, and leave no survivors
mkfake; FAKE_T="$FAKE_DIR"
HOME="$FAKE_T" bash "$REPO/install.sh" >/dev/null
TDIR="$FAKE_T/tmp dstack [probe]"
mkdir -p "$TDIR"
CANARY_T=$((34000 + $$ % 900))
printf 'sleep %s\n' "$CANARY_T" >> "$FAKE_T/.zshrc"
start=$(date +%s)
out_t="$(HOME="$FAKE_T" TMPDIR="$TDIR" bash "$REPO/install.sh" 2>&1)"
dur=$(( $(date +%s) - start ))
[ "$dur" -le 20 ] || fail "hostile TMPDIR: installer blocked ${dur}s"
case "$out_t" in *UNVERIFIED*) : ;; *) fail "hostile TMPDIR: not reported UNVERIFIED: $out_t" ;; esac
sleep 1
orph_t="$(pgrep -f "sleep $CANARY_T" || true)"
[ -z "$orph_t" ] || fail "hostile TMPDIR: verifier survivors (pid(s): $orph_t — kill manually)"
pass "hostile TMPDIR (spaces/brackets): bounded, UNVERIFIED, no survivors"

# (v) R8/R9 finding — compound alias whose first command bypasses the stub via
# `command`: must warn AND must NOT execute anything (non-executing verification).
# A fake `claude` executable first in PATH records any invocation.
mkfake; FAKE_V="$FAKE_DIR"
mkdir -p "$FAKE_V/bin"
printf '#!/bin/sh\necho "$@" >> "%s/invoked.log"\n' "$FAKE_V" > "$FAKE_V/bin/claude"
chmod +x "$FAKE_V/bin/claude"
HOME="$FAKE_V" bash "$REPO/install.sh" >/dev/null
printf 'path=("%s/bin" $path)\nalias claude=%s\n' "$FAKE_V" "'command claude --effort high; claude --effort ultracode'" >> "$FAKE_V/.zshrc"
out_v="$(HOME="$FAKE_V" bash "$REPO/install.sh" 2>&1)"
case "$out_v" in *"does not effectively"*) : ;; *) fail "compound alias silently accepted: $out_v" ;; esac
[ ! -e "$FAKE_V/invoked.log" ] || fail "verifier EXECUTED the alias body: $(cat "$FAKE_V/invoked.log")"
pass "compound alias warns and nothing is executed"

# (w) R8 finding 3 — KSH_ARRAYS is a valid user option; a clean install must stay
# warning-free (index-agnostic compare), while the joined-arg fixture still warns
mkfake; FAKE_W="$FAKE_DIR"
printf 'setopt KSH_ARRAYS\n' >> "$FAKE_W/.zshrc"
out_w="$(HOME="$FAKE_W" bash "$REPO/install.sh" 2>&1)"
case "$out_w" in *WARNING*) fail "KSH_ARRAYS clean install falsely warned: $out_w" ;;
  *) pass "KSH_ARRAYS clean install stays warning-free" ;; esac

echo "ALL PROBES PASSED"
```

Observed transcript:

```text
  ✓ PASS: hook appended on fresh install
  ✓ PASS: idempotent re-run
  ✓ PASS: dry-run leaves zshrc untouched
  ✓ PASS: skips when ~/.claude absent
  ✓ PASS: interactive zsh resolves ultracode alias
  ✓ PASS: no warning when hook effective
  ✓ PASS: override after hook triggers warning
  ✓ PASS: unsetopt aliases triggers warning
  ✓ PASS: substring-fake alias triggers warning
  ✓ PASS: noisy startup cannot mask an override
  ✓ PASS: blocking startup bounded (12s) and reported UNVERIFIED
  ✓ PASS: early exit 0 startup triggers warning
  ✓ PASS: exec-true startup triggers warning
  ✓ PASS: non-TTY early-exit + override triggers warning
  ✓ PASS: no orphan processes after timeout (tree kill)
  ✓ PASS: non-TTY return + override triggers warning (pty path)
  ✓ PASS: env-based nonce forge no longer works
  ✓ PASS: TERM at 0.05/0.3/2s: status 143, no summary, no verifier survivors
  ✓ PASS: global-alias rewrite triggers warning
  ✓ PASS: pre-existing wrapper function triggers warning
  ✓ PASS: joined single-argument alias triggers warning
  ✓ PASS: hostile TMPDIR (spaces/brackets): bounded, UNVERIFIED, no survivors
  ✓ PASS: compound alias warns and nothing is executed
  ✓ PASS: KSH_ARRAYS clean install stays warning-free
ALL PROBES PASSED
```

_Real-`$HOME` install run and fresh-shell alias capture are recorded below at final E2E._
## Gate status
- [ ] TDD: Red→Green→Refactor complete
- [ ] Codex (GPT-5.6 Sol) adversarial review consensus
- [ ] E2E capture verified
