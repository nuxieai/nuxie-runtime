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
`make scripted-rust-golden-runner`, and reading
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

## Required authoritative closeout

After the PR is open, run the clean, serialized performance lane with the
same three corpus IDs and the scripting-enabled release runners. Acceptance
requires all three Rust/C++ ratios below 10x. Only three independent quiet
sessions may tighten the affected `perf-corpus.toml` rows. The ordinary
required signoff must also run the focused scripted golden comparison for all
three IDs so exact draw and side-channel behavior remains pinned.
