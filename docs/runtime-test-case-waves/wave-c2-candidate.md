# Wave C2 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C2 covers the exact 73 Catch cases in pinned upstream files 59 through
66, from `layout_stack_test.cpp` through `listener_align_target_test.cpp`, at
upstream SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This candidate changes
test and evidence semantics only. It does not modify non-test runtime behavior
and does not declare Wave C2 accepted.

## Exact census

- 73/73 cases mapped, zero pending;
- classifications: 73 direct;
- outcomes: 52 pass and 21 expected-red;
- layout stack/layout: 14 pass / 12 expected-red;
- library assets: eight pass / two expected-red;
- line breaking: six pass / six expected-red;
- linear animation definition/instances: 15 pass / one expected-red;
- listener flags/alignment: nine pass / zero expected-red.

## Executable evidence

- Layout cases import the exact assets, advance the live Artboard owner, query
  retained layout bounds/world transforms, mutate the concrete style owner,
  and preserve the pinned assertion order.
- All nine layout Silver rows execute complete pinned action streams through
  the runtime/renderer and compare full SRIV output. The hug-artboard row
  restores its previously empty stream locally and reaches the missing
  computed frame-size seam.
- Library rows use the exact `nuxie::File` asset/artboard/view-model owners.
- Line-break rows decode the pinned RobotoFlex bytes through `RawTextFont`,
  exercise the private production shaper, and pass its concrete shaped indices
  through the production `Font::shapeText` annotation owner. Every pinned
  per-run break array and index is asserted. Paragraph direction uses the
  runtime's own `paragraph_base_is_rtl` owner, while line topology and bidi
  order use concrete `StandaloneLine` owners; no fake Font, raw-source anchor,
  aggregate-count proxy, constant-failure seam, or generic panic is used.
- Animation rows exercise distinct `RuntimeLinearAnimation` and
  `LinearAnimationInstance` owners per upstream case. Fixture-backed quantize,
  missing-keyed-object, and timeline-event rows retain their exact import and
  action flows.
- Listener flag rows route imported actions through the actual listener,
  transition, and state owners and run scheduled occurrences through the real
  executor filter. Listener alignment preserves the exact advance/pointer
  order on both pinned artboards.

All 21 ignored rows were forced individually with `CARGO_INCREMENTAL=0` and
failed at their declared concrete seams: three live layout gaps, nine full
layout SRIV differences, two missing script module names, six concrete
line-break differences, and the immutable animation quantize definition.

## Gates

- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, evidence-locator, and ignore-reason validator: 73/73 green;
- focused pass suites: 52/52 green;
- forced expected-red sweep: 21/21 reached the declared seams;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, and diff checks: green;
- default no-feature non-test LLVM IR contains no Wave C2 test-owner symbols.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0`. The three shared
source edits are late `cfg(test)` module/include declarations in `animation.rs`,
`listener_action.rs`, and `raw_text.rs`; their executable contents are confined
to disjoint Wave C2 test files.
