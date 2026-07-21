# Launcher-side capture — flag-layer probe (external observer)

Requested by review round 1 ("capture an externally observed fresh launch"). This is the
LAUNCHER's record — written by the session that spawned the probe, from its own captured
stdout/exit status, not by the probe session (contrast: [evidence-probe.md](evidence-probe.md)
is the inside-the-session view from the earlier, repo-cwd probe).

Date: 2026-07-10. Claude Code v2.1.205.

Command (scratch cwd per evidence-probe.md's prescribed fix; `command` bypasses any alias
so this tests the flag itself):

```
$ SCRATCH="$(mktemp -d)" && cd "$SCRATCH" && command claude --effort ultracode -p \
    "Answer with one word, yes or no: is ultracode mode active in this session?"
Yes
EXIT=0
```

Captured stdout was exactly `Yes`; exit status 0; no usage/flag error (an unsupported
flag exits non-zero with a usage message — deterministic external signal that the CLI
accepted and parsed `--effort ultracode`).

## Honest limits (surfaced)
- "Yes" is the probe model's self-report. The corroborating, non-self-attested signals:
  (a) flag accepted by the CLI parser (exit 0, no usage error); (b) the earlier probe's
  session context verbatim stated standing ultracode mode (evidence-probe.md); (c) the
  official doc's defined semantics for `--effort ultracode`
  (code.claude.com/docs/en/model-config, retrieved 2026-07-10).
- This exercises the FLAG, not the installed alias + `~/.zshrc` hook — those don't exist
  until the maintainer's manual activation (repo policy), which is exactly the M2 E2E
  Goal-gate check (fresh terminal after activation).
