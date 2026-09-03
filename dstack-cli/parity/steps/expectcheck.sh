# parity step: the harness's own proof — a declared difference that is not there
# A helper step: only `run.sh --only expectcheck` runs it (tests/r04_parity.rs does).
. "$PARITY_LIB"

expect_diff same-on-both "nothing differs here, so this declaration is stale"
call same-on-both -- printf '%s\n' same
