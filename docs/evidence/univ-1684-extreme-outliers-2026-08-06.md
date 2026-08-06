# UNIV-1684 extreme-outlier evidence

Issue: [UNIV-1684](https://universe.basis.dev/issue/UNIV-1684)

This slice targets the avoidable roots behind `car_widgets_v01`,
`gamepad_test`, and `script_create_text_runs` against pinned C++ runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The candidate started from Rust
commit `948538f2c4a9ba70a95fdb0a69ba0dbdf1023101`.

## Mechanisms

- `LayoutParticipant::displayValue` is folded into its host's retained collapse
  bit when the style changes and during initial lifecycle settlement. Draw-time
  collapse checks are therefore O(1), matching pinned
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
  Solo preservation, and imported `display:none` initial lifecycle tests.
- Per-VM/per-type userdata registration reuse, distinct instance field values,
  VM teardown without a strong-handle cycle, and cached field dispatch from a
  borrowed coroutine state.
- The `luaur-rt` `send` feature branch.
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

## Timing status

All runs used scripting-enabled release runners, 100 sequential frames at
60 Hz, five iterations, no warmups, median aggregation, and C++ first. The
release Rust runner provenance record was:

- source digest: `4f6d045fe2c9a6a93dddf2529a3e8a8b8a181ef7aea32d7e4915e1512420d4e3`
- binary SHA-256: `9f87bbc6648a532b1be04f7cfc4ac7f7c539a74cc4a8227ac3e2129e46c60a32`
- compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)`

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

## Retained local artifacts

The timing JSON and sampled stacks remain ignored under `target/`; hashes make
the evidence exact without committing host-specific profiles:

| Artifact | SHA-256 |
|---|---|
| `target/univ-1684-baseline.json` | `f4c3a6f6e6214ae27f9d03ae2b96452cceb31600b1b364c7793c7c6feed5bf07` |
| `target/univ-1684-baseline-rust-first.json` | `daad44d88dcd48a6138f335ed6b82be1427d38522781bce58c09c2c9b094034c` |
| `target/univ-1684-after-1.json` | `6f4558d918f2709dce40f658429c2de5c489dc0f43a3b2859dde6278aaea3ef9` |
| `target/univ-1684-after-2.json` | `13e51e66f81030cecaae899c63a9969b62355229eb8607d65deb1b1f30dcf5f7` |
| `target/univ-1684-after-3.json` | `8c78096090f130b19696aae2a30d5dccfc7a9411dc27c7ee290c3c2ff75890d7` |
| `target/univ-1684-car.sample` | `f11302b668ef8d9b4fb0e6f5099c15b6d4e538c8802ae809d2ae43f11a2cf17d` |
| `target/univ-1684-car-after.sample.txt` | `41e27f68735b44f03358eeea5c7140aba869da926d4893e6d32b409d4cf6f33f` |
| `target/univ-1684-gamepad.sample` | `67230f7731c390e6b1fb9530654bd3b8ee59f52e2059613689155c997caf7cfb` |
| `target/univ-1684-gamepad-hot.sample` | `a7dbc23a165cde72c21d077d26dcc3ab6195d0202bc8581effb8640d6ed01cef` |
| `target/univ-1684-gamepad-after.sample.txt` | `254e5cab9cb9c999956984c9303b4de415dcde6cc39469d7ccf19830aa8c29f9` |

## Required authoritative closeout

After the PR is open, run the clean, serialized performance lane with the
same three corpus IDs and the scripting-enabled release runners. Acceptance
requires all three Rust/C++ ratios below 10x. Only three independent quiet
sessions may tighten the affected `perf-corpus.toml` rows. The ordinary
required signoff must also run the focused scripted golden comparison for all
three IDs so exact draw and side-channel behavior remains pinned.
