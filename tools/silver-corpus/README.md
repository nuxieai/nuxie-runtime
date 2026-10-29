# Silver corpus

`silver-corpus` is the read-only adoption floor for upstream's SRIV v1
serialized-render corpus. It never rewrites a baseline.

The checked-in `silver-corpus.toml` is generated from active literal upstream
`silver.matches(...)`/`serializer()->matches(...)` calls after C++ comments are
removed. Six dynamically named layout-scroll cases and six `gridStackSilver`
helper cases are explicit generator data. `interpolator.sriv` and
`multitouch_debug.sriv` are orphaned public artifacts with unknown producers.
`stateful_list_props.sriv` has a known historical producer deliberately retired
upstream; it is classified separately and is not an executable obligation.

The runner validates all 282 streams structurally. For the 231 runtime cases,
the generator translates the replayable portion of each pinned C++ test body
into a shared action stream. Validation imports the recorded `.riv`, selects
the requested artboard/state machine/animation, applies those actions to the
Rust runtime, serializes the Rust render operations as SRIV v1, and compares
them operation by operation with the upstream baseline.

Every runtime row must be classified. Replayable differences are retained as
`diverges` findings with their first divergent operation in the note. Bodies
that cannot yet be represented or require an unported subsystem are
`unsupported-feature` with the blocker named. The 48 scripted producers now
have executable Luau replays in this runner. None is a Wasm guest producer;
Wasm/AssemblyScript/AnimaScript execution remains outside product scope.
Two baselines remain `provenance-unknown`; one is `retired-upstream`. The
exact-ID ledger and `min_cpp_rust_exact` ratchet prevent exact cases from being
silently downgraded. Reviewed CSS layout differences retain their recorded obligations and full-stream
checks. A CSS replacement in
`reviewed_layout_divergences` must remain an executable runtime or scripted layout
divergence, retain its C++ first-difference value, and match the SHA-256 of
the full approved Rust stream. The hash covers every frame and every
operation, so a later change cannot hide behind the first mismatch. Direct
integration assertions use this same closed ledger. Missing, unlisted,
duplicate, unsupported, changed or unhashed replacements fail.
Floor credit is given only after the first difference and full stream match.
Actual C++ equality is reported separately from reviewed CSS coverage. The
original SRIV baselines and epsilons stay intact.

Run:

```sh
make silver-corpus
cargo run -p silver-corpus -- compare expected.sriv actual.sriv
```

Use `--id <id>` to replay one manifest entry. An optional
`--rust-output-dir` also probes pre-generated `<id>.sriv` files operation by
operation. Classified divergences keep the suite green but are printed
prominently; an exact entry that changes fails.

## Geometry audit (2026-10-27)

Compared the translated test setup and source against upstream
`a203f5355840bbc22c4544fef9edc411c85f3c03`, preserving the original SRIV streams
and comparison tolerances. The governing differences are documented in
[`CSS_LAYOUT.md`](../../docs/CSS_LAYOUT.md), sections 2.2 and 5.

- `layout_grid_stack_grid_with_layout_participants`: intentional CSS automatic
  minima. Cell19 retains Shape22's natural height 221.80002, so the first row
  grows from 190 to 221.80002 and cell65 moves from y310 to y341.80002.
  Participant29 also retains the shape's natural width 151.38115. At frame44,
  its cell is 134.72658 wide: C++ scales the shape to 0.8899825, while the CSS
  minimum keeps its scale at 1. A temporary diagnostic changed only cell19's
  minimum height and participant29's minimum width to zero; **all 121 frames
  then matched the unchanged upstream stream**. Neither override belongs in
  production. The old missing-text-support classification was incorrect;
  the shared manifest now contains the complete replay as a layout divergence.
- `inventory_demo_test_v2`: the first difference is intentional CSS Fill sizing.
  Three Fill columns share 1640 points equally because their authored Auto
  flex bases map to CSS 0%. The middle 320-point item is centred at global
  x800. Upstream uses content bases 515, 320 and 430, then shares the remaining
  375 equally: column widths 640, 445 and 555 place that item at
  `140 + 640 + (445 - 320) / 2 = 842.5`. A temporary basis-only control reproduced
  those sizes and position. The October 29 full-replay audit additionally
  traced the later vertical offset to CSS shrinking: at frame 233, header
  object 9's authored height 173 shrinks to its 171.72614 content minimum
  when the content below grows. Object 35 consequently starts at y256.72614
  rather than y258. Upstream disables shrinking for fixed main-axis sizes;
  our documented translation deliberately retains `flex-shrink: 1`.
  Early clip-path rebuilds in the content-basis diagnostic also follow
  actual size changes: the nested ItemGrid's fixed-width scrollbar sibling
  shrinks below 12, expanding its adjacent clip from width 422. These are
  owner-level explanations. In a temporary control layered on content bases,
  pinning the header minimum to 173 removes its vertical offset; additionally
  pinning the scrollbar minimum to 12 removes exactly the three early clip
  rebuilds (12 operations). The first 233 of 340 frames then match. The remaining
  107 frames were subsequently isolated to automatic minimums at three nested
  boundaries. Setting only outer body21's minimum height to zero restores its
  height from 1523 to 789 and the hosted MenuGrid root from 1403 to 669, but
  MenuGrid child33 still overflows and puts footer37 at y1403. Setting child33's
  minimum height to zero restores its height to 637 and the footer to y669;
  the nested ItemGrid child3 still retains its 1371-point content minimum.
  Giving that child an explicit zero minimum restores its height to 637.
  With these three minimum controls layered on the header and scrollbar
  controls and restoring Auto Fill bases throughout the nested layout tree,
  **all 340 frames and all 602072 operations match the
  unchanged upstream stream**. Original actions, pointer coordinates, timing
  and comparison tolerances are retained. An additional ItemGrid child4
  minimum control makes no difference and is unnecessary in that combination;
  the complete set is sufficient, not a claim that every control is necessary.
  This isolates the full replay's differences to the documented CSS sizing
  contract without finding a separate scripting or runtime-port defect.
  All controls were removed; production retains CSS Fill bases, shrinking
  and automatic minima, and upstream equality remains intentionally ignored.

- `layoutstest_8-planets-grid`: the first pane boundary is an intentional
  combination of CSS Fill bases and automatic minima. In the 4377-point root,
  equal zero bases would give 2188.5 each, but the right pane's eight 284-point
  cells keep its automatic minimum at 2272, leaving 2105 on the left. C++ starts
  from content bases 284 and 2272 and shares 1821 free points equally, producing
  1194.5 and 3182.5. A temporary basis-only control restored exactly those pane
  sizes while retaining the automatic minima. A separate minimum-only control
  produced 2188.5 each. The content-basis control leaves 903 fewer
  `rewind`/`addRawPath` pairs across 164 frames. These are now located:
  paths 142/143 omit 163 each, 466/467 omit 146 each, 790/791 omit 141 each,
  and 236/560/884 omit one each. The last three upstream rebuilds change
  coordinates only within the existing comparator tolerance, and their
  effective path contents match at every draw/clip use.

  The other 900 concern local paths of Shape17/21 in three nested artboard
  instances. Their near-singular world matrices have a separately rounded
  determinant of zero, but sampled fused determinants at frame 0 are
  approximately -3.55e-29 and -1.71e-25. Upstream's unit-test producer explicitly builds with
  `--no_ffp_contract` (`tests/unit_tests/test.sh:112`); production contraction
  is documented in `docs/PORTING.md`. The shipping inverse therefore creates
  large relative coordinates and correspondingly large, source-equivalent
  cache tolerances. This accounts for the different rebuild decisions without
  identifying a missing cache invalidation. All six paths are drawn with
  default fill paints, never clipped or stroked. Paths 142/143 contain only
  Move/Close throughout. The other four briefly expand, then retain local
  contours after their world transforms collapse. Across every use where only
  Rust retains drawing segments, transformed control-point bounds have zero
  extent except path 791, whose maximum extent is 0.000015258789 pixels.
  Their positions can still differ because later CSS layout differs; those
  comparisons remain failures. This is bounded geometry evidence, not GPU
  pixel validation or a rendering-equivalence claim.

  The first residual path difference is a one-ULP connector at path 111:
  adjacent rounded-corner endpoints differ, so exact empty-segment pruning
  retains a line. A temporary scalar-contraction control removes it. Rounded
  path construction, pruning, matrix mapping and cache comparisons match the
  pinned source; no production arithmetic or cache correction is justified
  by these results. The effective-path audit still reports 160 differing path
  IDs across the controlled replay, including later layout geometry; accounting
  for the 903 rebuild pairs does not certify those other differences. All
  diagnostic controls were removed. The original upstream equality test remains
  ignored, with its comparator and production CSS behavior unchanged.

- `game_menu_ad_police_files`: the original first mismatch was a runtime CSS
  integration defect, not an intentional difference or missing test setup. The mounted ListItem
  is a `fit-content` flex row and receives an 800-point width offer. Its
  container sizing returned 1395.4121 from maximum-content bases instead of
  resolving a finite fit-content width. The summary text itself has a
  93.615234-point minimum and wraps correctly under a finite offer; its
  unbounded width is 1303.4121. The correction resolves explicit fit-content
  flex-container widths between their intrinsic minimum and maximum before
  laying out children. It preserves intrinsic queries, plain auto containers,
  and authored bounds, following the
  [CSS fit-content definition](https://www.w3.org/TR/css-sizing-3/#fit-content-size).
  This is a general container correction, not a width override for this asset.
  After that correction, the first difference moves to frame 14, operation
  4868, path 60: the suspect ListItem's description container (object 170)
  is 794.0928 wide instead of 716.91016, with the same 67.51367 height.
  This remaining difference follows the existing CSS text fit-content
  contract: text 179 keeps its chosen 754.0928 inline size rather than its
  676.91016 widest wrapped line; object 170 adds 20 points of padding on each
  side. Independent single-owner controls associate this path with that
  container: limiting object 170 to 321 changes path 60 to 321, limiting
  text owner 178 to 321 changes it to 361, and limiting separator 167 does
  not change it. The path's height follows object 170 in each control.
  The October 29 audit against `81e0ee93fb9a4ab117283a87838f20fdcac2044d`
  compared all 43,983 operations across 121 frames. Under the unchanged
  upstream comparator, only four right-edge x coordinates differ: two in path 60 and two in the 4-point-high path 35,
  both in frame 14. Path 35 has the same width difference, but its source
  owner has not been established. The active
  `game_menu_css_changes_only_two_rectangle_widths` regression checks these
  two complete rectangles and applies the unchanged upstream comparator to
  every other operation. The original C++ equality assertion remains ignored;
  no replacement stream or wider tolerance is introduced.

- `layout_aspect_ratio`: the October 29 producer audit found a test-porting
  error: the C++ producer binds the default view model to the artboard before
  constructing its state machine. The manifest now follows that order rather
  than binding only to an already-created machine. This correction does not
  remove the geometry difference. Object 12 has preferred ratio 2, auto sizes,
  Fill on both axes and stretch, inside object 9's 174-by-165 row with
  16-point padding. CSS flex-grow and stretch determine its size as 142 by
  133; upstream Yoga instead retains 142 by 71. The column pair similarly
  fills its 208-by-202 inner box despite preferred ratio 0.5. An active test
  checks both children's parent-relative sizes, positions and CSS inputs
  through all 33 animated frames. This preserves the preferred ratio while
  respecting the two axes already determined by CSS layout.

Diagnostic overrides are not runtime fixes or replacement baselines. Original
C++ equality assertions remain available explicitly for intentional differences;
these cases do not gain exact-parity credit.

Validation of this correction: all 149 fit-content unit tests and the existing
layout minimum, scroll, matrix, corpus-unit and grid/stack suites pass on the
build pool. A full runtime replay diagnostic executed 216 cases (15 unsupported
surfaces skipped); every stream is byte-identical between HEAD's container
implementation and this correction, and no exact case regressed. At the time of that correction, the normal
corpus validator still failed on pre-existing reviewed-stream hash drift in
`bankcard`, `data_viz_demo`, `global_view_models_scripting_test`,
`list_focus_order`, and `number_to_list_nested_children`. Both independent
baseline runs reproduced those same bytes; historical hashes were not rewritten.
That validation did not claim that the full corpus validator was green; the
subsequent hash investigation is documented below.


## Reviewed stream hash audit (2026-10-27)

The five failures noted above were investigated against historical Rust captures
and the complete upstream operation changes, rather than accepted from their
unchanged first CSS mismatch. Runtime revision `84d5cde02e` with its pinned
upstream `5551696a826ff546775fffe46ca78824b1f1dc51` reproduces the old hashes for
the four cases below exactly. Their assets and executable replay actions are
unchanged. Upstream `6771fdad5`, translated in `95dad0e6ae`, changed their C++
recordings for the same reasons:

| Case | Complete change from the recorded Rust stream |
| --- | --- |
| `data_viz_demo` | Removes 11 redundant `Rewind`/`AddRawPath` pairs for path 0 in frames 2–12. Every removed operation is identical to upstream's deletion; every remaining Rust operation byte is unchanged. |
| `global_view_models_scripting_test` | Removes one redundant path-0 reset/update pair in frame 1, exactly as upstream does. All remaining operation bytes are unchanged. |
| `list_focus_order` | Reorders paint initialization at 10 positions in frame 0. Every changed index and before/after operation byte sequence matches upstream. Geometry, draw-time colors and final paint colors are unchanged. |
| `number_to_list_nested_children` | Reorders paint initialization at 123 positions in frame 0, with the same complete byte-level correspondence and unchanged geometry/draw-time/final colors. |

A separate controlled pool replay on current source restored only the old host
style recascade and reproduced the first two historical hashes exactly. Restoring
only the old list-row entry timing reproduced the other two. These controls were
diagnostic only; the upstream improvements remain in the runtime. The four Rust
hashes are refreshed because their recordings were stale, while the approved CSS
first differences, original C++ streams, tolerances, exact-ID set and coverage
floor remain unchanged.

`bankcard` had two independent baseline problems. The original hash
`86f2d3515f0d1b4c094e35e9f88c1468ee2a92c54b544acda15e79845abe43fe`
reproduces exactly on macOS/aarch64 at its introduction, Rust `704761ef16`
with upstream `5551696a`. The same source on Linux/x86_64 produces
`a0408c25df1a78c1d68964d3dc181daec3c376b57f20e00dc3c2f25ed44be886`,
including with strict-fp. The platform difference is 123 small floating-point
fields across 43 operations; counts, ordering and integer fields are identical.
The existing SRIV comparator accepts these streams without tolerance changes.
A Mac-specific byte hash was therefore incorrectly treated as portable.

Separately, upstream `6c65f385`, translated in `032a1770f1`, reordered opacity
initialization. The 546 changed operation positions and their complete old/new
bytes match the upstream change. Geometry and draw-time colors are unchanged.
The current full-stream hashes are now recorded for each verified target:
Linux/x86_64 `b131c276bb5a6a7959ad2eb03dd45d6b5aeda42e9e587fdc846d27a153b77575`
and macOS/aarch64 `e385b79ca0968a6be70e024a1eb2e5bdd7580169cad248be223b8f5f88501c02`.
The verifier selects only the running target's hash and fails if no target has
been reviewed. It does not accept a set of alternative streams or normalize
floating-point bytes. The other four refreshed hashes reproduce on both hosts.

These five failures are stale or incorrectly platform-independent baseline
metadata, not new CSS divergences or runtime porting defects. Their already
reviewed CSS differences remain intentional and separately recorded. No runtime
math, upstream recording, comparison tolerance or exact-coverage obligation was
changed to clear them.

Validation after both independent reviews: the complete runtime lane succeeds
on the Linux/x86_64 and macOS/aarch64 build-pool targets. Each executes 216 of
231 selected cases: 192 compare to C++ within the unchanged comparator,
24 retain documented divergences, and 15 unsupported cases remain skipped.
All 30 Rust manifest/hash-guard unit tests pass on the pool, and manifest
regeneration/check is clean. The Python generator unit suite was not executed
because it has no configured pool target. This is runtime-lane validation,
not a claim of full scripted-lane or unsupported-feature parity.

## Bound-value and external-asset replay recovery (2026-10-27)

Six former unsupported entries now have complete executable upstream recipes:
`artboard_opacity_and_transform_test`, `image_computed_transform_bind`,
`solo_index_test`, `data_bind_blob_test`, `image_binding_with_listener`, and
`stateful_component_image_test`. These were shared-runner coverage gaps, not
approved CSS or Wasm exclusions. The recipes use the existing Rust runtime APIs.

Pointer coordinates read their bound numeric properties independently for every
input call. Computed image dimensions retain all eight upstream Catch-style
assertions and all 63 draws. The asset recipes retain authored instance-0
binding, serializing-factory image decoding, image clearing, and external blob
identity/byte assertions. JPEG/blob inputs are listed as dependencies but are
loaded only at their authored action; they are not eagerly imported as Rive files.
Existing Rive dependencies preserve their import behavior.

The six entries require exact comparison against the unchanged upstream streams.
No CSS behavior, recorded baseline, tolerance or existing obligation was relaxed;
the historical coverage floor increases from 209 to 215. Both independent review
passes checked the frozen translation, including the corrected dependency-loading
order. Temporary diagnostics from the earlier hash investigation are not part of
this change.

Pool validation: all 32 manifest/action tests, 27 Silver backfill tests (including
all six recovered recipes), and 8 scripted-property tests pass. The full Linux
runtime lane executes 222 cases: 198 compare to C++ under the unchanged
comparator, 24 retain their documented differences, and 9 remain unsupported.
The generator check is clean. Python generator unit tests were not run because
there is no configured pool target. No runtime implementation changes were needed.

## View-model rebinding replay recovery (2026-10-27)

Five additional recipes execute the pinned upstream producers:
`multi_listeners-rebind`, `rebind_with_nested_viewmodel`, `replace_vm_instance`,
`replace_vm_instance-list`, and `replace_vm_instance-double-nest`.
Named references retain the actual view-model handles, including the detached
old child mutated by the listener test. Explicit release actions match the C++
scope boundaries; replacement never reconstructs or clones a substitute tree.
The list cases append the same retained child that is subsequently modified.

The shared-parent/dependent producer uses a literal three-artboard replay owner
because its setup requires all three artboards before any machines, and all
three machines before the independent authored view models. It shares one Silver
factory, performs all 18 draws, and retains the original six parent/dependent
count assertions. Dispatch precedes the single-artboard runner's setup, so no
extra occurrence or resource is constructed.

Validation exposed an incorrect port of the test/tools-only
`ViewModelInstance::dependents()` accessor: it discarded state-machine entries,
where upstream returns every `DataBindContainer`. The accessor now returns the
complete ordered set using Rust's existing dependent enum; state-machine handles
remain weak. Binding behavior and the expected counts are unchanged. The source
and Rust integration reviews covered both this correction and reference lifetimes.

The five exact comparison obligations raise the coverage floor from 215 to 220.
Existing CSS exceptions, full-stream hashes, upstream recordings and tolerances
are unchanged.

Pool regression validation passes all 32 manifest/action tests and all 32 Silver
backfill tests, including the five new comparisons and the previous six recovered
cases. The shared-artboard replay passes every original parent/dependent count
assertion after the accessor correction. Manifest regeneration/check is clean.

## Stateful component replay recovery (2026-10-27)

The four remaining runtime-lane unsupported entries now have literal replay
owners: `stateful_artboard_swap`, `stateful_source_switch`,
`stateful_list_props_lifecycle`, and `transition_self_comparator_test`.
They preserve all upstream identity and property assertions, the retained removed
list item, click-only advances, empty list items, and invalid-index no-ops.
The recordings contain 33, 51, 25, and 18 draws respectively. Setup occurs before
renderer creation exactly as in the upstream producers, without the generic
runner constructing additional instances.

The new exact comparison obligations raise the coverage floor from 220 to 224.
Existing CSS exceptions, recorded streams, hashes and tolerances are unchanged.

Both independent reviews pass. Pool validation passes all 32 manifest/action
unit tests and all 36 backfill tests, including the four recovered producers.
These were missing replay coverage, not runtime bugs or intentional CSS/Wasm
exclusions; no runtime implementation changes were necessary. Review corrected
one replay-only temporary handle lifetime, and compilation corrected its module
import. Manifest regeneration/check is clean. Python generator unit tests were
not run because there is no configured pool target.

The full Linux pool runtime lane executes all 231 cases: 85 byte-exact,
122 operation-exact within the existing epsilon, 24 documented divergences,
and zero unsupported or pending runtime cases. The comparator case is byte-exact;
the other three use the existing operation comparator. This does not claim
scripted-lane completion or removal of the documented CSS differences.


## Scripted Silver replay coverage (2026-10-27)

All 48 catalogued scripted producers use Luau. The public producers provide no
Wasm-only Silver cases to exclude. Shared replay owners retain the original
embedded programs or literal standalone Lua, typed view-model construction,
identity assertions, callback ordering, recording boundaries, renderer balance,
and animated-oval stack/GC checks. Source compilation is confined to the Silver
tool; the runtime's production bytecode API is unchanged. The direct scripted
artboard renderer bridge is gated by the existing `upstream-test-seams` feature.

The scripted lane now executes producers before comparing their full streams.
It no longer counts standalone test metadata as execution evidence. Filtered
runs enforce the selected lane's obligations; an unfiltered run enforces the
combined floor. Existing CSS first differences and direct-test SRIV hashes are
preserved. The three long serialized demos retain their known first-difference
checks; this does not assert that every later difference has been adjudicated.

The full scripted pool run executes all 48 cases: 19 byte-exact, 23 exact within
the existing operation epsilon, and six established CSS differences. The grid
case is byte-exact; no new exception was introduced. The two create-scene and
list-index cases reproduce their existing direct-test full-stream hashes.
The earlier exact metadata for text-run creation was stale: its direct test
already documented a CSS difference, now enforced by the shared runner.

Both independent review passes covered all replay owners and the shared runner.
Corrections concerned harness setup/lifetimes, live pointer-coordinate reads,
metadata guards, type imports, and error handling. No production runtime behavior
was changed. Python generator tests have no pool target and were not executed;
manifest regeneration/check remains part of validation.

Final combined pool validation passes all 279 executable producers: 104 byte-exact,
145 operation-exact within the existing epsilon, and 30 documented differences.
There are no pending or unsupported executable cases. The three pre-existing
unknown-provenance baselines remain separate. All 32 manifest/action unit tests,
36 runtime backfill tests, and eight existing scripted-property tests pass.


## Historical baseline provenance audit (2026-10-27)

The full, non-shallow public upstream history resolves the origin of all three
files, but only one has a public producer. File origin is not evidence of a
replay recipe. None of these findings warrants adding an executable test.

| Baseline | Public history | Disposition |
| --- | --- | --- |
| `interpolator.sriv` | Added by [`e6b7fd580`](https://github.com/rive-app/rive-runtime/commit/e6b7fd58066576edfaf414ebf5eaa865063a0aed). The addition contains no matching producer; its new test writes `viewmodel_from_instance.sriv`. Full-history searches find no producer or rename lineage. | Keep `provenance-unknown`. Do not synthesize a test from its 67 bytes of paint/frame-size operations. |
| `multitouch_debug.sriv` | Added by [`a3cfedf2c`](https://github.com/rive-app/rive-runtime/commit/a3cfedf2cceef41b6cd6851563f4f5f16ea499dd). The only added producer writes `remove_from_list.sriv`; neither this delta nor public history names a `multitouch_debug` producer. | Keep `provenance-unknown`. The filename does not establish equivalence to the ordinary multitouch test. |
| `stateful_list_props.sriv` | [`ea7277bbe`](https://github.com/rive-app/rive-runtime/commit/ea7277bbe1d70583e7f7ebd594ddb0dc6fbc010d) adds “Stateful Component list with input/output bridge binds” in `component_test.cpp`. [`43a7dfcd7`](https://github.com/rive-app/rive-runtime/commit/43a7dfcd7f3458d19fe915b002f0b3e187d8c048) disables it and changes its fixture. [`a207d7884`](https://github.com/rive-app/rive-runtime/commit/a207d78846610d333e34688daa47acde5f600cad) removes the unused per-item VM cloning and bridge-binding implementation. | Mark `retired-upstream`, outside executable counts. Upstream explicitly rejected per-item stateful VM instances for overhead; replaying the old Silver against the changed fixture or restoring that behavior would contradict current upstream. |

Each baseline remains byte-identical to its public introduction at pin
`a203f5355840bbc22c4544fef9edc411c85f3c03`. SHA-256 values:

- `interpolator`: `0d1dfa8e41fd60b1ebb625f644823910c432a53ef9e2937574e72bd542a1b180`
- `multitouch_debug`: `cf6b08f9f374df5a02a001a78e4d5d887bd63c90e1f7cf4814125d174a34e5e0`
- `stateful_list_props`: `de4518d633626451ddb090b434d7a56d7329bb2b5a3bf959f9982bfb0dcb8ab9`

The active `stateful_list_props_lifecycle` producer remains executable and exact
under the existing comparator. Retirement applies only to the older distinct
baseline. No private upstream producer is inferred from public artifacts. All
282 baseline files remain structurally validated; the 279 active producers,
267 comparison obligations, CSS checks, and original baseline bytes are unchanged.

Both independent review passes passed. Pool validation passes 34 unit tests and
all 279 active producers (104 byte-exact, 145 operation-exact, 30 documented
differences), reporting one retired baseline and two unknown producers. There
are no new replay recipes or runtime changes. Generator/check and diff checks
pass; Python generator unit tests remain unexecuted because no pool target is
configured.
