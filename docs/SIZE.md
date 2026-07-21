# SDK Binary Size

This is the reproducible binary-size evidence for parity-closeout ticket
**#B-3**. The tracked artifact is now the post-Phase-R Darwin SDK link closure:
the portable `nux-capi` ABI with the pure-Rust `nuxie-renderer` and vendored
`wgpu` backend retained. It is measured with scripting both disabled and
enabled.

No new size budget is set in this document. Choosing one is the #B-3
**USER-GATE**.

## Current measurement

Measured 2026-07-20 from `main` runtime code at `1b6af6e2` plus the #B-3
measurement-only consumer harness. The committed evidence snapshot records the
exact measurement revision, artifact digests, toolchain, public-root inventory,
and symbol-size breakdown in
[`docs/evidence/size-b3-2026-07-20.md`](evidence/size-b3-2026-07-20.md). Two
consecutive runs of `make size-report` produced the same output and
byte-identical artifacts.

| `release-size` link closure | Bytes | MiB | Delta from scripting OFF |
|---|---:|---:|---:|
| Renderer ON, scripting OFF — tracked metric | **7,517,400** | **7.17** | — |
| Renderer ON, scripting ON | **8,318,616** | **7.93** | **+801,216 B (+10.7%)** |

The historical budget was **2.75 MiB = 2,883,584 B** per architecture. The new
renderer-on, scripting-off measurement is **4,633,816 B (+160.7%) above** that
number. This is informational only: `make size-report` does not enforce the old
budget or infer a replacement.

The scripting-off section layout reported by Apple's `size -m` is:

| Mach-O region | Bytes |
|---|---:|
| `__TEXT` segment | 6,848,512 |
| `__text` section | 4,672,800 |
| `__const` section in `__TEXT` | 1,073,020 |
| `__cstring` | 119,316 |
| `__unwind_info` | 169,456 |
| `__eh_frame` | 419,788 |
| `__DATA_CONST` segment | 524,288 |
| `__const` section in `__DATA_CONST` | 508,424 |
| `__DATA` segment | 16,384 |
| `__LINKEDIT` segment | 131,072 |

## Artifact contract

The measured files are:

```text
target/size-report/release-size-renderer-on-scripting-off/libnux_capi_full.dylib
target/size-report/release-size-renderer-on-scripting-on/libnux_capi_full.dylib
```

They are consumed-SDK **link-closure proxies**, not the raw `.a` archive and
not Cargo's callback-only `libnux_capi.dylib`. The report constructs each
artifact mechanically:

1. Build `nux-capi` as `staticlib` + `cdylib` under the `release-size` profile,
   with `--no-default-features --features size-report-roots`; add
   `nuxie/scripting` for the scripting-on variant. The measurement feature
   implies `apple-renderer` and is absent from normal SDK builds.
2. Verify the resolved dependency graph contains `nuxie-renderer` and the
   repository's vendored `wgpu` 30.0.0.
3. Verify the measurement consumer's 31 calls exactly match the public methods
   found in the renderer source: 16 inherent methods, seven `Factory` methods,
   and eight `Renderer` methods. Re-link the staticlib as one Mach-O dylib,
   retaining every public `_nux_*` C ABI export plus that exact consumer root.
4. Link with `-dead_strip -dead_strip_dylibs`, verify the C ABI export set is
   unchanged and both the exact consumer root and `wgpu_core` survived, then
   run `strip -S -x`.

This root set models an application consuming the full portable ABI and public
`WgpuFactory` / `WgpuFrame` renderer surface. It deliberately avoids two
misleading numbers:

- The raw static archive contains object code that a consuming linker removes,
  so its on-disk size is not application footprint.
- Merely enabling `nux-capi/apple-renderer` on Cargo's callback-only cdylib
  compiles the renderer but does not reference it. Fat LTO removes almost all
  renderer code, so that artifact does not measure Phase R.

Before the tooling correction, the unchanged report produced 3,782,736 B
(3.61 MiB) scripting-off and 4,684,272 B (4.47 MiB) scripting-on. Enabling
`apple-renderer` without link roots produced 3,783,168 B, only 432 B larger.
Those observations are the mechanical proof that the old artifact omitted the
renderer closure.

Actual application contribution can be smaller or larger depending on which
public APIs the host references, final-link settings, architecture, and App
Store thinning/compression. This report intentionally fixes those variables to
one conservative, reproducible per-architecture contract.

## Toolchain and target

| Input | Value |
|---|---|
| Target | Rust host `aarch64-apple-darwin`; Mach-O arm64 |
| Host | macOS 26.4.1 (25E253), Apple Silicon arm64 |
| Rust | `rustc 1.94.0 (4a4ef493e 2026-03-02)`, LLVM 21.1.8 |
| Cargo | 1.94.0 |
| Xcode | 26.6 (17F113) |
| macOS SDK | 26.5 |
| Clang | Apple clang 21.0.0 (`clang-2100.1.1.101`) |
| Linker | Apple `ld-1267` |
| Cargo profile | fat LTO, codegen-units=1, panic=unwind; `opt-level=z` |
| Final link | Darwin `clang -dynamiclib`, dead-strip closure, `strip -S -x` |

`release-size` inherits `[profile.release]`; the workspace's release panic
strategy is `unwind` because the Luau protected-error boundary requires it.
The size profile does not change the opt-level=3 release profile used by the
performance gates.

## Reproduce

```sh
make size-report
make size-report SIZE_BASELINE=1  # additionally measures the stripped opt=3 closure
```

The command fails rather than printing a partial number if the renderer or
vendored wgpu is absent, the committed renderer inventory differs from either
the public source API or the consumer harness, the compiled consumer root is
not unique, the C ABI closure is incomplete, or the linked export set changes.
The scripting-on variant must retain `nuxie-scripting` + `luaur-vm`, and the
scripting-off variant must retain neither. The command restores Cargo's
renderer-on/scripting-off `release-size` output after measuring both variants.

## Budget status — USER-GATE

The pre-Phase-R recommendation was **≤2.75 MiB per architecture**, with a
3.0 MiB alert, and tracked a different artifact that excluded the renderer.
Both numbers are now historical; neither is silently widened or repurposed.

The user must choose the new renderer-on budget and whether the blocking metric
tracks the scripting-off closure alone or requires both scripting variants.
Until that decision is recorded in `docs/parity-closeout-status.md`, #B-3 and
the size half of scorecard tier 5 remain pending.
