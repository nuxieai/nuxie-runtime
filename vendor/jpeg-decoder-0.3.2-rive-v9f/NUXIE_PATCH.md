# Nuxie Rive libjpeg-v9f parity patch

This directory starts from the crates.io `jpeg-decoder` 0.3.2 package
(archive SHA-256
`00810f1d8b74be64b13dbf3db89ac67740615d6c891f0e7b6179326533011a07`).
The `rive_v9f` feature replaces only the decoding mechanics that differ from
the JPEG owner compiled by Rive's WebGL2 renderer.

## Source authority

- Rive runtime commit:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Dependency: `renderer/dependencies/rive-app_libjpeg_v9f` (IJG libjpeg v9f).
- Rive build owner: `renderer/premake5_pls_renderer.lua`, compiled into the
  WebGL2 renderer with Emscripten 3.1.61.
- `jidctint.c` SHA-256:
  `800098c890c67fecf09b6626d6d74b27990d47257e0e00a148256770181341bd`.
- `jdcolor.c` SHA-256:
  `8c2a9f5f7ccfb72663cff4b489e0bf698dddbd542a4edd229428686b3056da06`.
- `jdmaster.c` SHA-256:
  `58565d73471ecdf9946ae26ae54829912bd67f40a025b146214213f0f18c55d8`.

## Behavioral delta

When `rive_v9f` is enabled:

- horizontal and vertical DCT scales are tracked separately;
- IJG v9f's default full-size fancy-upsampling selection is applied per
  component exactly as owned by `jpeg_calc_output_dimensions()`;
- the 8x8, 16x8, 8x16, and 16x16 integer inverse-DCT kernels use the constants,
  rounding points, arithmetic shifts, and output ordering from `jidctint.c`;
- YCbCr-to-RGB conversion uses `jdcolor.c`'s 16-bit lookup-table arithmetic,
  including its single combined rounding step for green;
- immediate and Rayon workers propagate rectangular scales and allocate/copy
  the corresponding rectangular output blocks.

The old `asmjs` target predicates were removed from the imported worker cfgs.
Rust no longer exposes that target; supported native and `wasm32` behavior is
unchanged by this manifest-compatibility cleanup.

The feature is opt-in. `nuxie-image-codec` enables it because browser WebGL2
must decode embedded JPEG assets through the same mechanics as the pinned Rive
source renderer.

## Differential evidence

- The 278x278 4:2:0 JPEG embedded in
  `clipping_and_draw_order.rive-stream` decodes to byte-identical RGBA upload
  bytes and produces a byte-identical WebGL2 frame.
- Deterministic 31x29 4:2:2 and 4:4:0 JPEGs generated and decoded by the pinned
  IJG v9f source were decoded independently through this fork; both RGB images
  compared with zero differing pixels and zero channel delta.
- The fork compiles with its complete native feature matrix, including Rayon,
  and under the renderer's `wasm32-unknown-unknown` build.
