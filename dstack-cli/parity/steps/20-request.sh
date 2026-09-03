# parity step: request new, open, show and approve, and check request (R13), with refusals (R11)
. "$PARITY_LIB"

# `request open` launches VSCode when `code` is on PATH (never -w), in the port as in the shell
# (D-14). So every open call here runs with a PATH that holds no `code` — the branch that prints
# the path instead, which is the only branch a sandbox may take: a harness that opened the
# maintainer's editor would be unusable.
BARE_PATH=/usr/bin:/bin

call check-request-none -- "$DSTACK" check request
call request-approve-none -- "$DSTACK" request approve
call request-show-none -- "$DSTACK" request show
call request-open-none -- "$DSTACK" request open
call request-new-bogus -- "$DSTACK" request new --bogus
call request-new-extra -- "$DSTACK" request new extra
call request-new-bad-type -- "$DSTACK" request new --type bogus
call request-new-bad-type-eq -- "$DSTACK" request new --type=bogus
# A value option in the last position: `shift 2` fails under `set -e`, so the shell exits 1
# without printing anything at all.
call request-new-no-type -- "$DSTACK" request new --type
call request-new-no-title -- "$DSTACK" request new --title

call request-new -- "$DSTACK" request new --type cli --title "the parity request"
call request-new-again -- "$DSTACK" request new --type cli
call request-show -- "$DSTACK" request show
# check request and request show take no options: resolve_target keeps what is left and the verb
# ignores it, so --bogus is not an error here.
call request-show-bogus -- "$DSTACK" request show --bogus
call request-open -- env PATH="$BARE_PATH" "$DSTACK" request open
call request-open-again -- env PATH="$BARE_PATH" "$DSTACK" request open
call request-show-after-open -- "$DSTACK" request show
call check-request -- "$DSTACK" check request
call check-request-bogus -- "$DSTACK" check request --bogus

# ── request approve, in a repository of its own ─────────────────────────────────────────
# Approval ends with `dstack cases sync`, which writes cases.tsv into the run it approves. That
# belongs to the approving run, not to the sandbox the harness diffs, so the scenario runs in a
# nested repository (its .dstack is not under $SANDBOX/.dstack) and the files approve writes are
# compared by the `cat` calls below as well as by every count the verbs print.
mkdir -p "$SANDBOX/approve"
( cd "$SANDBOX/approve" && git init -q \
  && git -c user.email=t@t -c user.name=t -c commit.gpgsign=false commit -q --allow-empty -m init )
IN='cd "$SANDBOX/approve" &&'
# The base commit of the nested repository is printed eight characters wide and is a different
# commit in each sandbox by construction; no standard mask covers it.
mask_call approve-run ' @ [^)]*\)' ' @ <GIT>)'
call approve-init -- sh -c "$IN"' exec "$DSTACK" init'
call approve-run -- sh -c "$IN"' exec "$DSTACK" run new approving --type cli'
call approve-new -- sh -c "$IN"' exec "$DSTACK" request new --type cli --title "the approved request"'
call approve-add -- sh -c "$IN"' exec "$DSTACK" req add "the approved row" --accept "an observable criterion"'
call approve-parked -- sh -c "$IN"' exec "$DSTACK" req add "a parked row" --from-answer'
# A request that does not validate is refused before anything is written: two stderr lines, the
# core's count and the verb's own sentence.
call approve-invalid -- sh -c "$IN"' exec "$DSTACK" request approve'
call approve-accept -- sh -c "$IN"' exec "$DSTACK" req accept R02 "the second criterion"'
call approve-one -- sh -c "$IN"' exec "$DSTACK" request approve'
call approve-file -- sh -c "$IN"' cat .dstack/runs/*/request.md'
call approve-stamp -- sh -c "$IN"' cat .dstack/runs/*/request.approved'
call approve-check -- sh -c "$IN"' exec "$DSTACK" check request'
call approve-show -- sh -c "$IN"' exec "$DSTACK" request show'
call approve-pending -- sh -c "$IN"' exec "$DSTACK" req add "a row after approval" --accept "the third criterion"'
call approve-two -- sh -c "$IN"' exec "$DSTACK" request approve'
call approve-file-2 -- sh -c "$IN"' cat .dstack/runs/*/request.md'
call approve-check-2 -- sh -c "$IN"' exec "$DSTACK" check request'
# With a draft snapshot that nothing edited afterwards the diff section prints nothing at all.
call approve-open -- sh -c "$IN"' exec env PATH='"$BARE_PATH"' "$DSTACK" request open'
call approve-three -- sh -c "$IN"' exec "$DSTACK" request approve'
call approve-file-3 -- sh -c "$IN"' cat .dstack/runs/*/request.md'
call approve-draft -- sh -c "$IN"' cat .dstack/runs/*/request.agent-draft.md'

# ── an edited draft, so the rendered diff itself is compared (D-15) ─────────────────────
# One unambiguous change to the title line between the snapshot and the approval: any shortest
# edit script has to be that one line, so the hunk is the same on both sides. Only the two header
# stamps are masked — BSD diff prints the mtime in the machine's local zone and the port prints
# it in UTC, which is the whole of what D-15 allows to differ here.
RD="$(ls -d "$SANDBOX/approve/.dstack/runs/"*)"
mask_call approve-edited '^(---|\+\+\+) (.*)[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$' '\1 \2<STAMP>'
sed 's/^# the approved request$/# the approved request, edited by hand/' "$RD/request.md" > "$RD/edited.md"
mv "$RD/edited.md" "$RD/request.md"
call approve-edited -- sh -c "$IN"' exec "$DSTACK" request approve'
call approve-file-4 -- sh -c "$IN"' cat .dstack/runs/*/request.md'

# ── every failure line of check request, on files written by hand ───────────────────────
rm -f "$RD/request.approved"
printf '# Questions (R51)\n\nWritten only by `dstack ask`.\n\n| Q | Question | Affects | Status |\n|---|---|---|---|\n| Q-01 | still open? | R01 | open |\n| Q-02 | assumed with no row? | R02 | assumed |\n' > "$RD/questions.md"
cat > "$RD/request.md" <<'BROKEN'
---
work_type: cli
route: merge nosuch-run
external_research: none
risk_axes: none,ux
design_review: auto
review: maybe
codex_effort: high
e2e: cli
unit_tests: on
korean_polish: on
stray_key: yes
---
# a broken request

- [x] **R01** a ticked box — accept: c1
- [ ] **R02** no accept segment
- [ ] **R03** fine — accept: c3
- [ ] **R02** a duplicate id — accept: c4
- [ ] not a row at all
- a bullet mentioning **R09** in prose
BROKEN
call check-request-broken -- sh -c "$IN"' exec "$DSTACK" check request'

# R43 is a warning, never a block: 13 live rows and 67 lines print two warn lines on stderr.
{
  printf -- '---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\nunit_tests: on\nvisual: none\nkorean_polish: on\n---\n# a big request\n'
  i=1; while [ "$i" -le 13 ]; do printf -- '- [ ] **R%02d** row %d — accept: criterion %d\n' "$i" "$i" "$i"; i=$((i + 1)); done
  i=1; while [ "$i" -le 40 ]; do printf 'filler line %d\n' "$i"; i=$((i + 1)); done
} > "$RD/request.md"
rm -f "$RD/questions.md"
call check-request-big -- sh -c "$IN"' exec "$DSTACK" check request'
call approve-cases -- sh -c "$IN"' cat .dstack/runs/*/cases.tsv'
