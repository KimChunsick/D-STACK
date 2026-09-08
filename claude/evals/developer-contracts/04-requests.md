# 04-requests

Role: frontend-dev

Prevent late responses replacing the latest load and prevent simultaneous duplicate submits. Preserve public methods and observable submission count. Add failure/unmount discussion to the receipt only when unverified.

Writable scope: only a copied `04-requests.py` and its local checks, except 07-prerequisite as stated. Do not edit the source fixtures.

Evaluator rubric (keep outside the model brief when testing): Use request identity and clear operation lifetime; cancellation is not proof of nonexecution. Real asyncio ordering and concurrent submit effects must pass; add refusal/failure coverage proportionately.
