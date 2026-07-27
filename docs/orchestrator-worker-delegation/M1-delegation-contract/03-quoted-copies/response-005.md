# Maintainer response — Round 005

Out of the reviewed corpus by the codex-review contract: this file is never bundled.

**[medium] Duplicate keys collapse before `fans.size` is evaluated.** Accepted, fixed. The
reproduction is right — `safe_load` merges `worker-fanout:` declared twice in one mapping, so
`dig` can only ever return the last value and a malformed node hides behind a valid duplicate. The
check now walks the Psych AST before loading and reports duplicate mapping keys by block and name.
Controlled with `r_dupkey`, which declares the key twice inside one mapping and is caught; the
earlier `r_dup` (a second fenced block) is kept because it exercises the other path.

**[medium] Types and a token are not semantics.** Accepted, fixed, with the cost stated rather than
absorbed. Each key must now state its decision, read off the parsed field: `declaration is
COMPLETE` / `WRITE SET IS DETERMINED` / `POSITIVE ISOLATION BENEFIT` for `delegate-when`,
`BASE IDENTITY IS VERIFIED` for `requires`, `verdict of PARALLEL` for `parallel-when` (which
subsumes and replaces the old bare-token assertion), `COMMITTED-DELIVERABLE` and `sandbox` for
`honest-scope`, `OUTRANKS` and `frontend-dev` for `frontend-takes-precedence`. Controlled with
`r_shape` and `r_sem`, the reviewer's own two examples.

The cost: this pins wording, so a reword that preserves meaning fails. That is unavoidable — a
check able to distinguish "required" from "forbidden" has nothing but wording to key on. The
false-positive control was rebuilt around the new boundary rather than dropped: it now rewords
outside the pinned phrase and must stay green.

**[medium][security] The parse-validity loop still runs `load_file`.** Accepted, fixed, and the
finding is a fair correction of my round-004 answer. I converted the parser I wrote and left the
one I did not, calling it out of scope; both read the same content in the same file, so the file was
not fixed. Both passes now use `safe_load(File.read(...))`. The narrowing this causes is real and
small: a fenced block using a YAML tag or alias would now fail the parse check. None does, and for a
configuration schema that failure is the correct outcome.

**[low][DX] Uncited process statements in the bundle.** Partly fixed, partly declined with a reason.
The `load_file` deferral comment is gone, because the deferral is gone. The `AGENTS.md` citation
stays and the reviewer is right that the policy text was not supplied for verification — but adding
a file to the bundle is exactly what this pipeline's own ratchet rule forbids mid-loop ("a finding
demanding a new file in scope becomes a follow-up for a separate review unit"). Carried as a
follow-up rather than acted on here, which is the rule applying to me, not an exemption from it.
