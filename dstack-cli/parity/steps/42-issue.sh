# parity step: `issue`, a verb the reference never had (R06, R11), as declared divergences
. "$PARITY_LIB"

# The shell-final tag carries no `issue` at all: its dispatcher answers every call below with
# `dstack: unknown command: issue (dstack help)` and exit 1. So this step is a set of declared
# divergences the way the unported nouns were handled (D-20 of run 20260902T085818Z_dstack-rust):
# what it proves is that the harness drives every entry of the verb and that the port answers each
# one without ending the step; what it cannot prove is a single character of those answers, since
# a reference that has no verb has nothing to compare them against. The port's own answers are
# pinned by claude/lint/fixtures/issue-new and tests/r01_issue_verb.rs.
#
# D-05 fixes the folder at $HOME/Documents/dstack-issues with no setting and no override, so a
# filing is kept off the machine's real Documents folder the only way there is: HOME per call,
# pointed inside the sandbox. What lands there is outside .dstack and reaches no store comparison.
HOMEDIR="$SANDBOX/home"
mkdir -p "$HOMEDIR"

issue() {   # $1 call name, then the arguments of the verb
  local name="$1"; shift
  call "$name" -- env HOME="$HOMEDIR" "$DSTACK" "$@"
}

# The reference's one line and the port's own, removed from both sides; the exit code differs
# wherever the port answers 0, so it is masked too.
no_such_verb() {   # $1 call name, $2 what the port answers instead
  expect_diff "$1" "the shell reference has no issue verb: it answers unknown command where the port $2"
  mask_call "$1" '^dstack: .*$' ''
  mask_call "$1" '^issues?[: ].*$' ''
  mask_call "$1" '^  sighting .*$' ''
  mask_call "$1" '^  .*\| sightings .*\| last .*$' ''
  mask_call "$1" '^[0-9]+$' '<RC>'
}

# ── the wrong usage of every entry (R11) ───────────────────────────────────────────────
no_such_verb issue-bare          "names the verb the noun is missing"
no_such_verb issue-unknown-verb  "names the verb it does not have"
no_such_verb issue-new-no-title  "prints the usage of issue new"
no_such_verb issue-new-bogus     "refuses an unknown option"
no_such_verb issue-new-no-source "names the field D-08 requires"
no_such_verb issue-new-extra     "refuses a second operand"
no_such_verb issue-list-extra    "refuses an operand it takes none of"
no_such_verb issue-list-bogus    "refuses an unknown option"

issue issue-bare          issue
issue issue-unknown-verb  issue bogus
issue issue-new-no-title  issue new
issue issue-new-bogus     issue new --bogus
issue issue-new-no-source issue new "the title of a filing" --symptom "it printed nothing" --repro "dstack issue new"
issue issue-new-extra     issue new "the title" again --symptom a --repro b --source c
issue issue-list-extra    issue list extra
issue issue-list-bogus    issue list --bogus

# ── the filing itself, and the sighting a repeat adds to it (D-06) ─────────────────────
no_such_verb issue-list-empty "lists an empty folder and exits 0"
no_such_verb issue-new-filed  "files the issue and exits 0"
no_such_verb issue-new-again  "adds a sighting to the file that is already there"
no_such_verb issue-list-one   "lists the one file with its sighting count"

issue issue-list-empty issue list
issue issue-new-filed  issue new "plan start refuses a file worktree" \
  --symptom "it exits 1 and prints nothing at all" \
  --repro "dstack plan start P4 --worktree ./notes.txt" \
  --source dstack-cli/src/verbs/plan/lifecycle.rs
issue issue-new-again  issue new "Plan start, refuses a file worktree!" \
  --symptom "it exits 1 and prints nothing at all" \
  --repro "dstack plan start P4 --worktree ./notes.txt" \
  --source dstack-cli/src/verbs/plan/lifecycle.rs
issue issue-list-one   issue list
