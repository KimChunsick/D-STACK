# 03-state

Role: frontend-dev

Receiving new server rows must preserve the current query and update displayed results. Keep original server facts and UI input separate; inspect derived-state ownership.

Writable scope: only a copied `03-state.py` and its local checks, except 07-prerequisite as stated. Do not edit the source fixtures.

Evaluator rubric (keep outside the model brief when testing): Prefer derived display instead of syncing two independent authoritative copies. Python is a state model: browser hooks, focus and accessibility remain unverified.
