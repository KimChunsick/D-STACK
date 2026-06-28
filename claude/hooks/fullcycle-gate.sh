#!/bin/bash
# Stop hook — full-cycle gate. Blocks the turn from ending while registered work is incomplete.
#
# HONEST SCOPE: this is a tripwire over the registered docs + the Codex review, NOT a sandbox.
# It cannot prove TDD/E2E actually ran (a checkbox is self-attested). What it DOES enforce
# mechanically: unchecked gate boxes block; tasks require a single registered Goal; every
# milestone heading needs a ticked milestone-E2E box; a ticked Codex gate requires a real
# codex-review.md with consensus. Together with the inject hook + adversarial Codex review,
# skipping becomes costly and visible — defense in depth, not a single airtight gate.
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

# Body of a "## <heading…>" section (prefix match), up to the next "## " line.
section() { awk -v h="## $2" 'index($0,h)==1{f=1;next} /^## /{f=0} f' "$1"; }

p=""   # accumulated problems

[ "${#goals[@]}" -gt 1 ] && p="$p more than one GOAL.md is active (exactly one Goal allowed);"
{ [ "${#tasks[@]}" -gt 0 ] && [ "${#goals[@]}" -eq 0 ]; } && p="$p task(s) active without a registered GOAL.md;"

for t in "${tasks[@]}"; do
  if section "$t" 'Gate status' | grep -qE '^- \[ \]'; then p="$p $t has unchecked task gates;"; fi
  if section "$t" 'Gate status' | grep -qiE '^- \[x\].*codex'; then
    d="$(dirname -- "$t")"
    if [ ! -s "$d/codex-review.md" ] || ! grep -qiE 'consensus:.*(agreed|resolved)' -- "$d/codex-review.md"; then
      p="$p $t ticked the Codex gate but $d/codex-review.md lacks an agreed/resolved consensus;"
    fi
  fi
done

for g in "${goals[@]}"; do
  gate="$(section "$g" 'Goal gate')"
  # Schema is REQUIRED (fail-closed): a registered Goal must carry a '## Goal gate' section
  # with a final 'GOAL E2E' box — a missing/typo'd schema cannot silently bypass G4.
  if [ -z "$gate" ]; then p="$p $g has no '## Goal gate' section (required gate schema missing);"; fi
  if ! printf '%s\n' "$gate" | grep -qiE '^- \[[ x]\] GOAL E2E'; then p="$p $g Goal gate is missing the final 'GOAL E2E' checkbox row;"; fi
  if printf '%s\n' "$gate" | grep -qE '^- \[ \]'; then p="$p $g has unchecked Goal-gate boxes (milestone/Goal E2E);"; fi
  for m in $(grep -oE '^### M[0-9]+' -- "$g" | grep -oE 'M[0-9]+' | sort -u); do
    printf '%s\n' "$gate" | grep -qiE "^- \[x\] $m E2E" || p="$p $g milestone $m has no ticked '$m E2E' Goal-gate box;"
  done
done

[ -z "$p" ] && exit 0
reason="full-cycle gate incomplete —$p  Resolve these, or remove a doc from .fullcycle-active to pause it. (Tripwire over registered docs + Codex review, not a sandbox: a ticked box is self-attested — do not fake it.)"
jq -n --arg r "$reason" '{decision:"block",reason:$r}'
