# Claude Code: ultracode on by default for every interactive launch.
#
# Ultracode (xhigh reasoning + automatic dynamic-workflow orchestration) is
# session-scoped upstream: the persisted effortLevel setting and
# CLAUDE_CODE_EFFORT_LEVEL reject it, and a bare "ultracode": true in
# settings.json is silently ignored (anthropics/claude-code#64817). A launch-time
# opt-in is the only durable route (this flag, `--settings '{"ultracode": true}'`,
# or a wrapper function); this repo uses the flag via alias.
# Requires claude >= 2.1.203 (older CLIs reject `--effort ultracode`).
#
# Installed by install.sh as ~/.claude/ultracode.zsh; the ~/.zshrc source hook
#   [ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"
# is appended idempotently by install.sh itself (it was a manual step once, and a
# zshrc rewrite silently dropped it — re-run ./install.sh after any zshrc churn).
# Note: the alias wraps EVERY `claude` invocation in interactive shells (subcommands
# and diagnostics included — the flag is inert where no session starts) and shadows
# any pre-existing `claude` alias. Escape hatch: `command claude …` bypasses it.
# Caveat: CLAUDE_CODE_EFFORT_LEVEL outranks the flag — any value other than `xhigh`
# leaves ultracode's workflow orchestration inactive even with this alias. Leave the
# env var unset (or set it to xhigh) for the default to actually take effect.
# Further prerequisites (documented limits, not enforced here): the session model
# must support xhigh effort, and workflows must not be disabled (a user/managed
# "disableWorkflows" setting removes ultracode's orchestration even when the alias
# passes the flag).
alias claude='claude --effort ultracode'
