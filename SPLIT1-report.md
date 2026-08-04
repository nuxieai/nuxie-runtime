# Artboard source-file decomposition report

## Result

`crates/nuxie-runtime/src/artboard.rs` started at 23,487 lines. The retained
root is 10,073 lines after moving the 11,387-line test body and 2,150 lines of
source-owner modules. Added module imports and the narrow visibility required
across child-module boundaries account for the difference between moved and
removed line totals.

The root remains the owner of `ArtboardInstance`, its storage and clone/drop
implementation, C++ `artboard.cpp` construction and ordered orchestration,
mixed dispatchers, and the pre-existing literal leaf includes. Public inherent
method paths remain `ArtboardInstance::*`. The two moved helper types are
re-exported from `crate::artboard` with `pub(crate)` visibility.

## Module tree and pinned C++ counterparts

| Rust module | Lines | Pinned C++ source counterpart |
| --- | ---: | --- |
| `crates/nuxie-runtime/src/artboard.rs` | 10,073 | `src/artboard.cpp` plus retained mixed orchestration |
| `crates/nuxie-runtime/src/artboard/advancing_component.rs` | 12 | `src/advancing_component.cpp` |
| `crates/nuxie-runtime/src/artboard/animation/property_recorder.rs` | 142 | `src/animation/property_recorder.cpp` |
| `crates/nuxie-runtime/src/artboard/artboard_component_list.rs` | 697 | `src/artboard_component_list.cpp` |
| `crates/nuxie-runtime/src/artboard/bones/bone.rs` | 10 | `src/bones/bone.cpp` |
| `crates/nuxie-runtime/src/artboard/nested_artboard.rs` | 379 | `src/nested_artboard.cpp` |
| `crates/nuxie-runtime/src/artboard/nested_artboard_layout.rs` | 181 | `src/nested_artboard_layout.cpp` |
| `crates/nuxie-runtime/src/artboard/node.rs` | 15 | `src/node.cpp` |
| `crates/nuxie-runtime/src/artboard/resetting_component.rs` | 52 | `src/resetting_component.cpp` |
| `crates/nuxie-runtime/src/artboard/shapes/paint/solid_color.rs` | 35 | `src/shapes/paint/solid_color.cpp` |
| `crates/nuxie-runtime/src/artboard/text/text_input.rs` | 106 | `src/text/text_input.cpp` |
| `crates/nuxie-runtime/src/artboard/text/text_style.rs` | 113 | `src/text/text_style.cpp` |
| `crates/nuxie-runtime/src/artboard/text/text_value_run.rs` | 44 | `src/text/text_value_run.cpp` |
| `crates/nuxie-runtime/src/artboard/text/text_variation_helper.rs` | 26 | `src/text/text_variation_helper.cpp` |
| `crates/nuxie-runtime/src/artboard/transform_component.rs` | 148 | `src/transform_component.cpp` |
| `crates/nuxie-runtime/src/artboard/virtualizing_component.rs` | 190 | `src/virtualizing_component.cpp` |
| `crates/nuxie-runtime/src/artboard/tests.rs` | 11,387 | Test-only body; no production C++ source owner |

## Row-by-row commits

1. `2ac481c3` — extract Artboard tests
2. `910f24f8` — extract `advancing_component.cpp` owner
3. `e48265ec` — extract `resetting_component.cpp` owner
4. `52f9636f` — extract `property_recorder.cpp` owner
5. `cf3d4a0c` — extract `text_style.cpp` owner
6. `fc735d6b` — extract `text_variation_helper.cpp` owner
7. `d5fce0f9` — extract `text_value_run.cpp` owner
8. `f7846495` — extract `transform_component.cpp` owner
9. `bb856e8a` — extract `solid_color.cpp` owner
10. `f0973b46` — extract `bone.cpp` owner
11. `18096444` — extract `node.cpp` owner
12. `2f971919` — extract `text_input.cpp` owner
13. `e5891d11` — extract `virtualizing_component.cpp` owner
14. `221d9978` — extract `artboard_component_list.cpp` owner
15. `dcaa0936` — extract `nested_artboard_layout.cpp` owner
16. `40fecf1e` — extract `nested_artboard.cpp` owner

Each production extraction updated its file-correspondence and frame-loop
ownership entries in the same commit. Physical owner-boundary registrations
were moved with unchanged site hashes. After every extraction commit,
`make runtime-frame-loop-port-check` and `make rust-attribution-check` passed;
the correspondence scatter count remained 155/155.

## Final gates

All required gates passed on branch `levi/split-artboard`:

| Gate | Result |
| --- | --- |
| `cargo test -p nuxie-runtime` | PASS |
| `cargo test -p nuxie --features scripting` | PASS (after refreshing the pinned C++ probe with `make cpp-probe`) |
| `make scripted-golden-compare` | PASS — 363 entries; 346 exact; 1,126 exact segments; 1,121 side-channel segments; 12 registered divergences; 5 registered not-yet cases |
| `make runtime-frame-loop-port-check` | PASS — 125 checker unit tests; 354/354 files classified; 76/76 members classified; scatter 155/155 |
| `make rust-attribution-check` | PASS — 10 checker unit tests; every in-scope Rust source classified |
| `make silver-corpus-test` | PASS — 21 library tests, 1 FL-E8 test, 3 runtime-frame-loop tests (1 registered ignore), and 19 manifest-generator tests |

The final post-review cleanup commits were `ef006a7b` (restore the test-only
resetting-kind name after the module move), `510fc232` (limit the owner-detector
test exemption to the exact extracted test file), and `2c7a6a4d` (scope that
test-only import under `cfg(test)`). `git diff --check` passes.
