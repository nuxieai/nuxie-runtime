# Wave A independent port review

Reviewed commit: `3cbcac6ef`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_a_review` lane

Verdict: **REJECTED**

## Operational definition

A ported test must recreate the upstream case's fixture, action order, runtime
owner, and assertion flow as executable Rust, or invoke a live C++/Rust
differential that executes that flow. A Rust safety or approved backend
adaptation may change the mechanism, but it must retain the upstream
observable behavior.

An expected-red test may be ignored, but running it must execute the translated
flow up to a concrete missing Rust behavior. Raw C++ text, comments, inert
pseudo-actions, source-name checks, and nearby or narrower tests are provenance
or supporting coverage, not case ports. A silver replay counts when it executes
the upstream silver fixture and action stream and compares the same pinned SRIV;
it does not cover additional direct assertions that the upstream C++ case made
outside that stream.

## Complete census

All 259 manifest rows were audited, not only the expected-red rows. Every
declared evidence path, line locator, and symbol resolves. The resolved body was
then compared with the pinned upstream case for fixture construction, action
order, owner under test, and assertions.

| upstream file | cases | accepted executable | incomplete or narrower | metadata/action only |
|---|---:|---:|---:|---:|
| `aabb_test.cpp` | 9 | 9 | 0 | 0 |
| `animation_state_instance_test.cpp` | 14 | 0 | 14 | 0 |
| `artboard_transform_test.cpp` | 6 | 6 | 0 | 0 |
| `audio_test.cpp` | 13 | 4 | 9 | 0 |
| `bezier_utils_test.cpp` | 11 | 2 | 9 | 0 |
| `binary_reader_test.cpp` | 5 | 5 | 0 | 0 |
| `bound_bones_test.cpp` | 1 | 1 | 0 | 0 |
| `bounds_test.cpp` | 3 | 1 | 2 | 0 |
| `cdn_asset_test.cpp` | 2 | 2 | 0 | 0 |
| `child_iterator_test.cpp` | 1 | 0 | 1 | 0 |
| `clip_test.cpp` | 5 | 4 | 0 | 1 |
| `color_glyph_test.cpp` | 22 | 19 | 3 | 0 |
| `color_test.cpp` | 2 | 2 | 0 | 0 |
| `command_queue_test.cpp` | 83 | 83 | 0 | 0 |
| `component_list_test.cpp` | 30 | 10 | 3 | 17 |
| `component_origin_test.cpp` | 3 | 3 | 0 | 0 |
| `component_test.cpp` | 8 | 5 | 3 | 0 |
| `contour_measure_test.cpp` | 6 | 6 | 0 | 0 |
| `cubic_value_test.cpp` | 1 | 1 | 0 | 0 |
| `dash_test.cpp` | 1 | 1 | 0 | 0 |
| `data_bind_container_test.cpp` | 12 | 8 | 4 | 0 |
| `data_bind_lists_test.cpp` | 4 | 4 | 0 | 0 |
| `data_binding_artboards_test.cpp` | 10 | 2 | 0 | 8 |
| `data_binding_blobs_test.cpp` | 5 | 1 | 4 | 0 |
| `data_binding_computed_values_test.cpp` | 2 | 1 | 1 | 0 |
| **total** | **259** | **180** | **53** | **26** |

The 180 accepted rows split into 176 executable direct ports, two executable
native-audio adaptations, and two native-audio adaptations that invoke a live
C++/Rust differential. No accepted row is declared with manifest status
`differential`; the two differential mechanisms are currently declared
`adapted` (`audio_test.cpp` cases 12 and 13).

## Accepted evidence

The accepted set includes compacted tests when the compacted Rust body still
executes every upstream case's behavior. Examples include the nine AABB cases,
all five binary-reader cases, all 83 command-queue cases, all six contour
measure cases, and the two color cases. The command-queue evidence preserves
the pinned byte fixtures and case-specific queue operations; the shared Rust
symbol for upstream cases 11 and 12 performs both cancellation variants rather
than merely serving as a common locator.

Approved mechanism substitutions were accepted only when the behavior remains
executable. `audio_test.cpp` case 6 exercises playback ownership, volume, and
artboard-stop lifecycle through native audio; case 10 executes the scripted
audio flow. Cases 12 and 13 run the buffered-duration and MP3-duration/cache
comparisons through the C++ probe when the pinned probe is configured.

Silver evidence was accepted where the upstream case itself is expressed by a
silver action stream and the Rust corpus replays that same stream against the
same expected SRIV. Expected-red divergence after a real replay is acceptable
evidence of a translated test; an empty or unsupported action list is not.

## Rejected evidence

### Metadata or inert actions: 26

Eighteen rows call `pending_literal_port`: `clip_test.cpp` case 2 and
`component_list_test.cpp` cases 1-15, 28, and 30. The helper validates the
shape of a raw C++ string and panics. It constructs no fixture, invokes no Rust
runtime owner, performs no action, and evaluates no upstream assertion. These
are useful frozen-source anchors, but not executable ports.

Eight more rows (`data_binding_artboards_test.cpp` cases 1, 2, and 5-10) use
`execute_until_missing_cross_file_registration`. File import is live, but the
declared `Select`, `Create`, `Bind`, `Advance`, `Draw`, `Set`, and related
actions are inert enum values/string checks. Execution advances to a generic
`Compare` panic without performing the upstream action flow. They are action
metadata, not expected-red translations.

### Incomplete or narrower mappings: 53

The main failure modes are wrong-owner testing, omitted assertions, and a
small synthetic proxy standing in for a fixture-driven upstream case:

- All 14 animation-state cases map to one matrix test that constructs
  `LinearAnimationInstance` directly and manually multiplies elapsed time by
  state speed. It bypasses `AnimationStateInstance`, the owner named and
  exercised by every upstream case, so it cannot detect owner/lifecycle
  translation errors.
- Nine audio mappings are narrower. Case 1 maps engine initialization to WAV
  decode; case 2 omits reader rate/level/playback behavior; cases 4 and 5 replace
  engine-outliving-sound behavior with artboard clone lifetime; cases 7-9 omit
  the direct/nested event-count assertions. Case 11 calls Rust duration only
  once and therefore misses the upstream second-call cache assertion.
- Nine bezier mappings do not preserve the action or owner. Case 1 loses the
  in-place alias operation and pinned randomized flow; case 2 calls the same
  explicit-slice API twice rather than testing the null input; cases 7-9 test
  local Rust reconstructions instead of the production GLSL behavior; cases 10
  and 11 only locate shader names before a generic expected-red panic.
- `bounds_test.cpp` case 1 derives a would-be local value by dividing scaled
  world bounds instead of calling the local-bounds owner. Case 3 reduces the
  upstream matrix across shapes, text, groups, images, n-slices, custom
  components, and layouts to two text objects.
- The child-iterator evidence manually filters graph collections rather than
  executing the typed child/object iterators under test.
- Three color-glyph cases miss the owner or assertions: the cache case calls a
  stateless extractor twice, the options case clones a font instead of invoking
  option derivation, and the shaping case checks only that render commands are
  nonempty.
- `component_list_test.cpp` cases 16, 20, and 21 replay useful silver streams,
  but omit direct C++ assertions respectively covering offset/scroll index and
  running physics, initial child positions, and the initial item count.
- `component_test.cpp` cases 5 and 6 point to small synthetic swap/source tests
  rather than their complex upstream fixtures. Case 7 maps a list lifecycle to
  an unsupported silver entry with no executable actions.
- `data_bind_container_test.cpp` cases 9-12 exercise nearby latch/reconcile or
  queue behavior, but do not invoke the upstream container update calls,
  call-count assertions, polled target-to-source path, or dirt-origin sequence.
- `data_binding_blobs_test.cpp` cases 1-3 test adjacent bindable/wrapper/import
  behavior rather than the upstream store/apply/id-only flows; case 5 points to
  an unsupported silver entry with no actions. Case 4 is accepted because it
  executes the directly-set empty-blob behavior.
- `data_binding_computed_values_test.cpp` case 2 points to an unsupported empty
  silver action list and omits the eight direct before/after size assertions.

## Methodology implications

Wave A demonstrates that schema completeness and resolvable symbols are
necessary but not semantic proof. The shard's `max_pending: 0` ratchet and its
declared `direct`/`adapted` statuses currently overstate the evidence by 79
cases.

Future promotion must apply these rules per upstream case:

1. Compare the executable fixture, actions, owner, and assertions, not names or
   nearby coverage.
2. Let one Rust test cover multiple rows only when that body actually executes
   every row's behavior. Parameter arithmetic outside the upstream owner is not
   equivalent coverage.
3. Require expected-red bodies to reach the concrete missing seam after live
   translated work. Raw source and inert action descriptions remain `pending`.
4. Accept a silver replay for the silver behavior it performs, but separately
   translate any direct C++ assertions outside that stream. An empty action list
   cannot prove a case.
5. Record approved backend or Rust-safety adaptations explicitly, without
   allowing them to substitute a different behavior or owner.

## Evidence runs

- The 259 manifest rows were enumerated against the pinned upstream checkout;
  every upstream ordinal/name and every Rust evidence path, line, and symbol
  resolved. This was followed by the semantic body comparison summarized
  above; locator resolution alone was not used to accept a row.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test
  upstream_wave_a_expected_red -- --list` compiled successfully and discovered
  all 18 ignored raw-source anchors.
- Running `component_list_case_01_direct_port_expected_red` with `--ignored
  --exact` failed at the common `pending_literal_port` panic with exit 101,
  before constructing its upstream fixture or executing any case action.
- `CARGO_INCREMENTAL=0 cargo test -p silver-corpus --test wave_a -- --list`
  compiled successfully and discovered all 30 Wave A silver entry points.
  Their acceptance was adjudicated case by case from the referenced corpus
  action stream and any additional assertions in the upstream C++ body.

## Required correction

Promote only the 180 accepted rows. Reset the 53 incomplete/narrow rows and 26
metadata/action-only rows to `pending` / `unverified`, or replace each mapping
with an executable translation (or live differential) before promotion. The
26 provenance records may remain alongside the pending rows, but cannot satisfy
the ratchet.

This review does not request production or test fixes. It establishes the
semantic acceptance boundary that later Wave A correction work must meet.
