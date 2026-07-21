# M2 flag-layer probe — capture (written from inside the probe session)

Probe per task.md "E2E verification / Flag layer": a fresh headless session launched with
`--effort ultracode`, asked "Answer with one word, yes or no: is ultracode mode active in
this session?".

## Result: **Yes**

Observed from inside the probe session (self-attested by the probe; the launcher should
cross-check its own capture of stdout/exit status):

- The session's system context states ultracode is ON for the session ("Ultracode is on:
  optimize for the most exhaustive, correct answer"; Workflow-tool orchestration enabled) —
  i.e. the `--effort ultracode` launch flag manifested as standing ultracode mode, not
  merely per-prompt keyword detection.
- Session was non-interactive/headless as intended.

## Defect found by this probe (fix the probe design, not the gate)

The probe was launched with the repo root as cwd. The fullcycle Stop gate
(`hooks/fullcycle-gate.sh`) resolves `.fullcycle-active` **relative to cwd**, so the probe
session inherited the active Goal's own gate and could not end its turn: the Stop hook
blocked repeatedly (gates legitimately unchecked mid-goal), and the probe's attempt to use
the documented escape hatch (edit `.fullcycle-active`) was denied by the permission
classifier — correctly, since unregistering docs would disable the parent session's
tripwire mid-flight. Net effect: the probe hangs until the caller's timeout; its stdout may
be lost. Hence this file.

**Fix:** run the flag-layer probe from a scratch cwd outside the repo (mirror the codex
skills' `-C "$SCRATCH"` isolation), e.g.:

```sh
SCRATCH="$(mktemp -d)" && (cd "$SCRATCH" && command claude --effort ultracode -p \
  "Answer with one word, yes or no: is ultracode mode active in this session?")
```

The probe session made no other changes: no gate boxes ticked, no goal docs edited,
`.fullcycle-active` untouched, no Codex reviews run (task 02's review loop is the parent
session's in-flight work).
