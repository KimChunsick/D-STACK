# Maintainer response — Round 006 (reopened unit)

Not bundled. **Verdict was `approve-with-fixes`** — the reopened live-CWD deletion is addressed and
everything remaining is low and non-blocking.

## F023 [low] cleanup leaks on trapped signals and when the session id is absent — AGREED in part

The leak on a signalled exit is deliberate now: the trap is gated on `<run-dir>/exit`, so it keeps
the directory whenever quiescence is unknown. That is the safer direction and it is what the
reopening was for. The `$CLAUDE_CODE_SESSION_ID` expansion under `set -u` is a real ordering nit and
the residual prose describing the old unconditional deletion was stale; both rewritten.

## F024 [low] the source-count regex — AGREED, narrowed again

`https://-` counted as 1 and `<https://example.com>` deduplicated separately from the bare URL.
Narrowed further. It matters because zero sources is a documented fallback trigger.

## F025 [low] "the current block was run end-to-end" — AGREED

The reopened fence has no recorded run. That is why this unit's E2E box came off when it was
reopened, and it goes back on only after a run of the current form.

## F026 [low][security] disposition language — AGREED

"Covering SIGPROF … are changes to `claude/bin/dstack`" pre-assigns where a defect gets reviewed,
inside the reviewed payload. Moved to `findings.md`.
