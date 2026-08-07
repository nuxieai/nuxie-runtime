# SDK Binary Size

This is the reproducible binary-size evidence behind the SDK size budget.
The tracked artifact is the Darwin SDK link closure, renderer included:
the portable `nux-capi` ABI with the pure-Rust `nuxie-renderer` and vendored
`wgpu` backend retained. It is measured with scripting both disabled and
enabled.

**Budget (decided 2026-07-21, user-approved): 9 MiB = 9,437,184 B,
blocking for BOTH scripting variants.** `make size-report` fails when either
link closure exceeds it, and `make parity-scorecard` validates the recorded
evidence against `size.budget_bytes` in `parity-scorecard.toml`. A breach
reopens the budget USER-GATE with fresh measurements — the constant is never
silently raised.

**Suspension (coordinator decision, 2026-07-30, user-directed): the hard
gate above is suspended until the FL series (through FL-E) is complete.**
Rationale: FL-D adds ~94 files of DataBind/ViewModel ownership and FL-E the
live-draw owners; enforcing a fixed budget mid-port would either block
faithful porting or invite size-motivated compensation, both worse than
measuring honestly and deciding once the full surface exists. During the
suspension `make size-report` remains in every floor and prints the same
measurements plus a NOTE (never a failure) when a variant exceeds the 9 MiB
reference. At FL-E completion the budget USER-GATE reopens with complete
measurements and the binding number is set then.

History: the initial 8 MiB choice was made against the 2026-07-20
measurements below, which predate concurrent main `974aab66` (editor-cutover
runtime support). Re-measurement at `2f82f9e7`, including the 43rd audited
renderer root `Factory::make_gpu_canvas_image` that `974aab66` added to the
public surface, reported scripting OFF at 8,216,984 B (7.84 MiB) and
scripting ON at 9,118,104 B (8.70 MiB) — ON breached, the gate reopened the
same day, and the user approved the 9 MiB replacement (≈3.4% headroom over
scripting ON).

## Baseline evidence snapshot

Measured 2026-07-20 at source revision `d8091cd5`, using the then-current
42-entry core-renderer and Darwin-presentation consumer harness. The active
harness now audits 43 renderer-owned roots, including the opaque Metal
presenter used by the runtime-owned Apple adapter.
The committed baseline snapshot records its exact measurement revision,
artifact digests, toolchain, public-root inventory, and symbol-size breakdown in
[`docs/evidence/size-b3-2026-07-20.md`](evidence/size-b3-2026-07-20.md). Two
consecutive runs of `make size-report` produced the same output and
byte-identical artifacts.

| `release-size` link closure | Bytes | MiB | Delta from scripting OFF |
|---|---:|---:|---:|
| Renderer ON, scripting OFF — tracked metric | **7,534,056** | **7.19** | — |
| Renderer ON, scripting ON | **8,335,288** | **7.95** | **+801,232 B (+10.6%)** |

The historical budget was **2.75 MiB = 2,883,584 B** per architecture. The new
renderer-on, scripting-off measurement is **4,650,472 B (+161.3%) above** that
number. This is informational only: `make size-report` does not enforce the old
budget or infer a replacement.

The scripting-off section layout reported by Apple's `size -m` is:

| Mach-O region | Bytes |
|---|---:|
| `__TEXT` segment | 6,864,896 |
| `__text` section | 4,685,328 |
| `__const` section in `__TEXT` | 1,076,220 |
| `__cstring` | 119,544 |
| `__unwind_info` | 168,984 |
| `__eh_frame` | 421,356 |
| `__DATA_CONST` segment | 524,288 |
| `__const` section in `__DATA_CONST` | 509,896 |
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
   with `--no-default-features --features nuxie/renderer`; add
   `nuxie/scripting` for the scripting-on variant. Separately build
   the non-shipping `nuxie-size-report-roots` tooling crate.
2. Verify the resolved dependency graph contains `nuxie-renderer` and the
   repository's vendored `wgpu` 30.0.0.
3. Verify the measurement consumer's 43 calls exactly match the public renderer
   methods: 19 `WgpuFactory`/`WgpuFrame` methods, six opaque Metal-presenter
   methods, ten `Factory` methods, and eight `Renderer` methods. Re-link both staticlibs as
   one Mach-O dylib, retaining every public `_nux_*` C ABI export plus that
   exact consumer root.
4. Link with `-dead_strip -dead_strip_dylibs`, verify the C ABI export set is
   unchanged and both the exact consumer root and `wgpu_core` survived, then
   run `strip -S -x`.

This root set models an application consuming the full portable ABI, public
`WgpuFactory` / `WgpuFrame` renderer surface, and the opaque
`WgpuMetalPresenter` used by Apple presentation. The shipping XCFramework has
its own end-to-end artifact verification; this renderer closure intentionally
measures only the shared portable baseline. It
deliberately avoids two misleading numbers:

- The raw static archive contains object code that a consuming linker removes,
  so its on-disk size is not application footprint.
- Merely enabling `nuxie/renderer` on Cargo's callback-only cdylib compiles the
  renderer but does not reference it. Fat LTO removes almost all renderer code,
  so that artifact does not measure the renderer.

Before the tooling correction, the unchanged report produced 3,782,736 B
(3.61 MiB) scripting-off and 4,684,272 B (4.47 MiB) scripting-on. Enabling
the former `apple-renderer` feature without link roots produced 3,783,168 B,
only 432 B larger.
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

The command fails rather than printing a partial number if tracked sources are
dirty, the claimed source revision differs from `HEAD`, the renderer or
vendored wgpu is absent, the committed renderer inventory differs from either
the public source API or the consumer harness, the selector cannot reach every
root, the compiled consumer root is not unique, the C ABI closure is
incomplete, or the linked export set changes.
The scripting-on variant must retain `nuxie-scripting` + `luaur-vm`, and the
scripting-off variant must retain neither. The command restores Cargo's
renderer-on/scripting-off `release-size` output after measuring both variants.

## Budget status

The budget decision is recorded at the top of this document: 9 MiB, blocking
for both scripting variants, enforced by `make size-report` and validated by
`make parity-scorecard` against `size.budget_bytes` in `parity-scorecard.toml`.

The renderer-excluded recommendation that preceded it (**≤2.75 MiB per
architecture**, 3.0 MiB alert) tracked a different artifact and is historical;
neither number is silently widened or repurposed.
