#!/bin/bash
# Structural invariants for the full-cycle SKILL.md rewrite (plain bash, no deps).
#
# WHY each check exists:
#  - The orchestrator schedules from the YAML schema; a missing phase/key silently
#    reverts that behavior to ad-hoc prose interpretation (the defect the rewrite fixes).
#  - fullcycle-gate.sh parses WORK DOCS the templates produce; if the rewrite drifts a
#    hook-parsed string, every future goal doc is generated gate-invisible. These
#    strings are byte-frozen and asserted here.
set -u
SKILL="$(cd "$(dirname "$0")/.." && pwd)/SKILL.md"
fails=0
ok()   { printf 'ok   %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; fails=$((fails + 1)); }
has()  { grep -qF -- "$2" "$SKILL" && ok "$1" || fail "$1 (missing: $2)"; }
hasE() { grep -qE -- "$2" "$SKILL" && ok "$1" || fail "$1 (missing re: $2)"; }

[ -f "$SKILL" ] || { echo "FAIL SKILL.md not found at $SKILL"; exit 1; }

# 1. Machine-readable pipeline schema exists.
has  "yaml pipeline block"        'pipeline: full-cycle'
# 2. All 12 phases, stable ids, in the schema.
for p in P1-intent P2-triaxis P3-research P4-interview P5-decompose P6-scaffold \
         P7-tdd P8-taskdoc P9-review P10-unit-e2e P11-milestone-e2e P12-goal-e2e; do
  has "phase id $p" "id: $p"
done
# 3. Scheduling semantics: the keys the orchestrator dispatches on.
for k in 'declaration:' 'checker:' 'modes:' 'review-overlap:' 'worker-fanout:' \
         'worktree-lifecycle:' 'fan-in:' 'waits:'; do
  has "scheduling key $k" "$k"
done
# 4. The task-declaration grammar (deps/files suffix) is specified.
hasE "deps/files grammar"         'deps: \[.*\]; files: \[.*\]'
# 5. Fan-out is guarded by the deterministic checker, fail-closed.
has  "checker script named"       'check-parallel.sh'
hasE "fail-closed to serial"      'fail-closed|fails? closed'
# 6. Actual-diff containment (scope) gate is part of the contract.
hasE "scope containment verdict"  'scope'
# 7. Hook-frozen surfaces stay byte-compatible in the templates.
has  "goal gate heading"          '## Goal gate'
has  "task gate heading"          '## Gate status'
has  "GOAL E2E box"               '- [ ] GOAL E2E'
has  "TDD gate box"               '- [ ] TDD: Red→Green→Refactor complete'
has  "codex gate box"             '- [ ] Codex (GPT-5.6 Sol) adversarial review consensus'
has  "e2e gate box"               '- [ ] E2E capture verified'
has  "review series contract"     'codex-review-<NNN>.md'
# 8. Registry contract. The store moved from a lock-serialized `.fullcycle-active` file to
# `.dstack/active/`, mutated only through the `dstack` CLI — so the old assertions (the flat
# file, its `mkdir` lock) now pin a mechanism that no longer exists and were replaced, not
# dropped. `.fullcycle-active` must STILL appear, but only as the fail-loud cutover trigger.
has  "registry dir"               '.dstack/active/'
has  "session tag"                '$CLAUDE_CODE_SESSION_ID'
has  "CLI path is defined once"   'DS="$HOME/.claude/bin/dstack"'
# Bind each VERB to an ABSOLUTE invocation. The previous form — one `/.claude/bin/dstack`
# occurrence plus free-floating ` migrate` and ` unreg` substrings — was satisfied by text that
# had nothing to do with each other, so adding a bare `dstack status` left every assertion green
# while reintroducing the exact class the check claims to pin (a bare name resolves only if the
# reader happens to have ~/.claude/bin on PATH, and never in a non-interactive shell).
# Define the verb set ONCE. The bare-call scan below reuses it: the two lists drifted apart and
# the destructive `rm-run` ended up only in the positive loop, so a bare `dstack rm-run` in a
# fence was scanned for by nothing at all.
VERBS="reg unreg reclaim status migrate run-dir prune rm-run"
VERBS_RE="$(printf '%s' "$VERBS" | tr ' ' '|')"
for v in reg unreg reclaim status migrate; do
  hasE "verb '$v' called by absolute path" '("\$DS"|"\$HOME/\.claude/bin/dstack") '"$v"'([^A-Za-z0-9_-]|$)'
done
# And the negative, PER FENCE. Two things are checked inside each runnable block, because a shell
# variable does not cross fences: no bare `dstack <verb>`, and no use of "$DS" in a fence that
# does not itself define it. Concatenating the fences (the earlier form) accepted a fence
# containing only `"$DS" status` as long as some OTHER fence defined DS — which is an unset
# variable at run time under `set -u`, the exact regression the check claims to pin. Prose may
# still name `dstack reg` in backticks; that is a reference, not something anyone runs.
ftmp="$(mktemp -d)" || { echo "FAIL: cannot create a temp dir — fence checks did NOT run" >&2; exit 1; }
tmp=""
# ONE trap covering every temp dir, armed as soon as the first exists. A second `trap … EXIT`
# REPLACES the first (bash has no trap stack), and a normal-path `rm -rf` at the end does nothing
# when an assertion exits early.
trap 'rm -rf "$ftmp" ${tmp:+"$tmp"}' EXIT
awk -v d="$ftmp" '/^```bash$/{f=1;n++;next} /^```/{f=0} f{print >> (d "/f" n ".sh")}' "$SKILL" \
  || { echo "FAIL: fence extraction failed — the checks below would have printed ok on nothing" >&2; exit 1; }
# A silent zero-fence extraction is the failure this whole block exists to catch: every loop
# below iterates an empty set and reports ok, which reads as "verified" and verified nothing.
set -- "$ftmp"/f*.sh
[ -e "$1" ] || { echo "FAIL: no bash fence was extracted from $SKILL" >&2; exit 1; }
bare=""; unbound=""
for b in "$ftmp"/f*.sh; do
  [ -e "$b" ] || continue
  grep -qE '(^|[^/A-Za-z0-9_$"])dstack ('"$VERBS_RE"')([^A-Za-z0-9_-]|$)' "$b" \
    && bare="$bare $(basename "$b")"
  if grep -qF '"$DS"' "$b" && ! grep -qF 'DS="$HOME/.claude/bin/dstack"' "$b"; then
    unbound="$unbound $(basename "$b")"
  fi
done
if [ -z "$bare" ]; then ok "no bare dstack call in a runnable block"
else fail "no bare dstack call in a runnable block (fences:$bare)"; fi
if [ -z "$unbound" ]; then ok "every fence using \$DS defines it"
else fail "every fence using \$DS defines it (fences:$unbound)"; fi
# 9. Standing behavior preserved.
has  "quick skip token"           '[quick]'
hasE "language boundary"          '[Kk]orean'
# 10. External waits keep registration (consult decision — no unreg-pause for waits).
hasE "waits keep registration"    'registered'
# 11. Round-1 review hardening: schema must be typed and the lifecycle executable.
hasE "phases carry per field"     'per: (goal|review-unit|milestone)'
has  "phases carry gate field"    'gate:'
has  "three-way verdicts"         'INVALID'
has  "fanout requires list"       'requires:'
has  "post-seal reopen rule"      'reopen'
has  "worker commits in worktree" 'COMMITS'
has  "merge precedes completion"  'Merge precedes P10'
has  "declared-path cleanliness"  'declared-path cleanliness'
has  "resource isolation gate"    'resource isolation'
has  "canonical path rule"        'canonical'
# 12. Every fenced ```yaml block must PARSE — grep-able keywords in broken YAML were
# exactly the round-1 failure mode. ruby ships with macOS; absent ruby = explicit skip.
if command -v ruby >/dev/null 2>&1; then
  tmp="$(mktemp -d)"
  awk -v d="$tmp" '/^```yaml$/{f=1;n++;next} /^```/{f=0} f{print >> (d "/b" n ".yaml")}' "$SKILL"
  found=0
  for b in "$tmp"/b*.yaml; do
    [ -e "$b" ] || continue
    found=1
    if ruby -ryaml -e 'YAML.load_file(ARGV[0])' "$b" >/dev/null 2>&1; then
      ok "yaml parses: $(basename "$b")"
    else
      fail "yaml block does NOT parse: $(basename "$b")"
    fi
  done
  [ "$found" -eq 1 ] || fail "no fenced yaml blocks found in SKILL.md"
else
  echo "skip yaml-parse checks (ruby unavailable)"
fi

if [ "$fails" -gt 0 ]; then echo "== $fails failure(s)"; exit 1; fi
echo "== all checks passed"
