# parity step: the quick track and the Stop gate (R13), with one wrong-usage call per verb (R11)
. "$PARITY_LIB"

# Both nouns read state the steps before this one leave behind: quick reads .dstack/quick and the
# gate reads whatever run CURRENT names. So the step opens a run of its own, clears the quick
# table and puts CURRENT back at the end — what it compares never depends on which steps ran.
prev="$(cat "$SANDBOX/.dstack/local/CURRENT")"
# adopt prints the owner heartbeat, whose pid is the pid of whatever called dstack, and run new
# prints the base commit eight characters wide — one past the short hash the harness masks.
mask_call run-adopt-prev ':[0-9]+@' ':<PID>@'
mask_call run-new-quicks ' @ [^)]*\)' ' @ <GIT>)'

call run-pause-prev  -- "$DSTACK" run pause
call run-new-quicks  -- "$DSTACK" run new quicks --type cli
quicks_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$quicks_id"
rm -f "$SANDBOX/.dstack/quick/STATE.md"
mkdir -p "$SANDBOX/artifacts"

# A deps table whose review=on tool cannot be probed: R105 refuses the quick task before its
# directory exists. It is passed per call, so the table the other steps read is untouched.
printf 'name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n' > "$SANDBOX/.deps-noreview.tsv"
printf 'git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n' >> "$SANDBOX/.deps-noreview.tsv"
printf 'codex\tcommand -v dstack-absent-tool\tnpm install -g @openai/codex\t-\tyes\tgoal-closing\treview=on\t\n' \
  >> "$SANDBOX/.deps-noreview.tsv"

# ── the wrong usage of every quick verb (R11) ──────────────────────────────────────────
# A value option in the last position: the shell's `shift 2` fails under set -e, which ends the
# command with exit 1 and nothing printed at all.
call quick-bare           -- "$DSTACK" quick
call quick-unknown-verb   -- "$DSTACK" quick bogus
call quick-new-bogus      -- "$DSTACK" quick new --bogus
call quick-new-no-slug    -- "$DSTACK" quick new
call quick-new-extra      -- "$DSTACK" quick new one two
call quick-new-bad-slug   -- "$DSTACK" quick new Bad_Slug
call quick-new-bad-type   -- "$DSTACK" quick new okslug --type bogus
call quick-new-type-noval -- "$DSTACK" quick new okslug --type
call quick-status-no-slug -- "$DSTACK" quick status
call quick-status-nosuch  -- "$DSTACK" quick status nosuch
call quick-resume-no-slug -- "$DSTACK" quick resume
call quick-resume-nosuch  -- "$DSTACK" quick resume nosuch
call quick-close-no-slug  -- "$DSTACK" quick close
call quick-close-nosuch   -- "$DSTACK" quick close nosuch
call quick-close-bogus    -- "$DSTACK" quick close --bogus
call quick-close-extra    -- "$DSTACK" quick close one two
call quick-close-noval    -- "$DSTACK" quick close nosuch --abandon
# cmd_quick_list reads no argument at all, so an unknown option is the plain listing.
call quick-list-bogus     -- "$DSTACK" quick list --bogus

# ── quick new: the defaults and every flag that changes them (R99) ─────────────────────
call quick-list-empty    -- "$DSTACK" quick list
call quick-new-default   -- "$DSTACK" quick new tidy-readme
call quick-new-again     -- "$DSTACK" quick new tidy-readme
call quick-new-discuss   -- "$DSTACK" quick new discussed --discuss --research
call quick-new-validate  -- "$DSTACK" quick new validated --type=library --validate
call quick-new-full      -- "$DSTACK" quick new full-run --type docs-writing --full
call quick-new-no-tool   -- env DSTACK_DEPS="$SANDBOX/.deps-noreview.tsv" "$DSTACK" quick new needs-review --review
call quick-list-four     -- "$DSTACK" quick list

# ── a slug that is a path, not a name (D-10) ───────────────────────────────────────────
# The shell joins the positional slug into $QUICK unchecked: an absolute one replaces the quick
# root entirely, and `.` or `..` reads a directory outside the quick tree and reports success.
# Like the --run/--quick identifiers, the port refuses anything that is not a plain name before
# it touches the filesystem, so each call below differs by design and by the same wording.
for name in slug-abs-status slug-abs-resume slug-abs-close slug-abandon-abs \
            slug-dotdot-status slug-inner-status slug-dot-status slug-parent-status; do
  expect_diff "$name" "D-10: the shell joins the slug into a path; the port refuses a slug that is not a plain name"
  mask_call "$name" '^dstack: (quick task not found: |quick slug must be a plain name).*$' ''
done
# `.` and `..` are directories, so the shell answers them with a whole listing and exit 0.
for name in slug-dot-status slug-parent-status; do
  mask_call "$name" '^(quick: |  dir: |  state row:|  request\.md: |  R rows ).*$' ''
  mask_call "$name" '^[0-9]+$' '<RC>'
done

call slug-abs-status     -- "$DSTACK" quick status /tmp
call slug-abs-resume     -- "$DSTACK" quick resume /tmp
call slug-abs-close      -- "$DSTACK" quick close /tmp
call slug-abandon-abs    -- "$DSTACK" quick close /tmp --abandon "nothing to abandon"
call slug-dotdot-status  -- "$DSTACK" quick status ../x
call slug-inner-status   -- "$DSTACK" quick status a/b
call slug-dot-status     -- "$DSTACK" quick status .
call slug-parent-status  -- "$DSTACK" quick status ..

# ── what a quick task still needs, and what it looks like once it has it ───────────────
call quick-status-fresh  -- "$DSTACK" quick status tidy-readme
call quick-resume-fresh  -- "$DSTACK" quick resume tidy-readme
call req-add-quick       -- "$DSTACK" req add "the quick row" --accept "the quick criterion" --quick tidy-readme
call quick-resume-pending -- "$DSTACK" quick resume tidy-readme
call approve-quick       -- "$DSTACK" request approve --quick tidy-readme
call quick-status-approved -- "$DSTACK" quick status tidy-readme
call quick-resume-approved -- "$DSTACK" quick resume tidy-readme

# The report is the gate of `quick close` (R79): with nothing proven it exits 1 and the task
# stays open, and the same call passes once the evidence row is there.
call quick-close-unmet -- "$DSTACK" quick close tidy-readme
printf 'R01 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/q-R01.txt"
call ev-quick -- "$DSTACK" evidence add --quick tidy-readme --r R01 --case c1 --kind cli \
  --artifact artifacts/q-R01.txt --produced-by "the parity step"
call quick-close-done    -- "$DSTACK" quick close tidy-readme
call quick-close-abandon -- "$DSTACK" quick close full-run --abandon "the parity step is over"
call quick-status-closed -- "$DSTACK" quick status tidy-readme
call quick-list-closed   -- "$DSTACK" quick list

# ── the Stop gate: the two things it looks at in this worktree (R33, R65, R99, R101) ───
# An open run with no request.md yet, next to the two quick items still open.
call gate-fresh -- "$DSTACK" gate
# cmd_gate reads no argument at all, so an unknown option is the plain verdict (R11).
call gate-bogus -- "$DSTACK" gate --bogus

# A repository the pipeline was never started in is not something the gate has an opinion about.
mkdir -p "$SANDBOX/elsewhere"
git -C "$SANDBOX/elsewhere" init -q >/dev/null 2>&1
call gate-no-store -- sh -c 'cd "$1/elsewhere" && exec "$2" gate' sh "$SANDBOX" "$DSTACK"
rm -rf "$SANDBOX/elsewhere"

# A state row whose directory was removed by hand: the table is the gate's input, `check
# coverage` cannot even find a request behind it, and a checker that failed without naming a
# MISSING line still has to reach the verdict.
printf '| ghost | open | 2026-01-01T00:00:00Z | |\n' >> "$SANDBOX/.dstack/quick/STATE.md"
call quick-status-ghost -- "$DSTACK" quick status ghost
call quick-list-ghost   -- "$DSTACK" quick list
call gate-ghost         -- "$DSTACK" gate
grep -v '^| ghost |' "$SANDBOX/.dstack/quick/STATE.md" > "$SANDBOX/state.new"
mv "$SANDBOX/state.new" "$SANDBOX/.dstack/quick/STATE.md"

# The three conditions the run half names besides coverage: a row pending approval, an open
# question and a request nobody approved. A pending row is minted by `req add --from-answer`,
# which belongs to another step, so the marker is written here directly.
call gate-req-new -- "$DSTACK" request new --type cli --title "the gate request"
call gate-req-add -- "$DSTACK" req add "the gated row" --accept "the gate criterion"
call gate-ask-add -- "$DSTACK" ask add "what the gate should ask" --affects R01
printf -- '- [ ] **R02** the row nobody approved yet — accept: the gate names it — status: pending-approval\n' \
  >> "$run_dir/request.md"
call gate-pending -- "$DSTACK" gate

call gate-ask-answer -- "$DSTACK" ask answer Q-01 "the answer the gate waited for" \
  --decision "what the answer decided"
call gate-approve  -- "$DSTACK" request approve
call gate-approved -- "$DSTACK" gate

# Coverage passes once every live row has a covering task and a recorded evidence row. plan add
# and task add write the same file; it is written here so the gate depends on no verb of theirs.
cat > "$run_dir/plan.json" <<'JSON'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"gate","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"gate","files":["artifacts"],"deps":[],
              "status":"in-progress","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"gate","covers":["R01","R02"],
                         "files":["artifacts"],"deps":[],"commit":"","done_at":""} ] } ] }
JSON
printf 'R01 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/gate-R01.txt"
printf 'R02 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/gate-R02.txt"
call gate-ev-01 -- "$DSTACK" evidence add --r R01 --case c1 --kind cli \
  --artifact artifacts/gate-R01.txt --produced-by "the parity step"
call gate-ev-02 -- "$DSTACK" evidence add --r R02 --case c1 --kind cli \
  --artifact artifacts/gate-R02.txt --produced-by "the parity step"
call gate-covered -- "$DSTACK" gate

# A paused run is deliberately invisible here: `run pause` is the escape hatch from a repeating
# block (R101), so it has to actually stop the block.
mask_call gate-adopt ':[0-9]+@' ':<PID>@'
call gate-pause  -- "$DSTACK" run pause
call gate-paused -- "$DSTACK" gate
call gate-adopt  -- "$DSTACK" run adopt "$quicks_id" --force

# Nothing open at all: the two clear lines the Stop hook reads as a pass.
call gate-abandon-discussed -- "$DSTACK" quick close discussed --abandon "the parity step is over"
call gate-abandon-validated -- "$DSTACK" quick close validated --abandon "the parity step is over"
call gate-clear-run   -- "$DSTACK" gate
call gate-pause-again -- "$DSTACK" run pause
call gate-clear-none  -- "$DSTACK" gate

# The run this step opened is closed again, so what it leaves behind is inert, and CURRENT goes
# back to the run the steps after this one expect.
call run-close-quicks -- "$DSTACK" run close "$quicks_id" --abandon "the parity step is over"
call run-adopt-prev   -- "$DSTACK" run adopt "$prev" --force
