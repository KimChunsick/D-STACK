# parity step: the ledger verbs — cases sync/render (R13), with their refusals (R11)
. "$PARITY_LIB"

run_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$run_id"

# request approve belongs to P6; this step writes the two files it produces, byte for byte the
# same on both sides, so the ledger verbs are the only difference the harness can see.
approve() {
  printf 'sha256 %s  approved_at %s\n' \
    "$(shasum -a 256 "$run_dir/request.md" | awk '{print $1}')" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$run_dir/request.approved"
}

# ── cases sync / render ────────────────────────────────────────────────────────────────
call cases-render-none     -- "$DSTACK" cases render
call cases-sync-no-request -- "$DSTACK" cases sync

cat > "$run_dir/request.md" <<'REQ'
---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: off
codex_effort: high
e2e: cli
unit_tests: off
visual: none
korean_polish: off
---
# ledger parity request

- [ ] **R01** the command prints what it counted — accept: stdout carries "checked N"
- [ ] **R02** the command refuses bad input — accept: exit code 1 with a reason
REQ

call cases-sync-unapproved -- "$DSTACK" cases sync
approve
call cases-sync            -- "$DSTACK" cases sync
call cases-sync-again      -- "$DSTACK" cases sync
call cases-render          -- "$DSTACK" cases render
call cases-render-extra    -- "$DSTACK" cases render extra
call cases-sync-positional -- "$DSTACK" cases sync "$run_id"
call cases-sync-no-run     -- "$DSTACK" cases sync nosuch
call cases-sync-bogus      -- "$DSTACK" cases sync --bogus

# quick new belongs to P14: the directory is what resolve_target looks for, so the step makes one.
mkdir -p "$SANDBOX/.dstack/quick/demo"
call cases-sync-both       -- "$DSTACK" cases sync "$run_id" --quick demo
call cases-sync-quick      -- "$DSTACK" cases sync --quick demo

# The second request adds the three marker rows, a pending row and two live children, and turns
# unit_tests on: sync keeps every row it already wrote and expands only what is new.
cat > "$run_dir/request.md" <<'REQ'
---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: skip
review: off
codex_effort: high
e2e: cli
unit_tests: on
visual: none
korean_polish: off
---
# ledger parity request

- [ ] **R01** the command prints what it counted — accept: stdout carries "checked N"
- [ ] **R02** the command refuses bad input — accept: exit code 1 with a reason
- [ ] **R03** a row the owner dropped — accept: nothing — withdrawn: the owner dropped it
- [ ] **R04** a row for a later Goal — accept: nothing — deferred: the next Goal
- [ ] **R05** a row that was split — accept: nothing — superseded-by: R06,R07
- [ ] **R06** the first child row — accept: the ledger names it
- [ ] **R07** the second child row — accept: the ledger names it
- [ ] **R08** a row nobody approved yet — accept: it prints — status: pending-approval
REQ

call cases-sync-markers   -- "$DSTACK" cases sync
call cases-render-markers -- "$DSTACK" cases render

# ── evidence add / retire ──────────────────────────────────────────────────────────────
# The artifacts live outside .dstack, so the store comparison sees only what the verbs wrote.
mkdir -p "$SANDBOX/artifacts"
printf 'R01 and R02 verified: checked 2, missing 0\n' > "$SANDBOX/artifacts/r01.txt"
printf 'R06 verified: checked 1, missing 0\n'         > "$SANDBOX/artifacts/r06.txt"
printf 'the run that names no requirement at all\n'   > "$SANDBOX/artifacts/quiet.txt"
printf 'a note the reviewer left behind\n'            > "$SANDBOX/artifacts/note.md"
printf 'R01 was proven a day before this run opened\n' > "$SANDBOX/artifacts/old.txt"
touch -t "$(date -u -v-1d +%Y%m%d%H%M 2>/dev/null || date -u -d '1 day ago' +%Y%m%d%H%M)" \
  "$SANDBOX/artifacts/old.txt"
: > "$SANDBOX/artifacts/empty.txt"

call ev-missing-option -- "$DSTACK" evidence add --r R01
call ev-cap-r          -- "$DSTACK" evidence add --R R01 --case c1 --kind cli --artifact artifacts/nosuch.txt --produced-by cmd
call ev-cap-r-equals   -- "$DSTACK" evidence add --R=R01 --case c1 --kind cli --artifact artifacts/nosuch.txt --produced-by cmd
call ev-unknown-arg    -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd --nope x
call ev-no-row         -- "$DSTACK" evidence add --r R99 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd
call ev-withdrawn      -- "$DSTACK" evidence add --r R03 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd
call ev-deferred       -- "$DSTACK" evidence add --r R04 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd
call ev-superseded     -- "$DSTACK" evidence add --r R05 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd
call ev-bad-kind       -- "$DSTACK" evidence add --r R01 --case c1 --kind bogus --artifact artifacts/r01.txt --produced-by cmd
call ev-bad-status     -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by cmd --status bogus
call ev-no-directory   -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact nosuch/dir/x.txt --produced-by cmd
call ev-no-file        -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/nosuch.txt --produced-by cmd
call ev-zero-byte      -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/empty.txt --produced-by cmd
call ev-old-mtime      -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/old.txt --produced-by cmd
call ev-no-mention     -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/quiet.txt --produced-by cmd

call ev-add            -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by "bash run.sh --only 23-ledger"
call ev-add-again      -- "$DSTACK" evidence add --r R01 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by "bash run.sh --only 23-ledger"
call ev-shared-refused -- "$DSTACK" evidence add --r R02 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by "bash run.sh"
call ev-shared         -- "$DSTACK" evidence add --r R02 --case c1 --kind cli --artifact artifacts/r01.txt --produced-by "bash run.sh" --shared "one run proves both rows" --note "the second row"
call ev-review-row     -- "$DSTACK" evidence add --r R01 --case c2 --kind review --artifact artifacts/note.md --produced-by "the reviewer" --status abstain --note "no runner on this machine"
call ev-blocked-row    -- "$DSTACK" evidence add --r R07 --case c1 --kind review --artifact artifacts/note.md --produced-by "the reviewer" --status blocked --shared "the note answers both children"
call ev-absolute       -- "$DSTACK" evidence add --r R06 --case c-test --kind test --artifact "$SANDBOX/artifacts/r06.txt" --produced-by "cargo test"
call cases-render-evidence -- "$DSTACK" cases render

call ev-retire-bogus   -- "$DSTACK" evidence retire --bogus
call ev-retire-missing -- "$DSTACK" evidence retire --r R01
call ev-retire-absent  -- "$DSTACK" evidence retire --r R01 --case zz --why "nothing to retire"
call ev-retire-open    -- "$DSTACK" evidence retire --r R06 --case c1 --why "still open"
call ev-retire         -- "$DSTACK" evidence retire --r R01 --case c1 --why "the artifact was overwritten"
call ev-retire-again   -- "$DSTACK" evidence retire --r R01 --case c1 --why "the artifact was overwritten"
call cases-render-retired -- "$DSTACK" cases render

# The quick branch of the start instant: a quick task has no meta.tsv, so its request.md is the
# earliest an artifact may be (there is no STATE.md row here either).
cat > "$SANDBOX/.dstack/quick/demo/request.md" <<'REQ'
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
# the quick task of the ledger step

- [ ] **R01** the quick task prints its counts — accept: stdout carries "checked N"
REQ
printf 'sha256 %s  approved_at %s\n' \
  "$(shasum -a 256 "$SANDBOX/.dstack/quick/demo/request.md" | awk '{print $1}')" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$SANDBOX/.dstack/quick/demo/request.approved"
printf 'R01 verified in the quick task: checked 1, missing 0\n' > "$SANDBOX/artifacts/quick-r01.txt"

call cases-sync-quick-ok -- "$DSTACK" cases sync --quick demo
call ev-quick            -- "$DSTACK" evidence add --quick demo --r R01 --case c1 --kind cli --artifact artifacts/quick-r01.txt --produced-by "the quick task"
call ev-quick-old        -- "$DSTACK" evidence add --quick demo --r R01 --case c2 --kind cli --artifact artifacts/old.txt --produced-by "the quick task"
call cases-render-quick  -- "$DSTACK" cases render --quick demo

# ── check coverage / worker report ─────────────────────────────────────────────────────
# The long R01 line is ASCII on purpose: `cut -c1-120` counts characters under a UTF-8 locale
# and bytes under C, so a line that has to be cut must read the same either way.
cat > "$SANDBOX/artifacts/report-all.txt" <<'RPT'
## Report
run verify: ok
R01: satisfied - the parity step recorded artifacts/r01.txt and the harness compared both stores byte for byte, so this line runs past the hundred and twentieth character
R02: satisfied — artifacts/r01.txt
R07: blocked — the owner has to choose between the two children
R09: unsatisfied — nothing proves it yet
RPT
cat > "$SANDBOX/artifacts/report-partial.txt" <<'RPT'
## Report
run verify: ok
R01: satisfied — artifacts/r01.txt
RPT

call check-coverage-no-plan -- "$DSTACK" check coverage
call worker-no-plan-file    -- "$DSTACK" worker report --plan P1 --from artifacts/report-all.txt

# A live row with a covering task and no evidence at all, and the plan that covers it. plan add
# and task add belong to P10, so the step writes the file they produce.
printf -- '- [ ] **R09** a row with a task and no evidence — accept: the coverage line says MISSING(evidence)\n' >> "$run_dir/request.md"
cat > "$run_dir/plan.json" <<'JSON'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"ledger","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"ledger","files":["artifacts"],"deps":[],
              "status":"in-progress","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"ledger","covers":["R01","R02","R07","R09"],"files":["artifacts"],
                         "deps":[],"commit":"","done_at":""} ] } ] }
JSON

call check-coverage         -- "$DSTACK" check coverage
call check-coverage-extra   -- "$DSTACK" check coverage extra
call check-coverage-quick   -- "$DSTACK" check coverage --quick demo
call check-coverage-no-run  -- "$DSTACK" check coverage --run nosuch

call worker-bogus        -- "$DSTACK" worker report --bogus
call worker-no-from      -- "$DSTACK" worker report --plan P1
# worker report's own loop takes only the two-word form, so `--plan=P1` is the usage error.
call worker-eq-plan      -- "$DSTACK" worker report --plan=P1 --from artifacts/report-all.txt
call worker-no-file      -- "$DSTACK" worker report --plan P1 --from artifacts/nope.txt
call worker-quick        -- "$DSTACK" worker report --plan P1 --from artifacts/report-all.txt --quick demo
call worker-no-plan      -- "$DSTACK" worker report --plan P9 --from artifacts/report-all.txt
call worker-all-reported -- "$DSTACK" worker report --plan P1 --from artifacts/report-all.txt
call worker-partial      -- "$DSTACK" worker report --plan P1 --from artifacts/report-partial.txt
call check-coverage-unreported -- "$DSTACK" check coverage
call cases-render-final  -- "$DSTACK" cases render

# A value-taking option as the last argument: the shell's `shift 2` fails and set -e ends the
# run with exit 1 and nothing printed at all.
call worker-plan-no-value    -- "$DSTACK" worker report --plan
call ev-cap-r-no-value       -- "$DSTACK" evidence add --R
call ev-r-no-value           -- "$DSTACK" evidence add --r
call ev-retire-case-no-value -- "$DSTACK" evidence retire --r R01 --case

# `dstack worker report` with no arguments at all: the shell expands an empty REST array under
# set -u, so bash ends it with its own error line naming the reference's worker.sh — a path no mask
# covers and a crash D-11 says the port does not reproduce. Both lines are removed here; the
# exit code is what the two implementations still have to agree on.
expect_diff worker-no-args "D-11: the shell crashes on an empty REST array (set -u) where the port prints its usage line"
mask_call worker-no-args '^.*worker\.sh: line [0-9]+: REST\[@\]: unbound variable$' ''
mask_call worker-no-args '^dstack: usage: dstack worker report .*$' ''
call worker-no-args -- "$DSTACK" worker report
