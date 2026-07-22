#!/bin/bash
# Contract tests for check-parallel.sh — the deterministic fan-out gate.
#
# WHY: LLM-declared task independence is brittle ("false independence"), so fan-out is
# allowed only on this checker's PARALLEL verdict. Every rule here is a safety
# property: INVALID (broken graph/state — serial cannot fix it) must never collapse
# into SERIAL; overlap must catch prefix/case aliases; declared paths must not
# traverse symlinks; completion state must be internally consistent; scope must
# collect the full changed set from git itself (a caller cannot narrow it); and only
# the '## Milestones & tasks' section — fences excluded — is graph data. macOS bash 3.2.
set -u
CP="$(cd "$(dirname "$0")/.." && pwd)/check-parallel.sh"
fails=0; dir="$(mktemp -d)"; trap 'rm -rf "$dir"' EXIT
ok()   { printf 'ok   %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; fails=$((fails + 1)); }
expect() { # <label> <want_rc> <want_stdout_prefix> <mode+args...>
  local label="$1" want_rc="$2" want_out="$3"; shift 3
  local out rc
  out="$(bash "$CP" "$@" 2>/dev/null)"; rc=$?
  if [ "$rc" -eq "$want_rc" ] && case "$out" in "$want_out"*) true ;; *) false ;; esac; then
    ok "$label"
  else
    fail "$label (rc=$rc want $want_rc; out='$out' want '$want_out*')"
  fi
}

[ -f "$CP" ] || { echo "FAIL check-parallel.sh not found at $CP"; exit 1; }

# GOAL fixtures live inside a git repo (the checker binds path checks to one).
R="$dir/goals"; git init -q "$R"
goal() { local f="$R/$1.md"; cat > "$f"; printf '%s\n' "$f"; }
G() { git -C "$1" -c user.email=t@t -c user.name=t "${@:2}"; }

# ── fixtures ──────────────────────────────────────────────────────────────────
g_indep="$(goal indep <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c, lib/util/]
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"
g_edge="$(goal edge <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c]
- [ ] **T02** b — two. deps: [T01]; files: [src/b.c]
EOF
)"
g_edge_done="$(goal edge_done <<'EOF'
## Milestones & tasks
### M1 — f
- [x] **T01** a — one. deps: []; files: [src/a.c]
- [ ] **T02** b — two. deps: [T01]; files: [src/b.c]
- [ ] **T03** c — three. deps: []; files: [src/c.c]
EOF
)"
g_closure="$(goal closure <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c]
- [x] **T02** b — checked with open dep. deps: [T01]; files: [src/b.c]
EOF
)"
g_overlap="$(goal overlap <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/shared.c]
- [ ] **T02** b — two. deps: []; files: [src/shared.c]
EOF
)"
g_prefix="$(goal prefix <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/]
- [ ] **T02** b — two. deps: []; files: [src/deep/b.c]
EOF
)"
g_case="$(goal casev <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [Src/A.c]
- [ ] **T02** b — two. deps: []; files: [src/a.c]
EOF
)"
g_cycle="$(goal cycle <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: [T02]; files: [src/a.c]
- [ ] **T02** b — two. deps: [T01]; files: [src/b.c]
EOF
)"
g_unknown="$(goal unknown <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: [T09]; files: [src/a.c]
EOF
)"
g_dupfield="$(goal dupfield <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; deps: []; files: [src/a.c]
EOF
)"
g_glob="$(goal globf <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/*.c]
EOF
)"
g_dotdot="$(goal dotdot <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [../etc/passwd]
EOF
)"
g_noncanon="$(goal noncanon <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/./a.c]
EOF
)"
g_nofiles="$(goal nofiles <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []
EOF
)"
g_wrapped="$(goal wrapped <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — a long description that wraps across
  physical lines before the declaration lands.
  deps: []; files: [src/a.c]
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"
g_empty="$(goal emptyf <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: []
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"
g_dupid="$(goal dupid <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c]
- [ ] **T01** b — dup id. deps: []; files: [src/b.c]
EOF
)"
g_goaldocs="$(goal goaldocs <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [docs/somegoal/x.md]
EOF
)"
mkdir "$dir/outside"; ln -s "$dir/outside" "$R/escape"
g_symlink="$(goal symlink <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [escape/output]
EOF
)"
g_fenced="$(goal fenced <<'EOF'
## Milestones & tasks
### M1 — f
```
- [ ] **T01** fake — an example row inside a fence. deps: []; files: [x.c]
```
- [ ] **T01** real — actual. deps: []; files: [src/a.c]
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"
g_nosection="$(goal nosection <<'EOF'
## Something else
- [ ] **T01** a — one. deps: []; files: [src/a.c]
EOF
)"
g_fencepre="$(goal fencepre <<'EOF'
Prose before.
```
## Milestones & tasks
### M1 — ex
- [ ] **T01** fake — a fully fenced example section. deps: []; files: [x.c]
```
## Milestones & tasks
### M1 — f
- [ ] **T01** real — actual. deps: []; files: [src/a.c]
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"
g_nested="$(goal nested <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c]
  - [ ] **T09** nested lookalike — not a peer row. deps: []; files: [x.c]
- [ ] **T02** b — two. deps: []; files: [src/b.c]
EOF
)"

# ── plan verdicts ─────────────────────────────────────────────────────────────
expect "independent pair → PARALLEL"          0 "PARALLEL" plan "$g_indep" T01 T02
expect "direct dep edge → SERIAL"             1 "SERIAL"   plan "$g_edge" T01 T02
expect "unready dep (unticked) → SERIAL"      1 "SERIAL"   plan "$g_edge" T02
expect "dep done (ticked) → PARALLEL"         0 "PARALLEL" plan "$g_edge_done" T02 T03
expect "exact file overlap → SERIAL"          1 "SERIAL"   plan "$g_overlap" T01 T02
expect "dir-prefix overlap → SERIAL"          1 "SERIAL"   plan "$g_prefix" T01 T02
expect "case-variant overlap → SERIAL"        1 "SERIAL"   plan "$g_case" T01 T02
expect "empty files → SERIAL (ineligible)"    1 "SERIAL"   plan "$g_empty" T01 T02
expect "wrapped declaration parses"           0 "PARALLEL" plan "$g_wrapped" T01 T02
expect "fenced example row is ignored"        0 "PARALLEL" plan "$g_fenced" T01 T02
expect "fenced pre-section example ignored"   0 "PARALLEL" plan "$g_fencepre" T01 T02
expect "indented lookalike row is not graph"  0 "PARALLEL" plan "$g_nested" T01 T02
expect "indented lookalike id is unknown"     2 "INVALID"  plan "$g_nested" T09
# ── INVALID is blocking, never collapsed to SERIAL ────────────────────────────
expect "dependency cycle → INVALID"           2 "INVALID"  plan "$g_cycle" T01 T02
expect "unknown dep id → INVALID"             2 "INVALID"  plan "$g_unknown" T01
expect "duplicate deps field → INVALID"       2 "INVALID"  plan "$g_dupfield" T01
expect "glob in files → INVALID"              2 "INVALID"  plan "$g_glob" T01
expect "parent traversal → INVALID"           2 "INVALID"  plan "$g_dotdot" T01
expect "non-canonical ./ → INVALID"           2 "INVALID"  plan "$g_noncanon" T01
expect "missing files field → INVALID"        2 "INVALID"  plan "$g_nofiles" T01
expect "duplicate task id → INVALID"          2 "INVALID"  plan "$g_dupid" T01
expect "goal-docs path declared → INVALID"    2 "INVALID"  plan "$g_goaldocs" T01
expect "unknown candidate id → INVALID"       2 "INVALID"  plan "$g_indep" T01 T99
expect "symlink-traversing path → INVALID"    2 "INVALID"  plan "$g_symlink" T01
expect "checked-with-open-dep → INVALID"      2 "INVALID"  plan "$g_closure" T01
expect "checked candidate → INVALID"          2 "INVALID"  plan "$g_edge_done" T01 T03
expect "no Milestones section → INVALID"      2 "INVALID"  plan "$g_nosection" T01
# ── scope: committed-state containment, identity-bound, clean-tree ────────────
# The GOAL.md must live in the SAME repository as the worktree, so scope fixtures
# carry their own goal file inside the work repo.
W="$dir/w"; git init -q "$W"
/bin/cat > "$W/GOAL.md" <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [src/a.c, lib/util/, GOAL.md]
EOF
printf 'base\n' > "$W/README"; G "$W" add README GOAL.md; G "$W" commit -qm base
B="$(git -C "$W" rev-parse HEAD)"
BR="$(git -C "$W" branch --show-current)"
mkdir -p "$W/src" "$W/lib/util/deep"
printf 'a\n' > "$W/src/a.c"; printf 'x\n' > "$W/lib/util/deep/x.c"
G "$W" add src/a.c lib/util/deep/x.c; G "$W" commit -qm work
expect "scope: declared committed clean → PASS"   0 "PASS"      scope "$W/GOAL.md" T01 "$W" "$B" "$BR"
printf 's\n' > "$W/stray.txt"                 # ANY uncommitted/untracked file breaks cleanliness
expect "scope: unclean tree → VIOLATION"          1 "VIOLATION" scope "$W/GOAL.md" T01 "$W" "$B" "$BR"
rm "$W/stray.txt"
printf 'o\n' > "$W/src/other.c"; G "$W" add src/other.c; G "$W" commit -qm sneak
expect "scope: undeclared COMMITTED → VIOLATION"  1 "VIOLATION" scope "$W/GOAL.md" T01 "$W" "$B" "$BR"
expect "scope: wrong branch → INVALID"            2 "INVALID"   scope "$W/GOAL.md" T01 "$W" "$B" not-that-branch
G "$W" branch alt "$B"; G "$W" checkout -q alt
printf 'z\n' > "$W/zzz"; G "$W" add zzz; G "$W" commit -qm altwork
ALT="$(git -C "$W" rev-parse HEAD)"; G "$W" checkout -q "$BR"
expect "scope: base not ancestor → INVALID"       2 "INVALID"   scope "$W/GOAL.md" T01 "$W" "$ALT" "$BR"
expect "scope: foreign-repo GOAL → INVALID"       2 "INVALID"   scope "$g_indep" T01 "$W" "$B" "$BR"
expect "scope: unknown task → INVALID"            2 "INVALID"   scope "$W/GOAL.md" T99 "$W" "$B" "$BR"
expect "scope: bad base commit → INVALID"         2 "INVALID"   scope "$W/GOAL.md" T01 "$W" deadbeef "$BR"
expect "scope: missing args → INVALID"            2 "INVALID"   scope "$W/GOAL.md" T01 "$W" "$B"
# Symlink materialized under a directory-ownership declaration is rejected even
# when committed on a clean tree (writes through it are invisible to git).
WS="$dir/ws"; git init -q "$WS"
/bin/cat > "$WS/GOAL.md" <<'EOF'
## Milestones & tasks
### M1 — f
- [ ] **T01** a — one. deps: []; files: [lib/util/, GOAL.md]
EOF
G "$WS" add GOAL.md; G "$WS" commit -qm base
BS="$(git -C "$WS" rev-parse HEAD)"; BRS="$(git -C "$WS" branch --show-current)"
mkdir -p "$WS/lib/util"; ln -s "$dir/outside" "$WS/lib/util/esc"
G "$WS" add lib/util/esc; G "$WS" commit -qm linkwork
expect "scope: symlink under dir ownership → VIOLATION" 1 "VIOLATION" scope "$WS/GOAL.md" T01 "$WS" "$BS" "$BRS"
# ── usage hygiene ─────────────────────────────────────────────────────────────
expect "no such goal file → INVALID"          2 "INVALID"   plan "$dir/absent.md" T01
cp "$g_indep" "$dir/nogit.md"
expect "goal outside a git repo → INVALID"    2 "INVALID"   plan "$dir/nogit.md" T01
expect "bad mode → INVALID"                   2 "INVALID"   frobnicate "$g_indep" T01

if [ "$fails" -gt 0 ]; then echo "== $fails failure(s)"; exit 1; fi
echo "== all checks passed"
