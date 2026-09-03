# parity step: what a store file that cannot be read does to a verb (R03/R13, D-12)
. "$PARITY_LIB"

# A run of its own: the ledgers of the sandbox run were written by the steps before this one, and
# what is asserted here is the answer to a file that is *not* there.
prev="$(cat "$SANDBOX/.dstack/local/CURRENT")"
# run new prints the base commit eight characters wide — one past the short hash the harness masks.
mask_call run-new-reads ' @ [^)]*\)' ' @ <GIT>)'

call run-pause-prev -- "$DSTACK" run pause
call run-new-reads  -- "$DSTACK" run new reads --type cli
reads_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$reads_id"

# ── (a) the file is not there: the empty answer both implementations give ────────────────
call absent-ask-list     -- "$DSTACK" ask list
call absent-dec-list     -- "$DSTACK" decision list
call absent-check-dec    -- "$DSTACK" check decisions
call absent-check-req    -- "$DSTACK" check request
call absent-req-status   -- "$DSTACK" req status
call absent-cases-render -- "$DSTACK" cases render
call absent-next         -- "$DSTACK" next
call absent-status       -- "$DSTACK" status
call absent-status-line  -- "$DSTACK" status --oneline
call absent-verify       -- "$DSTACK" verify
call absent-report       -- "$DSTACK" report
call absent-review-close -- "$DSTACK" review close --scope plan --id P1 --why "nothing to close"

# The ledgers every call below reads: an approved request with its cases, one answered question,
# one decision, one plan with a task, and one sealed round in review/index.tsv.
call req-new      -- "$DSTACK" request new --type cli --title "the reading request"
call req-add      -- "$DSTACK" req add "the row" --accept "the criterion"
call ask-add      -- "$DSTACK" ask add "the question" --affects R01
call ask-answer   -- "$DSTACK" ask answer Q-01 "the answer" --decision "what the answer decided"
call dec-add      -- "$DSTACK" decision add "the decision" --affects R01
call req-approve  -- "$DSTACK" request approve
call cases-sync   -- "$DSTACK" cases sync
call ms-add       -- "$DSTACK" milestone add first
call plan-add     -- "$DSTACK" plan add one --milestone M1 --files src
call task-add     -- "$DSTACK" task add one --plan P1 --covers R01 --files src
printf '001\tplan\tP1\tcodex-review-001.md\t2026-09-02T00:00:00Z\t0\t0\t1\n' > "$run_dir/review/index.tsv"

# ── (b) the file is there and cannot be read: D-12 says cannot decide (exit 2) ───────────
# The shell reads on with an empty result — awk and grep print nothing for a file they cannot
# open, and jq's own failure only sometimes reaches the exit code — so every call below differs by
# design. Each declaration removes the two implementations' error lines; what the shell printed
# past the point where the port stopped is removed per call, and mask_rc marks the calls whose
# exit code the divergence changes.
declare_unreadable() {   # $1 call name, $2 the store file the call cannot read
  expect_diff "$1" "D-12: the shell reads on with an empty $2; the port cannot decide (exit 2)"
  mask_call "$1" '^dstack: cannot read .*$' ''
  mask_call "$1" '^(awk: can.t open file |jq: error: Could not open file |jq: parse error).*$' ''
  mask_call "$1" '^ source line number [0-9]+$' ''
}
mask_rc() { mask_call "$1" '^[0-9]+$' '<RC>'; }

# questions.md: the ledger review round 040 found counted as zero questions.
chmod 000 "$run_dir/questions.md"
declare_unreadable unreadable-check-req questions.md
mask_rc unreadable-check-req
mask_call unreadable-check-req '^  (questions|size|approved): .*$' ''
mask_call unreadable-check-req '^check request: fields .*$' ''
declare_unreadable unreadable-ask-list questions.md
mask_rc unreadable-ask-list
mask_call unreadable-ask-list '^rows [0-9]+, open .*$' ''
call unreadable-check-req -- "$DSTACK" check request
call unreadable-ask-list  -- "$DSTACK" ask list
chmod 644 "$run_dir/questions.md"

chmod 000 "$run_dir/decisions.md"
declare_unreadable unreadable-dec-list decisions.md
mask_rc unreadable-dec-list
mask_call unreadable-dec-list '^(decisions: |D \| |rows [0-9]+, answered ).*$' ''
declare_unreadable unreadable-check-dec decisions.md
mask_rc unreadable-check-dec
mask_call unreadable-check-dec '^  rows [0-9]+, covered .*$' ''
call unreadable-dec-list  -- "$DSTACK" decision list
call unreadable-check-dec -- "$DSTACK" check decisions
chmod 644 "$run_dir/decisions.md"

chmod 000 "$run_dir/cases.tsv"
declare_unreadable unreadable-req-status cases.tsv
mask_rc unreadable-req-status
mask_call unreadable-req-status '^(request: |R \| |R01 \| |rows [0-9]+, live |cases: ).*$' ''
declare_unreadable unreadable-verify cases.tsv
mask_rc unreadable-verify
mask_call unreadable-verify '^(R01 FAIL: |branch containment: |verify: checked ).*$' ''
declare_unreadable unreadable-report cases.tsv
mask_rc unreadable-report
mask_call unreadable-report '^(\||report: run |MET [0-9]|requirement coverage rate |$).*$' ''
call unreadable-req-status -- "$DSTACK" req status
call unreadable-verify     -- "$DSTACK" verify
call unreadable-report     -- "$DSTACK" report
chmod 644 "$run_dir/cases.tsv"

# review/index.tsv: the bundle goes outside the store with --out, so the shell finishing what the
# port refuses leaves no file for the store comparison to find. Both sides already exit 2 here.
chmod 000 "$run_dir/review/index.tsv"
declare_unreadable unreadable-review-bundle review/index.tsv
call unreadable-review-bundle -- "$DSTACK" review --scope milestone --milestone M1 --out bundle.txt
chmod 644 "$run_dir/review/index.tsv"

# plan.json: jq's failure reaches the exit code in next (2 on both sides) and not in status (0).
chmod 000 "$run_dir/plan.json"
declare_unreadable unreadable-next plan.json
declare_unreadable unreadable-status plan.json
mask_rc unreadable-status
mask_call unreadable-status '^(store: |worktree: |current: |quick open: |open runs in store:).*$' ''
call unreadable-next   -- "$DSTACK" next
call unreadable-status -- "$DSTACK" status
chmod 644 "$run_dir/plan.json"

# ── (c) the file is there and does not parse: jq fails in the shell too, with exit 5 ─────
cp "$run_dir/plan.json" "$SANDBOX/plan.json.bak"
printf '{"v":2,"milestones":[\n' > "$run_dir/plan.json"
declare_unreadable broken-next plan.json
mask_rc broken-next
call broken-next -- "$DSTACK" next
cp "$SANDBOX/plan.json.bak" "$run_dir/plan.json"
rm -f "$SANDBOX/plan.json.bak" "$SANDBOX/bundle.txt"

# ── (d) CURRENT: the file that names the run of this worktree (P15, D-12) ────────────────
# `cat` prints nothing for a file it cannot open, so the shell answers about a worktree whose run
# it could not see; the port cannot decide. meta.tsv has no call here on purpose: with that table
# unreadable the shell's touch_owner rewrites it with the owner rows alone before the verb reads
# anything, so a call would compare two stores the reference itself had emptied — the port's side
# of that one is pinned by tests/r03_store_readers.rs.
chmod 000 "$SANDBOX/.dstack/local/CURRENT"
for c in unreadable-current-status unreadable-current-verify; do
  declare_unreadable "$c" CURRENT
  mask_rc "$c"
  mask_call "$c" '^cat: .*$' ''
done
mask_call unreadable-current-status '^(store: |worktree: |current: |quick open: |open runs in store:).*$' ''
mask_call unreadable-current-verify '^(pwd:|common-dir:|main root:|store:|worktree:|branch:|HEAD:|CURRENT:).*$' ''
call unreadable-current-status -- "$DSTACK" status
call unreadable-current-verify -- "$DSTACK" run verify
chmod 644 "$SANDBOX/.dstack/local/CURRENT"

# The store both implementations end with has to be the same one, so the run this step opened is
# left the way every other step leaves its own.
call run-pause-reads -- "$DSTACK" run pause
