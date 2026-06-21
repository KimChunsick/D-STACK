# gemini/ — placeholder

Extensibility stub for Google Gemini CLI config. No authored Gemini config exists
yet (`~/.gemini` is not present on this machine).

## To onboard Gemini later

1. Put only **authored** Gemini artifacts here (e.g. `GEMINI.md`, settings) — no
   secrets, same rules as `AGENTS.md` §Golden rules.
2. Add explicit `!`-allow lines to `.gitignore` for each file (the agent dir is
   `deny-all` by default).
3. Add a `gemini` entry to the `install.sh` map — but use **`copy` mode, not a
   symlink**: Gemini CLI intentionally **does not follow symlinked context files**
   (GH google-gemini/gemini-cli#11547, closed "not planned"). A symlinked
   `~/.gemini/GEMINI.md` is silently ignored. Re-run `install.sh` after editing.
4. Add a `tests/test_gemini_artifacts.sh` guard and keep `tests/run.sh` green.
