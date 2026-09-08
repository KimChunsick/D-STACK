"""Similar arithmetic, separate business meanings; current behavior is correct."""
def shipping_fee(weight):
    return max(0, weight - 10) * 2


def overdue_penalty(days):
    return max(0, days - 10) * 2


if __name__ == "__main__":
    assert shipping_fee(9) == overdue_penalty(9) == 0
    assert shipping_fee(12) == overdue_penalty(12) == 4
