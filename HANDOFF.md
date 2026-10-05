# Runtime main green handoff

Work is on `codex/runtime-main-green` in `/Users/levi/dev/nuxie-dev/.claude/worktrees/codex-runtime-main-green`, freshly cloned at `51e10355228ebd42eaa77203ae85a9baaeebba4b`. Scope follows `/Users/levi/dev/nuxie-handoff/R1-runtime-main-green.md`, read in full. No tracked or working-tree `AGENTS.md` or `CLAUDE.md` exists in this clone. No other checkout was edited. No push, PR, merge, stash, or history rewrite was performed. Levi owns the push.

Issue: [UNIV-3593](https://universe.basis.dev/issue/UNIV-3593).

## Commits

- `2df4a1e98`: `fix(capi): record modulate_color in ABI v4 layout`. Records the existing field at offset 328 and changes the recorded size from 328 to 336. No other layout value, header, or ABI shape changed.
- `e921289ac`: `fix(renderer): guard debug-only layout assertion in release`. Adds one `#[cfg(debug_assertions)]` to the assertion in `LogicalFlush::pushImageMeshDrawExecutable`.
- `97ad81a2e`: `fix(fixtures): ignore generated listener input RML copies`. Adds two narrowly scoped ignore patterns for four downloaded/generated outputs.
- The documentation commit containing this handoff records evidence only. All compiled source and Android qualification refer to `97ad81a2e`.

Each commit uses Conventional Commits and carries `Refs UNIV-3593` plus the Codex co-author trailer.

## Environment and command execution

Host checks used Rust/Cargo 1.97.1 (the machine's Homebrew toolchain), `CARGO_TARGET_DIR=$PWD/target/green`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=1`, `CARGO_PROFILE_TEST_DEBUG=1`, `RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime`, and `PYTHONDONTWRITEBYTECODE=1`. The debug settings keep host artifacts smaller while retaining debug information for the Apple library linker. `target/debug -> green/debug` was an ignored, local compatibility symlink for the Makefile's hardcoded C/Swift smoke-library paths.

Android used the packager's pinned Rust 1.94.1, cargo-ndk 4.1.2, NDK 29.0.14206865 at `/opt/homebrew/share/android-commandlinetools/ndk/29.0.14206865`, and API 23. `CARGO_INCREMENTAL` and every `CARGO_PROFILE_*` override were removed from its environment. The packager assigns its private target inside this clone at `target/nux-capi-android/build/cargo`; it does not use another checkout's cache.

Before every build/check invocation, the evidence runner executed `df -h /Users/levi` and checked available bytes against 10 GiB, refusing to launch below that floor. No build crossed that launch condition. Initial clone preparation also checked disk. Logs and the exact check-command journal are in `/tmp/runtime-main-green-evidence/`; they are outside the deleted build target.

## Renderer upstream relationship

This is a port correction, not a new divergence. Read-only inspection of upstream `renderer/include/rive/renderer/render_context.hpp:926` found `RIVE_DEBUG_CODE(bool m_hasDoneLayout = false;)`. `include/rive/rive_types.hpp` defines that macro only in debug builds, and upstream `renderer/src/render_context.cpp:3574` uses `assert(m_hasDoneLayout)` in `pushImageMeshDraw`. C++ release `assert` removes the expression; Rust `debug_assert!` still type-checks it. Guarding the Rust assertion with the field's existing cfg preserves upstream's release behavior without changing the field or runtime layout. No sync-upstream divergence entry is needed.

The renderer audit covered other layout/frame/tessellation debug fields, render-buffer map/unmap counters, and raw-path mutation locks. Their accesses already have statement, block, or function cfg guards; no further changes were needed. Native Metal and Android Vulkan release builds exercise the affected source.

## Fixture choice

Ignore the four generated copies rather than commit them. `tools/fetch-test-assets.sh` pins `sync/listener_input_values.rml` to upstream asset commit `abf676e79e616e003893dc941471c90ccbaf5c6e`, verifies SHA-256 `1f8300a80d674178151bdf9d3e9788f53d56be503e9a5e3e250c4a6f91c542ed`, and copies it into `fuzz_import`, `fuzz_runtime`, and `fuzz_pointer` seed directories. Existing `.gitignore` already excludes generated `.riv` downloads and seed copies, with explicit exceptions for committed fixtures. The new patterns follow that convention and cover only this RML file:

- `/fixtures/sync/listener_input_values.rml`
- `/fuzz/seeds/**/listener_input_values.rml`

Untouched main's `make fixtures` succeeded but left all four paths untracked. After the fix, `make fixtures` and an explicit empty-status assertion passed on a clean committed tree.

## Baseline comparison and checks

Baseline means source at `51e103552`, before edits. The baseline Metal compiler emitted E0609 before the fix was applied; Cargo then finished already-running dependency jobs before returning 101. The default-feature release build passed even on main because it does not enable the mechanical renderer. The additional `nux-capi/apple-metal` build is the direct before/after proof for the release defect.

| Run | Exact command | Exit | Outcome |
| --- | --- | --- | --- |
| baseline-layout | `python3 tools/check-nux-capi-layout.py` | 1 | FAIL: record omits modulate_color. |
| baseline-fixtures | `make fixtures; git status --short` | 0 | Command succeeds; status shows four untracked RML outputs. |
| baseline-fmt | `make fmt-check` | 2 | FAIL: 119 files, listed below. |
| baseline-lint | `make lint-gate` | 0 | PASS: counts 27 / 4927 / 350 / 574 / 41. |
| baseline-pr-gate | `make nux-capi-pr-gate` | 2 | FAIL: only ABI layout contract; portable smoke and 54 distribution tests pass. |
| baseline-release | `cargo build --release -p nux-capi -p nuxie-renderer` | 0 | PASS: default features do not compile the affected port. |
| record-layout | `python3 tools/check-nux-capi-layout.py --record > crates/nux-capi/abi-layout-v4.json` | 0 | PASS: only callback size and new field offset differ. |
| final-layout | `python3 tools/check-nux-capi-layout.py` | 0 | PASS. |
| final-surface | `python3 tools/check-nux-capi-surface.py` | 0 | PASS: runtime/platform-only shipped surface. |
| final-fixtures | `make fixtures && git status --short && test -z "$(git status --porcelain --untracked-files=all)"` | 0 | PASS: regenerated assets and empty git status. |
| final-android-contract | `make nux-capi-android-contract-test` | 0 | PASS: 25 tests; build and publish plans only, no publication. |
| final-capi-tests | `cargo test --locked -p nux-capi` | 0 | PASS: 117 passed, 0 failed, 0 ignored across 13 result groups including empty groups/doc tests. |
| baseline-release-metal | `cargo build --release -p nux-capi -p nuxie-renderer --features nux-capi/apple-metal` | 101 | FAIL: E0609 for m_has_done_layout at render_context_cpp.rs:6658. |
| final-pr-gate | `make nux-capi-pr-gate` | 0 | PASS: both top-level gates; C dynamic/static and Swift smoke, export checks, layout/surface, 54 distribution tests (19 + 10 + 18 + 7). |
| final-lint | `make lint-gate` | 0 | PASS: deny crates nuxie and nuxie-schema; warn counts match main exactly: audio 27, runtime 4927, binary 350, Metal 574, C API 41. |
| final-fmt | `make fmt-check` | 2 | FAIL: identical 119-file baseline set; no modified renderer file in this set. |
| final-release | `cargo build --release -p nux-capi -p nuxie-renderer` | 0 | PASS: nux-capi and nuxie-renderer default release. |
| final-release-metal | `cargo build --release -p nux-capi -p nuxie-renderer --features nux-capi/apple-metal` | 0 | PASS: native Apple Metal release; former E0609 path compiles. |
| final-android | `make nux-capi-android` | 0 | PASS: arm64-v8a and x86_64 release libraries; ABI-v4 exports/header/layout, ELF architecture/dependencies/16-KiB LOAD alignment, provenance, checksums, and size qualification. |

## Inspection, edits, and local administration

Commands below ran only in the designated clone unless an absolute read-only path or `/tmp` evidence path is shown. Repeated read-only progress queries are grouped.

| Command or command group | Outcome |
| --- | --- |
| `cat /Users/levi/dev/nuxie-handoff/R1-runtime-main-green.md`; subsequent `sed -n '1,260p'` read | Read the complete brief. |
| `df -h /Users/levi`; `test ! -e /Users/levi/dev/nuxie-dev/.claude/worktrees/codex-runtime-main-green` | Disk above floor; destination absent before clone. |
| `git clone https://github.com/nuxieai/nuxie-runtime.git /Users/levi/dev/nuxie-dev/.claude/worktrees/codex-runtime-main-green` | Fresh clone succeeded. |
| `git switch -c codex/runtime-main-green origin/main` | Branch created at `51e103552`. |
| `ls -la AGENTS.md CLAUDE.md .gitmodules`; `rg --files --hidden -g AGENTS.md -g CLAUDE.md -g '!vendor/**' -g '!.git/**'`; `git ls-files '*AGENTS.md' '*CLAUDE.md'` | No repo instruction files or submodule manifest found. The `ls`/no-match searches returned nonzero for those absent files. |
| `git status --short`; `git status --short --untracked-files=all`; `git rev-parse HEAD`; `git log -1 --format='%h %s'` | Verified initial base and clean tree, then the four generated untracked RML files before the ignore fix. |
| `cat .gitignore`; `sed`/`rg` reads of `Makefile`, `tools/check-nux-capi-layout.py`, `tools/fetch-test-assets.sh`, `tools/build-nux-capi-android.sh`, Cargo manifests, and renderer source | Located layout record generation, fixture downloads/copies, hardcoded smoke paths, pinned Android build behavior, and renderer feature gates. |
| `rg` searches for `cfg(debug_assertions)`, `debug_assert!`, layout/frame/tessellation fields, map/unmap counters, and raw-path mutation locks across `crates/nuxie-renderer/src` | One missing guard found. Other relevant accesses already guarded; saved audit extracts in `/tmp/runtime-main-green-evidence/`. |
| Read-only `rg`/`sed` of `/Users/levi/dev/oss/rive-runtime/renderer/src/render_context.cpp`, `renderer/include/rive/renderer/render_context.hpp`, and `include/rive/rive_types.hpp` | Confirmed upstream debug-only field and C++ assert semantics. No upstream edits. |
| `ls rust-toolchain*`; `rustup toolchain list`; `rustc --version` | No repo toolchain file; identified host 1.97.1 and installed pinned Android 1.94.1. The unmatched toolchain glob returned nonzero. |
| Searches for `renderer/src/mod.rs`, `mechanical_port/mod.rs`, `exact_source_modules.rs`, and `native.rs` | Those guessed paths did not exist; followed `lib.rs`'s `include!("native_root.rs")` instead and confirmed feature-gated mechanical port. |
| `mkdir -p /tmp/runtime-main-green-evidence`; write `run.py`, `fix.py`, and handoff assembly files there | Created local evidence/command runner, narrow edit script, and documentation draft. The runner checks disk and records each check's exact command, exit status, and output path. |
| `mkdir -p target`; `ln -s green/debug target/debug` | Private target compatibility symlink created for smoke tests. |
| `python3 /tmp/runtime-main-green-evidence/fix.py` | Added exactly one Rust cfg and two ignore lines. Layout regeneration is separately recorded above. |
| `git diff --stat`; `git diff -- crates/nux-capi/abi-layout-v4.json .gitignore crates/nuxie-renderer/src/mechanical_port/source/renderer/src/render_context_cpp.rs`; `git diff --check` | Reviewed the three minimal fixes; whitespace check passed. |
| `git add crates/nux-capi/abi-layout-v4.json`; `git commit` with the recorded ABI fix message and required trailers | Created `2df4a1e98`. |
| `git add crates/nuxie-renderer/src/mechanical_port/source/renderer/src/render_context_cpp.rs`; `git commit` with the recorded renderer fix message and required trailers | Created `e921289ac`. |
| `git add .gitignore`; `git commit` with the recorded fixture fix message and required trailers | Created `97ad81a2e`. |
| `git diff --check 51e103552..HEAD`; `git diff --stat 51e103552..HEAD`; Python inspection of `git diff 51e103552 -- crates/nux-capi/include` | Whitespace clean; three implementation files, eight insertions and one deletion; headers unchanged. |
| Python parsing of `test result`, `Ran ... tests`, and `Diff in ...` records; `rg '^== lint-gate'` | Counted 117 passing C API tests, 54 passing PR distribution tests, identical lint counts, and identical 119-file fmt sets. |
| Repeated `tail`, `rg`, `cat commands.jsonl`, `du -sh target`, `df -h /Users/levi`, `date`, `git status`, and `git log` queries | Read-only progress/evidence inspection; disk stayed above the launch floor. Queries for a log not yet created returned missing-file errors before that command started. |
| `ps -axo pid,etime,pcpu,comm` and later `ps -axo pid,etime,pcpu,command` | First process inspection was sandbox-denied; escalated read-only inspection confirmed active compilation. No process was killed. |

No required check was skipped. Formatting remains the independently reproduced main failure and was deliberately not repaired. The full runtime suite and superproject editor/performance commands were not requested by this brief and were not run; no superproject checkout was modified. No extra unit test was added for the cfg line: the formerly failing native release compilation is its direct regression check, alongside the existing ABI and package contracts.

## Formatting baseline file list

`make fmt-check` exits 2 on both untouched main and this branch with Rust 1.97.1. Both outputs identify exactly these 119 files. The modified `render_context_cpp.rs` is not in the list. None of these formatting issues was changed.

```text
crates/nux-capi/src/android_vulkan.rs
crates/nux-capi/src/android_vulkan/deferred.rs
crates/nux-capi/src/apple_metal.rs
crates/nux-capi/src/asset_hooks.rs
crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore.rs
crates/nuxie-ore-metal/src/mechanical_port/source/renderer/src/ore/metal/ore_context_metal_mm.rs
crates/nuxie-ore-metal/src/ore_cmd/ore_make_recording.rs
crates/nuxie-render-api/src/serializing.rs
crates/nuxie-render-api/tests/serialized_replay_test.rs
crates/nuxie-renderer/build.rs
crates/nuxie-renderer/src/deferred/cmd/deferred_replayer.rs
crates/nuxie-renderer/src/deferred/cmd/tests/artboard_bitmap_cache_test.rs
crates/nuxie-renderer/src/deferred/cmd/tests/deferred_session_attachment_test.rs
crates/nuxie-renderer/src/deferred/cmd/tests/layer_mask_geometry_test.rs
crates/nuxie-renderer/src/deferred/cmd/tests/layer_mask_test.rs
crates/nuxie-renderer/src/deferred/cmd/tests/mod.rs
crates/nuxie-renderer/src/deferred/cmd/tests/ore_deferred_target_test.rs
crates/nuxie-renderer/src/deferred/gm/mod.rs
crates/nuxie-renderer/src/deferred/gm/ore_deferred_resource.rs
crates/nuxie-renderer/src/deferred/gm/ore_deferred_target.rs
crates/nuxie-renderer/src/deferred/gm/ore_gm_helper.rs
crates/nuxie-renderer/src/deferred/gm/ore_render_deferred_canvas.rs
crates/nuxie-renderer/src/deferred/gm/render_canvas_dag.rs
crates/nuxie-renderer/src/deferred/gm/stroke_position_shapes.rs
crates/nuxie-renderer/src/deferred/ore/ore_deferred_bookkeeping_test.rs
crates/nuxie-renderer/src/deferred/ore/ore_deferred_context.rs
crates/nuxie-renderer/src/exact_gpu_canvas.rs
crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/makefile.rs
crates/nuxie-renderer/tests/native_metal_resource_shaders.rs
crates/nuxie-runtime/src/host_state_machine.rs
crates/nuxie-runtime/src/mechanical_port/source/advancing_component.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/linear_animation_instance.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/listener_types/mod.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/listener_viewmodel_change.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/nested_state_machine.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/state_machine_instance.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/state_machine_listener.rs
crates/nuxie-runtime/src/mechanical_port/source/animation/state_machine_listener_single.rs
crates/nuxie-runtime/src/mechanical_port/source/artboard_component_list.rs
crates/nuxie-runtime/src/mechanical_port/source/constraints/draggable_constraint.rs
crates/nuxie-runtime/src/mechanical_port/source/core.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/context/context_value/native_binding.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/converters/data_converter.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/converters/data_converter_group.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/converters/data_converter_interpolator.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/data_bind.rs
crates/nuxie-runtime/src/mechanical_port/source/data_bind/data_bind_container.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/animation/listener_types/mod.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/core_registry.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/mod.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/selection_style_base.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/shapes/paint/color_channels_base.rs
crates/nuxie-runtime/src/mechanical_port/source/generated/shapes/paint/fill_base.rs
crates/nuxie-runtime/src/mechanical_port/source/layout/layout_participant.rs
crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs
crates/nuxie-runtime/src/mechanical_port/source/listener_group.rs
crates/nuxie-runtime/src/mechanical_port/source/scene.rs
crates/nuxie-runtime/src/mechanical_port/source/scripted/scripted_data_converter.rs
crates/nuxie-runtime/src/mechanical_port/source/scripted/scripted_transition.rs
crates/nuxie-runtime/src/mechanical_port/source/semantic/semantic_data.rs
crates/nuxie-runtime/src/mechanical_port/source/shapes/image.rs
crates/nuxie-runtime/src/mechanical_port/source/shapes/paint/gradient_stop.rs
crates/nuxie-runtime/src/mechanical_port/source/shapes/paint/solid_color.rs
crates/nuxie-runtime/src/mechanical_port/source/text/font_hb.rs
crates/nuxie-runtime/src/mechanical_port/source/text/text.rs
crates/nuxie-runtime/src/mechanical_port/source/text/text_engine.rs
crates/nuxie-runtime/src/mechanical_port/source/text/text_input.rs
crates/nuxie-runtime/src/mechanical_port/source/text/text_modifier_group.rs
crates/nuxie-runtime/src/mechanical_port/source/viewmodel/viewmodel_instance.rs
crates/nuxie-runtime/src/mechanical_port/source/viewmodel/viewmodel_instance_trigger.rs
crates/nuxie-runtime/src/scripting/native_artboard.rs
crates/nuxie-runtime/src/video/objects.rs
crates/nuxie-runtime/tests/cpp_probe_text_native.rs
crates/nuxie-runtime/tests/semantic_focus_runtime.rs
crates/nuxie-runtime/tests/support/upstream_focus_traversal_9b.rs
crates/nuxie-runtime/tests/upstream_artboard_transform.rs
crates/nuxie-runtime/tests/upstream_color_glyph.rs
crates/nuxie-runtime/tests/upstream_data_binding_fonts.rs
crates/nuxie-runtime/tests/upstream_decoded_file_native.rs
crates/nuxie-runtime/tests/upstream_hidden_hit_targets.rs
crates/nuxie-runtime/tests/upstream_layout_corner_radius.rs
crates/nuxie-runtime/tests/upstream_listener_input_value.rs
crates/nuxie-runtime/tests/upstream_pointer_button_2dbbfe18.rs
crates/nuxie-runtime/tests/upstream_quiet_rows.rs
crates/nuxie-runtime/tests/upstream_scroll_input.rs
crates/nuxie-runtime/tests/upstream_scroll_velocity.rs
crates/nuxie-runtime/tests/upstream_semantic_artboard_swap.rs
crates/nuxie-runtime/tests/upstream_semantic_data_lifecycle.rs
crates/nuxie-runtime/tests/upstream_settled_layers_3330baec.rs
crates/nuxie-runtime/tests/upstream_text_input_native.rs
crates/nuxie-runtime/tests/upstream_trigger_callback_mutation.rs
crates/nuxie-runtime/tests/upstream_wave_a_core.rs
crates/nuxie-runtime/tests/upstream_wave_a_expected_red.rs
crates/nuxie-runtime/tests/upstream_wave_b_expected_red.rs
crates/nuxie-runtime/tests/upstream_wave_c9.rs
crates/nuxie-scripting/src/gpu_canvas_ore/shader.rs
crates/nuxie-scripting/src/vm/lua_canvas.rs
crates/nuxie-scripting/src/vm/lua_font.rs
crates/nuxie-scripting/src/vm/lua_mesh.rs
crates/nuxie-scripting/src/vm/lua_paint.rs
crates/nuxie-scripting/src/vm/lua_renderer.rs
crates/nuxie-scripting/tests/upstream_scripting_wake_advance.rs
crates/nuxie-scripting/tests/vm_boot.rs
crates/nuxie/tests/audio_core.rs
crates/nuxie/tests/command_queue.rs
crates/nuxie/tests/upstream_audio.rs
crates/nuxie/tests/upstream_data_binding_artboards.rs
crates/nuxie/tests/upstream_data_binding_cycle.rs
crates/nuxie/tests/upstream_data_binding_images_direct.rs
crates/nuxie/tests/upstream_data_binding_viewmodels.rs
crates/nuxie/tests/upstream_scripting_artboard.rs
crates/nuxie/tests/upstream_scripting_listener_action.rs
crates/nuxie/tests/upstream_solo.rs
tools/renderer-replay/src/main.rs
tools/rust-golden-runner/src/main.rs
tools/silver-corpus/src/action.rs
tools/silver-corpus/tests/upstream_scripting_properties.rs
tools/silver-corpus/tests/upstream_scroll.rs
tools/silver-corpus/tests/wave_a.rs
```

## Android size evidence

The qualified size report is retained at `/tmp/runtime-main-green-evidence/android-size-report.json`:

```json
{
  "artifactName": "NuxieRuntimeAndroid.zip",
  "budgetSha256": "1976ebeede5376f1c38a0164e81190bd5708cb31d2fc6bd8a418b385a3e78f91",
  "headroomBytes": {
    "archiveBytes": 5862383,
    "expandedBytes": 6221119,
    "fileBytes": {
      "include/nux_capi.generated.h": 409343,
      "jniLibs/arm64-v8a/libc++_shared.so": 3292728,
      "jniLibs/arm64-v8a/libnux_capi.so": 8948800,
      "jniLibs/x86_64/libc++_shared.so": 3567368,
      "jniLibs/x86_64/libnux_capi.so": 7304384
    }
  },
  "maximums": {
    "archiveBytes": 25165824,
    "expandedBytes": 67108864,
    "fileBytes": {
      "include/nux_capi.generated.h": 524288,
      "jniLibs/arm64-v8a/libc++_shared.so": 12582912,
      "jniLibs/arm64-v8a/libnux_capi.so": 29360128,
      "jniLibs/x86_64/libc++_shared.so": 12582912,
      "jniLibs/x86_64/libnux_capi.so": 29360128
    }
  },
  "measurements": {
    "archiveBytes": 19303441,
    "expandedBytes": 60887745,
    "fileBytes": {
      "include/nux_capi.generated.h": 114945,
      "jniLibs/arm64-v8a/libc++_shared.so": 9290184,
      "jniLibs/arm64-v8a/libnux_capi.so": 20411328,
      "jniLibs/x86_64/libc++_shared.so": 9015544,
      "jniLibs/x86_64/libnux_capi.so": 22055744
    }
  },
  "releaseTag": "android-runtime-v0.4.10",
  "schemaVersion": 1
}

```

## Final cleanup and handoff

The private `target/` tree, including `target/green`, the smoke symlink/artifacts, and Android build/package artifacts, was deleted after successful qualification. Logs remain in `/tmp/runtime-main-green-evidence/`. The Android archive is intentionally not retained or published.

Cleanup command: `rm -rf /Users/levi/dev/nuxie-dev/.claude/worktrees/codex-runtime-main-green/target`, then `test ! -e target`. Documentation was assembled from the evidence files, reviewed for em dashes and whitespace, and committed with `git add HANDOFF.md` and `git commit -m "docs: record runtime main green verification"` plus the required issue/co-author trailers. Final `make fixtures`, `git status --short`, `git diff --check 51e103552..HEAD`, and the target-absence assertion passed. No build was repeated after deleting the target.
