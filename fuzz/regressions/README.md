# Fuzz regressions

Committed reproducers for findings from the fuzz targets.

## Layout

- `fuzz/regressions/<target>/` — reproducers for **fixed** bugs. `make
  fuzz-regressions` replays these with `-runs=0` (just execute the inputs) and
  must exit cleanly. Add a reproducer here in the same commit that lands the
  fix.
- `fuzz/regressions/open/` — reproducers for **known-open** findings that still
  crash or hang. These are archived here so the exact input is not lost, but
  they are **not** replayed by `make fuzz-smoke` or `make fuzz-regressions`
  (that would wedge the gate). Move a file up into
  `fuzz/regressions/<target>/` when its finding is fixed.

## FIXED: read-fonts 0.41 panic on a VALID zero-contour glyph with padding

Upstream regression googlefonts/fontations#1962, fixed by fontations#1965
(upstream commit `26503f2a0a24f9d0504d7ec6cb0fc4cba0e68a58`). A legal TrueType
simple glyph may declare zero contours while its `loca`-allocated range keeps
trailing alignment bytes. `read-fonts` 0.41.0 `read_points_fast` treated that
padding as flag data and wrote `flags[0]` into an empty point buffer —
`glyf.rs:244: index out of bounds: the len is 0 but the index is 0` — reached
from `StaticTextSlice::render_data` -> `OutlineGlyph::draw_unhinted` ->
`load_simple`. The pinned C++ runtime accepts the same glyph and produces an
empty outline, so the font must stay accepted; the repair is the dependency
upgrade `skrifa` 0.44.0 -> 0.45.1 (`read-fonts` 0.42.1 on the outline path),
plus distilled font-fixture unit tests in `crates/nuxie-runtime/src/text.rs`
(`embedded_font_validation_accepts_empty_glyph_with_padding`).

Reproducers (replayed by `make fuzz-regressions` under `fuzz_runtime/`):

- `fuzz_runtime-panic-readfonts-empty-glyph-padding-component-list.riv`
  (806,803 bytes, SHA-256
  `e165da6113e8d604f1c73085f154a32a31cf9e0a6432d667d354a8036cc54fa1`)
- `fuzz_runtime-panic-readfonts-empty-glyph-padding-data-binding.riv`
  (833,949 bytes, SHA-256
  `0f6a27d5246a588236825a2e28bfba581ee09380c8b0315757b2ee05b6b4c6a1`)

Provenance: the original libFuzzer artifacts (SHA-256
`33025ff6407e479bfb45a47bf8d58fbbcbc23a32785f059bf79d5e92c07b9eb7`, 833,949
bytes, and `4312ad12551b3d6c56ed73083d3878032f780708fb83cb34fefd75d69e28fb60`,
806,803 bytes — mutations of the `component_list_2.riv` /
`data_binding_test.riv` seeds) were lost with a destroyed scratch worktree
before they could be committed. These reproducers were regenerated
deterministically from the same two seeds: every simple glyph with contours in
the embedded font is rewritten in place as a zero-contour glyph whose
remaining `loca` range is zero padding (container framing, `loca`, and all
table offsets unchanged). Verified equivalent to the originals at the pinned
pre-fix revision `93ed556b` (read-fonts 0.41.0): both panic at the identical
`glyf.rs:244` site; post-upgrade all three targets execute them cleanly, and
the pinned C++ golden runner renders both with empty outlines (exit 0).

## FIXED: retained-text assertion PANIC when setter dirt lands between advance and draw

`fuzz_pointer` finding (first seen on CI run 30750657425, a branch that only
touched manifest tooling — the crash is input-dependent, not branch-dependent).
Replaying pointer events after an advance and then drawing panicked at
`crates/nuxie-runtime/src/draw.rs` `RuntimeTextDrawOwner::retained_frame`:
"Text render styles were not rebuilt during component update". A state-machine
pointer listener / data-bind write runs a property setter
(`ArtboardInstance::set_string_property` et al.) whose
`mark_text_changed_for_local` marks the Text's retained owner dirty; nothing
rebuilds before the harness's next draw, and the FL-E5 (`5522ef85`) migration
assertion aborted.

The pinned runtime never asserts here: `TextValueRun::textChanged` ->
`Text::markShapeDirty` only publishes dirt (`text_value_run.cpp:11-14`,
`text.cpp:951-971`), `Text::buildRenderStyles` runs solely in the next update
pass, and `Text::draw` renders whatever `m_drawCommands` the previous update
built (`text.cpp:845-875`). FIX: `retained_frame` returns the stale retained
frame and the pending dirt survives for the next component update; the
scheduling is unchanged. Pinned by the unit test
`setter_dirt_between_update_and_draw_renders_the_stale_retained_text_frame`
(`crates/nuxie-runtime/src/draw.rs`), which asserts the stale replay is
byte-identical to the settled stream and that the next update rebuilds.

Reproducer (replayed by `make fuzz-regressions` under `fuzz_pointer/`):

- `fuzz_pointer-panic-text-render-styles-stale-draw.riv` (808,740 bytes,
  SHA-256
  `a8fab1dcda3f9723d6fe0d13faac34ddac7d5b795e4997f30d7fdc6938e9bfb6`) —
  `cargo fuzz tmin` minimization of the local libFuzzer artifact
  `crash-220ae79e6f1fe140be215c65da4b72eecf0a3f39` (833,950 bytes, SHA-256
  `30f3724ddde8e40837296b6e20405c71bfa5136a09df1562adf1133986a87d8f`), a
  mutation of the `data_binding_test.riv` seed. The CI artifact
  (`crash-b662aecc0aa586b7221b12c9e1423f32c572a97c`, one InsertByte from the
  same seed) was not uploaded by the workflow; the local find crashes at the
  identical assertion site.

## FIXED: unbounded chain-walk HANGs on malformed input (not panics)

One finding class, several reachable sites. In every case `read_runtime_file`
**accepted** the file and the pipeline then entered an infinite loop (a HANG,
not a panic) while walking a parent/reference chain whose links a malformed file
had made cyclic. No cycle guard bounded the walk.

Reproducers (now replayed by `make fuzz-regressions` under `fuzz_runtime/`):

- `fuzz_runtime-hang-layout-parent-cycle-min.riv` (39 bytes, minimized) and
  `fuzz_runtime-hang-layout-parent-cycle-orig.riv` (581 bytes, original) —
  hung in `crates/nuxie-runtime/src/components.rs`
  `runtime_layout_chain_has_layout_component` (and its sibling
  `runtime_constrained_layout_ancestor`), reached from
  `ArtboardInstance::from_graph_with_artboards`. The
  `while let Some(...) { local_id = parent_local; }` layout-parent walk never
  terminated on a `parentId` cycle.
- `fuzz_runtime-hang-drawrules-refchain.riv` (297 bytes) — hung in
  `crates/nuxie-graph/src/lib.rs` `flattened_draw_rules_local` /
  `runtime_object_for_local` / `object_parent_id`, reached from
  `GraphFile::from_runtime_file` during draw-order computation. A cyclic
  draw-rule / object reference chain looped forever.
- `fuzz_pointer-hang-layout-control-parent-cycle.riv` (1,415 bytes, exact
  libFuzzer artifact, SHA-1
  `1e8c0e3e96e386ba46d941499975778cdf67adbf`) — hung in
  `crates/nuxie-runtime/src/draw.rs`
  `runtime_layout_control_size_for_path`, reached while preparing path
  geometry after pointer-event replay. Its component-parent chain contains the
  cycle `Shape 1 -> CubicDetachedVertex 53 -> PointsPath 2 -> Shape 1`. The
  topology guards terminated earlier parent-chain queries, but this later
  draw-phase layout-control walk had omitted the same visited-id guard.

C++ parity: the reference `nuxie-runtime` **also hangs** on the original input
(confirmed with the C++ golden runner — it spins in `Artboard::initialize` ->
`Path::onAddedClean`'s unbounded shape-parent walk). `Component::validate` only
checks that a parent resolves to a `ContainerComponent`; it does not reject
parent cycles.

FIX (coordinator decision 2026-07-09, v2-status item 27): affected
parent/reference walks carry a visited-id set, mirroring C++'s own cycle-guard idiom
(`DependencySorter::visit`'s `m_Perm`/`m_Temp` visited sets,
`src/dependency_sorter.cpp`; cf. `Artboard::validateObjects`'s bounded
`for (int cycle = 0; cycle < 100; cycle++)`, `src/artboard.cpp`). This is a
DELIBERATE divergence: where C++ hangs, we terminate the walk gracefully
(treated as no-ancestor / no-rule). It is unreachable on any valid file, so
golden-compare is unchanged (263/584). See the code comments at each guard site.

RECURRENCE (2026-08-02): the FL-E5 landing (`52411269`) ported
`Node::markLayoutNodeDirty`'s layout-parent walk into
`crates/nuxie-runtime/src/layout/layout_node_provider.rs`
`mark_layout_node_dirty` without the visited-id guard. A local timed
`make fuzz-smoke` mutation run rediscovered the class within 20s: a
`parentId` cycle hangs the walk, reached from
`ArtboardInstance::set_string_property` ->
`text_owner::mark_shape_dirty_with_layout` during the data-bind advance.
Fixed with the same visited-set idiom. Reproducer (replayed by
`make fuzz-regressions` under `fuzz_runtime/`):
`fuzz_runtime-hang-layout-node-provider-parent-cycle.riv` (854,113 bytes,
exact libFuzzer artifact `timeout-203a89819bfe1088eba52793113d446af9225546`,
SHA-256 `bbc6fcb8047af24df147def96e70c334ae6f964db90cf6a1247e8a8e30f1691a`).

RECURRENCE (2026-07-31): the FL-E3 landing (`93ed556b`) re-ported
`Path::onAddedClean`'s shape-parent walk into
`crates/nuxie-runtime/src/artboard.rs`
`ArtboardInstance::build_component_occurrence_relations` without the visited-id
guard, so `fuzz_runtime-hang-layout-parent-cycle-orig.riv` hung again (all
`sample` frames inside that walk). Fixed by adding the guard there plus two
sibling FL-E walks found by audit: the IK FK-chain walk in the same function
(a bone parent cycle re-registered a peer constraint and tripped the
uniqueness assert — a panic, same malformed-cycle class) and the
Shape-parent walk in `crates/nuxie-runtime/src/shapes/parametric_path.rs`
`property_changed`. A 694-byte mutation-found sibling of the orig input
(SHA-1 `9cbbb28260aff66f649225862c816dae8163a981`) hung identically pre-fix;
it was not preserved, and post-fix `make fuzz-smoke` runs clean.

Reproduce (any file above), from repo root:

    cd fuzz && rustup run nightly cargo fuzz run fuzz_runtime \
        regressions/fuzz_runtime/fuzz_runtime-hang-layout-parent-cycle-min.riv \
        -- -runs=0 -timeout=10

Post-fix this exits cleanly instead of timing out.
