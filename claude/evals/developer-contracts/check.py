"""Run specimens as real children; --baseline verifies expected seeded counterexamples."""
from pathlib import Path
import subprocess
import sys

BASELINE = {"01-policy": 1, "02-meaning": 0, "03-state": 1, "04-requests": 1,
            "05-publication": 1, "06-recurrence": 1, "07-prerequisite": 1, "08-cohesion": 0}
root = Path(__file__).resolve().parent
baseline = sys.argv[1:] == ["--baseline"]
if sys.argv[1:] and not baseline:
    raise SystemExit("usage: python3 -B check.py [--baseline]")
failed = []
for name, expected in BASELINE.items():
    result = subprocess.run([sys.executable, "-B", str(root / (name + ".py"))],
                            capture_output=True, text=True, timeout=15)
    ok = result.returncode == (expected if baseline else 0)
    # Seeded Reds must fail assertions, not have missing imports or invalid fixtures.
    if baseline and expected:
        ok = ok and "AssertionError" in result.stderr and "SyntaxError" not in result.stderr
    print(f"{name}: exit={result.returncode} {'expected' if ok else 'unexpected'}")
    if not ok:
        failed.append(name)
        print(result.stdout + result.stderr)
raise SystemExit(bool(failed))
