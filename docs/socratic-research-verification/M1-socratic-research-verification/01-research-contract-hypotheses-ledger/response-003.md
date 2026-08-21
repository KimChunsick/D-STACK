# Maintainer response — Round 003

- F11 (medium): Accept. The refusal condition now keys on "lacks an encoding for ANY of
  the three", covering partial schemas; carried blocks are encoded, missing ones flagged
  individually.
- F12 (low): Accept. Task record updated to the current contract shape.

Verification after fixes: `bash tests/secret-guard.sh` → PASS.
