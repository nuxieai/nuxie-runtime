# Wave B independent port review

Reviewed commit: `5c57a35c8`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_b_review` lane

Verdict: **REJECTED**

## Operational definition

A ported test must recreate the upstream case's fixture, action order, and
assertion flow as executable Rust, or invoke a live C++/Rust differential that
executes that flow. The port may remain `#[ignore]` and expected-red when a
specific missing Rust behavior prevents it from passing, but running the test
must reach the translated behavior up to that missing point. Preserving a C++
body in a Rust string, comment, snapshot, or other inert anchor is useful
frozen source metadata; it is not a test port and cannot claim `direct` case
proof.

## Complete census

All 269 evidence locators were resolved against their declared Rust source
line and symbol. Every one has the same operative Rust body: it passes a raw
C++ `TEST_CASE` string to `pending_literal_port`. That helper checks only that
the string begins with `TEST_CASE(` and has braces, then unconditionally
panics. It never constructs a Rust fixture, invokes a Rust runtime owner,
performs the upstream actions, or evaluates an upstream assertion.

| upstream file | declared cases | executable/translated ports | source-anchor-only |
|---|---:|---:|---:|
| `data_binding_converters_test.cpp` | 3 | 0 | 3 |
| `data_binding_cycle_test.cpp` | 7 | 0 | 7 |
| `data_binding_fonts_test.cpp` | 2 | 0 | 2 |
| `data_binding_images_test.cpp` | 10 | 0 | 10 |
| `data_binding_keyframes.cpp` | 5 | 0 | 5 |
| `data_binding_test.cpp` | 40 | 0 | 40 |
| `data_binding_viewmodels_test.cpp` | 3 | 0 | 3 |
| `decode_ktx2_test.cpp` | 11 | 0 | 11 |
| `default_state_machine_test.cpp` | 1 | 0 | 1 |
| `distance_constraint_test.cpp` | 1 | 0 | 1 |
| `draw_order_test.cpp` | 1 | 0 | 1 |
| `elastic_easing_test.cpp` | 2 | 0 | 2 |
| `enums_test.cpp` | 17 | 0 | 17 |
| `file_test.cpp` | 12 | 0 | 12 |
| `focus_test.cpp` | 85 | 0 | 85 |
| `follow_path_constraint_test.cpp` | 8 | 0 | 8 |
| `font_test.cpp` | 5 | 0 | 5 |
| `gamepad_test.cpp` | 7 | 0 | 7 |
| `global_view_model_binding_test.cpp` | 15 | 0 | 15 |
| `global_viewmodels_test.cpp` | 3 | 0 | 3 |
| `hittest_test.cpp` | 21 | 0 | 21 |
| `ik_constraint_test.cpp` | 1 | 0 | 1 |
| `ik_test.cpp` | 2 | 0 | 2 |
| `image_asset_test.cpp` | 2 | 0 | 2 |
| `image_decoders_test.cpp` | 5 | 0 | 5 |
| **total** | **269** | **0** | **269** |

One representative evidence symbol from each of the 25 upstream files was
inspected directly in addition to resolving every locator. The source anchors
cover diverse behavior but none translates it. Concrete examples include:

- `wave_b_data_binding_converters_test_001_direct_port_expected_red` embeds
  file loading, view-model mutation, state-machine advances, drawing, and a
  silver comparison as C++ text; Rust executes none of them.
- `wave_b_decode_ktx2_test_001_direct_port_expected_red` embeds the decoder
  call and rejection assertion, but never calls the Rust KTX2 decoder.
- `wave_b_enums_test_001_direct_port_expected_red` embeds
  `TestBinaryEnumOp<Flags>` in the raw string, but performs no Rust enum
  operation or comparison. Running this ignored test reaches only the common
  unconditional panic.
- `wave_b_focus_test_001_direct_port_expected_red` embeds construction and
  property checks for `FocusNode`, but constructs no Rust focus node.
- `wave_b_hittest_test_001_direct_port_expected_red` embeds the `HitTester`
  command stream, but sends no commands through the Rust hit-test owner.
- `wave_b_image_decoders_test_001_direct_port_expected_red` embeds PNG file
  loading, decoding, and bitmap assertions, but reads and decodes nothing in
  Rust.

The same structure was present in the representative symbols for file,
constraint, IK, font, gamepad, global-view-model, image-asset, and all other
Wave B file groups. There is no mixed subset to accept.

## Evidence runs

- The Wave B shard was checked against the pinned upstream census and all 269
  declared source lines, case names, evidence paths, line locators, symbols,
  ignore reasons, and outcomes resolved. The structural checker reported 269
  `direct` / 269 `expected-red`. This is locator/schema validation only; its
  current implementation does not inspect a Rust test body for translated
  behavior.
- The repository correspondence checker passed with the main 1,404-case
  ledger still at 1,404 `pending` / `unverified`; Wave B has not yet been
  promoted into that ledger.
- All 24 correspondence-checker unit tests passed.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test
  upstream_wave_b_expected_red -- --list` compiled the target and discovered
  all 269 ignored entry points.
- Running the ignored enum representative directly failed at the common
  `pending_literal_port` panic with exit 101 before any case behavior.

## Required correction

Reset the Wave B shard's 269 rows to `pending` / `unverified`, or replace each
anchor with a literal executable Rust translation (or live differential) and
then promote only that case. An expected-red translation must name and reach
its concrete missing behavior; a generic unconditional panic before fixture
construction is not acceptable evidence.

This result is scoped to Wave B. It also establishes a methodology rule with
no grandfathering: any earlier or later wave that uses source-body-only
expected-red anchors fails the same port definition and needs its own audit.
Wave A should therefore be reviewed separately under this criterion.
