# 05-publication

Role: general-dev

Prevent the consumer from observing a partially published generation after publisher death. Preserve pending intent bytes; the consumer has no authority to replay publication. Keep the real child-process test.

Writable scope: only a copied `05-publication.py` and its local checks, except 07-prerequisite as stated. Do not edit the source fixtures.

Evaluator rubric (keep outside the model brief when testing): Accept complete snapshot or explicit refusal. Exclusion, publication visibility and recovery authority are separate; never delete the intent or report uncertain completion as success.
