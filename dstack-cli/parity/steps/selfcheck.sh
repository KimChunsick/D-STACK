# parity step: the harness's own proof — one call that differs by construction
# The name has no NN- prefix, so a normal run skips it; only run.sh --self-check runs it.
. "$PARITY_LIB"

call which-impl -- printf '%s\n' "$PARITY_IMPL"
