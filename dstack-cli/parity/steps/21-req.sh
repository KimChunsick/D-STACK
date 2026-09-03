# parity step: the six req verbs over the R rows of the request (R13), with their refusals (R11)
. "$PARITY_LIB"

REQ="$SANDBOX/.dstack/runs/$(cat "$SANDBOX/.dstack/local/CURRENT")"
# Step 20 leaves a request behind when the whole harness runs; --only 21-req starts without one.
# Both implementations take the same branch, so the calls below start from the same file.
[ -f "$REQ/request.md" ] || "$DSTACK" request new --type cli --title "the parity request" >/dev/null

call req-add-bogus     -- "$DSTACK" req add --bogus
call req-add-no-text   -- "$DSTACK" req add
call req-add-no-accept -- "$DSTACK" req add "a row without a criterion"
# A value option in the last position: `shift 2` fails under `set -e`, so the shell exits 1
# without printing anything at all.
call req-add-no-operand -- "$DSTACK" req add "a row" --accept
call req-add-sep-text   -- "$DSTACK" req add "a — b" --accept "c"
call req-add-sep-accept -- "$DSTACK" req add "a" --accept "c — d"

call req-add-one -- "$DSTACK" req add "the first row" --accept "the first criterion"
call req-add-two -- "$DSTACK" req add "the second row" --accept="the second criterion"
call req-add-id-low -- "$DSTACK" req add "a lower id" --accept "c" --id R01
call req-add-id-shape -- "$DSTACK" req add "a bad id" --accept "c" --id R0x
call req-add-id-bare -- "$DSTACK" req add "a bare id" --accept "c" --id=R
call req-add-id -- "$DSTACK" req add "the fifth row" --accept "c5" --id R05
call req-add-from-answer -- "$DSTACK" req add "a parked row" --from-answer
call req-add-assumption-alone -- "$DSTACK" req add "x" --accept "c" --assumption
call req-add-from-alone -- "$DSTACK" req add "x" --accept "c" --from Q-01
call req-add-assumption-no-ledger -- "$DSTACK" req add "x" --accept "c" --assumption --from Q-01

# The question ledger is written by hand: `dstack ask` belongs to another Plan, and the rows the
# assumption path reads have to exist before it runs.
printf '# Questions (R51)\n\nWritten only by `dstack ask`.\n\n| Q | Question | Affects | Status |\n|---|---|---|---|\n| Q-01 | which default? | R01 | open |\n| Q-02 | answered already? | R02 | answered |\n' > "$REQ/questions.md"

call req-add-assumption-unknown -- "$DSTACK" req add "x" --accept "c" --assumption --from Q-09
call req-add-assumption-answered -- "$DSTACK" req add "x" --accept "c" --assumption --from Q-02
call req-add-assumption -- "$DSTACK" req add "the assumed row" --accept "c7" --assumption --from Q-01
call req-add-assumption-again -- "$DSTACK" req add "a second assumed row" --accept "c8" --assumption --from Q-01

call req-accept-usage -- "$DSTACK" req accept
call req-accept-bogus -- "$DSTACK" req accept --bogus
call req-accept-extra -- "$DSTACK" req accept R06 "one" "two"
call req-accept-missing -- "$DSTACK" req accept R99 "a criterion"
call req-accept-real -- "$DSTACK" req accept R01 "a criterion"
call req-accept -- "$DSTACK" req accept R06 "now observable"
call req-accept-again -- "$DSTACK" req accept R06 "again"

call req-split-usage -- "$DSTACK" req split R01
call req-split-bogus -- "$DSTACK" req split R01 --bogus
call req-split-no-operand -- "$DSTACK" req split R01 --into
call req-split-missing -- "$DSTACK" req split R99 --into R01,R02
call req-split-self -- "$DSTACK" req split R01 --into R01,R02
call req-split-unknown-child -- "$DSTACK" req split R01 --into R02,R99
call req-split-one-child -- "$DSTACK" req split R01 --into R02
call req-split -- "$DSTACK" req split R01 --into R02,R05
call req-split-again -- "$DSTACK" req split R01 --into R02,R05

call req-withdraw-usage -- "$DSTACK" req withdraw R05
call req-withdraw-missing -- "$DSTACK" req withdraw R99 --why "no such row"
call req-withdraw-sep -- "$DSTACK" req withdraw R05 --why "a — b"
call req-withdraw -- "$DSTACK" req withdraw R05 --why "the parity step withdraws it"
call req-withdraw-again -- "$DSTACK" req withdraw R05 --why "again"
call req-split-withdrawn -- "$DSTACK" req split R05 --into R02,R06
call req-defer -- "$DSTACK" req defer R07 --why "the parity step defers it"
call req-defer-again -- "$DSTACK" req defer R07 --why "again"
call req-split-deferred -- "$DSTACK" req split R07 --into R02,R06
call req-defer-no-operand -- "$DSTACK" req defer R07 --why

call req-status -- "$DSTACK" req status
# req status takes no option: resolve_target keeps what is left and the verb ignores it.
call req-status-bogus -- "$DSTACK" req status --bogus

# The ledger column of req status, from a cases.tsv written by hand — `dstack cases sync` is
# another Plan's verb, and the file goes away again so nothing downstream inherits it.
printf 'R\tcase\tkind\tstatus\tartifact\tsha256\tproduced_by\trecorded_at\tnote\nR01\tc1\tcli\tmet\ta.txt\t-\t-\t-\t-\nR01\tc2\tcli\tunmet\t-\t-\t-\t-\t-\n' > "$REQ/cases.tsv"
call req-status-cases -- "$DSTACK" req status
rm -f "$REQ/cases.tsv"

# R48: a row appended to an approved request is marked pending-approval. The stamp is written by
# hand (request approve is driven in step 20) and removed again, so the run is left unapproved.
printf 'sha256 0000000000000000000000000000000000000000000000000000000000000000  approved_at 2026-01-01T00:00:00Z\n' > "$REQ/request.approved"
call req-add-pending -- "$DSTACK" req add "a row after approval" --accept "c9"
call req-status-pending -- "$DSTACK" req status
rm -f "$REQ/request.approved"
