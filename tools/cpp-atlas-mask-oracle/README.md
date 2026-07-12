# C++ WebGPU Atlas-Mask Oracle

This harness produces a deterministic readback of the C++ renderer's WebGPU
`R16Float` feather atlas. It temporarily injects a single C++ executable into
`RIVE_RUNTIME_DIR`, applies `runtime.patch`, builds the exact `--with-dawn`
renderer configuration, then reverses the patch and removes only the injected
source directory.

The exporter draws one fixed closed square stroke:

* render target: `64 x 64`
* closed square: `(16,16) -> (48,16) -> (48,48) -> (16,48)`
* stroke: thickness `8`, miter join, butt cap, feather `20`
* frame: 4x MSAA, which selects atlas feather rendering
* atlas contract: `39 x 39` logical content at `(2,2)`, in the complete
  `48 x 48` physical allocation produced by C++'s 125% resource growth

The harness emits two artifacts. The mask uses the exact `RIVEMSK` version 1
Rust interchange format: a 20-byte
little-endian header (`magic`, `version`, `width`, `height`) followed by a
canonical, tightly row-packed `R16Float` payload. WebGPU's 256-byte copy rows
are stripped during export. The complete physical C++ atlas, including its
cleared unused tail, must be exactly `48 x 48`, making the canonical file
exactly `4628` bytes. The exporter validates the frame, logical allocation,
placement, and physical allocation, then fails on drift without cropping,
padding, or normalization.

`atlas-inputs.bin` uses the `RIVEATI` version 1 contract. Its 40-byte
little-endian header records the atlas batch range, contour count, and
tessellation dimensions, followed by canonical 16-byte contour records and
the complete tightly packed `RGBA32Uint` tessellation texture. Both artifacts
come from the same submitted C++ frame.

```sh
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh --preflight
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh
python3 tools/cpp-atlas-mask-oracle/format_test.py
RIVE_CPP_ATLAS_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-mask.r16f" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atlas_mask_oracle_matches_fixed_rust_mask_when_configured \
  -- --exact --ignored --nocapture
RIVE_CPP_ATLAS_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-inputs.bin" \
  cargo test -p nuxie-renderer \
  tests::cpp_webgpu_atlas_input_oracle_matches_fixed_rust_inputs_when_configured \
  -- --exact --ignored --nocapture
```

The configured comparator is ignored by ordinary test suites and requires a
nonempty absolute `RIVE_CPP_ATLAS_MASK` or `RIVE_CPP_ATLAS_INPUTS` path;
invoking either test without its variable is an error.

`--preflight` proves that the temporary patch applies and reports each missing
Dawn prerequisite without building or changing the runtime checkout.
It also requires Naga exactly at version `30.0.0`, which the renderer's WGSL
shader-generation step invokes while Premake generates the isolated build
files. By default the harness uses `$HOME/.cargo/bin/naga`, matching
`tools/generate-renderer-shaders.sh`, and prepends that executable's directory
to the build `PATH`; the caller's `PATH` does not need to include Cargo's bin
directory. `RIVE_ATLAS_MASK_NAGA=/absolute/path/to/naga` selects another
executable named `naga`, still subject to the exact version check.

On macOS with Xcode 26 or later, `build.sh` temporarily changes Dawn
PartitionAlloc's `mac_no_default_new_delete_symbols` setting from
`-fvisibility-global-new-delete=force-hidden` to an empty `cflags` list.
Xcode 26's SDK libc++ declares these symbols with default visibility, so
forcing hidden visibility causes the known declaration mismatch. The patch is
checked before use, skipped when Dawn is already compatible, and reversed on
exit.

The same Xcode-26 branch temporarily appends
`treat_warnings_as_errors=false` to Dawn's generated `out/release/args.gn`.
This keeps legacy unsafe-buffer diagnostics visible but prevents the new clang
default from promoting them to build-stopping errors. An explicit user value is
never overwritten. It also sets `use_lld=false`, making Dawn emit regular
archives that the Premake executable's Apple `ld` link step can consume. Before
either temporary edit, the harness snapshots `args.gn`; its exit trap restores
that snapshot and verifies byte equality with `cmp`, including blank lines.
