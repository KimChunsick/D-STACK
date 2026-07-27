#!/bin/bash
# Stop hook — full-cycle gate. States the incomplete work of every registered document ONCE per
# user turn, then lets the turn end (see ONE BLOCK PER TURN below). It is not a wall.
#
# HONEST SCOPE: this is a tripwire over the registered docs + the Codex review, NOT a sandbox.
# It cannot prove TDD/E2E actually ran (a checkbox is self-attested), nor can it out-parse a
# determined faker: a *lying* Consensus line ('resolved' over a rejecting review body), work
# hidden under a nested/fenced heading (the gate is section-scoped), or a milestone written in
# setext form are self-attestation limits no regex closes. What it DOES enforce mechanically:
# unchecked gate boxes (any GFM marker) block; tasks require a single registered Goal; every ATX
# 'M<n>' milestone heading needs a ticked milestone-E2E box; every registered task needs a
# contiguous codex-review-001.md…NNN series whose latest file has exactly one clean positive
# Consensus line (a strict allow-list — 'disagreed' / 'unresolved' / 'agreed was not reached' /
# 'agreed to reject' all fail). A legacy codex-review.md alone never passes. Together with the
# inject hook + adversarial review, skipping becomes costly and visible — defense in depth, not
# a single airtight gate.
#
# Registry: `.dstack/active/`, one JSON record per registered document ({v,session,doc,ts}),
# written only by `dstack` (see claude/bin/dstack). Records appear via `ln`, so a record is
# either absent or complete — this scan never sees a half-written one. It DOES have to tolerate
# entries vanishing between listing and opening, because POSIX does not promise directory
# iteration is a snapshot; a record that disappears mid-scan is a deregistration that raced us.
# A record that exists but will not parse is NOT skipped: it is reported, because silently
# ignoring an unreadable record is exactly how work becomes ungated without anyone noticing.
#
# Per-session scoping: each record carries its owner session. This Stop hook enforces only the
# records IT owns, keyed on $CLAUDE_CODE_SESSION_ID (the same id Claude Code injects into hook +
# Bash subprocesses; it is also the `session_id` on hook stdin). SELF-ATTESTATION LIMIT: like a
# ticked checkbox, the owner field is written by the guarded agent — a wrong owner (bug or
# intent) can make this Stop ignore a doc, and `/clear` rotates the id so a doc registered before
# it becomes an orphan no live session owns (blocks nobody — an accepted, documented cost;
# `dstack reclaim <doc>` adopts it deliberately). This is not a new *malicious* bypass: `dstack
# unreg` (the escape hatch) already does the same. FAIL-CLOSED: any record we cannot attribute —
# an empty owner, an unparseable record, or an empty $CLAUDE_CODE_SESSION_ID (id unknown) — is
# enforced by EVERY session, so uncertainty blocks rather than silently clears.
#
# Escape hatch (avoids deadlock): `$HOME/.claude/bin/dstack unreg <doc>` to pause that
# document. The CLI is always called by absolute path — nothing puts ~/.claude/bin on PATH, so a
# remediation message naming a bare `dstack` tells the reader to run something that does not
# resolve, which is the same as telling them nothing.
#
# CUTOVER IS FAIL-LOUD. A repository still holding a non-empty legacy `.fullcycle-active` is
# refused outright rather than read alongside the new store. Reading both would leave no answer
# to "which one is authoritative", and reading only the new one would silently un-gate every
# document still recorded in the old file.
#
# ONE BLOCK PER TURN (deliberate, documented weakening): Claude Code sets `stop_hook_active`
# on a Stop it already forced by a previous block, and overrides the hook entirely after eight
# consecutive blocks without progress (raisable via CLAUDE_CODE_STOP_HOOK_BLOCK_CAP). Blocking
# a continuation again cannot say anything the first block did not — the reason is already in
# the transcript, in front of the user — but it does spend another full-context turn, and after
# eight of them the gate is bypassed anyway. Worse, a blocked turn can never end, so the harness
# path that re-invokes the agent when a BACKGROUND command finishes is unreachable: waiting on a
# 15-25 minute Codex round degenerates into repeated one-line status turns. So the gate states
# the incomplete work exactly once per user turn, then lets the turn end. What this gives up is
# real: an agent told once may simply stop, and nothing here forces it back. The tripwire was
# always self-attested (a ticked box is a claim, not proof); this trades a coercion the platform
# caps at eight anyway for a session that can actually wait on its own review.
LC_ALL=C
export LC_ALL

# Blocking must not depend on jq: if the only way to emit a verdict is a tool that might be
# missing, then a missing tool silently OPENS a gate whose whole purpose is to be fail-closed.
# The fallback message is deliberately free of quotes and backslashes so it needs no escaping.
block() {
  # jq being PRESENT is not the same as jq WORKING. A jq that exits non-zero printed nothing,
  # which the harness reads as "no decision" — the gate opens. Verify the emission succeeded
  # and produced output; otherwise fall back to a static block string that needs no tool.
  # Status 0 and nonempty output was not enough. A jq that exits 0 while emitting `not-json`
  # satisfied both and got printed as the verdict; the harness needs valid JSON carrying
  # decision:"block", so that path OPENED the gate while looking like it had blocked. Re-parse
  # what we are about to print and check the field that actually matters. If jq is broken enough
  # to fail this, the same brokenness fails the re-parse, and the static string below runs.
  if command -v jq >/dev/null 2>&1; then
    out="$(jq -n --arg r "$1" '{decision:"block",reason:$r}' 2>/dev/null)" && [ -n "$out" ] \
      && printf '%s' "$out" | jq -e '.decision == "block" and (.reason|type) == "string"' >/dev/null 2>&1 && {
      printf '%s\n' "$out"; exit 0
    }
  fi
  printf '{"decision":"block","reason":"full-cycle gate: could not read its own state or format a verdict (jq missing or failing). Refusing to open. Fix jq, then retry."}\n'
  exit 0
}
command -v jq >/dev/null 2>&1 || block ''

# Consume stdin before anything else so the writer is never left blocked. Strict boolean
# identity, not truthiness: the string "true", 1, or a non-empty object must NOT open the gate.
# Anything else (bad JSON, empty stdin, field absent) falls through to full enforcement —
# uncertainty blocks.
cont="$(jq -r 'if .stop_hook_active == true then "y" else "n" end' 2>/dev/null)" || cont=""
if [ "$cont" = "y" ]; then exit 0; fi     # STATUS checked: jq can print "y" and then fail

# STATE IS ANCHORED AT THE REPOSITORY ROOT, NOT AT THE INVOCATION CWD. `dstack` resolves the
# root with git and writes there; a hook that resolved `.dstack/active` relative to wherever it
# happened to be invoked would find nothing from any subdirectory and open the gate on active
# work. The two must agree, so this resolves the root exactly the way `dstack` does. Outside a
# repository there can be no store (dstack refuses to run there), so there is nothing to gate;
# a git that is missing or a root we cannot enter is uncertainty, and uncertainty blocks.
command -v git >/dev/null 2>&1 || block "full-cycle gate: git is unavailable, so the repository root that anchors gate state cannot be resolved. Refusing to open."
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  root="$(git rev-parse --show-toplevel 2>/dev/null)" || root=""
  [ -n "$root" ] || block "full-cycle gate: inside a git work tree but its root could not be resolved. Refusing to open."
else
  # Status 128 is NOT proof that no repository exists: a broken GIT_DIR, an unreadable object
  # store, or a permission problem returns 128 too, and reading that as "nothing to gate" opens
  # the gate inside a live checkout (verified: `GIT_DIR=/definitely/missing git rev-parse
  # --is-inside-work-tree` returns 128 from within this repository). Prove absence independently
  # before believing it — walk up looking for repository metadata.
  st=$?; [ "$st" -eq 128 ] || block "full-cycle gate: git failed with status $st while locating the repository root. Refusing to open."
  # PHYSICALLY resolved, not `$PWD`. From a logical path like `/tmp/repo-link/sub` the walk
  # examined `/tmp/repo-link`, `/tmp`, `/` — and never saw `/real/repo/.git`, so a fatal git
  # failure inside a real checkout still exited 0.
  # Explicit git environment is itself evidence a repository was intended here. With `GIT_DIR`
  # or `GIT_WORK_TREE` naming external metadata there is no in-tree `.git` for the walk to find,
  # so a broken `GIT_DIR` produced 128, the walk found nothing, and the gate opened inside a live
  # worktree — the very failure the walk was added to close, just through a different door.
  for v in "${GIT_DIR:-}" "${GIT_WORK_TREE:-}" "${GIT_COMMON_DIR:-}"; do
    [ -n "$v" ] && block "full-cycle gate: git failed with status 128 while GIT_DIR/GIT_WORK_TREE/GIT_COMMON_DIR is set — the repository is unreachable, not absent. Refusing to open."
  done
  d="$(pwd -P)" || block "full-cycle gate: cannot resolve the working directory physically, so no repository could be ruled out. Refusing to open."
  case "$d" in /*) : ;; *) block "full-cycle gate: physical working directory '$d' is not absolute. Refusing to open." ;; esac
  while :; do
    if [ -e "$d/.git" ] || [ -L "$d/.git" ]; then
      block "full-cycle gate: git reports no repository, but '$d/.git' exists — the repository is unreadable, not absent. Refusing to open."
    fi
    # A `.dstack/` store is this pipeline's own evidence that work is registered here. A worktree
    # with external metadata has no `.git` to find, but it does have this.
    if [ -e "$d/.dstack" ] || [ -L "$d/.dstack" ]; then
      block "full-cycle gate: git reports no repository, but the state store '$d/.dstack' exists — this is a working repository whose git metadata is unreachable. Refusing to open."
    fi
    [ "$d" = "/" ] && break
    nd="$(dirname -- "$d")" \
      || block "full-cycle gate: could not walk up from '$d' to prove no repository exists. Refusing to open."
    [ -n "$nd" ] || block "full-cycle gate: dirname returned nothing while walking up from '$d'. Refusing to open."
    # Non-progress that is not `/` means the walk cannot reach the root, so absence was never
    # proven. The old `break` here fell through to `exit 0` — the fallback failing OPEN, which
    # is the same defect it was added to fix.
    [ "$nd" = "$d" ] && block "full-cycle gate: walking up from '$d' stopped making progress before reaching '/'. Cannot prove no repository exists. Refusing to open."
    d="$nd"
  done
  exit 0
fi
cd -- "$root" 2>/dev/null || block "full-cycle gate: cannot enter the repository root that anchors gate state. Refusing to open."

legacy=".fullcycle-active"
active=".dstack/active"

# Symlink checks are per-component, not just on the top directory: `.dstack` being a real
# directory says nothing about `.dstack/active`, and either one — or the legacy file — can
# redirect reads outside the repository.
for p in .dstack "$active" "$legacy"; do
  [ -L "$p" ] && block "full-cycle gate: '$p' is a symlink. Gate state must not be reachable through one. Refusing to open."
done
[ -e .dstack ] && [ ! -d .dstack ] && block "full-cycle gate: '.dstack' exists and is not a directory. Refusing to open."

# `-s` is false for a FIFO, a device node, or a directory, so a non-regular thing occupying the
# legacy name read as "absent" here while `dstack status` refused the same state through
# `require_plain`. Two tools disagreeing about the registry namespace is the class this cutover
# check exists to prevent.
if [ -e "$legacy" ] && [ ! -f "$legacy" ]; then
  block "full-cycle gate: '$legacy' exists and is not a regular file, so the legacy registry cannot be read or migrated. Refusing to open."
fi
if [ -s "$legacy" ]; then
  block "full-cycle gate refusing to run: a non-empty legacy $legacy is still present alongside the .dstack/ store, so there is no single authoritative registry. Run \"$HOME/.claude/bin/dstack\" migrate (it refuses anything it cannot carry over losslessly), then retry."
fi
# No store at all: nothing was ever registered here, and there is nothing to validate.
if [ ! -e .dstack ] && [ ! -L .dstack ]; then exit 0; fi

# A store EXISTS, so its schema marker is a prerequisite for reading it — and that check has to
# happen BEFORE the "no active/ means nothing is registered" shortcut. It did not: a store with
# `version: 2` and no `active/` made this hook exit 0 while every `dstack` mutation refused the
# same store, so the writer said "unsupported schema" and the gate said "all clear".
[ -L .dstack/version ] && block "full-cycle gate: '.dstack/version' is a symlink. Refusing to open."
if [ -f .dstack/version ]; then
  # Both producers checked; a failed `wc` or `cat` must not read as "some other value" and fall
  # into the same branch as a genuinely wrong marker.
  vn="$(wc -c < .dstack/version 2>/dev/null)" || block "full-cycle gate: cannot size '.dstack/version'. Refusing to open."
  vn="$(printf '%s' "$vn" | tr -d ' ')"
  vr="$(cat -- .dstack/version 2>/dev/null)" || block "full-cycle gate: cannot read '.dstack/version'. Refusing to open."
  { [ "$vn" = "2" ] && [ "$vr" = "1" ]; } || block "full-cycle gate: '.dstack/version' is not the schema marker this hook understands (expects exactly '1' plus a newline). Refusing to open on a store it cannot interpret."
else
  block "full-cycle gate: '.dstack' exists but '.dstack/version' does not. Refusing to open on a store with no schema marker."
fi

if [ -e "$active" ] || [ -L "$active" ]; then
  [ -d "$active" ] || block "full-cycle gate: '$active' exists but is not a directory. Refusing to open on a malformed store."
else
  exit 0                                  # store present and readable, nothing registered in it
fi

# This session's identity. An EMPTY value already meant "unknown ⇒ enforce everything", but a
# malformed one did not: any nonempty `sid` took part in the foreign-owner comparison, so a value
# like `bad/slash` — which `dstack` would never write and never accepts — was simply "not equal
# to" every valid owner, and the gate skipped every record. Fail-closed has to cover our OWN
# identity, not just the stored one: a session id we cannot trust is the same as not knowing it.
sid="${CLAUDE_CODE_SESSION_ID:-}"
case "$sid" in
  '') : ;;
  *[!A-Za-z0-9_-]*) sid="" ;;          # unusable identity ⇒ treat as unknown ⇒ enforce all records
esac

# The record key is a SHA-1 of the lowercased document path, so verifying a record's filename
# against its own content needs a digest tool. Missing it is uncertainty, and uncertainty blocks.
if   command -v shasum  >/dev/null 2>&1; then SHA1TOOL='shasum -a 1'
elif command -v sha1sum >/dev/null 2>&1; then SHA1TOOL='sha1sum'
else block "full-cycle gate: neither shasum nor sha1sum is available, so record keys cannot be verified. Refusing to open."
fi

# A glob over an UNREADABLE directory expands to the literal pattern; every `-e` below then
# fails and the loop body never runs, so a permission problem in the registry looks exactly like
# an empty registry and the gate opens. `-d` earlier proved the type, not readability. Prove the
# traversal itself succeeded before letting "found nothing" mean "nothing is registered".
ls -1a -- "$active" >/dev/null 2>&1 \
  || block "full-cycle gate: the record directory '$active' exists but cannot be listed, so an empty scan proves nothing. Refusing to open."

goals=(); tasks=(); seen=""; bad=""
# Hidden entries are enumerated too: globbing only `*` left anything dot-prefixed invisible,
# and an invisible entry in a namespace this gate trusts is exactly what must not be possible.
for rec in "$active"/* "$active"/.[!.]* "$active"/..?*; do
  [ -e "$rec" ] || [ -L "$rec" ] || continue        # unmatched glob, or vanished mid-scan
  base="${rec##*/}"
  case "$base" in .tmp.*|*.lock) continue ;; esac   # transient by contract, not registrations
  # -L FIRST: a dangling symlink fails -f, so testing -f first skipped it silently.
  [ -L "$rec" ] && { bad="$bad $base(symlink)"; continue; }
  [ -f "$rec" ] || { bad="$bad $base(not-a-regular-file)"; continue; }
  # The filename IS the key. Anything else in this namespace was not written by `dstack`.
  case "$base" in
    [0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]) : ;;
    *) bad="$bad $base(not-a-record-name)"; continue ;;
  esac
  # ONE read of the file; every field then comes from those same bytes. Reading the path twice
  # could observe two generations of a record that a concurrent reclaim replaced in between,
  # pairing one session's owner with another session's document. Schema is validated rather
  # than assumed: a record whose version or field types we do not recognise is reported, never
  # partially believed. Fields are extracted separately on purpose — packing them onto one line
  # would need a delimiter no document path can contain, and there is no such byte available
  # here (command substitution strips NUL).
  # `cat`'s status matters: an unreadable record used to yield an empty `raw`, which the schema
  # predicate then rejected — the right verdict by accident, for the wrong reason, and it would
  # have reported "off-schema" for what is really "could not be read".
  raw="$(cat -- "$rec" 2>/dev/null)" || { bad="$bad $base(unreadable)"; continue; }
  if ! printf '%s' "$raw" | jq -e '(.v|type)=="number" and .v==1 and (.session|type)=="string" and (.doc|type)=="string" and (.doc|length)>0 and (.ts|type)=="string"' >/dev/null 2>&1; then
    # Present but unreadable or off-schema. Surfaced, never skipped — see the header note.
    # `ts` is part of the published record, so validating three of four fields called a record
    # `dstack` would never write well-formed.
    [ -f "$rec" ] && bad="$bad ${rec##*/}"
    continue
  fi
  # Extraction STATUS is checked. A failed jq here used to leave an empty `doc`, which the
  # docs/-prefix test then skipped silently.
  owner="$(printf '%s' "$raw" | jq -r '.session')" || { bad="$bad $base(owner-unreadable)"; continue; }
  doc="$(printf '%s' "$raw" | jq -r '.doc')"       || { bad="$bad $base(doc-unreadable)"; continue; }
  # Skip a record only when it is provably someone else's: it has an owner, we know our own id,
  # and they differ. Everything else (ours, or unattributable — no owner / empty id) falls
  # through and is enforced. (An owner no live session holds — a record stranded by `/clear`
  # rotating the id — is an orphan by construction: nobody enforces it. That is the accepted
  # orphan semantics, NOT the fail-closed path, which covers only *unattributable* records.)
  # The owner must satisfy the same grammar `dstack` enforces. Without this, a nonempty but
  # malformed owner is "not equal to mine" and gets skipped by EVERY session — unattributable
  # state that nobody enforces, which is the opposite of fail-closed.
  # `dstack` never writes an empty owner, so one here is a corrupt record, not an
  # "unattributable but otherwise fine" one. It used to fall through to enforcement, which
  # happened to block — but it also meant the hook and the CLI disagreed about whether the
  # record was valid. Both call it invalid now; `bad` still blocks, so nothing is weakened.
  case "$owner" in
    ''|*[!A-Za-z0-9_-]*) bad="$bad $base(bad-owner)"; continue ;;
  esac
  # STRUCTURE IS CHECKED FOR EVERY RECORD, before ownership is consulted. A record whose filename
  # is not the key of its own `doc` is addressable by nothing: `dstack unreg` cannot find it, so
  # nobody can release it, and its stated owner cannot be trusted to mean anything either. It is
  # not another session's business — it is a broken store, and a broken store blocks.
  # (This is the same invariant `read_record` enforces in `dstack`; the two are deliberately
  # separate implementations so the tripwire never depends on the CLI being installed. Change one,
  # change both.)
  # Digest status separately from `cut`: a pipeline reports its LAST command's status, so a
  # digest tool that printed 40 hex and then failed was accepted as authoritative.
  # Every producer's status, not just the last one in the pipeline. A failing `tr` followed by a
  # successful `shasum` returns 0 and the digest of an EMPTY stream
  # (da39a3ee5e6b4b0d3255bfef95601890afd80709) — one constant every record would "match".
  kfold="$(printf '%s' "$doc" | tr '[:upper:]' '[:lower:]')" \
    || { bad="$bad $base(key-could-not-be-computed)"; continue; }
  [ -n "$kfold" ] || { bad="$bad $base(key-could-not-be-computed)"; continue; }
  kraw="$(printf '%s' "$kfold" | $SHA1TOOL 2>/dev/null)" \
    || { bad="$bad $base(key-could-not-be-computed)"; continue; }
  key="${kraw%% *}"
  if [ "$key" != "$base" ]; then bad="$bad $base(key-does-not-match-its-own-doc)"; continue; fi
  case "$doc" in
    docs/*) : ;;
    # Not a silent skip any more. `dstack` refuses to register a non-docs/ path, so one sitting
    # here means the store was written by something else — and a record the gate ignores while
    # looking registered is precisely the "ungated but looks gated" state this replaces.
    *) bad="$bad $base(doc-not-under-docs/)"; continue ;;
  esac
  # IDENTITY, not a prefix. `docs/*` plus "the final component is a real file" accepted
  # `docs/../../outside/GOAL.md`, and `section` would then open and parse a file outside the
  # repository — this hook runs in EVERY repo, so that is a global read primitive. It also
  # accepted `docs/x/../real/GOAL.md`, which `dstack unreg` canonicalises to a different key and
  # therefore cannot release. Resolve the parent physically and require the result to be exactly
  # `$root/$doc`; that one comparison covers `..`, `.`, symlinked parents, and escaping the repo.
  case "/$doc/" in
    */../*|*/./*) bad="$bad $base(doc-path-has-a-dot-component)"; continue ;;
  esac
  ddir="$(cd -- "$(dirname -- "$doc")" 2>/dev/null && pwd -P)" || ddir=""
  if [ -z "$ddir" ] || [ "$ddir/${doc##*/}" != "$root/$doc" ]; then
    bad="$bad $base(doc-is-not-its-own-canonical-path)"; continue
  fi
  # The parent is resolved physically, but the FINAL COMPONENT was only appended — and on
  # case-insensitive APFS `-f docs/goal.md` succeeds when the real file is `docs/GOAL.md`. So a
  # record spelled `goal.md` passed here while `dstack` resolves and stores `GOAL.md`: the two
  # tools disagreeing about one file, and the wrongly-spelled one classified as a task rather
  # than the Goal. Recover the real spelling from the directory listing and require an exact
  # match. A listing failure blocks; it is not "no match".
  # `-a`: this listing and `canon`'s in `dstack` must enumerate the same set, or a dot-prefixed
  # document is stored by one tool and rejected by the other.
  dnames="$(cd -- "$ddir" 2>/dev/null && ls -1a 2>/dev/null)" \
    || { bad="$bad $base(doc-directory-unreadable)"; continue; }
  printf '%s\n' "$dnames" | grep -qxF -- "${doc##*/}" \
    || { bad="$bad $base(doc-spelling-is-not-the-on-disk-one)"; continue; }
  # The document invariant runs BEFORE ownership filtering, for the same reason the key check
  # does: a record whose document is missing, symlinked, or unrepresentable is a broken store,
  # and a broken store blocks whoever notices it. Filtering first meant a foreign-owned record
  # for a deleted document was skipped here and called invalid by `dstack status` — the two
  # tools disagreeing about the same bytes, which is exactly what the shared invariant exists
  # to prevent.
  if [ -L "$doc" ]; then bad="$bad $base(doc-is-a-symlink)"; continue; fi
  if [ ! -f "$doc" ]; then bad="$bad $base(doc-missing:$doc)"; continue; fi
  # Same representability rule `canon` enforces: a control or non-ASCII byte means the CLI
  # would refuse to derive this record's identity, so the hook must not accept it either.
  case "$doc" in *[!\ -~]*) bad="$bad $base(doc-has-a-control-or-non-ascii-byte)"; continue ;; esac
  if [ -n "$sid" ] && [ "$owner" != "$sid" ]; then continue; fi
  # Dedupe on the KEY (the record filename), not on a string-delimited set of document paths:
  # an accepted path may contain the delimiter sequence and mask a genuinely distinct document.
  case " $seen " in *" $base "*) continue ;; esac
  seen="$seen $base"
  # Case-insensitive: `dstack` now stores the on-disk spelling, but a store written by an
  # older build (or by hand) can still carry `goal.md`, and misfiling the Goal as a task
  # silently drops the one-Goal rule.
  # Match the BASENAME exactly, not a `*goal.md` suffix over the whole path: the old pattern also
  # classified `docs/x/notgoal.md` and `docs/x/my-goal.md` as Goals, so a task-shaped document
  # could satisfy the one-Goal structural count by itself.
  case "$(printf '%s' "${doc##*/}" | tr '[:upper:]' '[:lower:]')" in
    goal.md) goals+=("$doc") ;;
    *)       tasks+=("$doc") ;;
  esac
done

# Body of a "## <heading…>" section (prefix match), up to the next "## " line. Deliberately
# NOT fence-aware: stripping fenced blocks (to spare an example checkbox) proved net-negative —
# an unbalanced/lone ``` then strips real gate rows to EOF. A worked example inside a gate
# section instead just over-blocks (safe, recoverable); real gates live outside code fences.
# The frozen heading must match as a WHOLE TOKEN, not as a bare prefix. `index($0,h)==1` also
# matched `## Gate status-old` and `## Goal gate-archived`, so a renamed or typo'd heading still
# had its checkbox rows read as if the required section were present. But a plain equality test is
# too strict the other way and broke the live gate: the pipeline's own template writes
# `## Goal gate (Stop-hook enforced — …)`, and requiring exact equality made a real, correct
# GOAL.md report "no '## Goal gate' section". The rule that satisfies both: the heading must be
# followed by end-of-line or WHITESPACE. A parenthetical after a space is the documented form; a
# `-archived` glued straight on is a different heading.
# Exact heading, or the documented parenthetical form — nothing else. An earlier repair for the
# opposite bug (exact-only stopped matching `## Goal gate (Stop-hook enforced — …)` and broke the
# live gate) swung too far and accepted any whitespace-delimited suffix, so `## Gate status
# archived` satisfied the required section while the frozen heading was absent.
section() { awk -v h="## $2" '$0==h || index($0, h " (")==1 {f=1;next} /^ {0,3}#{1,6}([ \t]|$)/{f=0} f' "$1"; }

CONSENSUS_FIELD_RE='^[-[:space:]>#*+._)0-9]*(✅|❌)?[[:space:]]*consensus:'
CONSENSUS_SEALED_RE='^[-[:space:]>#*+._)0-9]*((✅|❌)[[:space:]]*)?consensus:[*_[:space:]]*(disagreed|agreed|resolved)[[:punct:][:space:]]*((✅|❌)[[:punct:][:space:]]*)?$'

sealed_round_ok() {
  local f="$1" count line
  count="$(grep -icE "$CONSENSUS_FIELD_RE" -- "$f" || true)"
  [ "$count" -eq 1 ] || return 1
  line="$(awk 'NF { line=$0 } END { print line }' "$f")"
  printf '%s\n' "$line" | grep -qiE "$CONSENSUS_FIELD_RE" || return 1
  printf '%s\n' "$line" | grep -qiE "$CONSENSUS_SEALED_RE"
}

# Validate the entire canonical review namespace and print its latest round. Suffixes have a
# minimum width of three digits and grow naturally after 999. Counting first and generating
# expected names gives numeric, Bash-3-compatible ordering with no arbitrary round cap. Every
# round from 001 must exist and be a sealed, nonempty, regular, nonsymlink, text file ≤64KB;
# malformed reserved names fail rather than being ignored. The legacy singleton may coexist as
# migration history, but it is never authoritative.
review_series_latest() {
  local d="$1" f base latest="" expected=1 round_count=0
  for f in "$d"/codex-review*.md; do
    [ -e "$f" ] || [ -L "$f" ] || continue
    base="${f##*/}"
    case "$base" in
      codex-review.md)
        continue
        ;;
      *)
        printf '%s\n' "$base" | grep -qE '^codex-review-[0-9]{3,}\.md$' || return 1
        round_count=$((round_count + 1))
        ;;
    esac
  done
  while [ "$expected" -le "$round_count" ]; do
    printf -v base 'codex-review-%03d.md' "$expected"
    f="$d/$base"
    [ ! -L "$f" ] && [ -f "$f" ] && [ -s "$f" ] || return 1
    [ "$(wc -c < "$f")" -le 65536 ] || return 1
    grep -Iq . -- "$f" || return 1
    sealed_round_ok "$f" || return 1
    latest="$f"
    expected=$((expected + 1))
  done
  [ -n "$latest" ] || return 1
  printf '%s\n' "$latest"
}

# A round carries a genuine positive consensus only when its one Consensus line is the final
# nonblank line. Blank lines after it are harmless; appended prose means the round is not sealed.
# Multiple such lines mean a later exchange was appended to the same document, which violates
# the one-file-per-round contract. The positive-only whitelist avoids an unwinnable negation
# lexicon and rejects trailing prose where "agreed to reject" could hide.
consensus_ok() {
  local line
  sealed_round_ok "$1" || return 1
  line="$(awk 'NF { line=$0 } END { print line }' "$1")"
  printf '%s\n' "$line" | grep -qiE '^[-[:space:]>#*+._)0-9]*(✅[[:space:]]*)?consensus:[*_[:space:]]*(agreed|resolved)[[:punct:][:space:]]*(✅[[:punct:][:space:]]*)?$'
}

p=""   # accumulated problems

# A record we could not read might be guarding anything, so it counts as a problem rather than
# as an absence. Fail-closed: an unreadable registry is not an empty one.
[ -n "$bad" ] && p="$p unreadable record(s) in $active:$bad — inspect or remove them (\"$HOME/.claude/bin/dstack\" status);"

[ "${#goals[@]}" -gt 1 ] && p="$p more than one GOAL.md is active (exactly one Goal allowed);"
{ [ "${#tasks[@]}" -gt 0 ] && [ "${#goals[@]}" -eq 0 ]; } && p="$p task(s) active without a registered GOAL.md;"

for t in "${tasks[@]}"; do
  gs="$(section "$t" 'Gate status')"
  # Schema REQUIRED (fail-closed, mirrors the Goal gate): a registered task must carry a
  # '## Gate status' section holding at least one checkbox row — a missing or prose-only
  # section cannot silently bypass task enforcement. Checkbox patterns accept any GFM list
  # marker ([-*+]), leading whitespace, and >1 space after it, so '* [ ]' / '+ [ ]' / an
  # indented / a double-spaced box cannot hide an unchecked gate.
  if [ -z "$gs" ]; then p="$p $t has no '## Gate status' section (required gate schema missing);"; continue; fi
  if ! printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[[ xX]\]'; then p="$p $t Gate status has no checkbox rows (prose cannot stand in for gates);"; continue; fi
  if printf '%s\n' "$gs" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[ \]'; then p="$p $t has unchecked task gates;"; fi
  # Codex (GPT-5.6 Sol) adversarial review is mandatory per task. Require a structurally valid
  # numbered series and a positive consensus in its latest round, independent of checkbox labels.
  d="$(dirname -- "$t")"
  if ! review="$(review_series_latest "$d")" || ! consensus_ok "$review"; then
    p="$p $t lacks a valid latest codex-review-<NNN>.md with one agreed/resolved consensus (mandatory per task);"
  fi
done

for g in "${goals[@]}"; do
  gate="$(section "$g" 'Goal gate')"
  # Schema is REQUIRED (fail-closed): a registered Goal must carry a '## Goal gate' section
  # with a final 'GOAL E2E' box — a missing/typo'd schema cannot silently bypass G4. Checkbox
  # patterns accept any GFM marker ([-*+]) + indentation, matching the task side.
  if [ -z "$gate" ]; then p="$p $g has no '## Goal gate' section (required gate schema missing);"; fi
  if ! printf '%s\n' "$gate" | grep -qiE '^[[:space:]]*[-*+][[:space:]]+\[[ xX]\] GOAL E2E'; then p="$p $g Goal gate is missing the final 'GOAL E2E' checkbox row;"; fi
  if printf '%s\n' "$gate" | grep -qE '^[[:space:]]*[-*+][[:space:]]+\[ \]'; then p="$p $g has unchecked Goal-gate boxes (milestone/Goal E2E);"; fi
  # Milestone headings match at ANY ATX level (#…######, incl. ≤3-space indent), case-insensitively,
  # normalized to upper-case 'M<n>' — so '# M1', '#### M1', '## M1', '### m1', '   ## M1' can't dodge
  # the required 'M<n> E2E' box. The trailing '([^0-9A-Za-z]|$)' is a word boundary so a topic heading
  # like '## M2M transport' or '## Milestones' is NOT misread as milestone 'M2'/'M'. (Setext headings
  # and 'M<n>' not as the first heading token are accepted blind spots — see HONEST SCOPE.)
  # Each producer's status is checked. As one `$( … | … | sort -u)` the loop saw only `sort`,
  # so a failed READ of the Goal document produced zero milestones and silently enforced no
  # milestone gate at all. grep status 1 is a legitimate "no milestone headings"; ≥2 is a read
  # error and must fail closed.
  mraw="$(grep -iE '^[[:space:]]*#{1,}[[:space:]]+M[0-9]+([^0-9A-Za-z]|$)' -- "$g")"; mst=$?
  [ "$mst" -le 1 ] || block "full-cycle gate: could not scan '$g' for milestone headings (grep status $mst), so no milestone gate could be proved. Refusing to open."
  mlist="$(printf '%s\n' "$mraw" | grep -oiE '#{1,}[[:space:]]+M[0-9]+' | grep -oiE 'M[0-9]+' | tr '[:lower:]' '[:upper:]' | sort -u)"; mst=$?
  [ "$mst" -le 1 ] || block "full-cycle gate: could not normalise the milestone headings of '$g' (status $mst). Refusing to open."
  for m in $mlist; do
    printf '%s\n' "$gate" | grep -qiE "^[[:space:]]*[-*+][[:space:]]+\[[xX]\] $m E2E" || p="$p $g milestone $m has no ticked '$m E2E' Goal-gate box;"
  done
done

[ -z "$p" ] && exit 0
block "full-cycle gate incomplete —$p  (Enforced for this session: its own owner-tagged records plus any unattributable ones; another session's records are not shown here.) Resolve these, or \"$HOME/.claude/bin/dstack\" unreg <doc> to pause that document. (Tripwire over registered docs + Codex review, not a sandbox: a ticked box is self-attested — do not fake it.)"
