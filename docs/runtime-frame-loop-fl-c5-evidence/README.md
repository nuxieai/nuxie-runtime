# FL-C5 evidence receipt protocol

Every copied floor receipt must contain the exact production candidate SHA as
its first line:

```text
FLOOR_RECEIPT_TREE_SHA=<40-character production commit SHA>
```

The operative E5 receipt set is:

- `floor7-browser.log`
- `floor7-pixel-same.log`
- `floor7-pixel-static.log`
- `floor7-size.log`

Each names immutable combined production candidate
`171b57033845cd5ce20222dd24604c8c2b27d120` on its first line. They replace
the `floor5-*` files as publication evidence. The complete floor5 set is
preserved under `superseded/`; it is historical E4 evidence and must not be
cited for the E5 candidate. All earlier floor generations are historical.

Upstream boundary commit `afe71e30` on 2026-07-30 removed
`nux-apple-runtime` and Apple packaging to establish a pure engine boundary.
E5 therefore has no Apple/XCFramework acceptance leg. Every historical Apple
receipt, including the original dirty-tree refusal and later clean packaging
runs, remains under `superseded/` as evidence of the coverage that applied
before that boundary. The size floor remains operative.

For future reruns, write the raw log outside this evidence directory and copy
and stamp it with:

```sh
python3 tools/runtime-frame-loop-port/stamp_floor_receipt.py \
  --repo-root "$PWD" \
  --source /path/to/raw-floor.log \
  --destination docs/runtime-frame-loop-fl-c5-evidence/floor7-name.log \
  --tree-sha <E5-production-full-SHA>
```

The wrapper validates the SHA and atomically installs the stamped copy.

Historical disclosure: `superseded/floor2-apple.log` ended with an attempt-1
dirty-tree packaging refusal;
`superseded/floor2-xcframework.log` is its successful clean attempt-2.
`superseded/floor-apple.log`, `floor3-apple.log`, `floor4-apple.log`, and
`floor5-apple.log` preserve the other historical Apple runs. None is
operative after `afe71e30`.

Independent rejection verdicts W39, W40, W45, W46, W47, W50, W51, W52,
W55, W56, and W57 are archived beside the floor receipts so internal
closeouts cannot be confused with independent acceptance. `W53-report.md`
is the round-five corrective handoff for W50/W51/W52. `W58-report.md` is the
round-six corrective handoff that maps the W55/W56/W57 findings to
production, differentials, structural negatives, and the pre-E4 acceptance
run. Corrective reports are not independent acceptance verdicts.

`W66-prereview.md` and `W68-reclear.md` preserve the two pre-freeze scout
verdicts. `W67-report.md` records the round-seven corrective, and
`W69-report.md` records the final alias-resistant ratchet correction. These
scout reports explain the path to the merged E5 candidate; they are not
post-publication independent acceptance verdicts.
