# parity step: dstack doctor, the repository sweep (R09, R13) and its wrong usage (R11)
. "$PARITY_LIB"

# doctor reads the tree its own binary sits in — deps.tsv, the agent definitions, the skills,
# settings.json, the worktree list and the source tree — not the sandbox store. Since P17 those
# are two different trees: the port reads the repository, the reference reads what --shell-ref
# extracted out of the shell-final tag, which is `claude` and `deps.tsv` and nothing else. Three
# sections are re-declared under D-20 for that, and every other section still compares byte for
# byte.
#
#   lib layout    the reference measures the shell files of the tag (its library, its dispatcher
#                 and the hook wrapper), the port measures dstack-cli/src/**/*.rs — different
#                 files, different counts. The rows are dropped and the count line is masked.
#   verb sweep    the reference sweeps only the `claude` half of the extraction, the port also
#                 sweeps README.md, CLAUDE.md, AGENTS.md and codex/, which hold 31 mentions more.
#                 Masking the count is what is left; the rows it comes from are still compared,
#                 and "without a function: 0" — the half of the section R13 rests on — is not
#                 masked on either side.
#   roster        the port's roster is the shell roster plus the hook entry of D-01 (R13), and
#                 claude/hooks/dstack-hook.sh names `dstack hook`: the reference has no such verb,
#                 so it reports that mention as unknown and ends the sweep — and the whole
#                 command — as failing. The mention row, the unknown count and the exit code are
#                 masked; the number of sections is not.
#
# `doctor --self` is not driven here: it runs every fixture of the repository (97 s in the shell)
# and its rows are the same fixture directories on both sides, so the harness would pay two
# minutes per run for what tests/r05_fixture_runner.rs proves in process.
expect_diff doctor-full "D-20: the reference is extracted from the shell-final tag, so its lib table measures that tree's shell files and its verb sweep only the claude half of it, where the port measures dstack-cli/src/**/*.rs and sweeps the whole repository; and the port's roster carries the hook entry of D-01 that claude/hooks/dstack-hook.sh mentions"
# The rows of the lib table: "<file> | <lines> | <responsibility>", the only three-column rows
# with a number in the middle that doctor prints.
mask_call doctor-full '^  [^|]+ \| [0-9]+ \| .*$' ''
mask_call doctor-full '^  files [0-9]+, lines [0-9]+, over the limit' '  files <N>, lines <N>, over the limit'
mask_call doctor-full '^  roster: [0-9]+ entries,' '  roster: <N> entries,'
# The owner heartbeat of the run CURRENT names carries the pid of whatever called dstack.
mask_call doctor-full ':[0-9]+\)$' ':<PID>)'
mask_call doctor-full '^  .*: dstack hook$' ''
mask_call doctor-full '^  mentions: [0-9]+, unknown verbs: [0-9]+$' '  mentions: <N>, unknown verbs: <N>'
mask_call doctor-full '^doctor: 8 sections, [0-9]+ failing$' 'doctor: 8 sections, <N> failing'
mask_call doctor-full '^[0-9]+$' '<RC>'
call doctor-full -- "$DSTACK" doctor

# ── the wrong usage of the noun (R11) ──────────────────────────────────────────────────
# cmd_doctor takes --self or nothing at all: anything else is the usage line and exit 1.
call doctor-bogus -- "$DSTACK" doctor --bogus
call doctor-extra -- "$DSTACK" doctor one two
