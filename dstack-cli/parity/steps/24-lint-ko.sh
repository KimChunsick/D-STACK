# parity step: lint-ko over files, stdin and changed files (R06), with its refusals (R11)
. "$PARITY_LIB"

# The sandbox has no .dstack/project/ko-scope.tsv, so both sides fall back to the same
# ~/.claude/lint/ko-scope.tsv. Every scope of that table gets a file here.
mkdir -p "$SANDBOX/claude/lint" "$SANDBOX/claude/skills/a" "$SANDBOX/notes" "$SANDBOX/app"
cat > "$SANDBOX/README.md" <<'KO'
# 안내

정본은 이 파일이에요. 나머지 값은 advisory예요.
이 옵션은 병렬 실행을 가능하게 한다.
설정에 있어서 중요한 값이에요.
즉, 이 값은 — 참고로 — 기본값이에요.
회의에서의 결정과 회의에의 결정이에요.
필요한 것은 방향이다.
이 게이트
게이트웨이는 그대로 둬요.
KO
# claude/lint/** is exempt, claude/skills/**/*.md is en (through the "**/" that also matches
# zero directories), *.ko.json is ko-data, and notes/ matches no row at all.
printf '정본은 여기예요.\n' > "$SANDBOX/claude/lint/note.md"
printf '정본은 여기예요.\n' > "$SANDBOX/claude/skills/x.md"
printf '정본은 여기예요.\n' > "$SANDBOX/claude/skills/a/b.md"
printf '정본은 여기예요.\n' > "$SANDBOX/notes/plain.txt"
printf '{"hi":"정본이에요"}\n' > "$SANDBOX/app/copy.ko.json"
printf 'id\tkind\tpattern\tseverity\treplacement\texample\tsource\tlevel\nK01\tregex\t정본\tS1\t기준이 되는 곳\t정본이에요.\tv1\tword\n' \
  > "$SANDBOX/rules-min.tsv"

printf '정본은 여기예요.\n설정에 있어서 좋아요.\n' > "$SANDBOX/in-ko.txt"
printf '정본을 고쳐요\n' > "$SANDBOX/in-msg.txt"

# ── files ──────────────────────────────────────────────────────────────────────────────
call file-one       -- "$DSTACK" lint-ko README.md
call file-report    -- "$DSTACK" lint-ko --report README.md
call file-fragment  -- "$DSTACK" lint-ko --fragment README.md
call file-many      -- "$DSTACK" lint-ko --report README.md claude/lint/note.md claude/skills/x.md claude/skills/a/b.md notes/plain.txt app/copy.ko.json
call file-missing   -- "$DSTACK" lint-ko --report gone.md
call file-directory -- "$DSTACK" lint-ko notes
call file-absolute  -- "$DSTACK" lint-ko "$SANDBOX/README.md"
call file-rules     -- "$DSTACK" lint-ko --rules "$SANDBOX/rules-min.tsv" --report README.md
call file-rules-gone -- "$DSTACK" lint-ko --rules "$SANDBOX/no-such-table.tsv" README.md

# ── stdin ──────────────────────────────────────────────────────────────────────────────
call_stdin stdin-path      "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path README.md
call_stdin stdin-path-eq   "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path=README.md
call_stdin stdin-fragment  "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path README.md --fragment
call_stdin stdin-exempt    "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path claude/lint/note.md --report
call_stdin stdin-unclassed "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path notes/plain.txt --report
call_stdin stdin-outside   "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --path /elsewhere/README.md --report
call_stdin stdin-msg       "$SANDBOX/in-msg.txt" -- "$DSTACK" lint-ko --stdin --scope commit-msg --report
call_stdin stdin-msg-eq    "$SANDBOX/in-msg.txt" -- "$DSTACK" lint-ko --stdin --scope=commit-msg
call_stdin stdin-no-path   "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin
call_stdin stdin-other     "$SANDBOX/in-ko.txt" -- "$DSTACK" lint-ko --stdin --scope en

# ── changed ────────────────────────────────────────────────────────────────────────────
# Two repositories of their own: `git diff --name-only HEAD` and `git ls-files --others` are
# read for the worktree lint-ko is standing in, and the sandbox itself carries whatever the
# earlier steps left behind. The names are deliberately all-lowercase in the second one — the
# shell pipes the union through `sort -u`, which orders by the collation of the UTF-8 locale
# _ko_locale sets, while the port orders by bytes; the two agree here and differ on a mixed-case
# set (reported with P9, not worked around).
git init -q "$SANDBOX/kolint-a"
printf '정본은 이 파일이에요.\n이 게이트\n' > "$SANDBOX/kolint-a/README.md"
call changed-one    -- sh -c 'cd "$SANDBOX/kolint-a" && exec "$DSTACK" lint-ko --changed'
call changed-report -- sh -c 'cd "$SANDBOX/kolint-a" && exec "$DSTACK" lint-ko --changed --report'

git init -q "$SANDBOX/kolint-b"
mkdir -p "$SANDBOX/kolint-b/claude/lint" "$SANDBOX/kolint-b/docs" "$SANDBOX/kolint-b/notes" "$SANDBOX/kolint-b/app"
printf '정본이에요.\n' > "$SANDBOX/kolint-b/docs/guide.md"
printf '정본이에요.\n' > "$SANDBOX/kolint-b/notes/plain.txt"
( cd "$SANDBOX/kolint-b" \
  && git add docs/guide.md notes/plain.txt \
  && git -c user.email=t@t -c user.name=t -c commit.gpgsign=false commit -q -m one )
printf '정본이에요. 설정에 있어서 좋아요.\n' > "$SANDBOX/kolint-b/docs/guide.md"
printf '정본이에요.\n' > "$SANDBOX/kolint-b/claude/lint/note.md"
printf '{"hi":"정본"}\n' > "$SANDBOX/kolint-b/app/copy.ko.json"
call changed-mixed -- sh -c 'cd "$SANDBOX/kolint-b" && exec "$DSTACK" lint-ko --changed --report'

# lint-ko needs no store at all: a commit message from a repository that never ran dstack init
# still has to be checkable, which is why the dispatcher skips the root resolution for this noun.
mkdir -p "$SANDBOX/../nogit"
printf '정본을 고쳐요\n' > "$SANDBOX/../nogit/msg.txt"
call_stdin nogit-msg "$SANDBOX/../nogit/msg.txt" -- sh -c 'cd "$SANDBOX/../nogit" && exec "$DSTACK" lint-ko --stdin --scope commit-msg --report'

# ── wrong usage (R11) ──────────────────────────────────────────────────────────────────
call usage-none      -- "$DSTACK" lint-ko
call usage-bogus     -- "$DSTACK" lint-ko --bogus
call usage-dash      -- "$DSTACK" lint-ko -r
call usage-rules-eq  -- "$DSTACK" lint-ko --rules=x README.md
call usage-path-none -- "$DSTACK" lint-ko --path
call usage-scope-none -- "$DSTACK" lint-ko --stdin --scope
call usage-rules-none -- "$DSTACK" lint-ko --rules
