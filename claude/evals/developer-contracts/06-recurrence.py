"""Previous review fixed CLI bypass; API still misses the shared refusal."""
def permitted(record):
    return record["status"] == "ready" and not record["revoked"]


def cli(record):
    return permitted(record)


def api(record):
    return record["status"] == "ready"


if __name__ == "__main__":
    for status in ["ready", "pending"]:
        for revoked in [False, True]:
            record = dict(status=status, revoked=revoked)
            assert cli(record) == api(record) == (status == "ready" and not revoked)
