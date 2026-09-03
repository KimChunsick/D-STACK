# parity step: status (human and the ≤2KB hook line) and exec, with their refusals (R11)
. "$PARITY_LIB"

call status         -- "$DSTACK" status
call status-oneline -- "$DSTACK" status --oneline
# cmd_status reads only its first argument, so an unknown option is the human form.
call status-bogus   -- "$DSTACK" status --bogus

# The other two branches of the hook line: CURRENT naming a run that is gone, and no CURRENT.
current="$SANDBOX/.dstack/local/CURRENT"
keep="$(cat "$current")"
printf 'nosuch\n' > "$current"
call status-oneline-missing -- "$DSTACK" status --oneline
call status-missing         -- "$DSTACK" status
: > "$current"
call status-oneline-empty -- "$DSTACK" status --oneline
call status-empty         -- "$DSTACK" status
printf '%s\n' "$keep" > "$current"

call exec-no-label  -- "$DSTACK" exec
call exec-no-dashes -- "$DSTACK" exec ok
call exec-no-cmd    -- "$DSTACK" exec ok --
call exec-slash     -- "$DSTACK" exec bad/label -- echo hi
call exec-dotdot    -- "$DSTACK" exec .. -- echo hi
call exec-ok        -- "$DSTACK" exec ok -- echo hi
call exec-again     -- "$DSTACK" exec ok -- echo again
call exec-fail      -- "$DSTACK" exec fail -- sh -c 'exit 3'

call status-oneline-final -- "$DSTACK" status --oneline
