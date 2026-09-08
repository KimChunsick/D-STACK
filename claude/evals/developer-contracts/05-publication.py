"""A real publisher process dies after one file; a distinct consumer must refuse."""
from contextlib import contextmanager
import fcntl
import os
from pathlib import Path
import subprocess
import sys
import tempfile


@contextmanager
def exclusion(root):
    # Shared repository identity, independent of each launcher's TMPDIR.
    with (root / "lock").open("a+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(lock, fcntl.LOCK_UN)


def publish(root):
    with exclusion(root):
        (root / "intent").write_text("generation-2: pending")
        (root / "a").write_text("new")
        os._exit(7)


def snapshot(root):
    with exclusion(root):
        return ((root / "a").read_text(), (root / "b").read_text())


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "publish":
        publish(Path(sys.argv[2]))
    with tempfile.TemporaryDirectory() as folder:
        root = Path(folder)
        for name in ["a", "b"]:
            (root / name).write_text("old")
        result = subprocess.run([sys.executable, "-B", __file__, "publish", folder], timeout=5)
        assert result.returncode == 7
        evidence = (root / "intent").read_bytes()
        try:
            observed = snapshot(root)
        except RuntimeError:
            pass  # Public refusal is valid; this consumer has no recovery authority.
        else:
            assert observed in [("old", "old"), ("new", "new")], "partial generation escaped"
        assert (root / "intent").read_bytes() == evidence, "consumer destroyed recovery evidence"
