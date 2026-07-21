# Codex adversarial review — Round 008

## Review scope
Re-review

## GPT findings
[severity:medium][security] The backup-suffix repair omits exact protected basenames, allowing their backup copies into the index.
Evidence: `COMP_RE` anchors `auth.json`, `.netrc`, `id_rsa`, `history.jsonl`, and `config.toml` to the complete component. Suffix handling was added only to extension-shaped families. After `git add -f`, paths such as `claude/skills/full-cycle/auth.json.bak`, `config.toml.old`, or `.netrc.backup` are invisible to section 5 and do not match section 4. The recorded backup scenarios test only PEM/token/database families.
Suggested direction: At the `COMP_RE` policy boundary, define backup semantics for every protected exact basename and mirror them in `.gitignore` and the probe battery. Handle intentional exceptions such as public-key `.pub` files explicitly.
Illustrative example:
```text
index: .../auth.json.bak
exact matcher: ^auth\.json$
section 4: no match
section 5: already indexed
result: PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Test ordinary and force-added `.bak`, `.old`, and `.backup-*` variants of every exact protected basename outside fixed probe paths; each protected variant must fail.

[severity:medium][security] `scan_components` is NUL-safe only between pathnames; its locale- and newline-oriented `grep` predicate can treat protected raw-byte components as safe.
Evidence: Each raw component is serialized with `printf '%s\n'` and processed by line-oriented `grep -iE`. Under the current `C.UTF-8` locale, `printf 'password\377\n' | grep -qEi 'password'` returns status 1, permitting a force-added component containing `password` plus an invalid UTF-8 byte. A component such as `private.pem.<LF>backup` is split into two grep records and misses `\.(pem|...)(\..+)?$`, although `*.pem.*` matches that filename. `private.pem.` is another mismatch because Git’s final `*` may be empty while the regex requires `.+`. Existing newline scenarios place the newline in a directory component separate from the protected name, so they do not cover this failure.
Suggested direction: Match each complete component as bytes under an explicit `LC_ALL=C` boundary without adding a line delimiter, using a Bash predicate or another length-aware mechanism. Make zero-length wildcard behavior agree with the corresponding Git patterns.
Illustrative example:
```text
component bytes: private.pem.<LF>backup
.gitignore glob: *.pem.*  -> match
grep records:    "private.pem." + "backup" -> neither matches
forced index:    guard PASS
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Create protected components containing embedded newlines and invalid UTF-8 bytes, plus empty wildcard tails such as `private.pem.`; force-add them and require rejection under `C`, `C.UTF-8`, and available UTF-8 locales.

[severity:low][security] Failure diagnostics emit untrusted pathname bytes directly to the terminal.
Evidence: `fail` uses `echo`, and multiple callers interpolate raw indexed or untracked pathnames and components. A protected filename containing an OSC or other terminal-control sequence is matched and then emitted unchanged, allowing display manipulation and, in supporting terminals, clipboard-oriented control sequences.
Suggested direction: Escape control and non-printable bytes at the diagnostic-output boundary while retaining the original bytes for policy matching.
Illustrative example:
```text
indexed component: password<ESC><terminal-control-payload>
scan: match -> fail -> raw terminal bytes emitted
```
Reviewer caveat: This illustrative example is only the reviewer's opinion, not a patch to copy verbatim. Adapt it to the actual codebase and verify the result.
Verification: Capture diagnostics for filenames containing ESC, BEL, carriage return, and newline bytes; assert that output contains escaped representations and no executable terminal-control bytes.

GPT verdict: reject — The sole retained guard still has deterministic indexed-path bypasses for protected backup and raw-byte filename variants.

## Maintainer response
1. **Agreed (exact-basename backup chains) — fixed, policy and matcher together.**
   Every exact protected basename now tolerates a dot-suffix chain in BOTH places:
   `.gitignore` gained `**/auth.json.*`, `**/.netrc.*`, `**/id_rsa.*` (+dsa/ecdsa/
   ed25519), `**/history.jsonl.*`, `**/config.toml.*`, `**/.DS_Store.*`; `COMP_RE`
   mirrors with `(\..*)?$` tails. The `.pub` question was decided the strict way:
   this repo never tracks key material of any kind, so `id_rsa.pub` is denied too
   (deliberate, recorded — an exception would be a policy change to propose to the
   user). Verified (scenario Y): `auth.json.bak`, `config.toml.old`,
   `.netrc.backup`, `id_rsa.pub`, `history.jsonl.gz` — each proven ignored, then
   force-added — all fail; ignore battery extended with the same names.
2. **Agreed (locale/newline/empty-tail matching) — fixed at the matcher mechanism.**
   The guard now runs under `LC_ALL=C` (byte semantics; invalid UTF-8 cannot evade),
   and `scan_components` matches each WHOLE component with bash `[[ =~ ]]` under
   `nocasematch` — no line-oriented serialization, so an embedded newline stays one
   component. Wildcard-tail semantics now agree with git's empty-`*`: family tails
   are `(\..*)?$` / `([.-].*)?$`. Verified (scenario Z): newline-inside-component
   `private.pem.\nbackup` and empty-tail `private.pem.` force-added both fail; the
   invalid-UTF-8 component is verified at the matcher level (APFS refuses such
   filenames on disk — the on-disk fixture is impossible on this machine, real on
   Linux clones; the matcher-level check covers the mechanism).
3. **Agreed (raw diagnostics) — fixed.** Every discovered-pathname diagnostic
   (component scans, nested-ignore lists, addable-agents list) is now `printf %q`-
   escaped at the output boundary; matching still uses the original bytes.

Fixes not yet independently reviewed — sealing for re-review.

## Carried decisions
- Key material including public halves (`id_rsa.pub` etc.) is denied — strict-side
  default, revisitable only as a user policy decision.
- All prior dispositions unchanged.

Consensus: disagreed
