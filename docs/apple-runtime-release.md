# Apple runtime distributions

The native Apple distribution is built from this repository. Product policy
does not enter the product-neutral C ABI merely because its consumer is Apple.

## Product-neutral destination

`nux-capi` is one C distribution with an Apple-only build feature; it does not
produce or link a second Rust archive.

```text
nuxie-renderer AppleSurface (generic Metal mechanics)
                  |
                  v
       nux-capi + apple-metal
                  |
                  v
 product-neutral XCFramework (packaging follow-up)
                  |
                  v
        pure Swift SDK wrapper
```

Swift owns CAMetalLayer configuration, retained drawable acquisition, actor and
frame scheduling, and SDK/product concepts. Rust owns the renderer domain,
draw/presentation mechanics, and device health. It never calls UIKit/AppKit or
acquires a drawable.

The C ABI exposes only the product-neutral scheduling evidence described in
`player-scheduling-contract.md`. The Apple renderer clears an occurrence's
current render demand only after the exact revision returns `Presented`;
skips and failures preserve it. Swift remains responsible for deciding when
and how to retry, and the slim release must consume this ABI rather than
recreating product scheduling in a Rust Apple harness.

## Legacy product distribution

The existing product-shaped release remains during migration:

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

`nuxie-apple-adapter` compatibility-re-exports the renderer-owned generic
surface types and owns Apple image admission.
`nux-apple-runtime` owns the product-shaped C ABI, package authentication,
experience/screen sessions, panic containment, generated header, and module
map. It exports C only. This compatibility graph is not the destination for
new renderer APIs and is retired by later migration work.

The iOS SDK consumes the binary from a pure Swift package target and supplies
the ergonomic Swift API. It does not compile Rust or own another native crate.

## Legacy artifact contract

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

`artifact.json` schema 5 binds the archive to the runtime version, exact
build-source and release revisions, audited build-input digest, public-header fingerprint,
SwiftPM checksum, Rust/Xcode/SDK versions, build profile, Luaur version, and
minimum platforms. Each static library embeds the same runtime identity,
build-input digest, and build provenance. `BUILD_INPUTS.json` is canonical JSON
inside the XCFramework; it enumerates the non-dev Cargo dependency closure for
all five Apple targets, each local package input, registry checksums and
resolved features, Cargo provider/patch/lock inputs, distribution scripts,
headers, notices, deployment targets, and toolchain identities.
Build-affecting Cargo, Rust, compiler, linker, and profile environment
overrides are rejected. Repository Cargo configuration and recursive includes
are audited; configuration discovered from Cargo home or a parent checkout is
rejected. Cargo metadata is always resolved from the repository root, and the
actual build-tool executables are hashed, so discovery cannot depend silently
on the caller's current directory or PATH. Resolved external-provider package
payloads and the host/target Rust sysroot libraries are hashed as well.

`buildInputsHash` is the SHA-256 of that canonical manifest and is the
functional build-input identity. The SwiftPM checksum, not the closure digest,
is the artifact-byte identity. `buildSourceRevision` records the commit that
produced the bytes and remains part of the compiled runtime identity;
`releaseRevision` records the clean tagged commit qualifying those bytes. They
may differ when an intervening commit changes only files outside the audited
closure. The publisher recomputes and byte-compares the canonical closure
before advancing `releaseRevision`; a mismatch requires a rebuild. Thus an
unrelated documentation or tool change can reuse the existing archive and its
checksum, while any closure, provider, target-specific, header, notice, or
toolchain change requires a new artifact.

The Apple C boundary has no compatible-minor negotiation. A Swift consumer
pins one atomic artifact tuple:

- `runtimeVersion`;
- the full artifact `buildSourceRevision`;
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
notices, the canonical dependency-closure manifest, its recomputed digest,
embedded provenance, and archive checksum. Missing, null, noncanonical, stale,
or incompletely described build-input provenance fails verification.

## Release

Apple releases preserve the existing `apple-runtime-v*` namespace. The current
release is `apple-runtime-v0.3.1`; it contains the dependency-closure-qualified
XCFramework built after the runtime acquisition-contract closeout.

The repository intentionally has no release workflow that assumes an
unverified macOS runner. Merge the release change first, check out the exact
landed `origin/main` commit on a qualified Mac, then build and verify the
artifact from that clean commit. Create the annotated tag on the same commit
and run the guarded publisher without changing checkouts:

```sh
git fetch origin
git checkout --detach <exact-landed-origin-main-sha>
make apple-runtime-check
make apple-runtime-xcframework
git tag -a apple-runtime-v0.3.1 -m "Nuxie Apple runtime 0.3.1"
git push origin refs/tags/apple-runtime-v0.3.1
tools/publish-apple-runtime-release.sh apple-runtime-v0.3.1
```

Do not reuse a pre-merge artifact after a rebase merge. Even when its source
tree is byte-identical, its producing commit is not an ancestor of the landed
release commit and the publisher correctly rejects that provenance.

The publisher rejects a version mismatch, dirty checkout, tag/HEAD mismatch,
commit not reachable from `origin/main`, missing or invalid artifacts, a
changed audited closure, or an existing release. It stamps the tagged commit
as `releaseRevision` in a temporary metadata candidate without changing the
archive or its compiled runtime identity, then verifies the candidate against
the recomputed closure. Only a successful verification atomically replaces
the original metadata.
The producing revision must itself be clean and an ancestor of the tagged
release revision. It creates the GitHub release with
exactly the zip and `artifact.json`, downloads both, byte-compares them, and
verifies the downloaded artifact again. Existing published assets are never
replaced.

Verification is source-qualified: `releaseRevision` must equal the current
clean checkout and the artifact's clean build-source commit must be its
ancestor. This preserves build-at-A/release-at-B reuse without accepting a
fabricated or stale release coordinate.

The stable SwiftPM URL is:

```text
https://github.com/nuxieai/nuxie-runtime/releases/download/apple-runtime-v<crate-version>/NuxieRuntime.xcframework.zip
```

Use `swift package compute-checksum NuxieRuntime.xcframework.zip` and require
the result to equal `swiftPackageChecksum` from the sibling
`artifact.json`.
