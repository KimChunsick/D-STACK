# parity step: the question ledger (ask), the decision rows (decision) and check decisions (R11)
. "$PARITY_LIB"

rd="$SANDBOX/.dstack/runs/$(cat "$SANDBOX/.dstack/local/CURRENT")"

# Nothing in either ledger yet: the readers, the empty lists and the empty check.
call check-decisions-none -- "$DSTACK" check decisions
call decision-list-empty  -- "$DSTACK" decision list
call ask-answer-nofile -- "$DSTACK" ask answer Q-01 "an answer" --decision "we do it"
call ask-assume-nofile -- "$DSTACK" ask assume Q-01 "a default" --accept "stderr says why"
call ask-list-empty    -- "$DSTACK" ask list

# Wrong usage, one call per verb (R11). The -noval calls pin the shell's `shift 2` under set -e:
# an option whose operand is missing ends the run with exit 1 and prints nothing at all.
call usage-ask-add           -- "$DSTACK" ask add
call usage-ask-add-affects   -- "$DSTACK" ask add "which store format"
call usage-ask-add-bogus     -- "$DSTACK" ask add "which store format" --bogus
call usage-ask-add-pipe      -- "$DSTACK" ask add "one | two" --affects R01
call usage-ask-add-affects-pipe -- "$DSTACK" ask add "which store format" --affects "R01|R02"
call usage-ask-add-empty     -- "$DSTACK" ask add "which store format" --affects=
call usage-ask-add-extra     -- "$DSTACK" ask add "which store format" extra --affects R01
call usage-ask-add-noval     -- "$DSTACK" ask add "which store format" --affects
call usage-ask-answer        -- "$DSTACK" ask answer Q-01
call usage-ask-answer-noval  -- "$DSTACK" ask answer Q-01 "an answer" --decision
call usage-ask-assume        -- "$DSTACK" ask assume Q-01
call usage-ask-assume-noval  -- "$DSTACK" ask assume Q-01 "a default" --accept
call usage-ask-list          -- "$DSTACK" ask list --run nosuch

# The ledger itself: three questions, the second one through the = form of the option.
call ask-add-one   -- "$DSTACK" ask add "which store format" --affects R01,design
call ask-add-two   -- "$DSTACK" ask add "which retry policy" --affects=R01,R09,bogus
call ask-add-three -- "$DSTACK" ask add "which timeout" --affects R02

call ask-answer-one    -- "$DSTACK" ask answer Q-01 "the tab separated one" --decision "the ledger stays a single tsv"
call ask-answer-again  -- "$DSTACK" ask answer Q-01 "twice" --decision "recorded once"
call ask-answer-nosuch -- "$DSTACK" ask answer Q-99 "x" --decision "y"

call ask-list -- "$DSTACK" ask list

# A request.md makes the affects column checkable: a token naming no row is a warning, never a
# refusal. `request new` belongs to P6's step, so the file is written here with plain shell —
# and only when it is missing, so the ported verb keeps whatever it wrote.
[ -f "$rd/request.md" ] || cat > "$rd/request.md" <<'REQUEST'
---
work_type: cli
---
# parity request

- [ ] **R01** the command prints what it counted — accept: stdout carries "checked N"
- [ ] **R02** the command refuses bad input — accept: exit code 1 with a reason
REQUEST

# `ask assume` mints the R row through `req add`, so it runs below the request.md above: the
# minted row is part of what is compared, not a call made where no request exists.
call ask-assume-ok    -- "$DSTACK" ask assume Q-02 "a fixed backoff of two seconds" --accept "stderr names the backoff"
call ask-assume-again -- "$DSTACK" ask assume Q-02 "twice" --accept "recorded once"

call ask-add-warns -- "$DSTACK" ask add "which cap" --affects R01,R09,bogus,design
call ask-add-known -- "$DSTACK" ask add "which order" --affects R01,R02
call ask-list-full -- "$DSTACK" ask list

# ── the decision rows and the check that every one of them reached something ──────────
call usage-decision-add         -- "$DSTACK" decision add
call usage-decision-add-affects -- "$DSTACK" decision add "the ledger stays a single tsv"
call usage-decision-add-bogus   -- "$DSTACK" decision add "the ledger stays a single tsv" --bogus
call usage-decision-add-pipe    -- "$DSTACK" decision add "one | two" --affects R01
call usage-decision-add-extra   -- "$DSTACK" decision add "one" extra --affects R01
call usage-decision-add-noval   -- "$DSTACK" decision add "one" --affects
call usage-decision-list        -- "$DSTACK" decision list --run nosuch
call usage-check-decisions      -- "$DSTACK" check decisions --run nosuch

call decision-add-plain   -- "$DSTACK" decision add "retries use a fixed backoff of two seconds" --affects R01
call decision-add-assumed -- "$DSTACK" decision add "the cap stays three" --affects R02 --assumed
call decision-add-equals  -- "$DSTACK" decision add "one responsibility per file" --affects=R01,R02
call decision-add-design  -- "$DSTACK" decision add "the store layer owns the formats" --affects design --design "the first round"

# plan.json and cases.tsv are what a D row reaches: a task with --covers, or a recorded evidence
# row. Both verbs belong to later Plans, so the two files are written here and removed again at
# the end of the step — this step leaves only what its own verbs wrote.
cat > "$rd/plan.json" <<'PLAN'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"only","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"only","files":["a.sh"],"deps":[],
               "status":"pending","worktree":"","started_at":"","done_at":"",
               "tasks":[ {"id":"T1","slug":"one","covers":["R01"],"files":["a.sh"],"deps":[],"commit":"","done_at":""},
                         {"id":"T2","slug":"two","covers":["R01"],"files":["b.sh"],"deps":[],"commit":"","done_at":""} ] } ] }
PLAN
printf 'R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\nR02\tC1\tcli\tmet\ta.txt\t-\tparity\t2026-01-01T00:00:00Z\t\n' > "$rd/cases.tsv"

call check-decisions-covered -- "$DSTACK" check decisions

# R14: a withdrawn row takes no task and no evidence, so a decision that affects only withdrawn
# rows is moot — covered, with the mark named. Every other row is covered here, so the check as a
# whole still exits 0.
call req-withdraw-r02   -- "$DSTACK" req withdraw R02 --why "the interview dropped it"
call decision-add-moot  -- "$DSTACK" decision add "bad input is refused with a reason on stderr" --affects R02
call check-decisions-moot -- "$DSTACK" check decisions

# From the second design round on, R55 wants a reason; the shell reads `--design` without the
# two-word rule, so a missing reason is empty here instead of ending the run.
call decision-add-design-2  -- "$DSTACK" decision add "the registry stays one table" --affects design --design
call decision-add-design-eq -- "$DSTACK" decision add "the harness masks five values" --affects design --design="the third round"
call check-decisions-reason -- "$DSTACK" check decisions
call decision-list          -- "$DSTACK" decision list

# One live R id that is neither tasked nor evidenced keeps a mixed row UNCOVERED: the moot rule
# forgives the marked ids, never the row.
call decision-add-mixed   -- "$DSTACK" decision add "retries use a fixed cap" --affects R02,R07
call check-decisions-mixed -- "$DSTACK" check decisions

rm -f "$rd/plan.json" "$rd/cases.tsv"
call check-decisions-uncovered -- "$DSTACK" check decisions
