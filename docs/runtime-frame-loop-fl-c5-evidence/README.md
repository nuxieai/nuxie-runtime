# FL-C5 evidence receipt protocol

Every copied floor receipt must contain the exact production candidate SHA as
its first line:

```text
FLOOR_RECEIPT_TREE_SHA=<40-character production commit SHA>
```

The operative E3 receipt set is:

- `floor4-apple.log`
- `floor4-browser.log`
- `floor4-pixel-same.log`
- `floor4-pixel-static.log`
- `floor4-size.log`

Each names immutable combined production candidate
`691c5262fb990e73bf13187ea923fe54341c25bc` on its first line. They replace
the `floor3-*` files as publication evidence. The complete floor3 set is
preserved under `superseded/`; it is historical E2 evidence and must not be
cited for the E3 candidate. `floor2-apple.log` remains at the packet root to
keep its disclosed failure prominent; the other five floor2 receipts remain
under `superseded/`. All floor2 files are historical and must not be cited
for E3.

For future reruns, write the raw log outside this evidence directory and copy
and stamp it with:

```sh
python3 tools/runtime-frame-loop-port/stamp_floor_receipt.py \
  --repo-root "$PWD" \
  --source /path/to/raw-floor.log \
  --destination docs/runtime-frame-loop-fl-c5-evidence/floor4-name.log \
  --tree-sha <P4-full-SHA>
```

The wrapper validates the SHA and atomically installs the stamped copy.

Historical disclosure: `floor2-apple.log` ended with an attempt-1 dirty-tree
packaging refusal; `superseded/floor2-xcframework.log` is its successful clean
attempt-2. Both are retained so that failure/rerun history is not hidden. The
operative P3 `floor3-apple.log` is a separate SHA-stamped clean run whose
product checks and XCFramework packaging both pass; it is now preserved under
`superseded/`. The operative E3 `floor4-apple.log` is another clean,
SHA-stamped run whose product checks and XCFramework packaging pass.

Independent rejection verdicts W39, W40, W45, W46, W47, W50, W51, and W52
are archived beside the floor receipts so internal closeouts cannot be
confused with independent acceptance. `W53-report.md` is the round-five
corrective handoff that maps the W50/W51/W52 findings to production,
differentials, structural negatives, and the pre-E3 acceptance run. It is a
corrective report, not an independent acceptance verdict.
