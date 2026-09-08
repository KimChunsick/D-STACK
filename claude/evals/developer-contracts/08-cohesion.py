"""One cohesive algorithm; extend negative-input validation without a framework."""
def runs(values):
    result = []
    for value in values:
        if result and result[-1][0] == value:
            result[-1] = (value, result[-1][1] + 1)
        else:
            result.append((value, 1))
    return result


if __name__ == "__main__":
    assert runs([]) == []
    assert runs([1, 1, 2, 1]) == [(1, 2), (2, 1), (1, 1)]
