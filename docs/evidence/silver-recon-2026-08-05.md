# Silver-corpus reconciliation evidence — 2026-08-05

Method: iterate `validate` -> scratch-patch the failing row -> regenerate -> revalidate,
on origin/main (rev b1f91278, fresh worktree) and on branch claude/jovial-chatelet-05ab64,
both against RIVE_RUNTIME_DIR=4ac7b327. Register issue: UNIV-1638.

# Silver-corpus validation delta

| id | main-state | branch-state | same_as_main |
|---|---|---|---|
| bidirectional_precedence-target_first | now-exact: byte exact | now-exact: byte exact | true |
| bidirectional_stateful_property | now-exact: byte exact | now-exact: byte exact | true |
| clipping_and_draw_order | now-exact: byte exact | now-exact: byte exact | true |
| component_list_child_origin | note-changed: frame 1, op 448 (rewind): expected rewind, got drawPath | note-changed: frame 1, op 448 (rewind): expected rewind, got drawPath | true |
| component_list_grouped | note-changed: frame 6, op 413 (transform), field tx: expected -90, got 0 | note-changed: frame 6, op 413 (transform), field tx: expected -90, got 0 | true |
| component_stateful_vm_instance | now-exact: byte exact | now-exact: byte exact | true |
| component_stateful_vm_instance_2 | now-exact: byte exact | now-exact: byte exact | true |
| computed_values_test | note-changed: frame 2, op 191 (addRawPath), field point: expected (301.00003, -0.0 (0x80000000)), got (301, -0.0 (0x80000000)) | note-changed: frame 2, op 191 (addRawPath), field point: expected (301.00003, -0.0 (0x80000000)), got (301, -0.0 (0x80000000)) | true |
| focus_collapsing | note-changed: frame 1, op 98 (color), field paint_id: expected 4, got 11 | note-changed: frame 1, op 98 (color), field paint_id: expected 4, got 11 | true |
| focusable_element | exact-diverged: frame 1, op 144 (color): expected color, got save | exact-diverged: frame 1, op 144 (color): expected color, got save | true |
| global_viewmodels_test-auto_instance | exact-diverged: frame 0, op 27 (color), field paint_id: expected 5, got 6 | exact-diverged: frame 0, op 27 (color), field paint_id: expected 5, got 6 | true |
| global_viewmodels_test-set_instance | note-changed: frame 0, op 27 (color), field paint_id: expected 5, got 6 | note-changed: frame 0, op 27 (color), field paint_id: expected 5, got 6 | true |
| image_fit_alignment_2 | now-exact: byte exact | now-exact: byte exact | true |
| layout_anim_bound | note-changed: frame 2, op 145 (rewind): expected rewind, got drawPath | note-changed: frame 2, op 145 (rewind): expected rewind, got drawPath | true |
| layout_anim_component_list | note-changed: frame 1, op 88 (rewind): expected rewind, got drawPath | note-changed: frame 1, op 88 (rewind): expected rewind, got drawPath | true |
| layout_anim_nested | note-changed: frame 1, op 85 (rewind): expected rewind, got drawPath | note-changed: frame 1, op 85 (rewind): expected rewind, got drawPath | true |
| layout_fixed_fill | note-changed: frame 1, op 56 (rewind): expected rewind, got drawPath | note-changed: frame 1, op 56 (rewind): expected rewind, got drawPath | true |
| layout_grid_stack_grid_with_layouts_size_changing | ok | now-exact: byte exact | false |
| scroll_test | note-changed: frame 0, op 53 (transform), field xy: expected -0.0 (0x80000000), got 0 | note-changed: frame 0, op 53 (transform), field xy: expected -0.0 (0x80000000), got 0 | true |
| stateful_keyed_trigger | now-exact: byte exact | now-exact: byte exact | true |
| text_feather_falloff | note-changed: frame 0, op 29 (feather), field paint_id: expected 12, got 8 | note-changed: frame 0, op 29 (feather), field paint_id: expected 12, got 8 | true |
| text_vertical_trim_test | note-changed: frame 3, op 220 (rewind): expected rewind, got drawPath | note-changed: frame 3, op 220 (rewind): expected rewind, got drawPath | true |
| unbound_stateful_component | note-changed: frame 0, op 10 (save): expected save, got color | note-changed: frame 0, op 10 (save): expected save, got color | true |

# Silver-corpus reconciliation log

## `origin/main`

- Revision: `b1f912780a7971d65e2fa58f6f9d3c3b99a72e6d`
- Logical validation iterations: 23 (22 enumerated failures, then 1 successful validation).
- Enumerator process runs: the initial process detected iterations 1–11 and stopped while scratch-patching iteration 11; the resumed process replayed iteration 11 from the 10 applied/checkpointed rows and completed through iteration 23.
- Patch failure: `global_viewmodels_test-auto_instance` (`exact-diverged`) initially failed because the id already had a dormant `DIVERGENCES` row while also appearing in `EXACT`. No partial scratch write occurred. The retry removed the `EXACT` row and replaced the dormant divergence detail with the validator-reported detail.
- Final summary: `silver-corpus summary: entries=252 provenanced=249 runtime=208 scripted=41 selected=252 executed=178 cpp-baseline-exact=252 cpp-rust-exact=82 byte-exact=62 epsilon=20 divergent=96 unsupported=30 pending=0 pending-scripted=41 diverges=96 unsupported-feature=30 provenance-unknown=3 operations=1263333 bytes=38379753`
- Final lane summary: `silver-corpus lane-summary: lane=all selected=252 byte-exact=62 epsilon=20 divergent=96 unsupported=30`
- Scratch files restored to `HEAD`; the baseline worktree `target/` was deleted.

## Branch working tree

- Branch: `claude/jovial-chatelet-05ab64`, using the current working-tree source changes with the two silver files restored to `HEAD` before enumeration.
- Logical validation iterations: 24 (23 enumerated failures, then 1 successful validation).
- Patch failures: none.
- Final summary: `silver-corpus summary: entries=252 provenanced=249 runtime=208 scripted=41 selected=252 executed=178 cpp-baseline-exact=252 cpp-rust-exact=83 byte-exact=63 epsilon=20 divergent=95 unsupported=30 pending=0 pending-scripted=41 diverges=95 unsupported-feature=30 provenance-unknown=3 operations=1263333 bytes=38379753`
- Final lane summary: `silver-corpus lane-summary: lane=all selected=252 byte-exact=63 epsilon=20 divergent=95 unsupported=30`
- Scratch files restored to `HEAD`; the branch worktree `target/` was retained.
