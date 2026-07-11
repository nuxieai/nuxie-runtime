# SDK Binary Size

Tracks the `nux-capi` cdylib size — the artifact embedded in customers' iOS /
Android apps. Since the Nuxie runtime ships as an SDK, cdylib size is an
adoption gate, so size is a tracked metric alongside the perf ratio
(docs/v2-status.md item 26(d)).

Regenerate the live numbers with:

```
make size-report              # release-size cdylib, scripting off + on
make size-report SIZE_BASELINE=1   # also builds unmodified `release` for the delta
```

All figures below are macOS `aarch64-apple-darwin` cdylib, on-disk uncompressed
bytes, measured on the branch that introduced this file. Host toolchain: rustc
1.94.0, Apple `ld-1267`.

## Profiles

`release-size` is a dedicated size profile (root `Cargo.toml`). It does **not**
touch `[profile.release]`, which the perf gate depends on staying at
`opt-level=3`, `lto=fat`, `codegen-units=1`, `panic=abort`.

```toml
[profile.release-size]
inherits = "release"   # fat LTO + codegen-units=1 + panic=abort
opt-level = "z"
strip = "symbols"
```

Override the opt-level ad hoc without editing the file:
`CARGO_PROFILE_RELEASE_SIZE_OPT_LEVEL=s cargo build --profile release-size -p nux-capi`.

## Baseline table

| Build | Bytes | MiB | vs release |
|---|---:|---:|---:|
| `release` (opt=3), unstripped — **baseline** | 4,179,920 | 3.99 | — |
| `release` (opt=3), `strip -x` | 3,691,640 | 3.52 | −11.7% |
| **`release-size` (opt=z + strip=symbols), scripting OFF** | **2,735,088** | **2.61** | **−34.6%** |
| `release-size` (opt=z + strip), scripting ON | 2,918,704 | 2.78 | −30.2% |

Scripting cost, apples-to-apples on the size profile: **+183,616 B (+6.7%)**.
On the unoptimized `release` profile the same feature costs +351,712 B (raw
4,179,920 → 4,531,632), because `opt-level=z` compresses the Luau VM harder
than `opt=3` does.

## Original per-lever deltas (each measured separately)

Starting from the unstripped `release` baseline (4,179,920 B):

| Lever | Result (B) | Δ from previous |
|---|---:|---:|
| `strip = "symbols"` (≈ `strip -x`) | 3,691,600 | −488,320 (−11.7%) |
| + `opt-level = "z"` | 2,617,024 | −1,074,576 (−29.1%) |
| + `opt-level = "s"` (alt.) | 3,234,608 | (z is −617,584 vs s) |
| + linker `-Wl,-dead_strip` | 2,617,024 | 0 (LTO already dead-strips) |
| + linker `-Wl,-icf,all` | n/a | **unsupported** by Apple `ld` |

Opt-level sweep on the size profile (with strip=symbols): `z` = 2,617,024 ·
`s` = 3,234,608 · `3` = 3,691,600. `z` wins decisively here and is the
committed default for the size profile; `s` is the perf-conscious middle
option (opt-level `z` can slow hot loops — but the perf gate runs the
untouched `release` profile, so this profile is free to pick `z`).

### Linker notes

- **ICF (identical-code folding)** — `-Wl,--icf=all` is an lld/ELF feature.
  Apple's linker (`ld-1267`, the current `ld-prime`) rejects `-icf` with
  `ld: unknown options: -icf`. There is no user-facing ICF flag on macOS;
  ld64/ld-prime does some literal/function dedup on its own at link time.
  If a future iOS/Android build path uses `lld`, `--icf=all` becomes a real
  lever there.
- **`-dead_strip`** yielded **zero** additional bytes: fat LTO plus the cdylib
  export roots already eliminate unreachable code at the IR level, and rustc
  passes dead-strip on Apple targets by default.
- **panic message trimming** — `panic=abort` is already set (inherited).
  Trimming the remaining panic *formatting/message* machinery needs the
  nightly `-Z build-std-features=panic_immediate_abort` path (see un-pulled
  levers); it is not a trivial flag on stable, so it was not applied here.

## What's in the binary (biggest contributors)

`.text` breakdown by crate, `release`/opt=3 unstripped, scripting OFF
(address-delta symbol sizing; `cargo-bloat` can't inspect a `cdylib`+`rlib`
crate, so this is `nm`-based):

| Crate | KiB | Note |
|---|---:|---|
| `nuxie_runtime` | 797 | core object/state-machine/draw model |
| fonts: `skrifa` + `harfrust` + `read-fonts` | 737 | **the text/shaping stack** |
| `std`/`core`/`alloc` | 501 | fmt + panic formatting + collections |
| generic trait impls (monomorphization) | 319 | |
| `taffy` | 190 | layout engine |
| `nuxie_binary` | 118 | .riv reader |
| `nuxie_graph` | 98 | |
| `nuxie_schema` | 58 | generated schema (code; table data lives in `__const`) |
| `luaur*` (Luau VM) | +194 | only present with scripting ON |

Answering the brief's question — the biggest contributors are **not** the
schema tables (58 KiB of code). They are `nuxie_runtime`, then the **text
stack** (skrifa/harfrust ≈ 737 KiB), then `std` formatting machinery. The
Luau VM adds ≈194 KiB of `.text` but compresses to +180 KiB net at opt=z.
Note also `__DATA_CONST/__const` ≈ 297 KiB of read-only table data (schema
property tables, vtables) that the `.text` breakdown doesn't capture.

## Budget recommendation

**Track the `release-size` scripting-OFF cdylib. Budget: ≤ 2.75 MiB per
architecture; alert if it exceeds 3.0 MiB.** Current value: **2.50 MiB**.

Rationale for a mobile paywall/flow SDK:
- On-disk uncompressed per-arch is the pessimistic figure. The App Store
  thins to the device architecture and compresses the binary, so the *download*
  impact on a customer's app is materially smaller than 2.5 MiB.
- Third-party iOS SDKs in this category (analytics, paywalls, feature flags)
  commonly land in the low-single-digit-MB range uncompressed; a self-contained
  animation + paywall runtime at 2.5 MiB is competitive and well inside the
  range where SDK size does not by itself block adoption. (Ballpark from
  general SDK-footprint norms; confirm against specific competitor SDKs before
  quoting externally.)
- The budget leaves ~250 KiB of headroom for near-term growth (renderer stubs,
  more binding surface) before the alert threshold, at which point the
  un-pulled levers below should be pulled.
- Scripting is +7% (~180 KiB) — cheap enough to ship on by default if the
  product needs it, or feature-gate for minimal builds.

## Levers NOT yet pulled (future work)

Ordered roughly by expected payoff vs. effort:

1. **Feature-gate the text/shaping stack** (~737 KiB, the single biggest
   removable chunk). Builds that render no text could drop skrifa/harfrust/
   read-fonts entirely behind a `text` feature. Highest-payoff slimming lever.
2. **`panic_immediate_abort`** via nightly build-std
   (`-Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort`).
   Strips panic message/formatting code out of `std` (overlaps the ~500 KiB
   std bucket). Requires a nightly toolchain and rebuilding std; changes panics
   to a bare `abort` with no message. Attractive for an SDK where panics abort
   into the host app anyway — pairs naturally with the capi `catch_unwind`
   firewall (item 26(d)).
3. **Feature-gate `taffy` layout** (~190 KiB) for builds that don't use the
   layout engine.
4. **Schema/`__const` table slimming** (~297 KiB of `__DATA_CONST` table data,
   plus 58 KiB schema code). Investigate whether the generated property/lookup
   tables can be made denser or partly `const fn`-computed rather than
   materialized.
5. **Monomorphization / generic bloat** (~319 KiB trait impls). Reduce over-
   generic hot paths or apply `-Z polymorphize` (nightly) — high effort, uncertain.
6. **`lld` + `--icf=all`** on the eventual iOS/Android device build paths
   (Apple `ld` can't do ICF; see linker notes). Real ICF can fold the many
   near-identical monomorphized instantiations.
7. **Phase R renderer choice.** No real renderer is linked yet. When a
   GPU/CPU rasterizer lands it will add substantial code; the renderer choice
   (full tessellation pipeline vs. a lighter path) is the largest *future*
   swing in SDK size and should be evaluated against this budget.

## How the metric is produced

`make size-report` → `tools/size-report.sh`. It builds `release-size`
(scripting off and on), optionally the `release` baseline (`SIZE_BASELINE=1`),
prints the tracked byte counts + deltas, and a `size -m` section breakdown.
The scripting-OFF `release-size` cdylib byte count is the number to record
alongside the perf ratio.
