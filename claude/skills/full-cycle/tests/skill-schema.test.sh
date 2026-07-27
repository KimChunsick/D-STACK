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
# The delegation gate stopped keying on parallelism. Without these, the whole change could be
# reverted to "PARALLEL verdict required" and every assertion here would stay green — which is
# exactly the blind spot that let the previous rewrite's registry assertions go stale.
has  "delegation keys on task shape" 'delegate-when:'
has  "wrong-to-delegate list"        'keep-in-the-orchestrator:'
has  "parallelism is scheduling"     'parallel-when:'
has  "containment is honest"         'honest-scope:'
# And the negative — WHERE the PARALLEL condition sits, read off the PARSED schema. Two earlier
# versions of this guard were wrong, in ways worth keeping written down. The first banned one exact
# phrase anywhere in the file: the live wording differed, so the real regression walked past it,
# while the banned wording under `parallel-when` — where it BELONGS — produced a false failure. The
# second scoped it by indentation with awk, but awk reads comments as content, so a key whose body
# is nothing but comments looked non-empty AND matched `grep PARALLEL`, while YAML loads that key as
# null. Text cannot answer a question about schema shape. Parsing can: comments are gone, and a
# null key is not a non-empty list.
tmp="$(mktemp -d)" || { echo "FAIL: cannot create a temp dir — yaml checks did NOT run" >&2; exit 1; }
awk -v d="$tmp" '/^```yaml$/{f=1;n++;next} /^```/{f=0} f{print >> (d "/b" n ".yaml")}' "$SKILL" \
  || { echo "FAIL: yaml fence extraction failed" >&2; exit 1; }
# The canonical decisions, one per line as `key|mode|text`. They live in a quoted heredoc rather
# than inside the ruby string because they contain apostrophes and backticks. `eq` means some entry
# must equal this after whitespace normalization; `sub` means the normalized field must contain it
# contiguously. Round 006 showed why token pins are not enough: `verdict of PARALLEL` is satisfied
# by "a verdict of PARALLEL must never permit concurrent execution", and `OUTRANKS` by
# "frontend-dev never OUTRANKS orchestrator retention". A phrase long enough to carry the decision
# cannot be negated without editing the phrase itself, which is exactly what should fail here.
cat > "$tmp/pins.txt" <<'PINS'
parallel-when|eq|a checker plan verdict of PARALLEL for the exact candidate set
delegate-when|sub|the declaration is COMPLETE — the checker returns non-INVALID for this task and its `files` list is non-empty.
delegate-when|sub|the WRITE SET IS DETERMINED — task.md states the intended behaviour and the declaration states where it lands, so the worker is implementing a decision rather than making one.
delegate-when|sub|there is a POSITIVE ISOLATION BENEFIT, decided from the declaration and the task doc rather than from a feeling
requires|sub|BASE IDENTITY IS VERIFIED, never assumed.
keep-in-the-orchestrator|sub|exploratory work, by the definition above
keep-in-the-orchestrator|sub|anything writing `docs/` or this pipeline's own skill files, which no worker may touch
honest-scope|sub|COMMITTED-DELIVERABLE containment, not write confinement
frontend-takes-precedence|sub|it OUTRANKS `delegate-when`
PINS
if command -v ruby >/dev/null 2>&1; then
  fanout_report="$(ruby -ryaml -e '
    # safe_load over File.read, never load_file: on Psych < 4 load_file constructs tagged Ruby
    # objects DURING the load, before any check below can look at the result. A checker that
    # deserializes the thing it is checking has the order backwards.
    #
    # Duplicate keys are caught at the AST, not after. safe_load collapses `worker-fanout:` twice
    # in one mapping down to the last one, so counting parsed values can never see the first — a
    # malformed node hides behind a valid duplicate and the count still reads 1.
    dups = []
    fans = []
    walk = nil
    walk = lambda do |node, where|
      if node.is_a?(Psych::Nodes::Mapping)
        seen = {}
        node.children.each_slice(2) do |k, _v|
          name = k.respond_to?(:value) ? k.value : k.to_s
          dups << (where + "/" + name) if seen.key?(name)
          seen[name] = true
        end
      end
      kids = node.respond_to?(:children) ? node.children : nil
      (kids || []).each { |c| walk.call(c, where) }
    end
    multi = []
    Dir[File.join(ARGV[0], "b*.yaml")].sort.each do |path|
      src = File.read(path)
      begin
        stream = Psych.parse_stream(src)
      rescue StandardError
        next
      end
      walk.call(stream, File.basename(path))
      # safe_load returns the FIRST document only, so a second document in the same fence would
      # carry a regressed worker-fanout past every check below. One document per fence, or fail.
      if stream.children.size != 1
        multi << (File.basename(path) + ":" + stream.children.size.to_s)
        next
      end
      doc = (YAML.safe_load(src) rescue nil)
      next unless doc.is_a?(Hash)
      node = doc.dig("scheduling", "modes", "worker-fanout")
      fans << node unless node.nil?
    end
    if multi.empty?
      puts "PASS every yaml fence holds exactly one document"
    else
      puts "FAIL yaml fences with more than one document: " + multi.join(", ")
    end
    if dups.empty?
      puts "PASS no duplicate yaml mapping keys"
    else
      puts "FAIL duplicate yaml mapping keys: " + dups.join(", ")
    end
    if fans.size != 1 || !fans[0].is_a?(Hash)
      puts "FAIL expected exactly one scheduling.modes.worker-fanout mapping, found #{fans.size}"
    else
      fan = fans[0]
      # Every list that DECIDES delegation. Naming a subset here was the defect in rounds 003 and
      # 004: a PARALLEL condition in ANY of them recouples delegation to parallelism, which is the
      # exact thing this change removed.
      gates = %w[delegate-when keep-in-the-orchestrator requires]
      # Non-empty is not enough: `- # commented out` parses as [nil], a list of nothing.
      list_ok = lambda { |v|
        v.is_a?(Array) && !v.empty? && v.all? { |e| e.is_a?(String) && !e.strip.empty? }
      }
      (gates + ["parallel-when"]).each do |k|
        if list_ok.call(fan[k])
          puts "PASS worker-fanout.#{k} is a list of non-blank entries"
        else
          puts "FAIL worker-fanout.#{k} is not a list of non-blank entries (parsed as #{fan[k].inspect})"
        end
      end
      %w[honest-scope frontend-takes-precedence].each do |k|
        v = fan[k]
        if v.is_a?(String) && !v.strip.empty?
          puts "PASS worker-fanout.#{k} carries text"
        else
          puts "FAIL worker-fanout.#{k} is not a non-empty string (parsed as #{v.class})"
        end
      end
      # Shape is not meaning, and neither is a token. Nonblank strings are satisfied by
      # `delegate-when: [a task exists]`; a bare /PARALLEL/ token is satisfied by
      # `parallel-when: [PARALLEL must never be used]`. Both invert the contract and pass. So each
      # key is pinned to the DECISION IT STATES, normalized, from $tmp/pins.txt.
      # This pins wording: a reword that keeps the meaning fails here and must be updated
      # deliberately. That is unavoidable — a check able to tell "required" from "forbidden" has
      # nothing but wording to key on — and it is the same trade every other assertion here makes.
      norm = lambda { |t| t.to_s.gsub(/\s+/, " ").strip }
      pins = Hash.new { |h, k| h[k] = [] }
      File.readlines(File.join(ARGV[0], "pins.txt")).each do |line|
        k, mode, text = line.chomp.split("|", 3)
        pins[k] << [mode, text] if text
      end
      pins.each do |k, wanted|
        v = fan[k]
        entries = v.is_a?(Array) ? v.map { |e| norm.call(e) } : [norm.call(v)]
        whole = entries.join(" ")
        missing = wanted.reject { |mode, text|
          mode == "eq" ? entries.include?(norm.call(text)) : whole.include?(norm.call(text))
        }
        if missing.empty?
          puts "PASS worker-fanout.#{k} states its decision verbatim"
        else
          missing.each { |mode, text| puts "FAIL worker-fanout.#{k} no longer states (#{mode}): #{text[0, 60]}" }
        end
      end
      hit = lambda { |v| v.is_a?(Array) && v.any? { |e| e.to_s =~ /PARALLEL/i } }
      bad = gates.select { |k| hit.call(fan[k]) }
      puts(bad.empty? \
        ? "PASS PARALLEL gates no delegation list (#{gates.join(", ")})" \
        : "FAIL PARALLEL is a delegation precondition again, under #{bad.join(", ")} (it belongs under parallel-when)")
    end
  ' "$tmp" 2>&1)" || fanout_report="FAIL worker-fanout schema check crashed: $fanout_report"
  while IFS= read -r line; do
    case "$line" in
      "PASS "*) ok   "${line#PASS }" ;;
      *)        fail "${line#FAIL }" ;;
    esac
  done <<< "$fanout_report"
else
  echo "skip worker-fanout placement check (ruby unavailable)"
fi
has  "post-seal reopen rule"      'reopen'
has  "worker commits in worktree" 'COMMITS'
has  "merge precedes completion"  'Merge precedes P10'
has  "declared-path cleanliness"  'declared-path cleanliness'
has  "resource isolation gate"    'resource isolation'
has  "canonical path rule"        'canonical'
# 12. Every fenced ```yaml block must PARSE — grep-able keywords in broken YAML were
# exactly the round-1 failure mode. ruby ships with macOS; absent ruby = explicit skip.
if command -v ruby >/dev/null 2>&1; then
  found=0
  for b in "$tmp"/b*.yaml; do
    [ -e "$b" ] || continue
    found=1
    if ruby -ryaml -e 'YAML.safe_load(File.read(ARGV[0]))' "$b" >/dev/null 2>&1; then
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
