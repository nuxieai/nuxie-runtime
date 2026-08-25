# Wave A final residual correction

Status: **PENDING FRESH FULL RE-REVIEW**

This correction is scoped to the 18 rows rejected by the first Wave A review:

- `clip_test.cpp` case 2;
- `data_bind_container_test.cpp` cases 9-12;
- `data_binding_artboards_test.cpp` cases 1, 2, and 5-10;
- `data_binding_blobs_test.cpp` cases 1-3 and 5; and
- `data_binding_computed_values_test.cpp` case 2.

No other Wave A row was re-adjudicated. The pre-existing one-line location
correction for `child_iterator_test.cpp` remains untouched.

## What changed

- The clip case now imports `artboardclipping.riv`, selects `Center`, advances,
  verifies both authored origins, draws the retained world path, checks all four
  framed points, disables `frameOrigin`, updates, draws again, and checks all
  four origin-space points.
- The four container cases now exercise the retained DataBind and container
  queue owners with the pinned scheduling, update/poll call counts, and complete
  dirt-origin assertion sequence. Case 10 is expected-red only after its exact
  `ToTarget` fixture reaches Rust's rejected target-to-source call.
- The eight artboard cases now execute imports, artboard selection, view-model
  construction and binding, state-machine advances, real renderer draws, local
  and cross-File Artboard assignments, trigger/pointer actions, and subsequent
  frames. Each expected-red body stops at the first concrete missing graph,
  bound-view-model, nested-occurrence, or SRIV-comparison boundary; enum values
  and string checks no longer stand in for actions.
- Blob cases 1-3 now run the actual retained instance-property and DataValue
  owners through store/swap/clear, live-value apply, and id-only apply flows.
  Case 5 performs every internal/external advance and draw, installs the live
  external blob, checks identity and byte count, then stops at the unavailable
  pinned SRIV comparator.
- The computed-image case executes the complete fixture, two initial advances,
  every animated frame, every draw, and all eight pinned number assertions. It
  is expected-red at the first real discrepancy: Rust reports initial
  `img1Width == 0` where pinned C++ accepts approximately `150`.

## Focused evidence

- Clip complete port: pass.
- Blob production-owner cases 1-3: 3/3 pass.
- Container cases 9, 11, and 12: pass; case 10 reaches its documented
  expected-red assertion.
- Computed-image case reaches the documented live `0` versus `150` assertion.
- Blob case 5 executes its complete live action stream and reaches only the
  missing SRIV comparator.
- Both new `nuxie` integration-test targets compile. Individual ignored reruns
  of the final four corrected artboard cases reach, respectively, the missing
  nested graph for global 7, the pinned SRIV comparator, the missing bound
  ViewModelInstanceArtboard setter, and the missing nested-occurrence query.
- `wave-a.json` parses as JSON and every one of these 18 evidence symbols and
  line anchors resolves at the recorded location.

This receipt does **not** accept Wave A. A fresh reviewer must re-evaluate the
complete shard, including these bodies and all previously accepted rows.
