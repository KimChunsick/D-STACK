## Carried decisions — Round 001
- Teardown guarantees are stated only for CATCHABLE termination. `SIGKILL` can orphan the launched
  round, so before relaunching a capture that has no terminal record, check `.launch/child` for a
  live pid or group — otherwise the retry pays for two concurrent rounds.
- The round file holds findings, bundle size, `## Carried decisions` and the consensus line, and NO
  maintainer response. Every place that says otherwise is a defect: three sites disagreed at once.
- The skip check iterates the ALLOWLIST with a fixed-string match per path. Scanning the bundle for
  the marker shape refuses valid bundles, because the bundle carries documents that quote the
  marker — demonstrated twice now.
- The assembler publishes skip status only inside the bundle it emits, so content it copies can
  still impersonate a marker for one of your own allowlisted paths. Closing that needs a separate
  channel from `assemble-review.sh` and is a FOLLOW-UP for that file's own review unit.
- A quoted array expansion `"${ALLOW[@]}"` is safe in both bash and zsh; the no-variables rule was
  only ever about an unquoted scalar.

Consensus: disagreed
