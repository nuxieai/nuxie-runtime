# UNIV-1684 extreme-outlier evidence

Issue: [UNIV-1684](https://universe.basis.dev/issue/UNIV-1684)

This slice targets the avoidable roots behind `car_widgets_v01`,
`gamepad_test`, and `script_create_text_runs` against pinned C++ runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The candidate started from Rust
commit `948538f2c4a9ba70a95fdb0a69ba0dbdf1023101`.

## Mechanisms

- `LayoutParticipant::displayValue` is folded into its host's retained collapse
  bit when the style changes, during initial lifecycle settlement, and after
  Solo, ancestor, or LayoutComponent reveal propagation. Draw-time collapse
  checks are therefore O(1), matching pinned
  `LayoutParticipant::syncStyleChanges` and `Component::isCollapsed`, instead
  of searching each owner's children for a participant on every draw.
- `luaur-rt` registers one userdata metatable per VM and Rust `TypeId`, matching
  pinned `lua_register_rive<T>`. Creating another renderer or `Mat2D` userdata
  allocates only its payload and reuses the raw registry metatable reference.
  Dispatch callbacks capture no Lua handles, and the owning VM releases cached
  registry references before `lua_close`.

## Focused validation

The following passed with only pre-existing generated/vendor warnings:

- Layout-participant display folding, parent-collapse preservation, inactive
  Solo preservation, imported `display:none` initial lifecycle, and transition-
  order coverage for Solo switches plus ancestor/LayoutComponent reveals.
- Per-VM/per-type userdata registration reuse, distinct instance field values,
  VM teardown without a strong-handle cycle, and cached field dispatch from a
  borrowed coroutine state via
  `cargo test -p luaur-rt-test-harness --test userdata_metatable_cache`.
- The `luaur-rt` `send` store through the same supported workspace harness via
  `cargo test -p luaur-rt-test-harness --features send --test userdata_metatable_cache`.
- The actual `nuxie-scripting` renderer integration test.
- `cargo check -p nuxie-runtime -p nuxie-scripting`.
- `make b6-audit-check` and the perf corpus/pinned-runtime checks.

## Stack evidence

The before `car_widgets_v01` sample places
`runtime_layout_participant_local` directly below
`runtime_component_is_collapsed_for_draw_component`; the participant lookup
appears 427 times in the sample. In the after sample, no sampled stack pairs
those symbols. The collapse predicate remains as the intended retained-bit
read (56 of 4,037 main-thread samples), while the 50 remaining participant
lookups occur under world-transform/layout work, not draw visibility.

The before `gamepad_test` hot sample repeatedly enters
`create_callback_function` beneath `create_userdata<ScriptedMat2D>` and
`create_userdata<ScriptedRenderer>` during `call_draw`. In the after sample,
`create_callback_function` and metatable registration have zero samples. One
`create_userdata<ScriptedMat2D>` payload-allocation sample remains, which is
the intended per-instance `lua_newrive` equivalent.

`script_create_text_runs` is an acceptance control in this slice, not a claimed
new stack attribution. It was already below the issue threshold in the clean CI
baseline at 8.343x, while `car_widgets_v01` and `gamepad_test` were the two
residual outliers. It remains in every timing and required scripted-golden run;
no mechanism in this change is attributed to an uncaptured script profile.

## Timing status

All runs used scripting-enabled release runners, 100 sequential frames at
60 Hz, five iterations, no warmups, and median aggregation. The three after
captures used C++ first; the local before controls deliberately used both
C++-first and Rust-first orders. The release Rust runner provenance record was:

- source digest: `4f6d045fe2c9a6a93dddf2529a3e8a8b8a181ef7aea32d7e4915e1512420d4e3`
- binary SHA-256: `9f87bbc6648a532b1be04f7cfc4ac7f7c539a74cc4a8227ac3e2129e46c60a32`
- compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`

Here `source digest` is the provenance script's `digest_state`: the SHA-256 of
the sorted workspace/member content-digest record. It and the runner hash can
be reproduced by checking out `74e8fbeb`, running
`make scripted-rust-golden-runner RUST_PROFILE=release`, and reading
`target/golden-gate/scripted-release.json`. Reproduction also requires the
recorded Rust compiler because the digest state includes `rustc --version`.

The repository quiet gate waited its complete 900-second bound, from 13:12:28
to 13:27:28 America/Los_Angeles. The one-minute load was 24 at expiry versus a
threshold of 9. The first capture therefore carried the repository's
`CONTENTION` marker; the next two were explicitly marked at loads 23 and 21.

Despite that contention, candidate Rust medians were directionally lower than
both local before-order controls:

| Fixture | Before Rust total ms, C++/Rust order | Before Rust total ms, Rust/C++ order | After Rust total ms, three sessions | Conservative reduction |
|---|---:|---:|---:|---:|
| `car_widgets_v01` | 299.516 | 287.260 | 243.707 / 261.106 / 251.193 | at least 9.1% |
| `gamepad_test` | 15.111 | 13.568 | 8.672 / 8.373 / 8.600 | at least 36.1% |
| `script_create_text_runs` | 28.199 | 29.139 | 21.546 / 23.795 / 25.397 | at least 9.9% |

The contended after ratios were:

| Fixture | Session 1 | Session 2 | Session 3 |
|---|---:|---:|---:|
| `car_widgets_v01` | 44.601x | 38.959x | 49.890x |
| `gamepad_test` | 14.844x | 15.730x | 14.029x |
| `script_create_text_runs` | 23.323x | 25.264x | 23.034x |

These ratios are diagnostic only. They are not comparable enough to the clean
CI before ratios (20.445x, 14.470x, and 8.343x respectively) to establish the
under-10x acceptance target. `perf-corpus.toml` is deliberately unchanged:
ratcheting requires an authoritative quiet run, and these local numbers must
not widen or tighten a ceiling.

## Durable diagnostic summary

The compact timing and sampled-stack summary is tracked in
`docs/evidence/univ-1684-extreme-outliers-summary.json`. It records the exact
before/after totals, ratios, capture conditions, runner identity, and relevant
stack counts used above. Host-specific raw `sample` output is intentionally not
claimed as durable repository evidence. The stack counts are transcribed local
diagnostics, not independently reproducible evidence; authoritative closeout
does not depend on them and requires a fresh final-tip capture below.

The summary is bound to candidate commit `74e8fbeb`. A later adversarial review
found a transition-order collapse bug, corrected in `bd6fd1f7`. These
diagnostics therefore do not describe the final runtime source tree. They
remain useful attribution evidence, but the corrected tip requires fresh
authoritative golden and performance capture.

## Corrected-tip profiling capture

A fresh release runner was built from corrected tip
`e2dd811814d291db303f5b34951d6020df66eebe` into a checkout-external artifact
root so repository-local target cleanup could not invalidate the capture. Its
source digest was
`9af483becc5ed50cb0c00f4133d70474e8e2fbbf00efc0ae9f7166524d5fc79e`,
its binary SHA-256 was
`16a97d0538105926a5b314eb4d59aeb2aa4585ecae7f3836dcd4a498f4e1a8d9`,
and the compiler remained `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`.

The local quiet gate expired after its bounded wait, so the corrected-tip
ratios remain diagnostic. One external C++-first capture at load 41.42 measured
`car_widgets_v01` 40.199x, `gamepad_test` 6.724x, and
`script_create_text_runs` 18.141x. The gamepad result is below the issue target
even under contention, but none of these numbers may ratchet the corpus.

Exact direct car timings isolated cold initialization: Rust advance was
40.962 ms for one frame, 39.782 ms for two frames, and 43.559 ms for 100
frames. C++ advance was 1.637 ms, 0.771 ms, and 1.135 ms respectively. Thus the
avoidable Rust advance debt is concentrated in the first lifecycle traversal,
not repeated frame replay.

An exact 100-frame Instruments Time Profiler launch produced 53 samples under
`advance_scene_to`; 47 passed through state-machine settlement, and 45 through
nested-artboard host-dirt update. The dominant avoidable leaves were 11 direct
samples in `RuntimeDrawableList::sort_draw_order`, with another five allocation
or proxy samples beneath it, plus four samples in
`clear_redundant_operations`. Static parity audit then identified the dispatch
error: every cloned Component begins `FILTHY`, which includes DrawOrder and
Clipping, while Rust ran the concrete Artboard-wide sort and clipping tails for
every scheduled child. Pinned C++ runs those tails only in `Artboard::update`.
The expected initial pass is therefore one sort and two clipping cleanups per
Artboard, not one/two per dependency node.

The host-specific trace bundle is not tracked, but its exported diagnostics are
content-addressed here:

- Time Profiler export SHA-256:
  `94a23ee88df78890e5deb67112a7f6eca6069f238a5ea0ce6ca3a733385d3107`
- Time Sample export SHA-256:
  `9435ed52eeb52311db0ec8cf4313becaa2e239ee05c5fe0345f164d0e86f0ef1`
- trace table-of-contents SHA-256:
  `be8f313575266c289473bf1a13397db7015d6b8eb45b38b75de8dfe7fac1f891`
- exact launch output SHA-256:
  `78306351ab1e22e5e4873471a3fadabb4bb5f10fe6ce95c55684213bcdbc476e`
- corrected-tip timing JSON SHA-256:
  `21ae1c5eac0c06ad17f31fe0cdae28cab3449fb33c093fb21097555922006979`

## Root-Artboard dispatch gate result

Commit `793d0c6d7c95d9e77de5c5d7bdbce93e702ce7c2` gates the concrete
Artboard draw-order and clipping tails to the root Artboard. Focused coverage
asserts one sort/two clipping cleanups for a root plus two FILTHY children, the
same counts for a mounted nested occurrence, zero calls for direct non-root
DrawOrder/Clipping dirt, and one sort/cleanup for a live DrawTarget property
change that publishes DrawOrder dirt to the root.

The release runner SHA-256 for that commit is
`9d4bc43aa6c007888bd1178375999360d2114ed57f305034ef214191528c36f8`.
The bounded quiet gate again expired under unrelated machine activity, at load
28 versus threshold 9. The resulting ratios are diagnostic only:
`car_widgets_v01` 25.905x, `gamepad_test` 12.087x, and
`script_create_text_runs` 17.812x. Car's Rust advance median nevertheless fell
from the immediately preceding contended 46.128 ms to 32.030 ms.

The exact post-gate car profile contains 35 samples under `advance_scene_to`.
Both `sort_draw_order` and `clear_redundant_operations` have zero samples,
confirming removal of the attributed hot path. Renderer preparation is now the
dominant captured phase: 95 samples under `synchronize_artboard_renderer`,
including 41 leaf samples in `runtime_shape_paint_commands` and 30 leaf samples
in `runtime_live_owned_shape_paint_blend_mode_value`. The latter repeatedly
walks static container ancestry to rediscover an owner already fixed by the
C++ parent topology, and is the next profile-backed narrow target. Paint-command
materialization is recorded separately and is not folded into that owner-lookup
slice.

The post-gate external diagnostics are content-addressed as follows:

- timing JSON SHA-256:
  `6dad515e63a521e4fc495adbeff25655839b8752b1aeb4d3bef9b16692485ff6`
- Time Profiler export SHA-256:
  `a1576e772a318778daaf869d4e69b344ee720d05ba570b79427a09c6b01244be`
- Time Sample export SHA-256:
  `ba426f61e6158e3866a38658cc8f56844e080f29116d9ef3a14539246de167c2`
- trace table-of-contents SHA-256:
  `dfc0c09753d02cad1b133854d895115a3a830e07c44eadd636de60c935851dbe`

## Retained opacity-owner result

Commit `aaa7efb4e7a68f0ac673e5a1bc7e84ee723fddc2` retains each paint
container's fixed opacity owner on its runtime shape during the existing
container-index construction pass. Live inherited blend reads now follow that
O(1) retained index instead of reopening the immutable component graph and
walking static ancestry. Focused coverage proves that every supported paint
container present in the text-style fixture, and its clone, retains the same
owner as the prior static resolver, and that a live inherited blend update
performs zero ancestry resolutions.

The scripting-enabled release runner SHA-256 for that commit is
`37639c25c77fbff5681a4dcc622daa3bb2b8e376682c6cab192027e3adf43d9a`.
The bounded quiet gate expired after 60 seconds at load 25 versus threshold 9,
so the car-only comparator remains diagnostic: C++ median 7.510 ms, Rust median
178.345 ms, and 23.748x. The acceptance target therefore remains open and the
corpus ceiling is unchanged.

The exact 100-frame post-change profile provides the structural result despite
the noisy timing environment. `opacity_owner_local` has zero inclusive and
zero leaf samples. `runtime_live_owned_shape_paint_blend_mode_value` fell from
31 inclusive/30 leaf samples in the immediately preceding exact profile to
1/1. `runtime_shape_paint_commands` remains separately visible at 55
inclusive/42 leaf samples and was not changed in this slice. The earlier
Artboard dispatch roots also remain absent: both `sort_draw_order` and
`clear_redundant_operations` have zero samples.

The post-owner external diagnostics are content-addressed as follows:

- timing JSON SHA-256:
  `6195b70a7832287606fb9a9a2d792ac4f7e6ed583110b578b9546f7a97d21386`
- Time Profiler export SHA-256:
  `a3a1f76400a9c599101fa2080e76de93c300a2ce6675cd4e8880a77554068cd9`
- Time Sample export SHA-256:
  `dff5a9894d5dba1e570e2bb221bb1a835338177afe4b5c1c8cef257df6629720`
- trace table-of-contents SHA-256:
  `5f4913ae1bcf13ea32d42ea58d844a5d62c94af97b89b56b580ba643343cd7b7`

## Required authoritative closeout

After the PR is open, run the clean, serialized performance lane with the
same three corpus IDs and the scripting-enabled release runners. Acceptance
requires all three Rust/C++ ratios below 10x. Only three independent quiet
sessions may tighten the affected `perf-corpus.toml` rows. The ordinary
required signoff must also run the focused scripted golden comparison for all
three IDs so exact draw and side-channel behavior remains pinned.
