# parity step: the review bundle — dstack review and check review-bundle (R13), with refusals (R11)
. "$PARITY_LIB"

run_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$run_id"
review_dir="$run_dir/review"

# The step owns the run's request and plan. Earlier steps leave their own behind and the first
# three refusals are about a run that has neither, so the step starts from nothing either way.
rm -f "$run_dir/request.md" "$run_dir/request.approved" "$run_dir/plan.json" "$run_dir/findings.md"
rm -rf "$review_dir"

# request approve syncs the ledger, which this step has no opinion about; the stamp file is all
# `dstack review` reads, so the step writes it the way step 23 does.
approve() {
  printf 'sha256 %s  approved_at %s\n' \
    "$(shasum -a 256 "$run_dir/request.md" | awk '{print $1}')" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$run_dir/request.approved"
}

# ── refusals before there is anything to bundle ────────────────────────────────────────
call review-no-args        -- "$DSTACK" review
call review-bad-scope      -- "$DSTACK" review --scope bogus
call review-plan-no-id     -- "$DSTACK" review --scope plan
call review-ms-no-id       -- "$DSTACK" review --scope milestone
call review-unknown-arg    -- "$DSTACK" review --scope plan --plan P1 --nope x
call review-scope-no-value -- "$DSTACK" review --scope
call review-plan-no-value  -- "$DSTACK" review --scope plan --plan
call review-no-request     -- "$DSTACK" review --scope plan --plan P1

cat > "$run_dir/request.md" <<'REQ'
---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: on
codex_effort: high
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# review parity request

- [ ] **R01** the bundle carries the request rows verbatim — accept: the REQUEST section repeats the approved line byte for byte
- [ ] **R02** the bundle carries only the plan's declared files — accept: no diff hunk names a file outside plan.files
- [ ] **R03** one file may not drown the other files' diffs — accept: a diff over 64KB is replaced by the skip line
- [ ] **R04** the ceiling refuses instead of truncating — accept: exit 1 and the reason names the ceiling
REQ

call review-not-approved -- "$DSTACK" review --scope plan --plan P1
approve
call review-no-plan      -- "$DSTACK" review --scope plan --plan P1

# plan add and task add belong to P10, so the step writes the file they produce. The declared
# worktree is "." on purpose: a bundle that carries the sandbox path would differ in length
# between the two sandboxes, and its byte count is one of the lines under comparison.
cat > "$run_dir/plan.json" <<'JSON'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"bundle","order":1}, {"id":"M2","slug":"ceiling","order":2} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"bundle-writer","files":["lib","docs"],"deps":[],
              "status":"in-progress","worktree":".","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"emit-diff","covers":["R01","R02"],"files":["lib"],
                         "deps":[],"commit":"","done_at":""},
                        {"id":"T2","slug":"cap-one-file","covers":["R03"],"files":["lib"],
                         "deps":["T1"],"commit":"","done_at":""} ] },
             {"id":"P2","milestone":"M2","slug":"ceiling","files":["big"],"deps":["P1"],
              "status":"ready","worktree":".","started_at":"","done_at":"",
              "tasks":[ {"id":"T3","slug":"refuse","covers":["R04"],"files":["big"],
                         "deps":[],"commit":"","done_at":""} ] },
             {"id":"P3","milestone":"M2","slug":"meta-worktree","files":["docs"],"deps":[],
              "status":"ready","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T4","slug":"fallback","covers":["R04"],"files":["docs"],
                         "deps":[],"commit":"","done_at":""} ] } ] }
JSON

call review-plan-missing -- "$DSTACK" review --scope plan --plan P9
call review-ms-missing   -- "$DSTACK" review --scope milestone --milestone M9
# quick new belongs to P14: the directory is what resolve_target looks for, so the step makes one.
mkdir -p "$SANDBOX/.dstack/quick/demo"
call review-quick        -- "$DSTACK" review --scope plan --plan P1 --quick demo

# ── the three branches of _emit_diff ───────────────────────────────────────────────────
# tracked (staged against the run's base_head), untracked, and one file over the 64KB cap. The
# second declared path stays empty so the "declared but never touched" branch is covered too.
mkdir -p "$SANDBOX/lib"
cat > "$SANDBOX/lib/tool.sh" <<'SH'
#!/usr/bin/env bash
# the helper the plan declared
say() { printf '%s\n' "$*"; }
SH
( cd "$SANDBOX" && git add lib/tool.sh )
cat > "$SANDBOX/lib/new.sh" <<'SH'
#!/usr/bin/env bash
# a file git has never seen; R69 wants it in the bundle all the same
SH
awk 'BEGIN{ for (i = 1; i <= 1200; i++) printf "%04d a line of the file whose diff must not drown the others\n", i }' \
  > "$SANDBOX/lib/huge.txt"

call review-plan     -- "$DSTACK" review --scope plan --plan P1
call review-plan-2   -- "$DSTACK" review --scope plan --plan P1
call review-plan-out -- "$DSTACK" review --scope plan --plan P1 --out out/mine.txt

# A plan with no worktree of its own falls back to meta.tsv, which holds the sandbox path: the
# two sandbox paths differ in length, so the byte count of that one bundle is masked. The bundle
# file itself is still compared byte for byte with the path normalized.
mask_call review-meta-wt 'bytes [0-9]+ of 512000' 'bytes <BYTES> of 512000'
call review-meta-wt  -- "$DSTACK" review --scope plan --plan P3

# ── the 512KB ceiling ──────────────────────────────────────────────────────────────────
mkdir -p "$SANDBOX/big"
i=1
while [ "$i" -le 10 ]; do
  awk -v n="$i" 'BEGIN{ for (j = 1; j <= 900; j++) printf "%s %04d a line of a file the plan declared and nobody will read\n", n, j }' \
    > "$SANDBOX/big/part-$i.txt"
  i=$((i + 1))
done
call review-ceiling -- "$DSTACK" review --scope plan --plan P2

# ── check review-bundle ────────────────────────────────────────────────────────────────
call check-bundle-usage   -- "$DSTACK" check review-bundle
call check-bundle-missing -- "$DSTACK" check review-bundle nosuch.txt
call check-bundle-good    -- "$DSTACK" check review-bundle "$review_dir/bundle-plan-P1-001.txt"
call check-bundle-noplan  -- "$DSTACK" check review-bundle "$review_dir/bundle-plan-P1-001.txt" --quick demo

cat > "$SANDBOX/no-scope.txt" <<'B'
=== REQUEST (frozen) ===
- [ ] **R01** the bundle carries the request rows verbatim — accept: the REQUEST section repeats the approved line byte for byte
B
call check-bundle-noscope -- "$DSTACK" check review-bundle no-scope.txt

cat > "$SANDBOX/bad-bundle.txt" <<'B'
=== REQUEST (frozen) ===
- [ ] **R01** the bundle carries the request rows verbatim — accept: the REQUEST section repeats the approved line byte for byte
- [ ] **R09** a row the plan never covered — accept: the checker names it

=== PLAN ===
plan: P1
slug: bundle-writer
status: in-progress
files: lib, docs
deps: (none)
T1 emit-diff covers: R02, R03 files: lib

=== CONTRACT ===
Your last line is `VERDICT: approve|reject`.
B
call check-bundle-bad -- "$DSTACK" check review-bundle bad-bundle.txt

cat > "$SANDBOX/empty-request.txt" <<'B'
=== REQUEST (frozen) ===
(the plan's tasks cover nothing, so no rows were copied)

=== PLAN ===
plan: P1
T1 emit-diff covers: R01, R02, R03 files: lib

=== CONTRACT ===
Your last line is `VERDICT: approve|reject`.
B
call check-bundle-empty -- "$DSTACK" check review-bundle empty-request.txt

# ── the milestone bundle ───────────────────────────────────────────────────────────────
call review-ms-nofindings -- "$DSTACK" review --scope milestone --milestone M1

cat > "$run_dir/findings.md" <<'F'
# findings — the review parity run

- the per-file cap is silent about which plan to split — resolved in P1
* an item that was closed the round before — resolved
F
call review-ms-noopen -- "$DSTACK" review --scope milestone --milestone M1

cat >> "$run_dir/findings.md" <<'F'
- the ceiling refusal names the ceiling but not the plan
  * an indented item the reader still owes
F
call review-ms -- "$DSTACK" review --scope milestone --milestone M1
call check-bundle-ms -- "$DSTACK" check review-bundle "$review_dir/bundle-milestone-M1-003.txt"

# A finding that names an R id the milestone never covered breaks the R69 count, so the bundle
# `review` just wrote is deleted again instead of shipping.
printf -- '- a finding that names R09, a row this milestone never covered\n' >> "$run_dir/findings.md"
call review-ms-deleted -- "$DSTACK" review --scope milestone --milestone M1

# ── review seal ────────────────────────────────────────────────────────────────────────
cat > "$SANDBOX/round.md" <<'M'
# codex review — P1

| R | verdict | evidence |
|---|---|---|
| R01 | covered | the REQUEST section repeats the approved rows |
| R02 | covered | the DIFF section names lib/ and docs and nothing else |
| R03 | partial | the skip line does not say how big the diff was |

[goal achievement] MINOR: the ceiling refusal never names a plan to split — bundle.rs

VERDICT: approve
M
# The tab-indented row is the point of the third one: _verdict_count reads spaces and tabs alike.
{ printf '| R01 | covered | the frozen rows |\n'
  printf '| R02 | absent | nothing in the diff touches it |\n'
  printf '\t| R03 |\tabsent\t| a tab-indented row counts too |\n'
  printf '\nVERDICT: reject\n'
} > "$SANDBOX/round-absent.md"
printf '# a round that judged nothing\n\nVERDICT: approve\n' > "$SANDBOX/round-no-rows.md"
printf '| R01 | covered | the frozen rows |\n\nverdict: approve\n' > "$SANDBOX/round-no-verdict.md"

call seal-no-args        -- "$DSTACK" review seal
call seal-unknown-arg    -- "$DSTACK" review seal --nope x
call seal-from-no-value  -- "$DSTACK" review seal --from
call seal-missing-file   -- "$DSTACK" review seal --from nosuch.md --scope plan --id P1
call seal-bad-scope      -- "$DSTACK" review seal --from round.md --scope bogus --id P1
call seal-no-id          -- "$DSTACK" review seal --from round.md --scope plan
call seal-quick-on-run   -- "$DSTACK" review seal --from round.md --scope quick
call seal-no-rows        -- "$DSTACK" review seal --from round-no-rows.md --scope plan --id P1
call seal-no-verdict     -- "$DSTACK" review seal --from round-no-verdict.md --scope plan --id P1
call seal                -- "$DSTACK" review seal --from round.md --scope plan --id P1
call seal-absent         -- "$DSTACK" review seal --from round-absent.md --scope plan --id P1
call seal-milestone      -- "$DSTACK" review seal --from round.md --scope milestone --id M1
call seal-quick          -- "$DSTACK" review seal --from round.md --quick demo --scope quick

# The integration table of a milestone bundle reads the sealed rounds back: the latest round of
# each plan wins, so this bundle must show the absent verdicts of round 002 and not of round 001.
cat > "$run_dir/findings.md" <<'F'
# findings — the review parity run

- the ceiling refusal names the ceiling but not the plan
F
call review-ms-sealed -- "$DSTACK" review --scope milestone --milestone M1

# ── review close ───────────────────────────────────────────────────────────────────────
call close-no-args        -- "$DSTACK" review close
call close-unknown-arg    -- "$DSTACK" review close --nope x
call close-scope-no-value -- "$DSTACK" review close --scope
call close-bad-scope      -- "$DSTACK" review close --scope bogus --why "no such scope"
call close-plan-no-id     -- "$DSTACK" review close --scope plan --why "no id at all"
call close-plan-missing   -- "$DSTACK" review close --scope plan --id P9 --why "no such plan"
call close-ms-missing     -- "$DSTACK" review close --scope milestone --id M9 --why "no such milestone"
call close-quick-on-run   -- "$DSTACK" review close --scope quick --why "the target is a run"
call close-plan           -- "$DSTACK" review close --scope plan --id P1 --why "the reviewer has no machine this week"
call close-plan-again     -- "$DSTACK" review close --scope plan --id P1 --why "the reviewer has no machine this week"
call close-ms             -- "$DSTACK" review close --scope milestone --id M1 --why "the milestone pass waits for the last plan"

# A quick task closes on its own live rows; the second one has none left to close.
cat > "$SANDBOX/.dstack/quick/demo/request.md" <<'REQ'
---
work_type: cli
route: quick
external_research: none
risk_axes: none
design_review: skip
review: on
codex_effort: medium
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# the quick task of the review step

- [ ] **R01** the quick task closes its review on purpose — accept: the R id reads ABSTAIN
- [ ] **R02** the closed record keeps the reason — accept: closed.tsv carries the why column
REQ
mkdir -p "$SANDBOX/.dstack/quick/empty"
cat > "$SANDBOX/.dstack/quick/empty/request.md" <<'REQ'
---
work_type: cli
route: quick
external_research: none
risk_axes: none
design_review: skip
review: off
codex_effort: medium
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# a quick task whose only row was withdrawn

- [ ] **R01** a row the owner dropped — accept: nothing — withdrawn: the owner dropped it
REQ
call close-quick-no-live -- "$DSTACK" review close --quick empty --scope quick --why "nothing left to close"
call close-quick         -- "$DSTACK" review close --quick demo --scope quick --why "the quick task ends its review here"
call close-quick-again   -- "$DSTACK" review close --quick demo --scope quick --why "the quick task ends its review here"

# The two branches that need the file gone. plan.json comes back straight after, so nothing the
# store comparison sees depends on the order of these two calls.
mv "$run_dir/plan.json" "$SANDBOX/plan.json.aside"
call close-plan-no-json -- "$DSTACK" review close --scope plan --id P1 --why "there is no plan file"
call close-ms-no-json   -- "$DSTACK" review close --scope milestone --id M1 --why "there is no plan file"
mv "$SANDBOX/plan.json.aside" "$run_dir/plan.json"
