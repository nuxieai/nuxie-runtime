# FL-C5 evidence receipt protocol

Every copied floor receipt must contain the exact production candidate SHA as
its first line:

```text
FLOOR_RECEIPT_TREE_SHA=<40-character production commit SHA>
```

After the orchestrator commits round-four production as P3, each floor writes
to a raw log outside this evidence directory. Copy and stamp it with:

```sh
python3 tools/runtime-frame-loop-port/stamp_floor_receipt.py \
  --repo-root "$PWD" \
  --source /path/to/raw-floor.log \
  --destination docs/runtime-frame-loop-fl-c5-evidence/floor3-name.log \
  --tree-sha <P3-full-SHA>
```

The wrapper validates the SHA and atomically installs the stamped copy. Do not
reuse the historical `floor2-*` results as P3 receipts. The evidence-only E2
pass must name P3, include all stamped reruns, and retain both failed attempts
and successful reruns when a floor requires a retry.

Independent review verdicts W39, W40, W45, W46, and W47 are archived beside
the floor receipts so internal closeouts cannot be confused with independent
acceptance.
