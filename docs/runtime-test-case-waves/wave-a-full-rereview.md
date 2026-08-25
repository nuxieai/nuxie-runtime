# Wave A full independent semantic re-review

Reviewed commit: `6bca8b958`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_a_full_rereview` lane

Verdict: **REJECTED**

## Acceptance rule

All 259 rows were re-read from the pinned C++ case and the committed Rust
evidence. No previous receipt, accepted count, resolved locator, or status was
grandfathered. A case was accepted only when its executable Rust body recreates
the pinned fixture, action order, production owner, and assertions, or when a
live differential executes that flow in both runtimes.

An expected-red body was accepted only when it performs real translated work
and reaches a concrete absent or divergent boundary. Frozen source, inert
actions, name checks, and nearby coverage do not count. A Silver replay counts
for the exact action stream and SRIV comparison that it executes; direct C++
assertions outside that stream require separate executable evidence.

## Complete census

`accepted pass` includes direct, approved-adaptation, and live-differential
rows whose declared outcome is pass. `accepted executable expected-red`
includes only ignored tests that reach a concrete boundary after live actions.

| upstream file | cases | accepted pass | accepted executable expected-red | incomplete or narrower | metadata-only |
|---|---:|---:|---:|---:|---:|
| `aabb_test.cpp` | 9 | 9 | 0 | 0 | 0 |
| `animation_state_instance_test.cpp` | 14 | 14 | 0 | 0 | 0 |
| `artboard_transform_test.cpp` | 6 | 5 | 1 | 0 | 0 |
| `audio_test.cpp` | 13 | 12 | 1 | 0 | 0 |
| `bezier_utils_test.cpp` | 11 | 10 | 1 | 0 | 0 |
| `binary_reader_test.cpp` | 5 | 5 | 0 | 0 | 0 |
| `bound_bones_test.cpp` | 1 | 1 | 0 | 0 | 0 |
| `bounds_test.cpp` | 3 | 1 | 2 | 0 | 0 |
| `cdn_asset_test.cpp` | 2 | 2 | 0 | 0 | 0 |
| `child_iterator_test.cpp` | 1 | 0 | 1 | 0 | 0 |
| `clip_test.cpp` | 5 | 4 | 0 | 1 | 0 |
| `color_glyph_test.cpp` | 22 | 19 | 3 | 0 | 0 |
| `color_test.cpp` | 2 | 2 | 0 | 0 | 0 |
| `command_queue_test.cpp` | 83 | 83 | 0 | 0 | 0 |
| `component_list_test.cpp` | 30 | 20 | 10 | 0 | 0 |
| `component_origin_test.cpp` | 3 | 1 | 2 | 0 | 0 |
| `component_test.cpp` | 8 | 3 | 5 | 0 | 0 |
| `contour_measure_test.cpp` | 6 | 5 | 1 | 0 | 0 |
| `cubic_value_test.cpp` | 1 | 1 | 0 | 0 | 0 |
| `dash_test.cpp` | 1 | 1 | 0 | 0 | 0 |
| `data_bind_container_test.cpp` | 12 | 11 | 1 | 0 | 0 |
| `data_bind_lists_test.cpp` | 4 | 2 | 2 | 0 | 0 |
| `data_binding_artboards_test.cpp` | 10 | 2 | 8 | 0 | 0 |
| `data_binding_blobs_test.cpp` | 5 | 4 | 1 | 0 | 0 |
| `data_binding_computed_values_test.cpp` | 2 | 0 | 2 | 0 | 0 |
| **total** | **259** | **217** | **41** | **1** | **0** |

The 258 accepted rows consist of 217 executable passes and 41 executable
expected-red ports. The declared mechanism census at the reviewed commit is
201 direct passes, 39 direct expected-red, 13 adapted passes, two adapted
expected-red, and four differential passes. The approved adaptations are
limited to native audio and Rust ownership/safety mechanics; they do not
replace the tested behavior. The four shader differentials compile the
repo-owned byte-identical GLSL into the original pinned C++ cases.

## Blocking row

`tests/unit_tests/runtime/clip_test.cpp#2`, **artboard is clipped correctly**,
is not a complete port.

The pinned case advances `Center`, then calls
`artboard->worldPath()->rawPath()` and asserts the four exact world-path points
`(0,0)`, `(500,0)`, `(500,500)`, and `(0,500)`. It disables
`frameOrigin`, updates components, calls that same owner again, and asserts
`(-250,-250)`, `(250,-250)`, `(250,250)`, and `(-250,250)`.

The cited Rust evidence,
`clip_artboard_is_clipped_correctly_complete_port`, never invokes or observes a
Rust world-path owner. It draws through `RecordingFactory`, checks that the
stream contains a translation and the four untransformed clip coordinates,
and computes `raw + 250 == world` inside the test. After disabling frame
origin it again searches the stream for raw coordinate strings. This proves
that a draw emitted a raw clip plus a transform; it does not prove that the
runtime's retained `worldPath` has the pinned points. A broken or absent
world-path owner can pass this test unchanged.

The row must therefore be reset from `direct/pass` or replaced with either:

1. a direct test that calls the actual retained world-path owner before and
   after `frameOrigin(false)` and compares all four points; or
2. an executable expected-red port that performs the fixture/advance/update
   actions and stops at the concrete missing world-path observation surface.

Recording-transform arithmetic in the test is not an equivalent owner.

## Re-audited accepted families

The prior 180 accepted rows were compared again, including all 83 command
queue cases and every compacted multi-row mapping. Shared symbols were accepted
only when the body performs each cited case's distinct operations and
assertions. The corrected families were also re-read in full:

- all 14 animation-state cases construct and advance the real retained state
  occurrence and inspect its owned animation instance;
- all 13 audio cases execute the native-audio adaptation, with case 2 red only
  at the exact resample-frame mismatch and the duration cases preserving the
  second cache call;
- all 11 Bezier cases either execute the production Rust owner or the live
  byte-identical production-GLSL harness;
- bounds, child-iterator, color-glyph, and component expected-red tests reach
  the named missing owner or live behavioral divergence after real fixture
  work;
- component-list rows combine complete Silver streams with the direct
  assertions omitted from those streams, including cases 16, 20, and 21;
- the four container residuals execute the retained queue/bind owners and
  exact call-count/origin sequences;
- all eight artboard residuals execute live imports, binding, mutation,
  advance, draw, and comparison work until their first concrete missing seam;
  and
- the four blob residuals and computed-image residual execute their actual
  retained owners and complete live streams up to the documented comparator or
  value divergence.

No accepted expected-red row is now metadata-only. The sole rejected row is a
narrow substitute for a different owner, not a locator or compilation issue.

## Evidence and validation

- The shard contains exactly 259 rows for the first 25 lexicographic upstream
  files. Every frozen upstream ordinal/name and every committed evidence path,
  line range, and symbol was resolved against `6bca8b958`; no locator failed.
- The committed shard parses as JSON and its declared outcomes total 218 pass
  and 41 expected-red. Semantic review demotes one of the 218 declared passes,
  yielding the 217/41/1 census above.
- The repository correspondence checker passed against the pinned checkout:
  157 files and 1,404 `TEST_CASE`s. Its main case ledger remains the independent
  all-pending ledger; that schema/denominator success does not promote this
  Wave A shard.
- `CARGO_INCREMENTAL=0 RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime cargo
  test -p nuxie-runtime --test upstream_wave_a_core` completed with its six
  concrete expected-red ports discovered.
- `CARGO_INCREMENTAL=0 RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime cargo
  test -p nuxie-runtime --test upstream_color_glyph` reported 18 pass and three
  ignored expected-red; the private shaping case supplies the twenty-second
  row.
- `CARGO_INCREMENTAL=0 RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime cargo
  test -p silver-corpus --test wave_a` reported 13 pass and 17 ignored
  expected-red, with all 30 entry points discovered.
- The focused component-list run completed before the Silver run. Later
  integration-suite attempts in the shared worktree encountered unrelated
  uncommitted edits from another active lane in `artboard.rs` and failed to
  compile at those edits. That shared-worktree contamination is not evidence
  for or against reviewed commit `6bca8b958`; no result from it was used to
  adjudicate a row.

Wave A cannot be accepted at 259/259 until the exact world-path row is
corrected and independently re-reviewed. No production or test file was
changed by this review.
