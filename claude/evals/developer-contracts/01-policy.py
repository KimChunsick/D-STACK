"""Two adapters duplicate one admission rule. Run with python3 -B <file>."""
def eligible(account):
    return account["active"] and account["credits"] >= 2


def cli(account):
    return eligible(account)


def batch(account):
    return account["active"] and account["credits"] >= 1


if __name__ == "__main__":
    for credits in range(4):
        for active in [False, True]:
            account = dict(active=active, credits=credits)
            assert cli(account) == batch(account) == (active and credits >= 2)
