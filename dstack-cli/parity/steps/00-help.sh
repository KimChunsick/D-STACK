# parity step: the roster, the dispatch errors, and one wrong-usage call per roster noun (R11)
. "$PARITY_LIB"

# The Rust roster carries the hook entry of D-01 and the issue entries of M1, which the shell does
# not have. Removing those lines and the count they change has to make the two rosters identical —
# that is the whole of the allowed difference, and the harness checks it instead of trusting the
# declaration.
for name in help dash-h dashdash-help no-args usage-help; do
  mask_call "$name" '^  hook ' ''
  mask_call "$name" '^  issue ' ''
  mask_call "$name" '^verbs: [0-9]+$' 'verbs: <N>'
  expect_diff "$name" "D-01 adds the hook line and M1 the issue lines to the Rust roster; the shell prints 59 verbs"
done

call help          -- "$DSTACK" help
call dash-h        -- "$DSTACK" -h
call dashdash-help -- "$DSTACK" --help
call no-args       -- "$DSTACK"
call bogus         -- "$DSTACK" bogus
call run-bogus     -- "$DSTACK" run bogus
call run-bare      -- "$DSTACK" run

# One wrong-usage call per roster noun (R11): the wording of these lines is what skills and hooks
# quote, so each one is pinned here. init, status, gate and help take no option they refuse and
# task and next answer their missing-state message before they parse anything — that is the
# current wording too, and the port has to reproduce it.
call usage-init      -- "$DSTACK" init --bogus
call usage-run       -- "$DSTACK" run new --bogus
call usage-exec      -- "$DSTACK" exec --bogus
call usage-request   -- "$DSTACK" request new --bogus
call usage-req       -- "$DSTACK" req add --bogus
call usage-ask       -- "$DSTACK" ask add --bogus
call usage-decision  -- "$DSTACK" decision add --bogus
call usage-milestone -- "$DSTACK" milestone add --bogus
call usage-plan      -- "$DSTACK" plan add --bogus
call usage-task      -- "$DSTACK" task add --bogus
call usage-next      -- "$DSTACK" next --bogus
call usage-cases     -- "$DSTACK" cases sync --bogus
call usage-evidence  -- "$DSTACK" evidence add --bogus
call usage-check     -- "$DSTACK" check coverage --bogus
call usage-verify    -- "$DSTACK" verify --bogus
call usage-report    -- "$DSTACK" report --bogus
call usage-review    -- "$DSTACK" review --bogus
call usage-worker    -- "$DSTACK" worker report --bogus
call usage-quick     -- "$DSTACK" quick new --bogus
call usage-status    -- "$DSTACK" status --bogus
call usage-gate      -- "$DSTACK" gate --bogus
call usage-doctor    -- "$DSTACK" doctor --bogus
call usage-lint-ko   -- "$DSTACK" lint-ko --bogus
call usage-help      -- "$DSTACK" help --bogus
