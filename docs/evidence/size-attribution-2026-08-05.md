# Binary-size attribution and competitive baseline — 2026-08-05

This snapshot measures the current SDK link closures at source revision
`b1f91278`, attributes the bytes per crate and per cross-cutting category, and
establishes a pin-matched C++ comparator built from `rive-runtime` `4ac7b327`
under the identical link discipline. It is analysis evidence for the
size-optimization effort; it does not change the suspended #B-3 budget gate
(see [SIZE.md](../SIZE.md)).

## Current measurements (`make size-report SIZE_BASELINE=1`)

Toolchain as recorded by the report; artifacts and symbol breakdowns under
`target/size-report/`.

| Stripped link closure | Bytes | MiB |
|---|---:|---:|
| release opt=3, renderer ON, scripting OFF | 13,067,976 | 12.46 |
| release-size opt=z, renderer ON, scripting OFF | 9,383,656 | 8.95 |
| release-size opt=z, renderer ON, scripting ON | 11,150,536 | 10.63 |

Growth since the 2026-07-20 evidence snapshot (`d8091cd5`, 7,534,056 B
scripting OFF): **+1,849,600 B (+24.5%)**, dominated by `nuxie_runtime` code
(FL-series DataBind/ViewModel surface) and renderer/wgpu growth.

## Competitive baseline

Two independent baselines agree on roughly the same target zone.

**Rive's published SDK impact** ([runtime-sizes](https://rive.app/docs/runtimes/runtime-sizes)):
iOS install-size impact **~4.66 MB** (download ~1.67 MB, App Thinning report);
Android arm64 install 7.03 MB including DEX and a 1.2 MB libc++ share.

**Pin-matched local comparator.** The pinned `rive-runtime` (`4ac7b327`) was
built with the Apple toolchain at its shipped release configuration
(premake `--config=release`: `-O2`, LTO bitcode archives, RTTI off), archives:
rive + rive_pls_renderer + rive_decoders + libpng/zlib/libjpeg/libwebp +
rive_harfbuzz + rive_sheenbidi + rive_yoga (+ luau_vm + miniaudio for the
scripted variant), text and layout enabled. Each set was linked exactly like
the Rust artifact: one consumer root referencing the full import → animate →
draw → Metal-flush surface plus `decodeImage`, `clang -dynamiclib
-dead_strip -dead_strip_dylibs -exported_symbols_list`, then `strip -S -x`.
Consumer source: session scratchpad `rive_closure_consumer.mm`; outputs under
`target/rive-size-comparator/`.

| C++ closure (arm64) | Bytes | MB |
|---|---:|---:|
| text+layout+PLS renderer, no scripting/audio | 2,161,264 | 2.16 |
| + Luau VM + miniaudio (scripted) | 2,941,968 | 2.94 |

Known asymmetries, both directions: the C++ closure's LTO internalization
dropped SheenBidi (~40 KB) and the libpng/jpeg/webp decode bodies (~350 KB
generous), and the non-scripted variant has no audio engine; the Rust closure
includes symphonia audio decoders, image codecs, and ICC color management.
Even crediting C++ with all of those (~2.6–3.0 MB adjusted), the ratio holds:

**Rust is ~3.3–4.1× the C++ runtime closure and ~1.9–2.3× Rive's shipped iOS
install impact.**

Section-level contrast (scripting OFF Rust vs non-scripted C++):

| Section | Rust | C++ |
|---|---:|---:|
| `__text` | 5,872,692 | 1,670,552 |
| `__TEXT,__const` | 1,283,592 | 154,332 |
| `__cstring` | 148,106 | 3,888 |
| `__unwind_info` + `__eh_frame`/`__gcc_except_tab` | 763,252 | 3,720 |
| `__DATA_CONST,__const` | 572,392 | 96,608 |

## Attribution (release-size, scripting OFF)

Per-crate `__text` next-address deltas over the unstripped closure (v0
mangling parsed to crate names; `_OUTLINED_FUNCTION_*` are opt-z machine
outlining shared across crates):

| Bytes | Crate/family |
|---:|---|
| 1,864,948 | Rust `core` + `alloc` + `std` (largely monomorphized generic infrastructure + fmt/panic machinery) |
| 875,180 | `nuxie_runtime` |
| 819,798 | wgpu stack: `naga` 411,316 + `wgpu_core` 304,948 + `wgpu_hal` 60,920 + `wgpu` 22,916 + `wgpu_types` 20,620 |
| 560,376 | text stack: `read_fonts` 218,056 + `skrifa` 174,508 + `harfrust` 167,812 |
| 522,584 | outlined fragments + compiler-rt (31.5k `_OUTLINED_FUNCTION_*`) |
| 275,672 | `nuxie_renderer` |
| 216,168 | image codecs: `image_webp` 62,932 + `jpeg_decoder` 36,428 + `png` 31,432 + `fdeflate` 8,432 + misc |
| 165,772 | audio: `symphonia_*` ~132 KB + `encoding_rs` 22,256 + `nuxie_audio` 5,476 |
| 161,320 | color management: `moxcms` 85,648 + `pxfm` 15,700 (+61,762 `__const`) |
| 146,124 | `serde_json` 47,628 + `zmij` (float fmt) + `serde` shims (+98 KB `__const`) |
| 90,748 | `taffy` |
| 63,764 | std backtrace: `gimli` + `rustc_demangle` + `addr2line` + `object` |
| 60,072 | `hashbrown` |

Const-side concentrations: `encoding_rs` **341 KB** (`__TEXT,__const` 123,930 +
`__DATA_CONST` 217,416) via `symphonia-metadata`'s ID3 charset tables;
`unicode_width` 92,664 + `codespan_reporting` via naga's WGSL diagnostics;
a single 75,868 B `log`-crate table; `zmij` 93,993; `harfrust` 91,846.

Cross-cutting categories (overlapping):

| Bytes (`__text`) | Category |
|---:|---|
| 229,552 | `fmt`/`Debug`/`Display` machinery |
| 150,816 | `Clone` impls (largest: `InstanceObjectStorage::clone` 16,684) |
| 106,784 | `generated_objects` property dispatch (`from_runtime_object` alone 59,760) |
| ~136,000 | product-layer code reachable from the closure (`project_data_converter` 89,148 + flow helpers) |

Scripting ON adds 1,766,880 B total: `luaur_*` ~591 KB `__text` (of which
`luaur_compiler` + `luaur_ast` = 135 KB runtime compilation — upstream C++
links only `luau_vm`), `nuxie_scripting` 121,908, plus const/unwind growth.
`ed25519_dalek`/`sha2` do not measurably survive this closure.

## Where the 4× comes from

1. **A WebGPU stack instead of a Metal renderer.** wgpu-core is a full
   WebGPU validation/tracking layer and naga is a complete WGSL frontend +
   MSL backend compiled into the SDK (plus diagnostics deps and log tables).
   Rive's entire PLS renderer closure is ~350 KB of `-O2` code with shaders
   precompiled offline. Delta attributable to the approach: **~1.3–1.6 MB**.
2. **Rust generic monomorphization + fmt/panic machinery.** core/alloc/std at
   1.86 MB `__text` plus several hundred KB of const/cstring panic and type
   strings; C++ counterpart is a few hundred KB. Partly structural, partly
   dietable (Debug derives, format!-heavy errors, per-callsite panics).
3. **Unwinding tables**: 763 KB vs ~4 KB. Cost of `panic = "unwind"`, which
   only the Luau protected-error boundary needs.
4. **Heavier media dependencies**: symphonia+encoding_rs (~500 KB total) vs
   miniaudio's decoders; moxcms ICC (~225 KB total) with no C++ counterpart.
5. **Generated-code shape**: giant monomorphized property-dispatch and
   from-graph builders (60 KB/30 KB single functions) vs C++'s per-type
   virtuals.

## Ranked levers

| # | Lever | Est. saving (OFF closure) | Effort/risk |
|---:|---|---:|---|
| 1 | Precompile shaders offline; drop naga front/back from the SDK build (keep for dev builds) | 550–750 KB | High effort; needs wgpu pipeline-creation seam or hal-direct path; parity-safe (shaders byte-identical) |
| 2 | Slim or bypass wgpu-core for the Apple SDK (hal-direct Metal like upstream PLS; keep wgpu for browser tier) | 400–700 KB beyond #1 | High effort, architectural; renderer goldens protect parity |
| 3 | `panic = "abort"` + no unwind tables for the scripting-OFF SDK profile (keep unwind only where luaur lives) | ~700 KB | Medium; audit capi panic boundary; panic-freedom lints already ratchet this |
| 4 | Patch out `symphonia-metadata`/`encoding_rs` (vendored patch like the luaur fork) or swap to minimal wav/mp3/flac decoders | 400–450 KB | Low–medium; contained; audio differentials exist |
| 5 | fmt/Debug/panic-string diet: strip `Debug` derives from runtime types in release, static error codes, `-Zlocation-detail=none` when toolchain allows | 300–500 KB | Medium; mechanical but wide |
| 6 | Feature-gate moxcms ICC transforms unless parity requires them (upstream has no CMS) | ~225 KB | Low; verify against image goldens |
| 7 | Move `project_data_converter` JSON (serde_json+zmij) behind the product seam per the UNIV-1621 extraction umbrella | ~250 KB | Low; already planned direction |
| 8 | Table-driven `generated_objects` property dispatch + builder codegen reshape | 100–200 KB | Medium; codegen change, differential-tested |
| 9 | Ship scripting as bytecode-only (drop `luaur_compiler`/`luaur_ast` from the SDK build) if the editor-emitted-bytecode contract allows | ~150 KB (ON) | Low code risk; contract question (bytecode matrix row) |
| 10 | Drop std backtrace machinery (falls out of #3, or build-std) | ~80 KB | Comes with #3 |

Realistic aggregate for the OFF closure: **-2.7 to -3.7 MB → ~5.2–6.2 MB**
with levers 3–8; adding the renderer levers (1–2) reaches **~4.5–5 MB**,
at or below Rive's shipped 4.66 MB iOS install impact. Matching the ~2.2 MB
pin-matched C++ closure additionally requires the fmt/monomorphization diet
to go deep (compact error/panic story, fewer generic instantiations), which
is a longer campaign.

Note the interaction with cleanup already planned: product-layer extraction
(FlowSession/SceneTx/data-converter) removes lever 7 naturally, and the
FL-series completion reopens the budget USER-GATE with these numbers as the
honest baseline.
