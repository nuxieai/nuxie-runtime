# Silver corpus

`silver-corpus` is the read-only adoption floor for upstream's SRIV v1
serialized-render corpus. It never rewrites a baseline.

The checked-in `silver-corpus.toml` is generated from active literal upstream
`silver.matches(...)`/`serializer()->matches(...)` calls after C++ comments are
removed. Six dynamically named layout-scroll cases and six `gridStackSilver`
helper cases are explicit generator data. `interpolator.sriv`,
`multitouch_debug.sriv`, and the upstream-disabled `stateful_list_props.sriv`
remain explicit `provenance-unknown` entries.

The runner validates all 282 streams structurally. For the 231 runtime cases,
the generator translates the replayable portion of each pinned C++ test body
into a shared action stream. Validation imports the recorded `.riv`, selects
the requested artboard/state machine/animation, applies those actions to the
Rust runtime, serializes the Rust render operations as SRIV v1, and compares
them operation by operation with the upstream baseline.

Every runtime row must be classified. Replayable differences are retained as
`diverges` findings with their first divergent operation in the note. Bodies
that cannot yet be represented or require an unported subsystem are
`unsupported-feature` with the blocker named. Scripted cases are adopted
independently as their literal C++ action/output replays land; the remainder
stay `pending-scripted`, and three baselines remain `provenance-unknown`. The
exact-ID ledger and `min_cpp_rust_exact` ratchet prevent exact cases from being
silently downgraded. The reviewed CSS layout differences retain all 206
historical IDs and the numeric floor. A CSS replacement in
`reviewed_layout_divergences` must remain an executable runtime layout
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
  those sizes and position. This explains the first mismatch; it does **not**
  certify every later operation in the scripted replay.

- `layoutstest_8-planets-grid`: the first pane boundary is an intentional
  combination of CSS Fill bases and automatic minima. In the 4377-point root,
  equal zero bases would give 2188.5 each, but the right pane's eight 284-point
  cells keep its automatic minimum at 2272, leaving 2105 on the left. C++ starts
  from content bases 284 and 2272 and shares 1821 free points equally, producing
  1194.5 and 3182.5. A temporary basis-only control restored exactly those pane
  sizes while retaining the automatic minima. A separate minimum-only control
  produced 2188.5 each. This identifies the first mismatch, not every later
  operation in the scripted replay.

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
  This explains the first remaining mismatch, not every later replay
  operation; the original C++ equality assertion remains ignored and no
  reviewed replacement stream is enrolled.

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
