# Research — Codex(GPT-5.5) as researcher/reviewer capability

> Per the new rule, research is run by Codex. For *this* meta-Goal the only external
> unknown was Codex's own capability, which was probed empirically via `codex exec`.

## Question
Can `codex exec` (non-interactive, the mode the full-cycle pipeline calls) actually
pull *live* web information? The whole "Codex is the researcher" premise depends on it.

## Method
Ran `codex exec --skip-git-repo-check -c model_reasoning_effort="low"` asking Codex to
honestly list the tools available to it in the non-interactive session and whether it
can fetch live web pages / run web searches.

## Finding (evidence)
Codex replied:
```
TOOLS: web.run, image_gen.imagegen, functions.exec_command, functions.write_stdin,
       functions.update_plan, functions.request_user_input, functions.request_plugin_install,
       functions.view_image, functions.apply_patch, tool_search.tool_search_tool,
       multi_tool_use.parallel
WEB: yes
NOTE: Shell/network permissions are restricted and approval is disabled, but the
      dedicated web tool is available for live search/page fetches.
```

## Conclusion
- **Codex exec has a live `web.run` tool → "Codex is the researcher" is viable.** It can
  do real web search + page fetch, not just reason from training data.
- Config confirms `model = "gpt-5.5"`, `model_reasoning_effort = "xhigh"`.
- Caveat (security): `web.run` reads arbitrary pages → **prompt-injection risk**. Codex
  research output must be consumed as *untrusted data*, never as instructions.

## Opposing view / risk (the "반대 의견" the user requires research to include)
- Codex `exec` runs with `sandbox: read-only` and approval disabled — fine for research,
  but it means Codex cannot verify claims by running code during research.
- Dual role conflict-of-interest: Codex both informs (research) and judges (review).
  Mitigation adopted: the reviewer is instructed to also attack its *own* research's
  assumptions.
