# parity step: the four hook events (R07, R13), with the wrong usage of the verb (R11)
. "$PARITY_LIB"

# `hook <event>` has no shell verb: D-20 makes its reference the pre-shrink wrapper, frozen under
# parity/ref, driven by the shell dispatcher through DSTACK_BIN — the first of the three places
# the wrapper looks. The Rust side runs the verb. Payloads and the model-rewrite log live outside
# the sandbox worktree, so nothing this step writes reaches the store comparison or the gate's
# `lint-ko --changed`.
REF="$(cd "$(dirname "$PARITY_LIB")/ref" && pwd -P)/dstack-hook.sh"
# The wrapper borrows meta_set from the reference's common.sh, whose last lines source three more
# files through $DSTACK_LIB; unset — which is how Claude Code calls it — `set -u` ends the wrapper
# with a bare exit 1 before the gate is asked, so the installed Stop hook has been failing quietly.
# The reference is given the variable here so the path its code describes is what the port is
# compared against (D-11: a reference path that crashes bash itself is not reproduced). The port
# reads no library at run time and needs nothing of the sort. The library is where the dispatcher
# itself looks for it, next to its own directory — the reference is extracted from a tag now, so no
# step may name the directory it lands in.
LIB=""
[ "$PARITY_IMPL" != shell ] || LIB="$(cd "$(dirname "$DSTACK")/../lib" && pwd -P)"
PAY="$SANDBOX/../hook-payloads"
mkdir -p "$PAY"
HOOK_LOG="$PAY/agent-model.log"

hook() {   # $1 call name, $2 payload file, then the arguments of the wrapper / of the verb
  local name="$1" stdin="$2"; shift 2
  if [ "$PARITY_IMPL" = shell ]; then
    call_stdin "$name" "$stdin" -- env DSTACK_BIN="$DSTACK" DSTACK_LIB="$LIB" DSTACK_HOOK_LOG="$HOOK_LOG" bash "$REF" "$@"
  else
    call_stdin "$name" "$stdin" -- env DSTACK_HOOK_LOG="$HOOK_LOG" "$DSTACK" hook "$@"
  fi
}

# The store keeps a line per hook run (.dstack/local/hooks/<event>.last), and the store comparison
# has no per-call masks, so a call whose answer is declared to differ has to leave no line: these
# run in the repository the pipeline was never started in, where log_hook writes nothing.
hook_away() {   # $1 call name, $2 payload file, $3 the event
  local name="$1" stdin="$2"; shift 2
  if [ "$PARITY_IMPL" = shell ]; then
    call_stdin "$name" "$stdin" -- env DSTACK_BIN="$DSTACK" DSTACK_LIB="$LIB" DSTACK_HOOK_LOG="$HOOK_LOG" \
      sh -c 'cd "$1" && exec bash "$2" "$3"' sh "$NOSTORE" "$REF" "$1"
  else
    call_stdin "$name" "$stdin" -- env DSTACK_HOOK_LOG="$HOOK_LOG" \
      sh -c 'cd "$1" && exec "$2" hook "$3"' sh "$NOSTORE" "$DSTACK" "$1"
  fi
}

# ── the payloads Claude Code sends ─────────────────────────────────────────────────────
turn() {      # $1 name, $2 hook_event_name, $3 cwd, $4 the fields after it (with a leading comma)
  printf '{"session_id":"s1","transcript_path":"/tmp/dstack-parity.jsonl","cwd":"%s","hook_event_name":"%s"%s}\n' \
    "$3" "$2" "$4" > "$PAY/$1.json"
}
pretool() {   # $1 name, $2 tool_name, $3 tool_input as JSON
  printf '{"session_id":"s1","cwd":".","hook_event_name":"PreToolUse","tool_name":"%s","tool_input":%s}\n' \
    "$2" "$3" > "$PAY/$1.json"
}

# A repository the pipeline was never started in: the events that carry a verdict exit 0 there.
git init -q "$SANDBOX/../nostore"
NOSTORE="$(cd "$SANDBOX/../nostore" && pwd -P)"

turn inject      UserPromptSubmit . ',"prompt":"go on"'
turn inject-away UserPromptSubmit "$NOSTORE" ',"prompt":"go on"'
turn stop        Stop . ',"stop_hook_active":false'
turn stop-active Stop . ',"stop_hook_active":true'
turn stop-away   Stop "$NOSTORE" ',"stop_hook_active":false'
printf 'this is not a JSON payload at all\n' > "$PAY/broken.json"

pretool am-no-model Agent '{"description":"probe","prompt":"do one thing","subagent_type":"general-purpose"}'
pretool am-fable    Agent '{"description":"probe","prompt":"do one thing","model":"fable"}'
pretool am-full-id  Agent '{"model":"claude-opus-5","description":"probe"}'
pretool am-inherit  Agent '{"description":"probe","model":"inherit","subagent_type":"recon"}'
pretool am-sonnet   Agent '{"description":"probe","model":"sonnet"}'
pretool am-opus     Agent '{"description":"probe","model":"opus"}'
# jq's `// {}` catches null and false; anything else that is not an object reaches `$ti +
# {model:"opus"}`, which jq refuses — a payload the hook cannot rewrite must not pass as one it
# approved. A repeated member reads back as its last value, collapsed onto the first one.
pretool am-ti-array  Agent '[]'
pretool am-ti-string Agent '"x"'
pretool am-ti-number Agent '3'
pretool am-ti-null   Agent 'null'
pretool am-ti-false  Agent 'false'
printf '{"session_id":"s1","cwd":".","hook_event_name":"PreToolUse","tool_name":"Agent"}\n' \
  > "$PAY/am-ti-missing.json"
pretool am-dup-model Agent '{"a":"1","model":"sonnet","b":"2","model":"fable"}'
pretool am-dup-opus  Agent '{"model":"fable","description":"a","model":"opus"}'
pretool am-dup-plain Agent '{"a":"1","b":"2","a":"3","model":"fable"}'

# README.md is ko-haeyo in the fallback scope table; 정본 is the K01 word rule (S1), 있어서 the
# K24 sentence rule, which an Edit fragment must not be judged by (R93).
pretool pw-write-deny     Write '{"file_path":"README.md","content":"정본은 이 파일이에요.\n"}'
pretool pw-write-allow    Write '{"file_path":"README.md","content":"훅을 옮겨요.\n"}'
pretool pw-write-unscoped Write '{"file_path":"notes/plain.txt","content":"정본은 이 파일이에요.\n"}'
pretool pw-edit-fragment  Edit  '{"file_path":"README.md","old_string":"a","new_string":"설정에 있어서 중요한 값이에요.\n"}'
pretool pw-edit-word      Edit  '{"file_path":"README.md","old_string":"a","new_string":"정본은 이 파일이에요.\n"}'
pretool pw-heredoc        Bash  '{"command":"cat > README.md <<KO\n정본은 이 파일이에요.\nKO\n"}'
pretool pw-heredoc-quoted Bash  '{"command":"cat >> docs/x.md <<'"'"'EOF'"'"'\n훅을 옮겨요.\nEOF\n"}'
pretool pw-redirect-only  Bash  '{"command":"printf hi > out.txt"}'
pretool pw-no-redirect    Bash  '{"command":"ls -l"}'
pretool pw-commit-deny    Bash  '{"command":"git commit --no-verify -m \"정본을 고쳐요\""}'
pretool pw-commit-allow   Bash  '{"command":"git -c commit.gpgsign=false commit -m '"'"'훅을 옮겨요'"'"'"}'
pretool pw-commit-heredoc Bash  '{"command":"git commit -F - <<MSG\n정본을 고쳐요\nMSG\n"}'
pretool pw-commit-bare    Bash  '{"command":"git commit --amend --no-edit"}'
pretool pw-empty-command  Bash  '{"command":""}'
pretool pw-other-tool     Read  '{"file_path":"README.md"}'
pretool pw-ti-array       Write '[]'
pretool pw-ti-string      Bash  '"ls -l"'
pretool pw-dup-path       Write '{"file_path":"notes/plain.txt","file_path":"README.md","content":"정본은 이 파일이에요.\n"}'

# A payload jq reads and serde_json does not: a number outside f64, nesting past serde_json's 128
# (jq stops at 256) or a lone low surrogate. The reference rewrites the model or lints the write
# and exits 0; the port blocks instead of judging fields it could not see (round 063), so each of
# these calls is a declared divergence — the whole answer differs, on purpose.
BIG="$(printf '9%.0s' $(seq 1 400))"
DEEP="$(printf '[%.0s' $(seq 1 129))1$(printf ']%.0s' $(seq 1 129))"
pretool am-e400      Agent "{\"description\":\"probe\",\"model\":\"fable\",\"budget\":1e400}"
pretool am-bigint    Agent "{\"model\":\"fable\",\"budget\":$BIG}"
pretool am-surrogate Agent '{"model":"fable","note":"\udc00"}'
pretool am-deep      Agent "{\"model\":\"fable\",\"path\":$DEEP}"
pretool pw-e400      Write "{\"file_path\":\"README.md\",\"budget\":1e400,\"content\":\"정본은 이 파일이에요.\\n\"}"
turn inject-e400 UserPromptSubmit . ',"prompt":"go on","budget":1e400'
turn stop-e400   Stop . ',"stop_hook_active":false,"budget":1e400'
# Both parsers read these two, and print the number differently: jq 1.7 keeps the literal through
# decNumber (1e2 → 1E+2, 1e-400 → 1E-400) where serde_json prints the f64 it read (100.0, 0.0).
pretool am-exponent  Agent '{"model":"fable","budget":1e2}'
pretool am-underflow Agent '{"model":"fable","budget":1e-400}'

# ── a run of this step's own ───────────────────────────────────────────────────────────
# The gate reads whatever run CURRENT points at, so the step opens one, and puts CURRENT back at
# the end — what it compares never depends on which steps ran before it.
prev="$(cat "$SANDBOX/.dstack/local/CURRENT")"
# run new prints the base commit eight characters wide — one past the short hash the harness
# masks — and adopt prints the owner heartbeat, whose pid is the pid of whatever called dstack.
mask_call run-new-hooks ' @ [^)]*\)' ' @ <GIT>)'
mask_call run-adopt-prev ':[0-9]+@' ':<PID>@'
mask_call stop-adopt ':[0-9]+@' ':<PID>@'
call run-pause-prev -- "$DSTACK" run pause
call run-new-hooks  -- "$DSTACK" run new hooks --type cli
hooks_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$hooks_id"
mkdir -p "$SANDBOX/artifacts"

# ── inject (UserPromptSubmit, R24) ─────────────────────────────────────────────────────
hook inject-fresh "$PAY/inject.json"      inject
hook inject-away  "$PAY/inject-away.json" inject
hook inject-broken "$PAY/broken.json"     inject

# ── agent-model (PreToolUse, tool Agent, R22) ──────────────────────────────────────────
hook am-no-model "$PAY/am-no-model.json" agent-model
hook am-fable    "$PAY/am-fable.json"    agent-model
hook am-full-id  "$PAY/am-full-id.json"  agent-model
hook am-inherit  "$PAY/am-inherit.json"  agent-model
hook am-sonnet   "$PAY/am-sonnet.json"   agent-model
hook am-opus     "$PAY/am-opus.json"     agent-model
hook am-not-agent "$PAY/pw-write-deny.json" agent-model
hook am-broken   "$PAY/broken.json"      agent-model
# jq prints its own "cannot be added" line before the wrapper's block line; the port has no jq to
# print one (D-11). Everything else about the two calls — stdout, the block line and the exit
# code — is compared.
jq_diagnostic() {
  expect_diff "$1" "D-11: jq's own 'cannot be added' line precedes the block line; the port has no jq"
  mask_call "$1" '^jq: error .*$' ''
}
jq_diagnostic am-ti-array
jq_diagnostic am-ti-string
jq_diagnostic am-ti-number
hook am-ti-array   "$PAY/am-ti-array.json"   agent-model
hook am-ti-string  "$PAY/am-ti-string.json"  agent-model
hook am-ti-number  "$PAY/am-ti-number.json"  agent-model
hook am-ti-null    "$PAY/am-ti-null.json"    agent-model
hook am-ti-false   "$PAY/am-ti-false.json"   agent-model
hook am-ti-missing "$PAY/am-ti-missing.json" agent-model
hook am-dup-model  "$PAY/am-dup-model.json"  agent-model
hook am-dup-opus   "$PAY/am-dup-opus.json"   agent-model
hook am-dup-plain  "$PAY/am-dup-plain.json"  agent-model

# ── pre-write (PreToolUse, tools Write|Edit|Bash, R93) ─────────────────────────────────
hook pw-write-deny     "$PAY/pw-write-deny.json"     pre-write
hook pw-write-allow    "$PAY/pw-write-allow.json"    pre-write
hook pw-write-unscoped "$PAY/pw-write-unscoped.json" pre-write
hook pw-edit-fragment  "$PAY/pw-edit-fragment.json"  pre-write
hook pw-edit-word      "$PAY/pw-edit-word.json"      pre-write
hook pw-heredoc        "$PAY/pw-heredoc.json"        pre-write
hook pw-heredoc-quoted "$PAY/pw-heredoc-quoted.json" pre-write
hook pw-redirect-only  "$PAY/pw-redirect-only.json"  pre-write
hook pw-no-redirect    "$PAY/pw-no-redirect.json"    pre-write
hook pw-commit-deny    "$PAY/pw-commit-deny.json"    pre-write
hook pw-commit-allow   "$PAY/pw-commit-allow.json"   pre-write
hook pw-commit-heredoc "$PAY/pw-commit-heredoc.json" pre-write
hook pw-commit-bare    "$PAY/pw-commit-bare.json"    pre-write
hook pw-empty-command  "$PAY/pw-empty-command.json"  pre-write
hook pw-other-tool     "$PAY/pw-other-tool.json"     pre-write
hook pw-broken         "$PAY/broken.json"            pre-write
unreadable_payload() {   # $1 call name, $2 what jq read and serde_json did not
  expect_diff "$1" "R07: jq reads this payload and serde_json refuses it ($2); the port blocks rather than take a branch meant for a payload nobody could read"
  mask_call "$1" '^\{"(hookSpecificOutput|decision)".*$' ''
  mask_call "$1" '^dstack-hook .*cannot decide.*$' ''
  mask_call "$1" '^dstack: .*$' ''
  mask_call "$1" '^[0-9]+$' '<RC>'
}
unreadable_payload am-e400      "a number outside f64"
unreadable_payload am-bigint    "a 400-digit integer"
unreadable_payload am-surrogate "a lone low surrogate"
unreadable_payload am-deep      "129 nested arrays, past serde_json's limit of 128"
unreadable_payload pw-e400      "a number outside f64"
unreadable_payload inject-e400  "a number outside f64"
unreadable_payload stop-e400    "a number outside f64"
# The number itself is what differs here, and nothing else: jq keeps the literal, the port prints
# the f64 serde_json read.
number_form() {   # $1 call name, $2 jq's form, $3 the port's form
  expect_diff "$1" "R07: jq 1.7 keeps a number literal through decNumber ($2) where the port prints the f64 serde_json read ($3); no Agent payload carries a number"
  mask_call "$1" '"budget":[^,}]*' '"budget":<NUM>'
}
number_form am-exponent  "1E+2" "100.0"
number_form am-underflow "1E-400" "0.0"
hook_away am-e400      "$PAY/am-e400.json"      agent-model
hook_away am-bigint    "$PAY/am-bigint.json"    agent-model
hook_away am-surrogate "$PAY/am-surrogate.json" agent-model
hook_away am-deep      "$PAY/am-deep.json"      agent-model
hook am-exponent  "$PAY/am-exponent.json"  agent-model
hook am-underflow "$PAY/am-underflow.json" agent-model
hook_away pw-e400      "$PAY/pw-e400.json"      pre-write
hook_away inject-e400  "$PAY/inject-e400.json"  inject
hook pw-ti-array       "$PAY/pw-ti-array.json"       pre-write
hook pw-ti-string      "$PAY/pw-ti-string.json"      pre-write
hook pw-dup-path       "$PAY/pw-dup-path.json"       pre-write

# ── stop (Stop, R33/R65/R99, D-13) ─────────────────────────────────────────────────────
# A run with no request.md yet: the gate's first condition, carried back as a block payload.
hook stop-fresh "$PAY/stop.json" stop

# The three conditions the run half names besides coverage, plus one open quick item. A pending
# row is minted by `req add --from-answer`, which belongs to another step, so it is written here.
call stop-req-new -- "$DSTACK" request new --type cli --title "the hook request"
call stop-req-add -- "$DSTACK" req add "the hooked row" --accept "the hook criterion"
call stop-ask-add -- "$DSTACK" ask add "what the hook should ask" --affects R01
printf -- '- [ ] **R02** the row nobody approved yet — accept: the hook names it — status: pending-approval\n' \
  >> "$run_dir/request.md"
call stop-quick-new -- "$DSTACK" quick new hooked
cat > "$run_dir/plan.json" <<'JSON'
{ "v": 2,
  "milestones": [ {"id":"M1","slug":"hook","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"hook","files":["artifacts"],"deps":[],
              "status":"in-progress","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"hook","covers":["R01","R02"],
                         "files":["artifacts"],"deps":[],"commit":"","done_at":""} ] } ] }
JSON
hook stop-open "$PAY/stop.json" stop

# D-13: a second stop in the same turn ends the turn, which is what lets a turn waiting on a
# background run be re-entered — a gate that could never let a turn end could never be woken up.
hook stop-active "$PAY/stop-active.json" stop

# Everything the gate named, answered: the block reason shrinks to what is still open.
call stop-ask-answer -- "$DSTACK" ask answer Q-01 "the answer the hook waited for" \
  --decision "what the answer decided"
call stop-approve -- "$DSTACK" request approve
printf 'R01 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/hook-R01.txt"
printf 'R02 verified: checked 1, missing 0\n' > "$SANDBOX/artifacts/hook-R02.txt"
call stop-ev-01 -- "$DSTACK" evidence add --r R01 --case c1 --kind cli \
  --artifact artifacts/hook-R01.txt --produced-by "the parity step"
call stop-ev-02 -- "$DSTACK" evidence add --r R02 --case c1 --kind cli \
  --artifact artifacts/hook-R02.txt --produced-by "the parity step"
call stop-quick-close -- "$DSTACK" quick close hooked --abandon "the parity step is over"
hook stop-covered "$PAY/stop.json" stop

# Nothing to look at at all: the clear verdict the Stop hook reads as a pass, with CURRENT empty
# so the transcript is recorded for no run either.
call stop-pause -- "$DSTACK" run pause
hook stop-clear "$PAY/stop.json" stop
call stop-adopt -- "$DSTACK" run adopt "$hooks_id" --force

hook_away stop-e400   "$PAY/stop-e400.json"  stop
hook stop-away   "$PAY/stop-away.json" stop
hook stop-broken "$PAY/broken.json"    stop

# ── the wrong usage of the verb (R11) ──────────────────────────────────────────────────
# No event at all, an event nobody registers, and an operand past the event, which is ignored.
hook usage-none    "$PAY/stop.json"
hook usage-unknown "$PAY/stop.json" bogus
hook usage-empty   "$PAY/stop.json" ""
hook usage-extra   "$PAY/stop.json" stop extra

# The run this step opened is closed again, so what it leaves behind is inert, and CURRENT goes
# back to the run the steps after this one expect.
call run-close-hooks -- "$DSTACK" run close "$hooks_id" --abandon "the parity step is over"
call run-adopt-prev  -- "$DSTACK" run adopt "$prev" --force
