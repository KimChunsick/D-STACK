# 01-global-config-xhigh

## Intent / Why
Make xhigh the **default reasoning effort for every Codex run on this machine that does
not explicitly set one**, via the global `~/.codex/config.toml`. Explicit `-c` pins keep
precedence by design — the skills (T02) pin the same xhigh value, so both layers agree:
unpinned calls inherit xhigh instead of silently running at `none`; pinned calls enforce
it. (This is the achievable reading of "무조건 xhigh": no call can silently drop below it.)

## What was done (what / why)
- Upgraded Codex CLI 0.143.0 → 0.144.0 via `brew upgrade --cask codex` — 0.143.0 rejected
  the gpt-5.6 family ("requires a newer version of Codex"); after upgrade
  `codex debug models` lists gpt-5.6-sol/terra/luna (efforts low…xhigh,max,ultra).
  **The upgrade preceded the Red capture**, so Red and Green ran on the same binary.
- Backed up `~/.codex/config.toml` to `config.toml.bak.20260710070640` (kept as the
  rollback path), then prepended `model_reasoning_effort = "xhigh"` **before** the
  `[projects]` table header (a top-level TOML key after a table header would silently
  belong to that table).
- Idempotence/placement verified (line-numbered): `grep -n '^model_reasoning_effort'`
  → `1:model_reasoning_effort = "xhigh"`; first table header `grep -n '^\['` →
  `3:[projects."/Users/won"]` — one key, top level, before any table. File: 4 lines,
  82 bytes, mode 600. TOML validity is proven by the consuming parser: Codex parses
  config.toml strictly at startup, and the bare run below exits 0 honoring the key
  (a stdlib `tomllib` cross-check was attempted; unavailable on this Python — the
  consuming parser is the authoritative one anyway). Re-application procedure (fresh
  machine): open the file in an editor — it is a handful of lines — and ensure the key
  exists exactly once, at top level (before any `[table]` header), with value `"xhigh"`.
  Then confirm positionally: `grep -n '^model_reasoning_effort' ~/.codex/config.toml`
  must print one line whose number precedes the first `grep -n '^\['` line. No blind
  string-check inserts; a hand-read of a tiny file beats a wrong automation.
- Rollback (safe under later edits): delete only the top-level key — the key sits at
  line 1, so `sed -i '' '1{/^model_reasoning_effort = "xhigh"$/d;}' ~/.codex/config.toml`
  (line-1-scoped: cannot touch a same-named key inside a later table; `-i ''` leaves no
  extra copy). **Then verify** — the sed is a silent no-op if a comment/blank/other key
  was inserted before line 1, pushing the key down: `grep -n '^model_reasoning_effort'
  ~/.codex/config.toml` must now print nothing at top level (any remaining hit must sit
  after the first `grep -n '^\['` header, i.e. inside a table); if a top-level line
  survives, the key moved — hand-edit it out. Do NOT blind-restore the `.bak` once other
  config changes may exist; the `.bak` is a disaster-recovery copy, not the rollback
  mechanism. Per-call override: `-c model_reasoning_effort=<level>` (CLI flags out-rank
  the file by design).

## Files changed (where / why)
- `~/.codex/config.toml` — add `model_reasoning_effort = "xhigh"` (machine-local; this file
  is gitignore-hard-denied and is NOT tracked by the repo — the repo carries no diff for it)
- Machine state: Codex CLI 0.143.0 → 0.144.0 (`brew upgrade --cask codex`) — required for
  the gpt-5.6 family. Rollback: `brew` casks don't version-pin; reinstalling an older cask
  is possible but unmanaged. Accepted residual: CLI version is machine state, not SSOT;
  T02's skills document the ≥0.144 prerequisite and its failure signature.

## Accepted residuals (surfaced, not hidden)
- The config file itself can never appear in a review bundle (the repo's own secret-deny
  model forbids it), so reviewers get captured transcripts + the placement checks above,
  not the raw file. The alternative — shipping config.toml to an external model — is worse.
- No in-repo regression test can watch an untracked machine-local file; the repo-enforceable
  layer is T02's command-anchored pin assertions. A fresh machine re-applies this step by
  hand (documented here); it is intentionally outside the SSOT because the file class is
  secret-bearing.
- `xhigh` is model-dependent per the config reference: a future call selecting a model
  without xhigh support could error or normalize. All models this system invokes (gpt-5.5,
  gpt-5.6 family) support xhigh; accepted residual for hypothetical other models.
- A CLI header proves what the client selected, not what the backend billed/executed —
  no externally observable evidence can close that gap; the header is the CLI's contract.
- Scope / precedence: this is a per-user default on a single-user machine (`~/.codex`);
  other UNIX users or an alternate `CODEX_HOME` are out of scope by definition of the ask.
  Higher-precedence layers out-rank the global default by design — explicit `-c`, a
  `--profile` carrying its own effort, a project-level `.codex/config.toml`, or managed
  config — so "no silent drop" holds against the *absence* of a lower-effort override, not
  against a deliberately-set one. None are configured here (no `[profiles]`; no project
  `.codex/config.toml` in the repo tree — verified), so today every unpinned call inherits
  xhigh.

## E2E verification
Sanitized live transcript of the **current state** (commands + full outputs + exit
status, home paths redacted): [evidence-transcript.md](evidence-transcript.md) —
generated at review round 3. It evidences the Green/current state only; the Red→Green
comparison below is **summarized historical evidence** (header excerpts + session ids) —
re-running Red would require reverting live machine config, which we decline to do.
Same command both times (only the config key toggled; same binary v0.144.0):
`codex exec --skip-git-repo-check --ephemeral -s read-only -C "$(mktemp -d)" "Reply with exactly: ok"`
- BEFORE (Red), session 019f48ea-f3f9-70a2-af27-322e301411e3, header:
  `OpenAI Codex v0.144.0 … model: gpt-5.5 … reasoning effort: none`
- AFTER (Green), session 019f48eb-4a79-7982-a3a2-cd5b9b9fe37b, header:
  `OpenAI Codex v0.144.0 … model: gpt-5.5 … reasoning effort: xhigh`
- Repo-cwd check (review round 2, addressing the empty-tmpdir objection): bare run from
  the repository working directory — `codex exec --ephemeral "Reply with exactly: ok"` —
  header `workdir: /Users/won/Desktop/Workspace/D-STACK … model: gpt-5.5 … reasoning
  effort: xhigh`, reply `ok`, **EXIT=0** (session 019f48f9-b2fc-7403-b063-e657a177e236).
  Full record mirrored in GOAL.md's M1 E2E section.

## Gate status
- [x] Verification: behavioral Red→Green probe complete (in lieu of TDD — no repeatable
  in-repo test can watch untracked machine state; residuals surfaced above)
- [x] Codex (GPT-5.6 Sol) adversarial review consensus (4 rounds → approve-with-fixes,
  final fix applied; see codex-review.md, `Consensus: resolved`)
- [x] E2E capture verified (sanitized live transcript + repo-cwd bare run EXIT=0 at
  xhigh; mirrored in GOAL.md M1 E2E record)
