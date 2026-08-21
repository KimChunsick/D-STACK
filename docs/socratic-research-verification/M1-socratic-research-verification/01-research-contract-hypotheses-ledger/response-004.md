# Maintainer response — Round 004

- F13 (medium): Accept. The no-channel case now has a defined terminal behavior:
  first-line refusal before artifact generation. Every format now lands in exactly one
  of: encode all three; encode some + flag the rest; refuse.
- F14 (low): Accept. Both residual stale passages updated.

Verification after fixes: `bash tests/secret-guard.sh` → PASS.
