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
implementation and this correction, and no exact case regressed. The normal
corpus validator still fails on pre-existing reviewed-stream hash drift in
`bankcard`, `data_viz_demo`, `global_view_models_scripting_test`,
`list_focus_order`, and `number_to_list_nested_children`. Both independent
baseline runs reproduced those same bytes; historical hashes were not rewritten.
This does not claim that the full corpus validator is green.
