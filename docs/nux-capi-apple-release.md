# Nuxie runtime C distribution

The supported Apple binary is rooted at `nux-apple-product-extension`. That
upper leaf composes the product-neutral `nux-capi` ABI with Nuxie's authored-
data converter registry and adds one explicit product-named configured-import
entrypoint. A single immutable source revision produces five thin static
libraries.
Those exact libraries are reused in two archives:

- `NuxieRuntime.xcframework.zip` contains iOS device, universal iOS simulator,
  and universal macOS libraries.
- `NuxieRuntime-iOS.xcframework.zip` contains the same iOS device and simulator
  libraries, without copying the macOS slices into an iOS SDK dependency.

Both archives expose exactly one Clang module, `NuxieRuntimeC`, backed by five
headers: the module map, product-extension umbrella header, baseline umbrella
header, generated portable header, and narrow Apple extension header. The
exported ABI is the union of three disjoint symbol partitions: portable, Apple
Metal, and the single product configured-import symbol. Experience, screen,
journey, SDK-session, package authentication, and product host-command
semantics belong to the Swift SDK and are rejected by the shipped-interface
source guard.

The portable scheduling contract is part of both packages:
`nux_player_step_result_scheduling` reports independent dirty, settled,
render-demand, revision, and optional monotonic-deadline facts, and
`nux_player_acknowledge_presented` consumes only the exact outstanding
occurrence revision. A successful Apple `Presented` disposition performs that
acknowledgement automatically; skipped or failed presentations preserve the
render demand. Both packaged C and Swift consumers compile and link these
symbols from `NuxieRuntimeC`.

The distribution has no product lifecycle runtime or migration facade. The
upper leaf installs only the authored-data converter registry before delegating
to baseline configured import. Package, experience, screen, session,
authentication, and product host-command policy remain Swift SDK
responsibilities and cannot enter the shipped Rust closure.

The `apple-runtime` target-graph check rejects `symphonia-metadata` and
`encoding_rs`. Apple audio keeps WAV/MP3/FLAC sample decoding but omits
container text-tag and artwork parsing that the public runtime never exposes.

The `apple-runtime` feature is also the device build tier. Scripting remains
enabled, but the target dependency graph contains only the Luau VM and accepts
validated editor-emitted bytecode; `luaur-compiler`, `luaur-ast`, and
`luaur-bytecode` are host/editor tools and must not appear as normal target
dependencies. The Apple compile/link matrix checks that boundary for every
packaged architecture. Build scripts may use the pinned compiler on the host to
precompile runtime-owned helper closures without adding it to an XCFramework
slice.

## Candidate qualification

From a clean checkout of the exact intended release commit:

```sh
make nux-capi-distribution-contract-test
make nux-capi-xcframeworks
```

The final command performs the five target builds once, strips embedded LLVM
bitcode, constructs both XCFrameworks, links clean C and pure-Swift consumers
for both the baseline and product entrypoints, and writes:

- `target/nux-capi-apple/artifact-set.json` (schema 6 provenance and checksums)
- `target/nux-capi-apple/SIZE_REPORT.json` (schema 2 exact compressed,
  expanded, per-slice, and representative linked sizes, plus signed deltas
  from the immutable v0.4.0 baseline)

Before release, replace the candidate sentinel in
`crates/nux-capi/size-budgets-v3.json` with reviewed release maxima. The
publisher fails closed while those values are unfrozen.

The committed v0.4.0 baseline records its tag, exact source revision, original
size-report SHA-256, and every measurement. Release maxima are frozen from the
qualified v0.6.0 authored-data build, rounded up independently to the next 1 MiB
boundary. This keeps a narrow allowance for provenance-only rebuild variation
while still ratcheting archives, expanded bundles, every thin slice, and the
larger of the baseline/product representative C and Swift linked binaries.

## Immutable release

Tag the landed commit as `apple-runtime-v<crate-version>`, then run:

```sh
tools/publish-nux-capi-release.sh apple-runtime-v<crate-version>
```

The publisher requires a clean commit that is exactly `origin/main` and whose
local and remote tag both resolve to that commit. It re-verifies both artifacts
and release size budgets, refuses an existing release, uploads all four assets
to a draft, downloads them, compares every byte, and re-verifies the downloaded
archives before making the draft public. Published assets are never replaced.
