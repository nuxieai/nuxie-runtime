# C++ WebGPU Atlas-Mask Oracle

This harness produces a deterministic readback of the C++ renderer's WebGPU
`R16Float` feather atlas. It temporarily injects a single C++ executable into
`RIVE_RUNTIME_DIR`, applies `runtime.patch`, builds the exact `--with-dawn`
renderer configuration, then reverses the patch and removes only the injected
source directory.

The exporter draws one fixed open cubic stroke:

* render target and atlas: `64 x 64`
* closed square: `(16,16) -> (48,16) -> (48,48) -> (16,48)`
* stroke: thickness `8`, miter join, butt cap, feather `20`
* frame: 4x MSAA, which selects atlas feather rendering

Output is the exact `RIVEMSK` version 1 Rust interchange format: a 20-byte
little-endian header (`magic`, `version`, `width`, `height`) followed by a
canonical, tightly row-packed `R16Float` payload. WebGPU's 256-byte copy rows
are stripped during export.

```sh
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh --preflight
RIVE_RUNTIME_DIR=/path/to/rive-runtime tools/cpp-atlas-mask-oracle/build.sh
python3 tools/cpp-atlas-mask-oracle/format_test.py
```

`--preflight` proves that the temporary patch applies and reports each missing
Dawn prerequisite without building or changing the runtime checkout.
It also requires `naga`, which the renderer's WGSL shader-generation step
invokes while Premake generates the isolated build files.

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
never overwritten, and the harness removes only the line it added on exit.
