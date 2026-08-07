# Apple runtime distribution

The complete native Apple distribution leaf is owned by this repository.
Portable runtime crates remain platform-neutral and cannot depend on it.

```text
portable runtime + optional product crates
                  |
                  v
      nuxie-apple-adapter
                  |
                  v
        nux-apple-runtime
                  |
                  v
 NuxieRuntime.xcframework (C module NuxieRuntimeFFI)
```

`nuxie-apple-adapter` owns CAMetalDrawable validation, surface lifecycle,
presentation completion/disposition, and Apple image admission.
`nux-apple-runtime` owns the product-shaped C ABI, package authentication,
experience/screen sessions, panic containment, generated header, and module
map. It exports C only. Objective-C, C++, and Swift SDK policy are not part of
the artifact interface.

The iOS SDK consumes the binary from a pure Swift package target and supplies
the ergonomic Swift API. It does not compile Rust or own another native crate.

## Artifact contract

`make apple-runtime-xcframework` creates:

- `target/apple-runtime/NuxieRuntime.xcframework`;
- `target/apple-runtime/NuxieRuntime.xcframework.zip`; and
- `target/apple-runtime/artifact.json`.

The XCFramework contains these static-library variants:

- `ios-arm64`;
- `ios-arm64_x86_64-simulator`; and
- `macos-arm64_x86_64`.

The minimum supported versions are iOS 15.0 and macOS 12.0. Every variant uses
the same `NuxieRuntimeFFI` module map, `nux_runtime.h`, generated header, and
complete public `nux_*` symbol contract. The archive root also contains
`LICENSE` and `THIRD_PARTY_NOTICES.md`.

`artifact.json` schema 3 binds the archive to the runtime version, exact
source revision, public-header fingerprint, SwiftPM checksum, Rust/Xcode/SDK
versions, build profile, Luaur version, and minimum platforms. Each static
library embeds the same runtime identity and build provenance.

The Apple C boundary has no compatible-minor negotiation. A Swift consumer
pins one atomic artifact tuple:

- `runtimeVersion`;
- the full `sourceRevision`;
- the versioned release asset URL; and
- the exact `swiftPackageChecksum`.

The ordinary `nux_experience_context_create` entry binds to the linked runtime
internally; a SwiftPM consumer does not need to duplicate identity constants.
Hosts that independently propagate release identity may instead call
`nux_runtime_bind` and `nux_experience_context_create_bound`; that strict path
requires exact version and revision equality.

## Qualification

Install the pinned toolchain and targets once:

```sh
rustup toolchain install 1.94.1 --profile minimal --component llvm-tools
rustup target add --toolchain 1.94.1 \
  aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios \
  aarch64-apple-darwin x86_64-apple-darwin
```

Then run:

```sh
make apple-runtime-check
make apple-runtime-xcframework
```

The build rejects a dirty tree unless
`NUX_APPLE_ALLOW_DIRTY=1` is explicitly set for a diagnostic build. Diagnostic
artifacts carry a content-distinct dirty source identity and cannot be
published. The verifier checks the exact public symbol manifest, all
architectures, identical headers, Swift import/link smoke tests, iOS 15 and
macOS 12 load commands, panic-unwind support, absence of embedded LLVM bitcode,
notices, provenance, and archive checksum.

## Release

Apple releases preserve the existing `apple-runtime-v*` namespace. The next
release is `apple-runtime-v0.3.0`; it is the first version whose XCFramework
and complete Apple native implementation are co-located here.

The repository intentionally has no release workflow that assumes an
unverified macOS runner. Build and verify the artifact on a qualified Mac,
merge the exact source, create an annotated tag on the merged commit, then run
the guarded publisher from that clean tagged checkout:

```sh
git tag -a apple-runtime-v0.3.0 -m "Nuxie Apple runtime 0.3.0"
git push origin refs/tags/apple-runtime-v0.3.0
tools/publish-apple-runtime-release.sh apple-runtime-v0.3.0
```

The publisher rejects a version mismatch, dirty checkout, tag/HEAD mismatch,
commit not reachable from `origin/main`, missing or invalid artifacts, or an
existing release. It verifies before upload, creates the GitHub release with
exactly the zip and `artifact.json`, downloads both, byte-compares them, and
verifies the downloaded artifact again. Existing published assets are never
replaced.

The stable SwiftPM URL is:

```text
https://github.com/nuxieai/nuxie-runtime/releases/download/apple-runtime-v<crate-version>/NuxieRuntime.xcframework.zip
```

Use `swift package compute-checksum NuxieRuntime.xcframework.zip` and require
the result to equal `swiftPackageChecksum` from the sibling
`artifact.json`.
