# Wave C1 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C1 covers the exact 62 Catch cases in pinned upstream files 51 through
58, from `image_mesh_test.cpp` through `layout_scroll_test.cpp`, at upstream
SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This candidate changes test and
evidence semantics only; it does not modify non-test runtime behavior and does
not declare Wave C1 accepted.

## Exact census

- 62/62 cases mapped, zero pending;
- classifications: 57 direct and five Rust-safety adaptations;
- outcomes: 45 pass and 17 expected-red;
- files 51-56: 14 pass / six expected-red;
- `layout_participant_test.cpp`: 18 pass / one expected-red;
- `layout_scroll_test.cpp`: 13 pass / ten expected-red.

The five adaptations retain the same externally observable fixture behavior
while respecting Rust ownership: shared immutable mesh-index storage; whole
Artboard occurrence cloning rather than individual heap-object cloning;
immutable graph/instance separation for clipping; immutable animation
definition lifetime; and exclusion of a collapsed Solo child from the retained
Taffy solve rather than retaining a C++ provider pointer for it.

## Executable evidence

- Image mesh, in-band asset loading, instancing, joystick flags, and grid cases
  use the exact pinned assets and live graph/artboard owners.
- Every grid/stack and layout-scroll Silver case executes its complete pinned
  action stream against the real renderer/runtime and compares the resulting
  SRIV operation stream.
- Participant cases preserve exact advance, property mutation, layout-provider,
  group/Solo, component-list, intrinsic-bounds, and world-transform owners.
- Non-Silver scroll cases execute the real imported ScrollConstraint,
  StateMachineInstance, component-list, physics, snap, pre-layout intent, and
  scrollbar owners. The hidden-layout intent row includes its live assertion
  flow in addition to the complete Silver stream.

All 17 ignored rows were forced individually. Every command ran exactly one
test and failed at its declared concrete seam: five grid/stack SRIV
differences; one missing retained grid-line query; missing pre-advance custom
PointsPath intrinsic geometry; eight layout-scroll SRIV differences; the live
hidden-layout index intent retaining offset zero at frame 6; and scrollbar
release retaining `is_scroll_bar_dragging`.

## Gates

- strict pinned identity, ordinal, source-line, exact-name, classification,
  adaptation, evidence-locator, and ignore-reason validator: 62/62 green;
- focused pass suites: 45/45 green;
- forced expected-red sweep: 17/17 reached the declared seams;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped formatting, JSON parsing, and diff checks: green;
- default no-feature non-test LLVM IR contains no Wave C1 test-owner symbols.

Every relied-on Cargo invocation, including all focused pass suites, all 17
forced-red commands, and the non-test IR build, ran with
`CARGO_INCREMENTAL=0`. No result from another lane's incremental build was
used. The only shared-source edit is a `cfg(test)` module declaration appended
at the end of `constraints.rs`, after all previously frozen source locators.
