# parity step: init on a fresh repository and the six run verbs (R13), with their refusals (R11)
. "$PARITY_LIB"

# init needs a repository that has no store yet; the sandbox already has one, so a nested
# repository is created here. It lives outside $SANDBOX/.dstack, so the store comparison at the
# end of the harness still sees only what the run verbs wrote.
git init -q "$SANDBOX/fresh"

# adopt prints the owner heartbeat, whose pid is the pid of whatever called dstack — a value that
# differs between the two passes by construction and that no standard mask covers.
for name in run-adopt run-adopt-live run-adopt-force run-adopt-back; do
  mask_call "$name" ':[0-9]+@' ':<PID>@'
done
# The base commit is printed eight characters wide, one past the short hash the harness masks,
# so the leftover character is masked here (the rule runs after the standard masks).
mask_call run-new-worktree ' @ [^)]*\)' ' @ <GIT>)'

call fresh-init       -- sh -c 'cd "$SANDBOX/fresh" && exec "$DSTACK" init'
call fresh-init-again -- sh -c 'cd "$SANDBOX/fresh" && exec "$DSTACK" init'
call init-again       -- "$DSTACK" init

call run-new-no-slug  -- "$DSTACK" run new
call run-new-bad-slug -- "$DSTACK" run new Bad_Slug --type cli
call run-new-bad-type -- "$DSTACK" run new two --type bogus
# The option loop of run new takes only the two-word form, so --type=cli is an unknown option.
call run-new-eq-type  -- "$DSTACK" run new two --type=cli
call run-new-refused  -- "$DSTACK" run new two --type cli

call run-new-worktree  -- "$DSTACK" run new two --worktree "$SANDBOX/wt-two" --type cli
call run-new-wt-exists -- "$DSTACK" run new three --worktree "$SANDBOX/wt-two" --type cli

call run-list        -- "$DSTACK" run list
call run-list-bogus  -- "$DSTACK" run list --bogus
call run-verify      -- "$DSTACK" run verify
call run-verify-bogus -- "$DSTACK" run verify --bogus

call run-pause       -- "$DSTACK" run pause
call run-pause-again -- "$DSTACK" run pause

call run-adopt-bogus   -- "$DSTACK" run adopt --bogus
call run-adopt-missing -- "$DSTACK" run adopt nosuch
call run-adopt         -- "$DSTACK" run adopt
call run-adopt-live    -- env CLAUDE_CODE_SESSION_ID=other "$DSTACK" run adopt
call run-adopt-force   -- env CLAUDE_CODE_SESSION_ID=other "$DSTACK" run adopt --force
call run-adopt-back    -- "$DSTACK" run adopt --force

two_id="$(ls "$SANDBOX/.dstack/runs" | grep '_two$')"
call run-close-bogus   -- "$DSTACK" run close --bogus
call run-close-missing -- "$DSTACK" run close nosuch --abandon "no such run"
call run-close-abandon -- "$DSTACK" run close "$two_id" --abandon "the parity step abandons it"

call run-list-final -- "$DSTACK" run list
