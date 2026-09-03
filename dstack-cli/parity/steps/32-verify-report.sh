# parity step: verify and report — the policy ceiling, the per-R states, the accepts, the branch
# line, the R79 table and the R01 metrics (R13), with one wrong-usage call per verb (R11)
. "$PARITY_LIB"

TAB="$PARITY_TAB"
run_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$run_id"

# verify and report read the request, the ledger, the plan and the review directory, and the
# steps before this one leave all four behind in the same sandbox. So the step starts from an
# empty ledger and writes every file it reads itself; both sandboxes are reset the same way, so
# the harness still compares like with like.
rm -f "$run_dir/request.md" "$run_dir/request.approved" "$run_dir/request.agent-draft.md" \
      "$run_dir/cases.tsv" "$run_dir/accepts.tsv" "$run_dir/metrics.tsv" "$run_dir/plan.json"
rm -f "$run_dir"/review/codex-review-*.md "$run_dir/review/index.tsv" "$run_dir/review/closed.tsv"
mkdir -p "$run_dir/review" "$SANDBOX/artifacts"

# meta.tsv rows are written by run new and by the Stop hook; the branch containment and the
# metrics read three of them, so the step sets them the way those two writers would.
meta_row() {   # $1 key, $2 value
  awk -F"$TAB" -v OFS="$TAB" -v k="$1" -v v="$2" '$1==k{$2=v} {print}' "$run_dir/meta.tsv" \
    > "$run_dir/meta.new"
  mv "$run_dir/meta.new" "$run_dir/meta.tsv"
}

# ── nothing to verify yet, and the wrong usage (R11) ───────────────────────────────────
call verify-no-request   -- "$DSTACK" verify
call verify-bogus        -- "$DSTACK" verify --bogus
call verify-positional   -- "$DSTACK" verify extra
call verify-no-run       -- "$DSTACK" verify --run nosuch
call verify-no-quick     -- "$DSTACK" verify --quick nosuch
# A value option in the last position: the shell's `shift 2` fails under set -e, which ends the
# command with exit 1 and nothing printed at all.
call verify-accept-noval -- "$DSTACK" verify --accept-abstain
call verify-why-noval    -- "$DSTACK" verify --why
call report-no-request   -- "$DSTACK" report
call report-bogus        -- "$DSTACK" report --bogus
call report-positional   -- "$DSTACK" report extra
call report-no-run       -- "$DSTACK" report --run nosuch
call report-no-quick     -- "$DSTACK" report --quick nosuch
call report-metrics-none -- "$DSTACK" report --metrics

# ── the request this step verifies ─────────────────────────────────────────────────────
# work_type cli brings unit_tests: on, so cases sync opens a c-test row next to every c1 row and
# a row is only PASS once both kinds carry recorded evidence (R74).
call request-new -- "$DSTACK" request new --type cli --title "the verify parity request"
call req-add-01 -- "$DSTACK" req add "the row with untouched cli evidence" --accept "verify prints R01 ok"
call req-add-02 -- "$DSTACK" req add "the row nothing proves yet" --accept "verify names the missing evidence"
call req-add-03 -- "$DSTACK" req add "the row the evidence could not decide" --accept "verify reads ABSTAIN"
call req-add-04 -- "$DSTACK" req add "the row the evidence found blocked" --accept "verify reads BLOCKED"
call req-add-05 -- "$DSTACK" req add "the row one case skipped" --accept "the report reads SKIPPED"
call req-add-06 -- "$DSTACK" req add "the row the owner dropped" --accept "the report reads WITHDRAWN"
call req-add-07 -- "$DSTACK" req add "the row of a later Goal" --accept "the report reads DEFERRED"
call req-add-08 -- "$DSTACK" req add "the row that was split" --accept "the report names its children"
call req-add-09 -- "$DSTACK" req add "the first child row" --accept "a sealed round judges it"
call req-add-10 -- "$DSTACK" req add "the second child row" --accept "the worker reports on it"
call req-add-11 -- "$DSTACK" req add "the row whose review was closed" --accept "verify reads ABSTAIN"

call req-withdraw -- "$DSTACK" req withdraw R06 --why "the owner dropped it"
call req-defer    -- "$DSTACK" req defer R07 --why "the next Goal takes it"
call req-split    -- "$DSTACK" req split R08 --into R09,R10
call request-approve -- "$DSTACK" request approve

# A row nobody approved yet is not live: verify skips it and the report reads it as UNMET:
# pending-approval. `req add --from-answer` mints such a row from the question ledger, which
# belongs to another step, so the marker is written here directly.
printf -- '- [ ] **R12** the row nobody approved yet — accept: the report prints it — status: pending-approval\n' \
  >> "$run_dir/request.md"
call cases-sync-pending -- "$DSTACK" cases sync

# plan add and task add are ported by P10; until that lands the file they write is written here,
# covering every live row so the report's coverage column is not the reason a row reads UNMET.
cat > "$run_dir/plan.json" <<'JSON'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"verify","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"verify","files":["artifacts"],"deps":[],
              "status":"in-progress","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"verify","covers":["R01","R02","R03","R04","R05","R09","R10","R11"],
                         "files":["artifacts"],"deps":[],"commit":"","done_at":""} ] } ] }
JSON

# ── the evidence rows the states are read from ─────────────────────────────────────────
for r in R01 R02 R03 R04 R05 R09 R10 R11; do
  printf '%s verified: checked 1, missing 0\n' "$r" > "$SANDBOX/artifacts/v-$r.txt"
done

# One artifact proves both rows of an R (a second row of the same R needs no --shared), so a
# status is one option away from the row above it.
rows() {   # $1 R, $2 status, $3 note
  call "ev-$1-cli" -- "$DSTACK" evidence add --r "$1" --case c1 --kind cli \
    --artifact "artifacts/v-$1.txt" --produced-by "the parity step" --status "$2" --note "$3"
  call "ev-$1-test" -- "$DSTACK" evidence add --r "$1" --case c-test --kind test \
    --artifact "artifacts/v-$1.txt" --produced-by "the parity step" --status "$2" --note "$3"
}
rows R01 met "-"
rows R03 abstain "no runner on this machine"
rows R04 blocked "the owner has to choose between the two children"
rows R05 skipped "the case moved to the next Goal"
rows R09 met "-"
rows R11 met "-"

# R10 keeps a worker's silence in the ledger (R68): the plan covers it and the report below does
# not name it, so `worker report` writes the unreported row verify has to refuse.
cat > "$SANDBOX/artifacts/worker.txt" <<'RPT'
## Report
run verify: ok
R01: satisfied — artifacts/v-R01.txt
R02: unsatisfied — nothing proves it yet
R03: satisfied — artifacts/v-R03.txt
R04: blocked — the owner has to choose between the two children
R05: satisfied — artifacts/v-R05.txt
R09: satisfied — artifacts/v-R09.txt
R11: satisfied — artifacts/v-R11.txt
RPT
call worker-silence -- "$DSTACK" worker report --plan P1 --from artifacts/worker.txt
rows R10 met "-"

# review seal is ported by P11; the round it writes is written here, so this step depends on no
# verb outside verify and report. The first round judges R09 partial and says nothing about R02,
# R10 or R11 — the three cases the second round and the close below separate.
cat > "$run_dir/review/codex-review-001.md" <<'ROUND'
# codex review round 001

| R | verdict | evidence in the diff |
|---|---|---|
| R01 | covered | the cli row and the test row |
| R03 | covered | the reviewer read the note |
| R04 | covered | the reviewer read the note |
| R05 | covered | the reviewer read the note |
| R09 | partial | half of the row is in the diff |

VERDICT: revise
ROUND
# `review close` (P11) records the deliberate stop: R11 was never re-verified after it, so it
# reads ABSTAIN however clean its evidence is.
printf 'plan\tP1\t001\tR11\t2026-01-01T00:00:00Z\tthe reviewer stopped on this row\n' \
  > "$run_dir/review/closed.tsv"

# ── the broken state ───────────────────────────────────────────────────────────────────
call verify-first -- "$DSTACK" verify
call verify-run   -- "$DSTACK" verify --run "$run_id"
call report-first -- "$DSTACK" report
call report-run   -- "$DSTACK" report --run "$run_id"

# The policy block of PROJECT.md is the ceiling a request may only narrow (R75): asking for
# capture evidence where the repository verifies cli makes every row unverifiable.
sed 's/^e2e: cli$/e2e: capture/' "$run_dir/request.md" > "$run_dir/over.md"
mv "$run_dir/over.md" "$run_dir/request.md"
call verify-over-ceiling -- "$DSTACK" verify
call report-over-ceiling -- "$DSTACK" report
sed 's/^e2e: capture$/e2e: cli/' "$run_dir/request.md" > "$run_dir/over.md"
mv "$run_dir/over.md" "$run_dir/request.md"

# The accepts, one refusal at a time (R11): a row that is not live, a row that is not ABSTAIN,
# and an accept without the reason R79 puts in the report.
call verify-accept-no-why  -- "$DSTACK" verify --accept-abstain R03
call verify-accept-unknown -- "$DSTACK" verify --accept-abstain R99 --why "nothing to accept"
call verify-accept-passing -- "$DSTACK" verify --accept-abstain R01 --why "already proven"
call verify-accept-two     -- "$DSTACK" verify --accept-abstain R03,R04 --why "the owner accepted both"
call verify-accept-again   -- "$DSTACK" verify --accept-abstain=R03 --why="a second accept row"
call verify-accepted       -- "$DSTACK" verify
call report-accepted       -- "$DSTACK" report

# run close runs verify in process (P5): a verdict of 1 is the refusal below, never a crash.
call run-close-refused -- "$DSTACK" run close

# ── what the rows still miss ───────────────────────────────────────────────────────────
rows R02 met "-"
# A skipped case does not prove an R (R74), so R05 keeps its skipped row and gets a second case
# that does: verify reads it as proven, the report still reads the row as SKIPPED.
call ev-R05-cli-2 -- "$DSTACK" evidence add --r R05 --case c2 --kind cli \
  --artifact artifacts/v-R05.txt --produced-by "the parity step"
call ev-R05-test-2 -- "$DSTACK" evidence add --r R05 --case c2-test --kind test \
  --artifact artifacts/v-R05.txt --produced-by "the parity step"
call ev-R10-worker -- "$DSTACK" evidence add --r R10 --case c-worker-P1 --kind review \
  --artifact artifacts/v-R10.txt --produced-by "the parity step" --note "the worker reported late"
# A later round supersedes an older verdict: R09 was partial in 001 and is covered here.
cat > "$run_dir/review/codex-review-002.md" <<'ROUND'
# codex review round 002

| R | verdict | evidence in the diff |
|---|---|---|
| R02 | covered | the cli row and the test row |
| R09 | covered | the rest of the row landed |
| R10 | covered | the worker filled its row |

VERDICT: approve
ROUND

call verify-accept-closed -- "$DSTACK" verify --accept-abstain R11 --why "the closed review is accepted"
call verify-clean         -- "$DSTACK" verify
call report-clean         -- "$DSTACK" report

# ── the artifact behind a recorded row (R74) ───────────────────────────────────────────
# A recorded artifact that no longer hashes to what the ledger holds is the ledger being edited
# by hand; putting the bytes back puts the sha back, so the run stays verifiable afterwards.
printf 'edited by hand after it was recorded\n' >> "$SANDBOX/artifacts/v-R01.txt"
call verify-tampered -- "$DSTACK" verify
printf 'R01 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/v-R01.txt"
mv "$SANDBOX/artifacts/v-R01.txt" "$SANDBOX/artifacts/moved-away.txt"
call verify-artifact-gone -- "$DSTACK" verify
mv "$SANDBOX/artifacts/moved-away.txt" "$SANDBOX/artifacts/v-R01.txt"

# ── branch containment (R38) ───────────────────────────────────────────────────────────
# The three lines _verify_branch can print besides "branch = base": a base that is not
# resolvable, a branch that is not, and a branch that does not contain its base. The commit and
# the branch are made and removed here, so the sandbox git repository ends as it started.
head0="$(git -C "$SANDBOX" rev-parse HEAD)"
git -C "$SANDBOX" branch behind
git -C "$SANDBOX" -c user.email=t@t -c user.name=t -c commit.gpgsign=false \
  commit -q --allow-empty -m "one commit the branch does not have"
meta_row branch behind
call verify-behind -- "$DSTACK" verify
meta_row branch nosuch-branch
call verify-branch-gone -- "$DSTACK" verify
meta_row base_branch nosuch-base
call verify-base-gone -- "$DSTACK" verify
meta_row branch "$(git -C "$SANDBOX" rev-parse --abbrev-ref HEAD)"
meta_row base_branch "$(git -C "$SANDBOX" rev-parse --abbrev-ref HEAD)"
git -C "$SANDBOX" reset -q --hard "$head0"
git -C "$SANDBOX" branch -D behind >/dev/null

# ── a quick task: no branch containment, and review: off really skips the round (R99) ──
mkdir -p "$SANDBOX/.dstack/quick/verified"
cat > "$SANDBOX/.dstack/quick/verified/request.md" <<'REQ'
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
# the quick task of the verify step

- [ ] **R01** the quick task prints its counts — accept: stdout carries "checked N"
- [ ] **R02** the quick task refuses bad input — accept: exit code 1 with a reason
REQ
printf 'sha256 %s  approved_at %s\n' \
  "$(shasum -a 256 "$SANDBOX/.dstack/quick/verified/request.md" | awk '{print $1}')" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$SANDBOX/.dstack/quick/verified/request.approved"
printf 'R01 verified in the quick task: checked 1, missing 0\n' > "$SANDBOX/artifacts/v-quick.txt"

call cases-sync-quick -- "$DSTACK" cases sync --quick verified
call verify-quick-open -- "$DSTACK" verify --quick verified
call ev-quick -- "$DSTACK" evidence add --quick verified --r R01 --case c1 --kind cli \
  --artifact artifacts/v-quick.txt --produced-by "the quick task"
call verify-quick -- "$DSTACK" verify --quick verified
call report-quick         -- "$DSTACK" report --quick verified
call report-quick-metrics -- "$DSTACK" report --quick verified --metrics

# ── the R01 metrics of a run ───────────────────────────────────────────────────────────
# Two sandboxes open their run a second apart, so the wall clock is the one metric they cannot
# agree on: the live calls are masked, and the metrics.tsv both stores keep is written by the
# last call below, whose stamps are fixed.
mask_call report-metrics-open '[0-9]+h [0-9]{2}m [0-9]{2}s' '<DUR>'
call report-metrics-open -- "$DSTACK" report --metrics

# transcript_path is recorded by the Stop hook; the transcript and its subagent siblings are
# written here so the token sums are read from real JSONL — a line that is not JSON and a line
# without a usage block are skipped rather than counted.
mkdir -p "$SANDBOX/transcript/session/subagents"
cat > "$SANDBOX/transcript/session.jsonl" <<'JSONL'
{"type":"user","message":{"role":"user","content":"hello"}}
{"type":"assistant","message":{"role":"assistant","usage":{"input_tokens":120,"output_tokens":30,"cache_read_input_tokens":900,"cache_creation_input_tokens":10}}}
not json at all
{"type":"assistant","message":{"usage":{"input_tokens":5}}}
JSONL
printf '%s\n' '{"type":"assistant","message":{"usage":{"input_tokens":7,"output_tokens":3}}}' \
  > "$SANDBOX/transcript/session/subagents/a.jsonl"
printf '%s\n' '{"type":"assistant","message":{"usage":{"output_tokens":11,"cache_read_input_tokens":4}}}' \
  > "$SANDBOX/transcript/session/subagents/b.jsonl"

mask_call report-metrics-gone '[0-9]+h [0-9]{2}m [0-9]{2}s' '<DUR>'
meta_row transcript_path "$SANDBOX/transcript/gone.jsonl"
call report-metrics-gone -- "$DSTACK" report --metrics
meta_row transcript_path "$SANDBOX/transcript/session.jsonl"
meta_row started_at 2026-01-01T00:00:00Z
meta_row closed_at 2026-01-01T01:02:03Z
call report-metrics -- "$DSTACK" report --metrics

# ── run close, with a real verify behind it (P5) ───────────────────────────────────────
call run-close     -- "$DSTACK" run close
call verify-closed -- "$DSTACK" verify --run "$run_id"

# The sandbox is shared with the steps that come after this one, so the run is put back the way
# `run close` found it: status open, closed_at empty and CURRENT restored.
meta_row status open
meta_row closed_at ""
printf '%s\n' "$run_id" > "$SANDBOX/.dstack/local/CURRENT"
