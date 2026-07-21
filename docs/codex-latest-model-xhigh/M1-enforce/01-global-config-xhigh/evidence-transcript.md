# Sanitized evidence transcript — T01 global config xhigh

Generated live on 2026-07-10 by re-running each command and piping through
a home-path/username redaction sed (no other edits).
The raw ~/.codex/config.toml itself is excluded by the repo secret-deny model.

```
$ codex --version
codex-cli 0.144.0

$ grep -n "^model_reasoning_effort" ~/.codex/config.toml
1:model_reasoning_effort = "xhigh"

$ grep -n "^\[" ~/.codex/config.toml   # every table header, line-numbered
3:[projects."~"]

$ grep -c "^model_reasoning_effort" ~/.codex/config.toml   # occurrence count
1

$ wc -l -c ~/.codex/config.toml && ls -l ~/.codex/config.toml ~/.codex/config.toml.bak.*
       4      82 ~/.codex/config.toml
-rw-------  1 <owner>  82  7월 10 07:06 ~/.codex/config.toml
-rw-------  1 <owner>  48  7월 10 07:06 ~/.codex/config.toml.bak.20260710070640

$ codex exec --ephemeral "Reply with exactly: ok"   # bare default, repo cwd; then echo EXIT=$?
Reading additional input from stdin...
OpenAI Codex v0.144.0
--------
workdir: ~/Desktop/Workspace/D-STACK
model: gpt-5.5
provider: openai
approval: never
sandbox: read-only
reasoning effort: xhigh
reasoning summaries: none
session id: 019f48ff-3479-7e32-8748-b7012d540f8b
--------
user
Reply with exactly: ok
codex
ok
tokens used
4,116
ok
EXIT=0
```
