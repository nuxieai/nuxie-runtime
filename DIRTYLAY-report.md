# Dirty-layout continuation report

Date: 2026-08-04  
Branch: `levi/perf-dirty-layout`  
Reference: Rive `artboard.cpp:1260-1414` at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Outcome

The precise dirty-layout behavior is now retained in the Rust runtime. Layout
style synchronization works from an exact member set, does nothing when that
set is empty, and triggers a solve only for relevant style changes. Solved
layout is retained per component instead of being reconstructed from transient
published bounds.

The interrupted `TextInput` clone test was salvageable. It was completed and
retained as `text_input_clone_rebuilds_scroll_link_and_drag_state_cold`, which
verifies that cloning rebuilds the scroll link while starting with cold drag
state.

## Port details

- `dirty_layout` is a `BTreeSet<usize>` and synchronizes only its exact members.
- Reentrant layout dirt is rejected while that exact-member style cleaning
  pass is active, matching `m_isCleaningDirtyLayouts`.
- Empty style synchronization is a no-op; layout calculation follows only a
  relevant synchronized style change.
- Each layout component retains its solved `Layout`, while the artboard retains
  solved bounds for publication and clean-frame reuse.
- Animated layout distinguishes solved artboard-space position from live
  animated width and height, including controlled parametric path invalidation.
- Host-owned transferred layout publishes descendants without incorrectly
  publishing the transferred root.
- Nested hug sizing refreshes when the recursively mounted component-list
  generation changes, and invalidates only stale hug axes.

The continuation's thin commits were:

- `d208e97c` — retain solved layout bounds per component.
- `fc2ccd21` — compare layout updates against retained solves.
- `d2f13866` — stabilize transferred nested hug sizing.
- `f1cf53cd` — gate transferred solves on host ownership.
- `52598d36` — replace provisional mounted gradient events.
- `0fc3fe67` — publish animated transferred layout bounds.
- `27bae051` — keep animated layout paths in artboard space.
- `88e044e6` — separate solved and animated layout control.
- `ca71ce06` — update the drawing ownership ledger for the extracted path
  composer so the checker follows the current source owner.
- `29049479` — reject reentrant layout dirt during the style cleaning pass and
  correct the PathComposer lifecycle citation found by final review.

## Correctness evidence

The release scripted golden comparison is byte-identical for every registered
exact entry:

```text
entries=363 exact=348 exact-segments=1132 side-channel-segments=1127
diverges=10 unsupported-feature=0 not-yet=5
```

Focused regression coverage includes empty and member-only synchronization,
cleaning-pass mutation rejection, retained solve comparison, animated
completion and live-frame path dirtiness, host-owned descendant publication,
recursive mounted-list hug generation, and the recovered `TextInput` clone
state test.

The final two-axis review found and corrected a stale PathComposer lifecycle
citation and the missing explicit cleaning-pass mutation guard. It also noted
that `0fc3fe67` was broader than the requested thin-increment shape. Repairing
that historical process deviation would require rewriting already banked
history, so it is reported here rather than rewritten.

Final gates:

- `cargo test -p nuxie-runtime` — PASS (992 principal unit tests plus integration targets).
- `cargo test -p nuxie --features scripting` — PASS after refreshing the pinned C++ probe.
- `cargo test -p silver-corpus --test runtime_frame_loop_backfill_bc` — PASS (3 passed, 1 ignored).
- `make b6-audit-check runtime-frame-loop-port-check runtime-drawing-port-check` — PASS.
- `make RUST_PROFILE=release scripted-golden-compare` — PASS with the counts above.
- `make perf-gate` — PASS for all 24 fixtures.

## Performance evidence

The measured code revision is `290494792ff3819f6f150510968df2177c6792f2`.
Both runs used the pinned C++ runtime, release scripting-enabled runners, 100
frames at 60 Hz, C++ first, and the median of five iterations with no warmups.

| Fixture | Baseline Rust ms/frame | Final Rust ms/frame | Change | Final Rust/C++ |
|---|---:|---:|---:|---:|
| `car_widgets_v01` | 1.563576 | 0.436322 | -72.09% | 20.734x |
| `zombie_skins` | 2.212570 | 0.502118 | -77.31% | 13.607x |

The checked-in raw reports are in
[`docs/evidence/dirtylay-2026-08-04/`](docs/evidence/dirtylay-2026-08-04/).
Their SHA-256 digests are:

- `baseline.json`: `7125199903d2962491c941aa087d720441141521af929839a462eb02aa0c34df`
- `final.json`: `f796385902c3d89b4ec01952b2b906b5881ebe6697d0321618d6ae59fa76f199`
- final `target/perf-gate.json`: `cd6b5c7ef5f96ca85303a1bc04ba1c25c25cac0272296b210557db9a6fce5db0`

No temporary artifacts were written under `/tmp`.
