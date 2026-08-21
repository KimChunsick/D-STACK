## Audit of hypotheses

### H1

Claim: Project X's test suite pass rate is 80%.

Grouped materials: ledger L1; recorded executable-check result: `5 passed, 5 failed, 10 total`.

Probes and answers:

Definition: What does “pass rate” mean in the ledger?
Grounding: data reading. The ledger defines unit `tests`, denominator `10`, transformation `passed/total`, and value `5/10 = 80%`. Under that stated definition, pass rate is `passed / total`.

Data reading: What does the recorded result compute to?
Grounding: recomputation. Recorded input says `5 passed, 5 failed, 10 total`. Arithmetic: `5 / 10 = 0.5 = 50%`, not 80%.

Assumption: What must be true for 80% to hold?
Grounding: formal reasoning over artifact text. With denominator `10`, an 80% pass rate would require `8 passed`. The recorded result says `5 passed`, so the claim cannot hold under the artifact’s own denominator.

Data-check reconciliation: L1 is refuted. Its formula label is correct, but its computed value is wrong: `5/10 = 50%`.

Verdict: refuted. The recorded executable output and ledger denominator directly refute the claimed 80% pass rate.

### H2

Claim: The Python walrus operator (`:=`) was introduced in Python 3.9.

Grouped materials: deferred check D1: count the number of lines in Project X's README file.

Probes and answers:

Definition: What feature is being claimed?
Grounding: external empirical source. Python’s official documentation defines assignment expressions using `:=` and says they were “Added in version 3.8” (Python docs, no page date, retrieved 2026-08-21).

Evidence: What does the strongest primary source say about the Python version?
Grounding: external empirical source. PEP 572, the standards-track PEP for assignment expressions, lists `Python-Version: 3.8` (created 2018-02-28, retrieved 2026-08-21).

Counterexample: What published source would falsify Python 3.9?
Grounding: external empirical source. Both PEP 572 and the Python language reference identify Python 3.8, not 3.9, as the version where assignment expressions were added.

Data/deferred-check bearing: What does counting Project X README lines establish about Python language history?
Grounding: formal reasoning. The deferred check D1 concerns a local README line count. It has no evidentiary bearing on when Python introduced assignment expressions. The artifact’s link from D1 to H2 is unsupported.

Data-check reconciliation: D1 is unresolved but irrelevant to H2, so it does not cap the verdict. The external primary sources refute H2.

Verdict: refuted. Official Python sources identify assignment expressions / the walrus operator as introduced in Python 3.8, not Python 3.9.

## Audit of findings

No decision-relevant non-H findings were present in the artifact.

## Audit of data checks

### L1

Claim fed: H1.

Data-reading probes:

Dataset/version: The only recorded executable input is `5 passed, 5 failed, 10 total`; no broader Project X run context is provided.

Unit and denominator: The ledger states unit `tests` and denominator `10`, matching the recorded total of `10`.

Transformation: The ledger states `passed/total`. Recomputed value is `5 / 10 = 0.5 = 50%`.

Outcome: refuted. L1’s arithmetic result is wrong; it reports `5/10 = 80%` when the correct value is 50%.

### Recorded executable-check result for H1

Input: `5 passed, 5 failed, 10 total`.

Data-reading outcome: upheld as internally consistent with itself: `5 + 5 = 10`, and pass rate under `passed/total` is `5/10 = 50%`.

Feeds: H1.

### D1

Claim linked by artifact: H2.

Bearing outcome: no bearing. Counting Project X README lines cannot establish or refute the Python version that introduced `:=`.

Feeds: none in substance; artifact links it to H2, but that linkage is not evidentiary.

## New deferred checks

none

## Verdict summary

| id | verdict | one-line grounds | unresolved checks |
|---|---|---|---|
| H1 | refuted | Recorded result says 5 passed out of 10, which is 50%, not 80%. | none |
| H2 | refuted | Official Python sources say assignment expressions / `:=` were added in Python 3.8, not 3.9. | D1 pending but irrelevant |
| L1 | refuted | `5/10 = 50%`, so the ledger’s `80%` computation is wrong. | none |

## Unverified

The current real test pass rate of Project X is unverified because no live Project X repository or fresh test run was provided. This does not affect the refutation of the artifact’s stated computation from its own recorded result.

D1’s README line count is unresolved, but it has no bearing on H2.

## Sources

- Python Enhancement Proposals, “PEP 572 – Assignment Expressions,” created 2018-02-28, retrieved 2026-08-21: https://peps.python.org/pep-0572/
- Python documentation, “6.12. Assignment expressions,” no page date, retrieved 2026-08-21: https://docs.python.org/3/reference/expressions.html#assignment-expressions