"""Read-only shared store dependency; editing this specimen is outside trial scope."""
class SharedStore:
    def snapshot(self):
        # Both adapter_a and adapter_b call this public boundary.
        return ("new", "old")


def adapter_a(store):
    return store.snapshot()


def adapter_b(store):
    return store.snapshot()


if __name__ == "__main__":
    assert adapter_a(SharedStore()) in [("old", "old"), ("new", "new")]
