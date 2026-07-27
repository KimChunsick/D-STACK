# 01-spin-fix

## Intent / Why
The `Stop` gate blocks turn-end unconditionally. Claude Code overrides a `Stop` hook only
after eight consecutive blocks without progress, so every legitimate wait on a long external
run costs up to eight turns, each re-sending the whole conversation context, and then bypasses
the gate anyway. Parsing the documented `stop_hook_active` field collapses that to one block
per turn-end attempt: the gate still says what is incomplete, once, and then lets the turn end
so the harness can re-invoke on background completion.

First task of the Goal and dependency-free on purpose — the hook is symlinked into
`~/.claude/hooks/`, so this Goal's own remaining turns benefit immediately.

## Design consult
Skipped — no trigger. Reading one documented JSON field crosses no module boundary, defines no
contract, and touches no persistence or sanitization path.

## What was done (what / why)
`fullcycle-gate.sh` now consumes its stdin JSON as its first action and exits success when
`stop_hook_active` is exactly boolean `true`, before any registry work.

Strict boolean identity rather than truthiness: `jq -r 'if .stop_hook_active == true then "y"
else "n" end'`. The obvious spelling, `jq -r '.stop_hook_active // false'`, also opens the gate
for the *string* `"true"`, and this file's standing rule is that uncertainty blocks. Bad JSON,
absent jq, an absent field, `null`, `1`, and empty stdin all fall through to full enforcement.

stdin is drained first so the hook never leaves the writer blocked, and nothing later in the
script reads stdin.

The HONEST SCOPE block gained a paragraph stating plainly what this gives up: an agent told once
may simply stop, and the hook no longer forces it back. The trade is stated rather than hidden,
because the platform caps the coercion at eight blocks anyway and then bypasses the gate, while a
turn that can never end also makes background-completion re-invocation unreachable.

## Files changed (where / why)
- `claude/hooks/fullcycle-gate.sh` — early `stop_hook_active` exit plus the HONEST SCOPE
  paragraph recording the deliberate weakening.

## Verification (direct run — repo policy: no TDD, no tests)
Ran the hook against crafted stdin fixtures with the live registry holding two documents that
have unchecked gates, so every non-continuation case must block:

| stdin | verdict | wanted |
|---|---|---|
| `{"stop_hook_active":true}` | ALLOW | ALLOW |
| `{"stop_hook_active":"true"}` (string) | block | block |
| `{"stop_hook_active":1}` | block | block |
| `{"stop_hook_active":false}` | block | block |
| `{"stop_hook_active":null}` | block | block |
| `{"session_id":"x"}` (field absent) | block | block |
| `not json` | block | block |
| empty stdin | block | block |

8/8 as intended. `bash -n` clean. `~/.claude/hooks/fullcycle-gate.sh` is a symlink to this file,
confirmed with `ls -l`, so the fix is live for the session that wrote it.
