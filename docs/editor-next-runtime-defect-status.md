# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: one quarantined Scene diagnostic, one landed additive Scene seam,
  a closed Apple artifact family, closed historical WebGL2 support-matrix and
  variable-font stale-oracle evidence, qualified supported-WebGPU stale-golden
  evidence awaiting user decision, parked authored-shader diagnosis, and
  deferred post-port verification;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `b1f58004332a73564ffdd9f8585838209604c4d1`;
- Editor's last consumed runtime:
  `e72323c808b91d706ba3b745396beaca7accd69a`;
- rows: 25 defects plus the reserved `LOC-010` tombstone;
- closed rows: `RT-ED-001`, `RT-ED-002`, `RT-ED-003`, `RT-ED-004`,
  `RT-ED-006`, `LOC-003`, `LOC-004`, `LOC-006`, `LOC-011`, `LOC-013`,
  `LOC-014`, `LOC-015`, `LOC-016`, `LOC-017`, and `LOC-019`;
- open rows: 10;
- state counts: 15 `closed`, 3 `intake-needs-evidence`, 3 `mapped`, 1
  `regression-reopened`, 1 `stale-oracle`, and 2 `reported`;
- formal/structured product children in the landed Editor snapshot: 11;
- candidate-linked product children: 10;
- union: 21, with no formal/candidate overlap;
- correction rows: 12.
- fixture rows: 25 total, with `RT-ED-001`, `RT-ED-002`, `RT-ED-003`,
  `RT-ED-004`, `LOC-003`, `LOC-011`, `LOC-013`, `LOC-014`, `LOC-015`,
  `LOC-016`, `LOC-017`, and `LOC-019` directly qualified.
- supported browser backend: WebGPU only, landed in runtime PR #47 at
  `95027109c89f651835c76646ebf4d8734f032f07`.
- latest control-plane landing: retained-owner correction PR #80
  rebase-merged at exact runtime main
  `22ba401a9f734eafe0fa3a5852e960e47a4c6121`; historical clip/font
  evidence closeout PR #78 remains the previous recorded evidence landing at
  `98bf5de1f9dfc5d280d29d295dcdc4e418f74c9b`;
- no active control-plane writer: the retained-ViewModel boundary is
  corrected and the historical WebGL2 and variable-font rows have complete
  no-production-change closeouts. The next control-plane action is the
  explicit user decision for `LOC-012`; do not close it before that decision;
- landed Scene-owned repair: `LOC-018` PR #66 exact head
  `2707280cb3507f8d5c2f48cfe58f1cf0990e9ed0` rebase-merged at
  `d7cef0a8b80411b8ef16bf8b48452ea42f71fbe3`. It covers all four concrete
  pinned C++ interpolators (`CubicEase`, `CubicValue`, `Elastic`, and
  `Scripted`) with exact fields, defaults, presence, and semantic ScriptAsset
  ordinal mapping. Its claim is limited to stable typed 409/420 hierarchy and
  the current
  Editor stream's +60 `LayoutComponent` plus +60 `LayoutComponentStyle`
  records (410 -> 530). The remaining ten product records and retained product
  traversal/order are Editor-owned; layout execution, dirt, pixels, and final
  product closure remain deferred post-port verification;
  the changed committed inbox record remains `intake-needs-evidence` because
  it names producer `38f5170f`, runtime pin `e72323c8`, and Journey acceptance
  `a2dbcd2c`, but does not separately label one full Editor assembly SHA.
  `P08-C06` is now its only structured formal child; the implementation is
  merged, but runtime execution and pixels cannot promote at this boundary;
- deferred FL-D owner family: `LOC-001`, with `LOC-002` and `LOC-005` as
  duplicate acceptance cases. The quarantined Scene-only candidate proved
  that one retained ViewModel-instance handle and generation-matched
  materialization are necessary. Migrating a
  `RuntimeOwnedViewModelInstance` schema in place while preserving compatible
  cells, child/list edges, aliases, and dependents maps to FL-D
  `viewmodel.owner`. The exact pinned-C++ image audit disproves the candidate's
  second premise: live `File` asset catalogs do not grow. Each
  `ViewModelInstanceAssetImage` instead owns a private retained `ImageAsset`,
  and `DataBindContextValueAssetImage` falls back to it when an immutable file
  ordinal does not resolve. The original `LOC-001` fixture authors both image
  assets before initial materialization, so the reported family waits only on
  FL-D `viewmodel.owner`. A distinct source-level dynamic-image acceptance is
  retained separately below for an identical post-FL-D C++/Rust/Editor
  differential, or an evidence-backed Editor-not-applicable disposition; it
  is not part of LOC-001 and is not yet promoted to a confirmed defect.
  Candidate
  `dcccdf4fb09275783f6910e5a4a01c028f2c817e` plus uncommitted correction
  diff SHA-256
  `5477057e14eab86a2d0b2b7c5e8e95e2c837bfa33624fa43c6dee9f24aeef981`
  are diagnostic only. The fallback remounted live owners and rejected
  previously accepted schema edits, and its append-only catalogs/tombstones
  invent lifecycle absent from C++, so it is quarantined; no repair landed,
  no writer is active. `LOC-001/002/005` rerun after FL-D `viewmodel.owner`;
- closed Apple artifact family: `LOC-015`, `LOC-016`, and `LOC-017` qualify
  exact runtime identity
  `0.2.0@b1f58004332a73564ffdd9f8585838209604c4d1`, Editor correction
  `233552c13929b09666a62ddff541eb8620d1882b`, and qualification-only iOS
  consumer `f9528fe4295de0a55d121fd7e5290374b22f03c5`. Artifact run
  `5ef5769f-d521-4471-8b91-b9f83acdd065` passed all six sentinels, the
  nine-screen Metal corpus, signed GPU canvas, 28 named animations at
  start/quarter/end, behavior, archive, and framework validators. Clients
  bind runtime version plus source revision; no separately versioned ABI,
  public publication, or iOS-main consumption is claimed or required;
- closed Editor-owned empty-text defect: `LOC-011` retains the independent
  pinned-C++/Rust proof that explicit empty text stays empty and draws no
  glyphs, while reviewed Editor fix
  `fc1a7e406494ee970bd93e456d1f5cfae468bfd4`, landed tree-identically as
  `3bc62bf82ac7d8518e89d093b46f92c727c5af7a`, repairs the actual lowering
  omission. The unchanged browser page reports both prices empty, with
  inspection SHA-256
  `7c9a264d6803d9729197f3cce89d04192e0bd55c386558011be1aeb8e4b89be2`;
- closed historical WebGL2 support-matrix row: `RT-ED-004` preserves the
  original 402×874/radius-57 four-cubic fixture, frame asset, focused command,
  and exact
  `frame.surface.finish_failed: unsupported renderer feature: WebGL2 clip layer allocation or path construction`
  failure as true historical evidence. The user decided on 2026-07-24 to
  remove WebGL2/FemtoVG/fallback support and require WebGPU; runtime
  `95027109c89f651835c76646ebf4d8734f032f07` is therefore a support-matrix
  landing, not a WebGL2 repair. At current runtime
  `e494995cb941fc8fd74ea8a7395a6ba3402c1fa1`, hosted run `30217608092`
  proves the same-runner corpus 1,468/1,468 exact with 1,375 byte-exact and
  zero divergences. `gm-clippedcubic2`, `riv-circle_clips` frame 0, and
  `riv-clip_tests` frame 0 are zero-delta in both final modes; browser
  `gm-cliprects` is zero-delta; unchanged Editor P04-C01 is 21/21. No live
  mechanism, child, parity repair, or writer remains;
- closed variable-font stale oracle: `LOC-013` now has a durable fail-closed
  driver over the exact 879,708-byte Inter font (SHA-256
  `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`),
  face 0, size 17, line height 22, and `wght` 400/500/600/700. Pinned C++ and
  Rust match all 64 glyph IDs and 1,507 outline commands; maximum advance
  drift is `5.1e-7`, maximum outline-coordinate drift is `3.1e-8`, and the
  four weight outline hashes remain distinct. The generated 880,306-byte
  typed `.riv` has SHA-256
  `121965b51165b5ed6198189236fc992d5cd1013665c442bad3a42172a43efcf8`;
  both 38-line renderer programs retain identical order/resources, and C++
  Dawn plus Rust wgpu produce the same 240×112 MSAA PNG SHA-256
  `8e54706fb740e462e58046a9b396cb535e335c454a1c1d06b2a6a814c8662287`
  with zero differing pixels. The retired WebGL2/old-baseline pixels are an
  Editor-owned stale oracle, not a current runtime defect; `P08-C08` remains
  historical candidate linkage, with no production touch set or repair SHA;
- open Editor-owned supported-WebGPU stale-golden evidence: `LOC-012` is
  qualified at `stale-oracle`, but closing it requires an explicit user
  decision because the expected image changed. At Editor checkpoint
  `3a16e76c6f8461c573afff278176302bff5b08b1` on runtime
  `ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`, the unchanged required-WebGPU
  visual/spacing gate passes 2/2 in 2.7s with the reviewed image containing
  the authored `#13253d` background, border, and clip-radius details instead
  of the prior white-background expectation. `P19-C08` remains a candidate
  link until the decision.
  The former 882,146-byte artifact
  `563da6e08c413f76eb1b728ce2d998098ae7ec1fada9e383daa5f44bb6973d16`
  cannot be regenerated exactly; COR-07 remains open for its missing
  backend/mode/surface/capture provenance, so no pinned-C++/Rust parity or
  closure claim is made;
- active production-repair lane: `LOC-009` is a confirmed physical
  shader-module error-scope defect requiring a new production landing; it is
  `regression-reopened`, with PR #54 / `7f1450dc` retained only as historical
  evidence, and is parked and frozen outside the shared tracking merge line
  until diagnosis can resume in a different reliable execution/model
  environment; replacement task
  `019f9f59-1ac6-7e32-b973-5deb6b457c05` ended without authoritative output;
- deferred/parked production lane: malformed embedded-font outline crash and
  its existing PR #60 writer at head
  `61d5d018aa036882d17cea1065a78d7f2e057547`; do not finish, rebase, or land it
  from this queue;
- deferred post-port verification: the completed `RT-ED-007` seam report
  proves correct Scene-emitted property 158 / source path `[0,0,0]` bytes and
  localizes the first divergence to
  `runtime_transition_duration_bindings` in the narrow `state_machine.rs` /
  `state_machine/bindables.rs` seam. This record makes no direct Runtime Fix
  request, schedule, or active writer lease; after the relevant state-machine
  port wave lands, Defects Fix reruns the unchanged set → fire → `advance(0)`
  acceptance and classifies it resolved or still open;
- current `RT-ED-007` proof: no committed SHA contains
  `bind_transition_duration_source`; the recovered e723 producer plus dirty
  `scene.rs` patch SHA-256
  `16492cda16a2f91da7d612c9348c6cca572b294d0d25b782c42ab686904ef57a`
  emitted exact 323-byte artifact SHA-256
  `b8e1696a3166959ab7afbca6d7e8ba4abaf99c9e04a15f144327699ce54ebe70`
  and normalized-dump SHA-256
  `aa199f8e58050272016865f24fd0792375ddddc0c48da83b236db282ef30fcf4`;
  fe0 drops the nested default and produces instant opacity 0.8, while pinned
  d788 produces 0.200000003 at `advance(0)` and 0.5 after another 0.5 seconds.
  Non-main `dd3be99c` appears to implement the seam but is not an ancestor of
  fe0/main; the uncommitted Scene patch has no landing claim;
- defects closed since the preceding Q0 report: 10;
- completed evidence dispositions in binding order: `LOC-006` on main, then
  batch rows `LOC-014`, `LOC-011`, `RT-ED-003`, and `LOC-019`, followed by
  Apple artifact rows `LOC-015`, `LOC-016`, and `LOC-017`, then historical
  support-matrix row `RT-ED-004` and variable-font stale-oracle row
  `LOC-013`;
- parked repair lane: resume `LOC-009` diagnosis only in a different reliable
  execution/model environment, then assign any production repair after a
  fresh coordinator review; do not close or consume the row without a reviewed
  new production SHA;
- current `LOC-009` proof: a temporary uncommitted real-GPU probe on exact
  `fe0a0a07` / tree `4512e0d7` returns `Ok` while Metal reports an uncaught
  max-bind-groups shader-module validation error; the 2,129-byte local log has
  SHA-256
  `93ecaae76c5bfd6252e5fb919087215a1c60a397dd5cfb9a8bc8bf64929b5611`;
- current `LOC-009` browser dependency: the canonical path crashes at
  `luaG_indexerror` / `luaD_throw`; this is the sole authoritative browser
  observation, and cause remains under investigation. Replacement task
  `019f9f59-1ac6-7e32-b973-5deb6b457c05` ended without authoritative output;
  the row is parked and frozen and has consumed nothing in this cycle;
- deferred post-port verification list (none is an active implementation
  assignment, request, schedule, or writer lease):
  - `LOC-007` retains
    `CARGO_INCREMENTAL=0 CARGO_HOME=/private/tmp/nuxie-editor-cargo-home bash tools/nuxie-editor-next/scripts/cargo.sh test -p browser-host --test product_host command_authored_resize_ --offline -- --nocapture`.
    Pinned d788 expects width/height callbacks to propagate
    `ParametricPath -> Path -> Shape -> PathComposer` dirt and progress
    geometry from 96×44 to 160×68 rather than retain one static hash.
  - `LOC-008` retains
    `CARGO_HOME=/private/tmp/nuxie-editor-cargo-home rustup run stable cargo test --manifest-path tools/rive-compiler/scene-shared/Cargo.toml -p nuxie-scene-compiler --lib document_lowering::tests::lowers_list_alias_projection_value_to_a_name_resolved_text_run_binding --offline -- --exact && PAGE_PARITY_ASSERT=1 pnpm --dir apps/nuxie-dashboard run test:visual:page --grep 'Real-Data Paywall / Paywall'`.
    The pinned-C++ shaper expectation is exact intrinsic width and multiline
    height: the 354-wide subtitle occupies 47.59375 over two lines, and
    intrinsic labels do not retain the 180-pixel fallback. At checkpoint
    `233552c1` on runtime `e72323c8`, the unchanged page is red by `166,969`
    pixels after the empty-value correction. The row is
    `intake-needs-evidence` because its changed source record does not
    separately label a full Editor SHA; this does not authorize a writer.
  - `LOC-018`'s remaining assembled ordinary-layout/TextStyle execution retains
    the exact `P08-C01` command:
    `pnpm --dir apps/nuxie-dashboard run test:visual:style && PAGE_PARITY_ASSERT=1 pnpm --dir apps/nuxie-dashboard run test:visual:page && PLAYWRIGHT_START_WEB_SERVER=1 NUXIE_PLAYWRIGHT_WORKERS=1 pnpm --dir apps/nuxie-dashboard run test:editor outline-visual-consistency.spec.ts unit-conversion-matrix.spec.ts safe-area-env-guide.spec.ts design-token-layout.spec.ts && NUXIE_EDITOR_PERF_REQUIRE_HEALTHY_ENV=1 NUXIE_EDITOR_STREAMING_CAPACITY=1 PLAYWRIGHT_START_WEB_SERVER=1 pnpm --dir apps/nuxie-dashboard run test:editor:perf`.
    Pinned C++ expects property-key-driven
    LayoutComponentStyle/TextStyle changes to dirty and reflow the same
    retained layout. This is separate from the already landed and consumed
    `RT-ED-005` generic number/color authoring primitive.
  - unreported FL-D dynamic-image source acceptance: pinned d788
    `file.cpp:310-355,1423,1492-1498`,
    `viewmodel_instance_asset_image.cpp:13-62`,
    `context_value_asset_image.cpp:13-48`, and
    `data_binding_images_test.cpp:179-233` keep the imported file catalog
    fixed while one image-valued ViewModel property privately swaps a decoded
    image and the same ViewModel instance, state machine, and artboard draw it.
    Rust currently stores only `AssetImage(u32)` and resolves only the file
    ordinal. After FL-D `viewmodel.owner` plus `databind.context`, build and
    run one identical C++/Rust/Editor dynamic-image stimulus, or record an
    evidence-backed Editor-not-applicable disposition, before classifying this
    source-level risk as resolved or a confirmed new defect. It is not an
    Editor-reported row, current dependency, implementation request, or writer.
  - `RT-ED-007` retains exact artifact SHA-256
    `b8e1696a3166959ab7afbca6d7e8ba4abaf99c9e04a15f144327699ce54ebe70`
    and unchanged
    `CI=1 pnpm --dir apps/nuxie-dashboard exec playwright test -c playwright.published-rive-conformance.config.ts --grep 'runtime-viewmodel-contract' --workers=1`.
    Pinned d788 produces opacity 0.200000003 at `advance(0)` and 0.5 after
    another 0.5s; fe0 keeps source=`None` and produces instant opacity 0.8.
  After each corresponding formal port wave lands, Defects Fix independently
  reruns each existing registered acceptance unchanged and classifies it
  resolved or still open; the unreported dynamic-image source risk first
  requires the three-layer differential described above;
- post-port verification escalates only for an actual simultaneous file-writer
  collision or a safety/data-loss issue, never merely because a formal port
  wave is in progress;
- `LOC-002` and `LOC-005` are mapped duplicate acceptance cases for the active
  `LOC-001` FL-D owner family. They have no separate qualification or writer
  lane, and none of the three has an active production writer.

Defects Fix owns intake, triage, pinned-C++ qualification, faithful repair
orchestration, independent verification, PR/landing tracking, and immutable
downstream handoff evidence for the complete Editor-reported queue. Editor may
merge before the queue is empty, and Editor consumption is not required to
close a verified landed repair. A genuinely non-port repair may be delegated
to a sole external owner, while a port-covered finding remains a tracked
formal-port-wave dependency with no implementation request, schedule,
assignment, or writer lease. Neither closes until the relevant landing or port
wave passes the unchanged independent acceptance.

The FL reservation in the atlas remains deliberately conservative after Q0.
No runtime, renderer, Scene, state-machine, Editor product, or compiler file is
authorized by this provenance-only slice.

The current textual FL handoff supersedes the older lease snapshot without
mutating that atlas table in this control-plane follow-up: FL-A is
independently promoted on `levi/fl-a` at
`f86d5ba0146697abc996310c62fa45e1f053144b`; FL-B production is blocked on the
recorded pre-advance `LinearAnimationInstance::didLoop` user safety/API
decision. Defects Fix's duplicate stable-Apple branch was canceled; Runtime
Fix owns that mechanical repair. Therefore all listed
runtime/graph/ledger reservations remain binding until the coordinator
publishes a new lease after that repair and decision.

The immutable Editor checkpoint records the completed WebGPU-only consumption
through runtime `e72323c8`: `P14-C01` is 4/4 green, `P14-C06` is 17/17 green,
and RT-ED-003 direct presentation is consumed. This intake does not use those product
results to self-promote an atlas row; independent state promotion remains a
separate step.

The stale conditional-visibility report is now independently closed without a
production change. At Editor checkpoint `7ca11e33` and runtime `e72323c8`, the
exact no-hover diagnostic passed 1/1 in Chromium. Its five frames
(`post-write-no-hover-1` through `-4`, then `post-capture-no-hover-5`) all stay
at draw count 30 with identical timing-stripped frame hashes, no red pixels in
any compositor capture, zero probe errors, and no rejected maps, device loss,
or uncaptured errors. Draw count 34 returns only after the separate
`hoverAt`/`clearHover` gesture. The fresh 70,947-byte run log has SHA-256
`1a9b91fcb8a64296a4c464ad1848be839e9bc91da8d9dfdef337707a0a09f328`;
the committed 171,784-byte machine report has SHA-256
`f78b93d7575c3543e57de49bd73dce5783648b4c5a258328cfdd1f5eeb2652b5`.

The same committed ledger has 11 unique structured runtime children, plus ten
candidate-linked children, for 21 unique affected children. `P08-C06` is now a
formal child of `LOC-008` and `LOC-018`; `LOC-011` has no active child after
its Editor fix, while open `LOC-012` retains the `P19-C08` candidate link
pending explicit user decision. Closed `RT-ED-004` retains only historical
WebGL2 evidence, and closed `LOC-013` retains `P08-C08` only as historical
candidate linkage.

## Defect inbox

The committed Editor repository is the mailbox:

- canonical branch: `origin/levi/editor-next-cutover-assembly`;
- inbox: `plans/nuxie-editor-next-runtime-defects.md`;
- linkage: `plans/nuxie-editor-next-parity-ledger.json`;
- last consumed checkpoint:
  `233552c13929b09666a62ddff541eb8620d1882b`;
- newest known checkpoint:
  `233552c13929b09666a62ddff541eb8620d1882b`;
- inbox SHA-256:
  `24e78816d3bafdd61903e4ea1b36ecb77e946accff847963b2ab886d9530b2ae`;
- linkage SHA-256:
  `07d345c82b8dfd18a06201f08726bafd233f13eabd3cca16c3a8d833f8759226`;
- unconsumed inbox records: 0;
- imported atlas rows: 25;
- changed records consumed at this boundary: `LOC-008`, `LOC-011`, and
  `LOC-018`.

Intake runs only after the current control-plane or scheduled batch reaches a
merge/block boundary. Missing record evidence becomes
`intake-needs-evidence`; it does not trigger chatty Editor coordination or
preempt active repairs. After reconciliation, the dependency/file-ownership
DAG is rebuilt and disjoint lanes refill available capacity. Dependency and
landing handoffs route through the coordinator; Defects Fix does not task or
poll Editor Fix directly.

Complete schema-v2 records use role-labeled column-zero, top-level bullets for
the Editor SHA, runtime SHA, exact command/reproducer, and result/evidence.
The checker accepts only the enumerated original and current committed inbox
labels; a combined Editor/runtime checkpoint needs two distinct SHAs, and an
unrelated continuation SHA or fixture code span cannot fill a missing role.

The recorded newest checkpoint is the last boundary observation, not a live
poll. The v2 checker proves both recorded commits belong to the canonical
local branch, hashes both committed inbox files, binds atlas IDs and ledger
links to their source records, and derives the unconsumed count. The scheduler
fetches the canonical branch only at a boundary; program completion requires a
fresh fetch, exact tip equality, and zero unconsumed records.

## Editor source snapshot

The last consumed Editor snapshot at intake cycle 4 is
`233552c13929b09666a62ddff541eb8620d1882b`. The pinned source checkout and
committed blobs used by the checker resolve to that exact SHA, its runtime
gitlink is `e72323c808b91d706ba3b745396beaca7accd69a`, and the three recorded
source artifacts match the commit byte-for-byte. This statement does not claim
that the canonical remote branch is still at the intake-boundary SHA; a later
tip is fetched and reconciled only at the next explicit intake boundary.

The landed snapshot hashes are:

- proposal:
  `905bf599f2058828e678bff118261a60fdda4a1a09f4557693b7247409b5beb9`;
- runtime defects:
  `24e78816d3bafdd61903e4ea1b36ecb77e946accff847963b2ab886d9530b2ae`;
- parity ledger:
  `07d345c82b8dfd18a06201f08726bafd233f13eabd3cca16c3a8d833f8759226`.

The earlier reviewed hashes remain in this file's Git history, but their formal
dependency map is stale and must not be used for qualification. Any later
artifact change makes the source-root check fail until a newly reviewed Editor
checkpoint is recorded.

The current checkpoint consumes runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. Producer checkpoint
`f9d798dd3b1f9b2dfdbeb74dcdf4485aae4519f6` emits target-0 WGSL plus
target-16 `BindingMap`; its exact one-UBO inner RSTB is SHA-256
`546517d0dc9fbdaf9585f3daa6e440628e62292d7cb8aa7253fd3019aa35713d`.
That producer checkpoint does not replace the immutable three-artifact source
snapshot above.

## Executable checks

Run the standalone checker tests:

```sh
python3 tools/editor-next-runtime-defects/test_check.py
```

Run the landed-snapshot atlas check:

```sh
tools/editor-next-runtime-defects/run-check.sh
```

The check is provenance-valid only while the source files retain those exact
hashes and the Editor checkout resolves the recorded checkpoint. Never use a
hash override.

`RT-ED-001` (`data_viz_demo`) and `RT-ED-002` (`db_health_tracker`) are closed
as stale observations after a focused current-pin scripted comparison produced
two exact entries and two exact segment streams with zero divergences. The
pinned C++ runner SHA-256 is
`b20b815c9f3fe30223b0c93ed9b162c0ec1f9031fc0001490d094bb006516a0b`.

`LOC-003` is also closed, but for a different reason. A pinned-source audit of
`include/rive/listener_type.hpp`, `src/listener_group.cpp`, and the state
machine pointer entry points found no held-duration or timed long-press
primitive. Rust already mirrors that listener vocabulary. Per the user's exact
C++ parity decision, adding a Rust-only timer would be a new product feature,
not a port repair; the Editor compiler therefore continues to fail closed for
the unsupported duration and its fully qualified regression test passes 1/1.

`F-ED-06` / `RT-ED-003` began at source baseline
`bc139955c7e2d30d9cf611dd14c24606fd13520a`. PR #55's final head
`a1c56b5a80c88db4f6cee6550795b6e242394c46` rebase-merged at
`e72323c808b91d706ba3b745396beaca7accd69a`; those commits have the same
tree. Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes that merge. The committed
browser proof records `getCurrentTexture=1`, `mapAsyncRead=0`, and
`putImageData=0` for every measured ordinary ProductHost presentation, while
explicit capture records `getCurrentTexture=0` and `mapAsyncRead=1`. The
product-host proof, static readback audit, and unchanged normal-timeout
device-frame drag gate are green, including the focused drag result 1/1.
Independent promotion confirms ordinary surface acquisition=1/MAP_READ=0/
putImageData=0, explicit readback surface acquisition=0/MAP_READ=1, Lost
recovery acquisitions=2/surfaces=2, persistent Lost typed and bounded,
renderer 418 pass/40 ignored, and corpus 1468 exact/837 byte-exact/0
divergent. The atlas row is closed; `P19-C03` consumption remains downstream
evidence rather than a repair prerequisite.

`F-ED-03` / `RT-ED-005` is classified as an API-surface gap, not a
low-level runtime defect. PR #49's final head
`f0bd914fbac1fd4cf82814216f2ddc88c3e32083` rebase-merged at
`08286481b4e7420768f625f901a944f313b84903`; those commits have the same tree.
That landing includes production commits
`4eec745b704e9920f67098138963dc973e7b2d87` and
`e2d274d8d3b8de3af705d18506a6d48eadebfc0c`, which port the pinned C++ generic
`DataBind.propertyKey` authoring contract into Scene while leaving the
FL-owned runtime mechanism unchanged. They add typed number/color binds,
converter-free direction selection, stable `LayoutComponentStyle` targets
for all four padding keys, exact target/property collision identity,
converter output validation matching C++ `Input`/`None`/`Any` semantics, and
encoded `File::import` behavior tests. Independent review found the missing
converter-free direction surface; the follow-up now proves numeric
`ToSource`, numeric source-first `TwoWay`, and color source-first `TwoWay`
through exact re-import and reverse propagation. Clean Editor checkpoint
`233552c13929b09666a62ddff541eb8620d1882b` consumes descendant runtime
`e72323c808b91d706ba3b745396beaca7accd69a`, including the generic
number/color paint primitive for existing `Stroke` and `SolidColor` targets.
Historical executor evidence remains green, but the changed committed
`RT-ED-005` inbox record is `intake-needs-evidence`: it does not separately
label one full Editor SHA and one full Runtime SHA, so this intake cycle cannot
promote the row yet. `P09-C01` is green for the generic property-target
primitive and remains a nonblocking Known Runtime Defect only for the separate
`LOC-002` retained-owner behavior. Ordinary layout padding and TextStyle
font-size/line-height remain under `P08-C01` / `LOC-018`; their post-port
dirt/reflow acceptance is not required to close the landed `RT-ED-005`
implementation.
Its executor battery was green:
the 721-test probe-armed workspace suite, 317/317 ordinary and
scripted golden entries with 647/647 segments each, 1468/1468 renderer rows,
CAPI, Apple, frame-loop, B-6, lint/format/diff, WebGPU-only browser, and the
8.74 MiB maximum SDK floor all passed.

The transferred Editor report cited historical C++ pin
`f4bb3025e263ad1a646ef6971358577a0aa6bfa2`. It is retained as provenance,
not silently treated as the current oracle. The relevant source set changed
before `d788e8ec6e8b598526607d6a1e8818e8b637b60c`: the current pin adds
generated property notifications, target observation, and explicit
reconcile-origin handling. `COR-01` therefore requires the F-ED source hashes,
fixture, executable probe, and behavioral assertions to use `d788e8ec`.

`F-ED-11` / `LOC-019` is landed, consumed, independently verified, and
closed. PR #51's final head
`22454fb58bc80d95174ca78d0c0d4d611b0d5a08` rebase-merged at
`ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`; those commits have the same
tree. Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes descendant runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. Its unchanged required-WebGPU
`P14-C06` command passes 17/17: ProductRuntimeTools source validation, both
retained-session fixtures, single and batch snapshots, and all 12 WebGPU
pixel fixtures reach readiness. Independent promotion verifies real Chrome
clean-null output at 64 pixels/32 red without a device error, invalid WGSL
preserving a concrete error, the full WebGPU matrix green, and the native
corpus 1468 exact/837 byte-exact/0 divergent. No queued hosted Apple lane is
relabeled green.

## Next queue

1. keep the quarantined Scene-only `LOC-001` candidate out of production and
   preserve unchanged `LOC-001` / `LOC-002` / `LOC-005` acceptance for
   independent rerun after FL-D `viewmodel.owner`; keep the unreported
   dynamic-image source acceptance in the deferred list for an identical
   post-FL-D C++/Rust/Editor differential, or an evidence-backed
   Editor-not-applicable disposition, before any defect classification;
2. keep `LOC-012` open at `stale-oracle`, retain P19-C08 and COR-07, and ask
   for the explicit user decision on the reviewed expected image; do not claim
   pinned-C++ parity from the non-reproducible historical artifact;
3. keep merged `LOC-018` in `intake-needs-evidence` until its committed source
   separately labels a full Editor assembly SHA, and keep `LOC-008` in the
   same fail-closed state for its missing full Editor SHA; preserve their exact
   evidence and do not conflate downstream product/runtime execution;
4. keep `LOC-009` outside the shared tracking merge line, parked, and frozen
   until diagnosis resumes in a different reliable execution/model
   environment; do not close or consume it without a reviewed new production
   landing;
5. retain `RT-ED-007`, `LOC-007`, `LOC-008`, and `LOC-018`'s remaining
   runtime layout/TextStyle execution as deferred post-port verification only;
   after each relevant formal port wave lands, rerun the unchanged acceptance
   and classify it resolved or still open, with no direct Runtime Fix request,
   schedule, or active writer lease;
6. keep PR #60 and the malformed embedded-font outline crash deferred/parked,
   then refill other disjoint qualification lanes from the reconciled
   ownership DAG.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
