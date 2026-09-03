# parity step: the roadmap — milestone and the plan verbs (R13), with their refusals (R11)
. "$PARITY_LIB"

# The verbs work in a run of their own: a plan.json another step left behind would shift every
# id minted here, so what the harness compares never depends on which steps ran before it.
prev="$(cat "$SANDBOX/.dstack/local/CURRENT")"
# adopt prints the owner heartbeat, whose pid is the pid of whatever called dstack, and run new
# prints the base commit eight characters wide — one past the short hash the harness masks.
mask_call run-adopt-prev ':[0-9]+@' ':<PID>@'
mask_call run-new-plans ' @ [^)]*\)' ' @ <GIT>)'

call run-pause-prev -- "$DSTACK" run pause
call run-new-plans  -- "$DSTACK" run new plans --type cli
plans_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
run_dir="$SANDBOX/.dstack/runs/$plans_id"

# One R row per state `task add --covers` has to judge: live, withdrawn, deferred, superseded
# and (after the approval) pending.
call req-new       -- "$DSTACK" request new --type cli --title "the roadmap request"
call req-add-one   -- "$DSTACK" req add "the first row" --accept "the first criterion"
call req-add-two   -- "$DSTACK" req add "the withdrawn row" --accept "the second criterion"
call req-add-three -- "$DSTACK" req add "the third row" --accept "the third criterion"
call req-add-four  -- "$DSTACK" req add "the deferred row" --accept "the fourth criterion"
call req-add-five  -- "$DSTACK" req add "the split row" --accept "the fifth criterion"
call req-withdraw  -- "$DSTACK" req withdraw R02 --why "the owner dropped it"
call req-defer     -- "$DSTACK" req defer R04 --why "the next Goal"
call req-split     -- "$DSTACK" req split R05 --into R01,R03
call req-approve   -- "$DSTACK" request approve
call req-add-late  -- "$DSTACK" req add "a row nobody approved yet" --accept "the late criterion"

# A quick task structurally cannot hold plans; the directory is what resolve_target looks for.
mkdir -p "$SANDBOX/.dstack/quick/roadmap"
call plan-add-quick -- "$DSTACK" plan add first --quick roadmap --files a

# Nothing minted yet: the verbs that read plan.json say where to start.
call plan-remove-no-json -- "$DSTACK" plan remove P1
call plan-edit-no-json   -- "$DSTACK" plan edit P1 --slug other
call plan-render-no-json -- "$DSTACK" plan render
call plan-start-no-json  -- "$DSTACK" plan start P1
call plan-done-no-json   -- "$DSTACK" plan done P1
call task-add-no-json    -- "$DSTACK" task add write --plan P1 --covers R01 --files src/a.rs
call task-done-no-json   -- "$DSTACK" task done T1 --commit HEAD
call next-no-json        -- "$DSTACK" next

# ── milestone add ──────────────────────────────────────────────────────────────────────
call ms-add-no-slug   -- "$DSTACK" milestone add
call ms-add-bad-slug  -- "$DSTACK" milestone add Bad_Slug
call ms-add-bogus     -- "$DSTACK" milestone add core --bogus
call ms-add-extra     -- "$DSTACK" milestone add core extra
call ms-add-no-value  -- "$DSTACK" milestone add core --after
call ms-add-core      -- "$DSTACK" milestone add core
call ms-add-wrap      -- "$DSTACK" milestone add wrap
call ms-add-bad-after -- "$DSTACK" milestone add mid --after M9
call ms-add-mid       -- "$DSTACK" milestone add mid --after M1
call ms-add-tail      -- "$DSTACK" milestone add tail --after=M2

# ── plan add ───────────────────────────────────────────────────────────────────────────
call plan-add-no-slug   -- "$DSTACK" plan add
call plan-add-bad-slug  -- "$DSTACK" plan add Bad_Slug --milestone M1 --files src/a.rs
call plan-add-bogus     -- "$DSTACK" plan add first --nope x
call plan-add-extra     -- "$DSTACK" plan add first second --milestone M1 --files src/a.rs
call plan-add-after     -- "$DSTACK" plan add first --milestone M1 --files src/a.rs --after P1
call plan-add-no-files  -- "$DSTACK" plan add first --milestone M1
call plan-add-abs-file  -- "$DSTACK" plan add first --milestone M1 --files /etc/passwd
call plan-add-dotdot    -- "$DSTACK" plan add first --milestone M1 --files ../outside.rs
call plan-add-glob      -- "$DSTACK" plan add first --milestone M1 --files 'src/*.rs'
call plan-add-no-ms     -- "$DSTACK" plan add first --files src/a.rs
call plan-add-bad-ms    -- "$DSTACK" plan add first --milestone M9 --files src/a.rs
call plan-add-bad-dep   -- "$DSTACK" plan add first --milestone M1 --files src/a.rs --deps P9
call plan-add-self-dep  -- "$DSTACK" plan add first --milestone M1 --files src/a.rs --deps P1
call plan-add-no-value  -- "$DSTACK" plan add first --milestone M1 --files
call plan-add-first     -- "$DSTACK" plan add first --milestone M1 --files src/a.rs,src/lib
call plan-add-second    -- "$DSTACK" plan add second --milestone M1 --files src/b.rs --deps P1
call plan-add-third     -- "$DSTACK" plan add third --milestone=M2 --files=src/lib/c.rs --deps=P1,P2

# ── plan insert ────────────────────────────────────────────────────────────────────────
call plan-insert-no-slug  -- "$DSTACK" plan insert --after P1
call plan-insert-no-after -- "$DSTACK" plan insert between --milestone M1 --files src/e.rs
call plan-insert-bad-after -- "$DSTACK" plan insert between --after P9 --files src/e.rs
call plan-insert-one      -- "$DSTACK" plan insert between --after P1 --files src/e.rs
call plan-insert-two      -- "$DSTACK" plan insert another --after P1 --files src/f.rs --deps P1

# ── plan edit ──────────────────────────────────────────────────────────────────────────
call plan-edit-no-plan   -- "$DSTACK" plan edit
call plan-edit-bad-plan  -- "$DSTACK" plan edit P9 --slug other
call plan-edit-nothing   -- "$DSTACK" plan edit P1
call plan-edit-bogus     -- "$DSTACK" plan edit P1 --nope x
call plan-edit-extra     -- "$DSTACK" plan edit P1 P2 --slug other
call plan-edit-bad-slug  -- "$DSTACK" plan edit P1 --slug Bad_Slug
call plan-edit-bad-files -- "$DSTACK" plan edit P1 --files /etc/passwd
call plan-edit-bad-dep   -- "$DSTACK" plan edit P2 --deps P9
call plan-edit-cycle     -- "$DSTACK" plan edit P1 --deps P2
call plan-edit-no-value  -- "$DSTACK" plan edit P1 --slug
call plan-edit-slug      -- "$DSTACK" plan edit P2 --slug renamed
call plan-edit-files     -- "$DSTACK" plan edit P2 --files src/b.rs,src/g.rs
call plan-edit-deps      -- "$DSTACK" plan edit P3 --deps=P2

# ── plan remove ────────────────────────────────────────────────────────────────────────
call plan-remove-none  -- "$DSTACK" plan remove
call plan-remove-bad   -- "$DSTACK" plan remove P9
call plan-remove-bogus -- "$DSTACK" plan remove --bogus
call plan-remove-used  -- "$DSTACK" plan remove P1
call plan-remove-ok    -- "$DSTACK" plan remove P1.2

# ── plan render ────────────────────────────────────────────────────────────────────────
call plan-render        -- "$DSTACK" plan render
# render takes no arguments: resolve_target keeps what is left over and the verb ignores it.
call plan-render-bogus  -- "$DSTACK" plan render --bogus
call plan-render-no-run -- "$DSTACK" plan render --run nosuch

# ── plan start ─────────────────────────────────────────────────────────────────────────
call plan-start-none     -- "$DSTACK" plan start
call plan-start-bad      -- "$DSTACK" plan start P9
call plan-start-bogus    -- "$DSTACK" plan start P1 --nope x
call plan-start-extra    -- "$DSTACK" plan start P1 P2
call plan-start-unmet    -- "$DSTACK" plan start P2
call plan-start-no-value -- "$DSTACK" plan start P1 --worktree
call plan-start-p1       -- "$DSTACK" plan start P1 --worktree "$SANDBOX/wt-P1"
call plan-start-again    -- "$DSTACK" plan start P1
# What R67 protects while a worker holds the files of the subtree.
call plan-insert-busy    -- "$DSTACK" plan insert late --after P1 --files src/h.rs
call plan-edit-busy      -- "$DSTACK" plan edit P1 --slug hurried
call plan-remove-busy    -- "$DSTACK" plan remove P1
# A --worktree path that is already there is recorded, and nothing is created.
call plan-start-existing -- "$DSTACK" plan start P1.1 --worktree "$SANDBOX/wt-P1"

# ── plan done ──────────────────────────────────────────────────────────────────────────
call plan-done-none    -- "$DSTACK" plan done
call plan-done-bad     -- "$DSTACK" plan done P9
call plan-done-pending -- "$DSTACK" plan done P2
call plan-done-p1      -- "$DSTACK" plan done P1
call plan-done-again   -- "$DSTACK" plan done P1

# The branch plan start would mint, already there: the refusal names it and the way out.
git -C "$SANDBOX" branch plan/P2-renamed HEAD
call plan-start-branch -- "$DSTACK" plan start P2 --worktree "$SANDBOX/wt-P2"
git -C "$SANDBOX" branch -D plan/P2-renamed >/dev/null
call plan-start-p2     -- "$DSTACK" plan start P2 --worktree "$SANDBOX/wt-P2"

call plan-render-final -- "$DSTACK" plan render
call roadmap-file -- cat "$run_dir/ROADMAP.md"
call state-file   -- cat "$run_dir/STATE.md"

# ── task add ───────────────────────────────────────────────────────────────────────────
call task-add-no-slug    -- "$DSTACK" task add
call task-add-bad-slug   -- "$DSTACK" task add Bad_Slug --plan P1.1 --covers R01 --files src/e.rs
call task-add-bogus      -- "$DSTACK" task add write --nope x
call task-add-extra      -- "$DSTACK" task add write more --plan P1.1 --covers R01 --files src/e.rs
call task-add-no-plan    -- "$DSTACK" task add write --covers R01 --files src/e.rs
call task-add-bad-plan   -- "$DSTACK" task add write --plan P9 --covers R01 --files src/e.rs
call task-add-done-plan  -- "$DSTACK" task add write --plan P1 --covers R01 --files src/a.rs
call task-add-no-files   -- "$DSTACK" task add write --plan P1.1 --covers R01
call task-add-glob       -- "$DSTACK" task add write --plan P1.1 --covers R01 --files 'src/*.rs'
call task-add-no-covers  -- "$DSTACK" task add write --plan P1.1 --files src/e.rs
call task-add-outside    -- "$DSTACK" task add write --plan P1.1 --covers R01 --files src/zz.rs
call task-add-not-an-id  -- "$DSTACK" task add write --plan P1.1 --covers bogus --files src/e.rs
call task-add-unknown-r  -- "$DSTACK" task add write --plan P1.1 --covers R99 --files src/e.rs
call task-add-withdrawn  -- "$DSTACK" task add write --plan P1.1 --covers R02 --files src/e.rs
call task-add-deferred   -- "$DSTACK" task add write --plan P1.1 --covers R04 --files src/e.rs
call task-add-superseded -- "$DSTACK" task add write --plan P1.1 --covers R05 --files src/e.rs
call task-add-pending    -- "$DSTACK" task add write --plan P1.1 --covers R06 --files src/e.rs
call task-add-no-value   -- "$DSTACK" task add write --plan P1.1 --covers
call task-add-one        -- "$DSTACK" task add write-first --plan P1.1 --covers R01 --files src/e.rs
call task-add-two        -- "$DSTACK" task add write-second --plan=P1.1 --covers=R01,R03 --files=src/e.rs --deps=T1
call task-add-bad-dep    -- "$DSTACK" task add write-third --plan P1.1 --covers R03 --files src/e.rs --deps T9
call task-add-cycle      -- "$DSTACK" task add write-third --plan P1.1 --covers R03 --files src/e.rs --deps T3
# The narrowed --files a task already left: R64 refuses the edit instead of leaving it broken.
call plan-edit-narrow    -- "$DSTACK" plan edit P3 --files src/lib/c.rs

# ── task done ──────────────────────────────────────────────────────────────────────────
# A commit object with no parent, a fixed empty tree, a fixed identity and a fixed date: the
# same sha in both sandboxes, so what task done records is compared like any other byte.
tree="$(git -C "$SANDBOX" mktree </dev/null)"
sha="$(GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_AUTHOR_DATE='2026-01-01T00:00:00 +0000' \
       GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t GIT_COMMITTER_DATE='2026-01-01T00:00:00 +0000' \
       git -C "$SANDBOX" -c commit.gpgsign=false commit-tree "$tree" -m "the task commit" </dev/null)"

call task-done-none      -- "$DSTACK" task done
call task-done-bad-task  -- "$DSTACK" task done T9 --commit "$sha"
call task-done-bogus     -- "$DSTACK" task done T1 --nope x
call task-done-extra     -- "$DSTACK" task done T1 T2 --commit "$sha"
call task-done-no-commit -- "$DSTACK" task done T1
call task-done-bad-sha   -- "$DSTACK" task done T1 --commit 0123456789abcdef0123456789abcdef01234567
call task-done-no-value  -- "$DSTACK" task done T1 --commit
call task-done-one       -- "$DSTACK" task done T1 --commit "$sha"
call task-done-again     -- "$DSTACK" task done T1 --commit "$sha"

# ── next ───────────────────────────────────────────────────────────────────────────────
# Two ready plans whose files overlap: the greedy set takes the first and says why it stopped.
call plan-add-fourth -- "$DSTACK" plan add fourth --milestone M2 --files src/lib
call plan-add-fifth  -- "$DSTACK" plan add fifth --milestone M2 --files src/lib/c.rs,docs
call plan-add-sixth  -- "$DSTACK" plan add sixth --milestone M2 --files docs

call next-bogus      -- "$DSTACK" next --bogus
call next-positional -- "$DSTACK" next P1
call next-max-zero   -- "$DSTACK" next --max 0
call next-max-bad    -- "$DSTACK" next --max abc
call next-max-empty  -- "$DSTACK" next --max=
call next-max-novalue -- "$DSTACK" next --max
call next            -- "$DSTACK" next
call next-max-one    -- "$DSTACK" next --max 1
call next-max-four   -- "$DSTACK" next --max=4
call next-max-biggest -- "$DSTACK" next --max 9223372036854775807
# A digit-only --max past what bash's test builtin can hold: `[ "$max" -ge 1 ]` fails there, so
# the shell lands on the same refusal and adds one diagnostic line of its own.
for name in next-max-overflow next-max-huge; do
  expect_diff "$name" "D-11: bash prints its own '[: integer expression expected' line naming the reference's next.sh"
  mask_call "$name" '^.*next\.sh: line [0-9]+: \[: [0-9]+: integer expression expected$' ''
done
call next-max-overflow -- "$DSTACK" next --max 9223372036854775808
call next-max-huge     -- "$DSTACK" next --max 99999999999999999999

# A policy value that is not a number falls back to the built-in cap, and says so.
cp "$SANDBOX/.dstack/project/PROJECT.md" "$SANDBOX/project.bak"
sed 's/^max_concurrent: 3$/max_concurrent: many/' "$SANDBOX/project.bak" > "$SANDBOX/.dstack/project/PROJECT.md"
call next-cap-default -- "$DSTACK" next
# A policy value no integer can hold is not refused anywhere: it is printed as it stands and the
# free slots come out of the same wrapping arithmetic bash does.
sed 's/^max_concurrent: 3$/max_concurrent: 9223372036854775808/' "$SANDBOX/project.bak" > "$SANDBOX/.dstack/project/PROJECT.md"
call next-cap-overflow -- "$DSTACK" next
cp "$SANDBOX/project.bak" "$SANDBOX/.dstack/project/PROJECT.md"

# R38: another open run's plans are a warning, never a block.
mask_call run-new-other ' @ [^)]*\)' ' @ <GIT>)'
mask_call run-adopt-plans ':[0-9]+@' ':<PID>@'
call run-pause-plans -- "$DSTACK" run pause
call run-new-other   -- "$DSTACK" run new other --type cli
other_id="$(cat "$SANDBOX/.dstack/local/CURRENT")"
call other-ms        -- "$DSTACK" milestone add core
call other-plan      -- "$DSTACK" plan add shared --milestone M1 --files src/e.rs,src/lib
call run-adopt-plans -- "$DSTACK" run adopt "$plans_id" --force
call next-cross      -- "$DSTACK" next

# ── plan start and the three shapes of --worktree ──────────────────────────────────────
# An existing path that is not a directory: the shell's `cd "$wt"` fails and set -e ends the run
# right there — nothing is written and no dstack line is printed at all.
printf 'a regular file, not a directory\n' > "$SANDBOX/wt-file"
expect_diff plan-start-file "D-11: bash prints its own 'cd: ...: Not a directory' line naming the reference's plan.sh; the port ends with cd's status and prints nothing"
mask_call plan-start-file '^.*plan\.sh: line [0-9]+: cd: .*: Not a directory$' ''
call plan-start-file -- "$DSTACK" plan start P4 --worktree "$SANDBOX/wt-file"
# An existing directory that is no git worktree is recorded as it stands, and nothing is created.
mkdir -p "$SANDBOX/wt-plain"
call plan-start-plain-dir -- "$DSTACK" plan start P4 --worktree "$SANDBOX/wt-plain"
# A path whose parent does not exist yet is created with its leading directories.
call plan-start-deep -- "$DSTACK" plan start P5 --worktree "$SANDBOX/deep/nest/wt-P5"

# ── a comma and a newline are the same separator ───────────────────────────────────────
# `tr ',' '\n'` makes a literal newline in a list option exactly what a comma is, an item is
# trimmed, and an empty item is dropped — so the store holds the same items either way.
nl_files="$(printf 'src/n1.rs\nsrc/n2.rs')"
nl_deps="$(printf 'P4\nP5')"
nl_covers="$(printf 'R01\nR03')"
call plan-add-newline-files -- "$DSTACK" plan add seventh --milestone M2 --files "$nl_files"
call plan-add-newline-deps  -- "$DSTACK" plan add eighth --milestone M2 --files src/n3.rs --deps "$nl_deps"
call plan-edit-newline      -- "$DSTACK" plan edit P7 --files "$(printf 'src/n1.rs\nsrc/n4.rs')"
call task-add-newline       -- "$DSTACK" task add write-newline --plan P1.1 --covers "$nl_covers" --files "$(printf 'src/e.rs\n')"
call task-add-empty-item    -- "$DSTACK" task add write-empty --plan P1.1 --covers "R01,,R03" --files "src/e.rs,"
call task-add-padded-item   -- "$DSTACK" task add write-padded --plan P1.1 --covers "$(printf 'R01\n  R03  ')" --files src/e.rs
call plan-add-newline-only  -- "$DSTACK" plan add ninth --milestone M2 --files "$(printf '\n  \n')"

# ── what the store holds ───────────────────────────────────────────────────────────────
call plan-json -- cat "$run_dir/plan.json"

# The two runs this step opened are closed again, so what it leaves behind is inert.
call run-close-other -- "$DSTACK" run close "$other_id" --abandon "the parity step is over"
call run-close-plans -- "$DSTACK" run close "$plans_id" --abandon "the parity step is over"
call run-adopt-prev  -- "$DSTACK" run adopt "$prev" --force
