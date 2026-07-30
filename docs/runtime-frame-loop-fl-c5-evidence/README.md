# FL-C5 evidence receipt protocol

Every copied floor receipt must contain the exact production candidate SHA as
its first line:

```text
FLOOR_RECEIPT_TREE_SHA=<40-character production commit SHA>
```

The operative E2 receipt set is:

- `floor3-apple.log`
- `floor3-browser.log`
- `floor3-pixel-same.log`
- `floor3-pixel-static.log`
- `floor3-size.log`

Each names immutable combined production candidate
`95333c41fe68ab6a2a5486874ffd0c59cd4381be` on its first line. They replace
the `floor2-*` files as publication evidence. `floor2-apple.log` remains at
the packet root to keep its disclosed failure prominent; the other five
floor2 receipts are under `superseded/`. All floor2 files are historical and
must not be cited for P3.

For future reruns, write the raw log outside this evidence directory and copy
and stamp it with:

```sh
python3 tools/runtime-frame-loop-port/stamp_floor_receipt.py \
  --repo-root "$PWD" \
  --source /path/to/raw-floor.log \
  --destination docs/runtime-frame-loop-fl-c5-evidence/floor3-name.log \
  --tree-sha <P3-full-SHA>
```

The wrapper validates the SHA and atomically installs the stamped copy.

Historical disclosure: `floor2-apple.log` ended with an attempt-1 dirty-tree
packaging refusal; `superseded/floor2-xcframework.log` is its successful clean
attempt-2. Both are retained so that failure/rerun history is not hidden. The
operative P3 `floor3-apple.log` is a separate SHA-stamped clean run whose
product checks and XCFramework packaging both pass.

Independent review verdicts W39, W40, W45, W46, and W47 are archived beside
the floor receipts so internal closeouts cannot be confused with independent
acceptance.
