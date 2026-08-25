# Wave A final independent confirmation

Reviewed commit: `36da806e4`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_a_final_confirmation` lane

Verdict: **REJECTED**

## Acceptance rule

All 259 rows were reconsidered under the executable-port rule. A passing port
must recreate the pinned fixture, action order, production owner, and
assertions in executable Rust, or run a live C++/Rust differential over that
flow. An expected-red port must execute the translated work up to one concrete
missing or divergent boundary. Frozen C++ source, inert action descriptions,
name checks, recording proxies for a different owner, and nearby coverage do
not count.

The committed evidence locator is part of the proof. Its exact line and symbol
must resolve, an expected-red reason must equal the test's `#[ignore]` reason,
and the row must satisfy the one-typed-locator schema. A test body in an
integration target that does not compile is not presently executable.

## Exact census

The body-level semantic census is 217 passing ports, 42 genuine expected-red
ports, zero incomplete/narrow bodies, and zero raw-source/inert/proxy bodies.
The corrected clip row accounts for the change from the preceding review's
218/41 declared census to 217/42.

Strictly applying locator and executability requirements to the reviewed
commit produces this certifiable census:

| upstream file | cases | executable pass | executable expected-red | incomplete or unexecutable | metadata-defective |
|---|---:|---:|---:|---:|---:|
| `aabb_test.cpp` | 9 | 9 | 0 | 0 | 0 |
| `animation_state_instance_test.cpp` | 14 | 14 | 0 | 0 | 0 |
| `artboard_transform_test.cpp` | 6 | 5 | 1 | 0 | 0 |
| `audio_test.cpp` | 13 | 10 | 0 | 2 | 1 |
| `bezier_utils_test.cpp` | 11 | 8 | 1 | 0 | 2 |
| `binary_reader_test.cpp` | 5 | 5 | 0 | 0 | 0 |
| `bound_bones_test.cpp` | 1 | 1 | 0 | 0 | 0 |
| `bounds_test.cpp` | 3 | 0 | 2 | 1 | 0 |
| `cdn_asset_test.cpp` | 2 | 0 | 0 | 0 | 2 |
| `child_iterator_test.cpp` | 1 | 0 | 1 | 0 | 0 |
| `clip_test.cpp` | 5 | 4 | 1 | 0 | 0 |
| `color_glyph_test.cpp` | 22 | 7 | 2 | 0 | 13 |
| `color_test.cpp` | 2 | 2 | 0 | 0 | 0 |
| `command_queue_test.cpp` | 83 | 83 | 0 | 0 | 0 |
| `component_list_test.cpp` | 30 | 19 | 8 | 0 | 3 |
| `component_origin_test.cpp` | 3 | 1 | 0 | 2 | 0 |
| `component_test.cpp` | 8 | 3 | 5 | 0 | 0 |
| `contour_measure_test.cpp` | 6 | 5 | 1 | 0 | 0 |
| `cubic_value_test.cpp` | 1 | 0 | 0 | 1 | 0 |
| `dash_test.cpp` | 1 | 1 | 0 | 0 | 0 |
| `data_bind_container_test.cpp` | 12 | 11 | 0 | 0 | 1 |
| `data_bind_lists_test.cpp` | 4 | 2 | 2 | 0 | 0 |
| `data_binding_artboards_test.cpp` | 10 | 2 | 7 | 0 | 1 |
| `data_binding_blobs_test.cpp` | 5 | 4 | 0 | 0 | 1 |
| `data_binding_computed_values_test.cpp` | 2 | 0 | 1 | 0 | 1 |
| **total** | **259** | **196** | **32** | **6** | **25** |

The 25 metadata-defective rows contain 17 otherwise-passing bodies and eight
otherwise-executable expected-red bodies. The six unexecutable rows contain
four otherwise-passing bodies and two expected-red bodies. These buckets are
disjoint and total 259.

## Clip correction

`tests/unit_tests/runtime/clip_test.cpp#2`, **artboard is clipped correctly**,
is now a genuine expected-red port. It no longer derives a would-be world path
from a recording transform. The test imports the exact
`artboardclipping.riv` fixture, instantiates and advances `Center`, and reads
the retained `runtime_shapes.paint_path_owner(0, World).retained.raw_path`
owner that corresponds to pinned `Artboard::worldPath()->rawPath()`.

The ordinary filtered run discovers the test as ignored with the committed
reason. The explicit ignored run passes the initial four points `(0,0)`,
`(500,0)`, `(500,500)`, and `(0,500)`. It calls `frameOrigin(false)`, updates
components, reads the same retained owner, and fails only because Rust leaves
those points unchanged instead of producing `(-250,-250)`, `(250,-250)`,
`(250,250)`, and `(-250,250)`. This is the exact concrete owner divergence;
the prior proxy blocker is resolved.

## Blocking metadata defects

The Wave A shard cannot pass its own strict evidence validation at
`36da806e4`. Upstream identity is sound: all 259 IDs, paths, ordinals, source
lines, and case names exactly match the pin. The failures are confined to 25
Rust evidence records.

### Seventeen stale exact function lines

The symbols exist, but not at their declared line:

- `bezier_utils_test.cpp#3`: `measure_non_inflect_cubic_rotation_direct_port`,
  declared 254, actual 324.
- `bezier_utils_test.cpp#5`:
  `find_cubic_convex_180_chops_lines_direct_port`, declared 345, actual 586.
- `cdn_asset_test.cpp#1`: `hosted_image_cdn_descriptor_matches_pinned_cpp`,
  declared 9937, actual 10132.
- `cdn_asset_test.cpp#2`: `hosted_font_cdn_descriptor_matches_pinned_cpp`,
  declared 9960, actual 10155.
- `color_glyph_test.cpp#8`: declared 155, actual 159.
- `color_glyph_test.cpp#10`: declared 186, actual 192.
- `color_glyph_test.cpp#12`: declared 200, actual 199.
- `color_glyph_test.cpp#13`: declared 208, actual 207.
- `color_glyph_test.cpp#14`: declared 219, actual 218.
- `color_glyph_test.cpp#15`: declared 231, actual 230.
- `color_glyph_test.cpp#16`: declared 248, actual 247.
- `color_glyph_test.cpp#17`: declared 260, actual 259.
- `color_glyph_test.cpp#18`: declared 273, actual 272.
- `color_glyph_test.cpp#19`: declared 281, actual 280.
- `color_glyph_test.cpp#20`: declared 287, actual 286.
- `color_glyph_test.cpp#21`: declared 294, actual 293.
- `color_glyph_test.cpp#22`: declared 300, actual 299.

The color-glyph symbols themselves resolve by name in
`crates/nuxie-runtime/tests/upstream_color_glyph.rs`; this is locator drift,
not a newly discovered semantic substitution.

### Five expected-red reason mismatches

For each row, `expected_red_reason` differs from the evidence test's exact
`#[ignore]` text:

- `audio_test.cpp#2`: the shard names concrete 10545/7030 results; the test
  says the 48 kHz and 32 kHz resamples round one frame longer than C++.
- `data_bind_container_test.cpp#10`: the shard says a ToTarget-only bind
  rejects `updateSourceBinding`; the test includes “a” and “currently”.
- `data_binding_artboards_test.cpp#8`: the shard says “bound ViewModel
  instance”; the test says “bound ViewModel”.
- `data_binding_blobs_test.cpp#5`: the shard names the unavailable pinned
  `data_bind_blob_test` comparator; the test says the pinned SRIV comparator is
  not wired into the integration tests.
- `data_binding_computed_values_test.cpp#2`: the shard names `computedWidth`
  at the first assertion; the test says `computedWidth/Height` remain zero
  instead of the pinned initial 150.

### Three schema-invalid multiple-locator rows

`component_list_test.cpp#16`, `#20`, and `#21` each contain two `rust-test`
locators: one for the direct assertions omitted by Silver and one for the
Silver replay. Both bodies are semantically necessary and were rechecked, but
the `nuxie-test-case-correspondence/v1` validator requires exactly one typed
locator per proven row. The ledger needs a schema-valid composite evidence
surface or a single test that executes both halves.

## Six execution-blocked rows

The integration target `crates/nuxie-runtime/tests/cpp_probe.rs` does not
compile at the reviewed commit with its required `tools` feature. Rust rejects
`Some(*value as i32)` at line 24799 because `ScriptArtboardSource` is not a
fieldless enum. Consequently these six cited rows cannot currently execute:

- `audio_test.cpp#12` and `audio_test.cpp#13`;
- `bounds_test.cpp#2`;
- `component_origin_test.cpp#1` and `component_origin_test.cpp#2`; and
- `cubic_value_test.cpp#1`.

Their resolved bodies were compared with the pinned cases and are not
raw-source or proxy evidence. This is an integration-target compilation
blocker, but the executable-port criterion does not permit counting an
unbuildable target as passing or executable-red.

## Rechecked families and adaptations

Every family was sampled against the pinned C++ source after the complete
row/locator census. The large compacted families still execute their distinct
case behavior: all 83 command-queue rows retain case-specific byte fixtures
and queue assertions; all 14 animation-state rows instantiate and operate on
the retained state owner; and the component-list direct tests and Silver
replays jointly retain the case-specific assertions that were previously
omitted.

No Wave A row cites `upstream_wave_a_expected_red.rs`, `pending_literal_port`,
raw source, or a symbol named as metadata/proxy evidence. The artboard action
stream now executes imports, selection, view-model creation/binding,
mutation, advance, input, draw, and concrete boundary checks rather than
iterating inert action names. The four Bezier GLSL rows invoke their live
differential harness and the repo-owned byte-identical shader authority.

The 13 audio rows are explicitly `native-audio` adaptations. The two
production GLSL helper rows are `rust-safety` adaptations for C++ null-pointer
mechanics; the four shader cases are live differentials. No Taffy, native
scripting, or language adaptation in Wave A substitutes a different tested
observable.

## Validation evidence

- Direct shard validation against the pinned first 259 cases found zero
  upstream identity errors and the exact 25 evidence-metadata failures above.
- The repository-wide checker and its 24 unit tests pass: 157 files and 1,404
  pinned `TEST_CASE`s. Its main case ledger deliberately remains all pending;
  that result does not promote this Wave A shard.
- The corrected clip test is discovered as one ignored test. Its explicit
  ignored run fails only at the second retained-world-path comparison.
- Representative `nuxie-runtime` integration targets pass or discover their
  expected reds: AABB (12 pass), artboard transform (5 pass/1 red), color glyph
  (18/3), contour measure (5/1), instancing (1/2), Wave A core (0/6), plus the
  blob tests (2 pass).
- The 14 animation-state tests pass. The 20 direct component-list tests report
  19 pass and one expected-red. The color-owner and zero-length-dash focused
  tests pass.
- `nuxie` integration targets report audio core 6 pass, command queue 90 pass,
  upstream audio 8 pass/1 red, bound bones 1 pass, data-binding artboards 8
  expected-red, and final residuals 2 expected-red. CDN owner tests pass 2/2.
- Silver Wave A reports 13 pass and 17 expected-red. Binary-reader and broader
  `nuxie-binary` suites pass. The renderer Bezier unit family reports 10 pass
  and 3 expected-red.
- The four Bezier live differentials pass respectively 1,850, 15, 74,370, and
  16,539 assertions.
- The `cpp_probe` target fails to compile at the exact enum cast described
  above; no result from that target was counted as executable.

Wave A is semantically translated, including the corrected clip owner, but it
is not certifiable at 259/259 until all 25 evidence records validate and the
six `cpp_probe`-backed rows can execute. This review changed no production or
test code.
