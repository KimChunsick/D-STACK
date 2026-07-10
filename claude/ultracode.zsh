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
# Installed by install.sh as ~/.claude/ultracode.zsh; sourced from ~/.zshrc:
#   [ -f "$HOME/.claude/ultracode.zsh" ] && source "$HOME/.claude/ultracode.zsh"
# Note: the alias wraps EVERY `claude` invocation in interactive shells (subcommands
# and diagnostics included — the flag is inert where no session starts) and shadows
# any pre-existing `claude` alias. Escape hatch: `command claude …` bypasses it.
alias claude='claude --effort ultracode'
