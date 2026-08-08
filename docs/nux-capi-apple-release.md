# Nuxie runtime C distribution

The supported Apple binary is built from `nux-capi`, the product-neutral C ABI
root. A single immutable source revision produces five thin static libraries.
Those exact libraries are reused in two archives:

- `NuxieRuntime.xcframework.zip` contains iOS device, universal iOS simulator,
  and universal macOS libraries.
- `NuxieRuntime-iOS.xcframework.zip` contains the same iOS device and simulator
  libraries, without copying the macOS slices into an iOS SDK dependency.

Both archives expose `NuxieRuntimeC`. During the one migration release only,
the same archive also exposes `NuxieRuntimeFFI`, backed by the explicitly
allowlisted legacy experience/session symbols. The modules have separate
headers because their historical typedefs overlap; consumers must not include
both header families in one C translation unit.

## Candidate qualification

From a clean checkout of the exact intended release commit:

```sh
make nux-capi-distribution-contract-test
make capi-migration-contract
make nux-capi-xcframeworks
```

The final command performs the five target builds once, strips embedded LLVM
bitcode, constructs both XCFrameworks, links clean C and pure-Swift consumers,
and writes:

- `target/nux-capi-apple/artifact-set.json` (schema 6 provenance and checksums)
- `target/nux-capi-apple/SIZE_REPORT.json` (compressed, expanded, per-slice,
  and representative linked sizes)

Before release, replace the candidate sentinel in
`crates/nux-capi/size-budgets-v3.json` with reviewed release maxima. The
publisher fails closed while those values are unfrozen.

The v0.4.0 maxima are the qualified measurements rounded up independently to
the next 1 MiB boundary. This keeps a narrow allowance for provenance-only
rebuild variation while still ratcheting archives, expanded bundles, every
thin slice, and representative C and Swift linked binaries.

## Immutable release

Tag the landed commit as `apple-runtime-v<crate-version>`, then run:

```sh
tools/publish-nux-capi-release.sh apple-runtime-v<crate-version>
```

The publisher requires a clean tagged commit already contained in
`origin/main`, re-verifies both artifacts and release size budgets, refuses an
existing release, uploads all four assets in one `gh release create` operation,
downloads them, compares every byte, and re-verifies the downloaded archives.
Published assets are never replaced.
