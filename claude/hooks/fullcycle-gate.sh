#!/bin/bash
# Stop hook — full-cycle gate. Blocks the turn from ending while registered work is incomplete.
#
# HONEST SCOPE: this is a tripwire over the registered docs + the Codex review, NOT a sandbox.
# It cannot prove TDD/E2E actually ran (a checkbox is self-attested), nor can it out-parse a
# determined faker: a *lying* Consensus line ('resolved' over a rejecting review body), work
# hidden under a nested/fenced heading (the gate is section-scoped), or a milestone written in
# setext form are self-attestation limits no regex closes. What it DOES enforce mechanically:
# unchecked gate boxes (any GFM marker) block; tasks require a single registered Goal; every ATX
# 'M<n>' milestone heading needs a ticked milestone-E2E box; every registered task needs a
# codex-review.md whose final Consensus line is a clean positive verdict (a strict allow-list —
# 'disagreed' / 'unresolved' / 'agreed was not reached' / 'agreed to reject' all fail). Together
# with the inject hook + adversarial review, skipping becomes costly and visible — defense in
# depth, not a single airtight gate.
#
# Escape hatch (avoids deadlock): remove a doc's line from .fullcycle-active to pause it.
f=".fullcycle-active"
[ -f "$f" ] || exit 0

goals=(); tasks=()
while IFS= read -r doc; do
  [ -z "$doc" ] && continue
  case "$doc" in docs/*) : ;; *) continue ;; esac      # only docs/ paths are honored
  [ -L "$doc" ] && continue                             # never follow a symlinked doc
  [ -f "$doc" ] || continue
  case "$doc" in *GOAL.md) goals+=("$doc") ;; *) tasks+=("$doc") ;; esac
done < "$f"

# Body of a "## <heading…>" section (prefix match), up to the next "## " line. Deliberately
# NOT fence-aware: stripping fenced blocks (to spare an example checkbox) proved net-negative —
# an unbalanced/lone ``` then strips real gate rows to EOF. A worked example inside a gate
# section instead just over-blocks (safe, recoverable); real gates live outside code fences.
section() { awk -v h="## $2" 'index($0,h)==1{f=1;next} /^## /{f=0} f' "$1"; }

# A codex-review.md carries a genuine positive consensus. Design is a POSITIVE-ONLY WHITELIST
# (fail-closed): a negation lexicon is unwinnable — 'reject', 'block', 'nack', 'wontfix', 'no-go'
# would all need listing, and any miss is a bypass. So we require the FINAL verdict line to be
# *exactly* a clean positive and reject everything else. The authoritative line is the LAST line
# that begins with 'Consensus:' after any leading markdown decoration — unordered/ordered list
# markers, blockquote, heading hashes, emphasis ([-[:space:]>#*+._)0-9]) — a multi-round loop
# supersedes earlier verdicts, and both the SELECT and VALIDATE greps share that decoration
# prefix so a later '1. Consensus: disagreed' / '### Consensus: disagreed' can't be skipped to
# let a stale 'agreed' win. It passes iff that line is 'Consensus: agreed' / 'Consensus: resolved'
# followed only by non-alphanumeric decoration (punctuation, a ✅) — NO trailing word, which is
# exactly where a negation like 'agreed to reject' would hide.
consensus_ok() {
  local line
  line="$(grep -iE '^[-[:space:]>#*+._)0-9]*consensus:' -- "$1" | tail -n1)"
  [ -n "$line" ] || return 1
  printf '%s\n' "$line" | grep -qiE '^[-[:space:]>#*+._)0-9]*consensus:[*_[:space:]]*(agreed|resolved)[^[:alnum:]]*$'
}

p=""   # accumulated problems

[ "${#goals[@]}" -gt 1 ] && p="$p more than one GOAL.md is active (exactly one Goal allowed);"
{ [ "${#tasks[@]}" -gt 0 ] && [ "${#goals[@]}" -eq 0 ]; } && p="$p task(s) active without a registered GOAL.md;"

for t in "${tasks[@]}"; do
  gs="$(section "$t" 'Gate status')"
  # Schema REQUIRED (fail-closed, mirrors the Goal gate): a registered task must carry a
  # '## Gate status' section holding at least one checkbox row — a missing or prose-only
  # section cannot silently bypass task enforcement. Checkbox patterns accept any GFM list
  # marker ([-*+]), leading whitespace, and >1 space after it, so '* [ ]' / '+ [ ]' / an
  # indented / a double-spaced box cannot hide an unchecked gate.
  if [ -z "$gs" ]; then p="$p $t has no '## Gate status' section (required gate schema missing);"; continue; fi
  if ! printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[[ xX]\]'; then p="$p $t Gate status has no checkbox rows (prose cannot stand in for gates);"; continue; fi
  if printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[ \]'; then p="$p $t has unchecked task gates;"; fi
  # Codex (GPT-5.5) adversarial review is a MANDATORY per-task phase, so require the
  # codex-review.md artifact + a positive final consensus for EVERY registered task —
  # unconditional, not keyed on a checkbox label (relabeling / omitting the box must not opt out).
  d="$(dirname -- "$t")"
  if [ ! -s "$d/codex-review.md" ] || ! consensus_ok "$d/codex-review.md"; then
    p="$p $t lacks a codex-review.md with a final agreed/resolved consensus (mandatory per task);"
  fi
done

for g in "${goals[@]}"; do
  gate="$(section "$g" 'Goal gate')"
  # Schema is REQUIRED (fail-closed): a registered Goal must carry a '## Goal gate' section
  # with a final 'GOAL E2E' box — a missing/typo'd schema cannot silently bypass G4. Checkbox
  # patterns accept any GFM marker ([-*+]) + indentation, matching the task side.
  if [ -z "$gate" ]; then p="$p $g has no '## Goal gate' section (required gate schema missing);"; fi
  if ! printf '%s\n' "$gate" | grep -qiE '^[[:space:]]*[-*+][[:space:]]+\[[ xX]\] GOAL E2E'; then p="$p $g Goal gate is missing the final 'GOAL E2E' checkbox row;"; fi
  if printf '%s\n' "$gate" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[ \]'; then p="$p $g has unchecked Goal-gate boxes (milestone/Goal E2E);"; fi
  # Milestone headings match at ANY ATX level (#…######, incl. ≤3-space indent), case-insensitively,
  # normalized to upper-case 'M<n>' — so '# M1', '#### M1', '## M1', '### m1', '   ## M1' can't dodge
  # the required 'M<n> E2E' box. The trailing '([^0-9A-Za-z]|$)' is a word boundary so a topic heading
  # like '## M2M transport' or '## Milestones' is NOT misread as milestone 'M2'/'M'. (Setext headings
  # and 'M<n>' not as the first heading token are accepted blind spots — see HONEST SCOPE.)
  for m in $(grep -iE '^[[:space:]]*#{1,}[[:space:]]+M[0-9]+([^0-9A-Za-z]|$)' -- "$g" | grep -oiE '#{1,}[[:space:]]+M[0-9]+' | grep -oiE 'M[0-9]+' | tr '[:lower:]' '[:upper:]' | sort -u); do
    printf '%s\n' "$gate" | grep -qiE "^[[:space:]]*[-*+][[:space:]]+\[[xX]\] $m E2E" || p="$p $g milestone $m has no ticked '$m E2E' Goal-gate box;"
  done
done

[ -z "$p" ] && exit 0
reason="full-cycle gate incomplete —$p  Resolve these, or remove a doc from .fullcycle-active to pause it. (Tripwire over registered docs + Codex review, not a sandbox: a ticked box is self-attested — do not fake it.)"
jq -n --arg r "$reason" '{decision:"block",reason:$r}'
