# Apple runtime exact-identity contract

## Objective

Replace client-visible Apple ABI major/minor negotiation with an exact runtime
build identity. An Apple client consumes one immutable XCFramework built from
one runtime version and source revision; the SwiftPM URL and checksum select
that artifact. The C layout remains an ABI, but clients no longer select,
negotiate, or branch on a separate ABI version.

## Public contract

1. The runtime exposes its `runtimeVersion` and full `sourceRevision` as one
   exact runtime identity.
2. A permanently stable primitive-only bootstrap call compares an
   independently compiled expected identity with the linked runtime and
   returns an opaque binding only on exact equality. A mismatch returns a
   typed runtime-identity-mismatch status and a null binding.
3. Strict context creation accepts that binding and validates it before
   reading any request pointer. The ordinary convenience entry binds to the
   linked runtime internally, so checksum-pinned clients do not need to
   duplicate generated identity constants. Context and session handles then
   need no repeated identity fields.
4. `NuxFlowConfiguredSessionDescriptor` and `NuxFlowSessionOperation` no
   longer carry `required_abi_major` or `minimum_abi_minor`.
5. Every caller-owned singleton input retains `struct_size`; the runtime
   requires the exact current size before reading the rest of the structure.
   In particular, the configured descriptor is exactly 40 bytes and the
   generic operation is exactly 48 bytes on supported 64-bit Apple targets.
6. Configured sessions always use the current typed `player_kind` contract.
   There is no ABI-1.5 fallback interpretation.
7. Artifact metadata and embedded build provenance contain runtime version,
   source revision, one exact runtime identity, a tooling-only public-contract
   fingerprint, artifact checksum, and toolchain provenance, but no
   client-facing ABI version fields.
8. Published SDK and XCFramework releases are atomic: the SDK pins the exact
   artifact URL/checksum. No compatible-minor range is accepted.
9. A host may check in an expected identity independently from the
   XCFramework's headers when it needs an additional release-identity check.
   The standard SwiftPM consumer does not need to do so.
10. The SwiftPM checksum remains the artifact-byte authority. The optional
    identity binding lets other hosts independently prove that their expected
    release and linked runtime match.

## Runtime repository scope

- `crates/nux-apple-runtime/src/lib.rs`
- `crates/nux-apple-runtime/src/session.rs`
- generated C header and C/Swift smoke tests
- `crates/nux-apple-runtime/build.rs`
- XCFramework build/verification scripts and artifact-validator tests
- Apple runtime release documentation and focused exact-identity evidence
- release the co-located distribution as runtime `0.3.0`; existing immutable
  `0.1.x` and `0.2.x` clients remain pinned to their existing artifacts

Do not change renderer, runtime frame-loop, Scene, or Editor code. Do not
publish a release in this slice.

## Downstream scope

After the runtime change lands and an exact immutable artifact is approved:

- remove legacy ABI negotiation from `nuxie-ios` without inventing duplicate
  generated runtime-identity constants;
- remove `sessionMinimumMinor` and ABI fields from native descriptors;
- pin the exact release URL/checksum;
- rerun the unchanged P18 artifact, animation-selection, and native-pixel
  corpus.

The downstream pin cannot be finalized before the runtime merge SHA and
release checksum exist.

## TDD acceptance

1. Exact runtime version + exact source revision succeeds.
2. Wrong version, wrong revision, empty identity, and malformed views return
   a null binding and runtime-mismatch or invalid-argument as appropriate.
3. A mismatched binding is rejected before a deliberately unreadable request
   pointer is touched.
4. Configured-session and operation structures have no ABI-version members;
   wrong `struct_size` fails before any trailing field is read.
5. The configured descriptor is exactly 40 bytes and the generic operation is
   exactly 48 bytes on the supported 64-bit Apple targets; legacy 48-byte
   descriptors and 56-byte operations fail closed.
6. Named state-machine and linear-animation selection still pass.
7. Generated C and Swift imports compile using only runtime identity.
8. Packaged metadata matches the embedded identity in every device/simulator
   library, the complete public-symbol manifest and header fingerprint agree,
   and the SwiftPM checksum verifies.
9. Focused negative packaging tests reject an obsolete metadata schema,
   client-visible ABI fields, malformed identity/fingerprint/checksum values,
   and missing or undeclared public symbols.
10. Full Apple product, artifact validator, header, strict Clippy, formatting,
    and relevant workspace floors remain green.
11. Cargo reuses an unchanged local build, but creating, changing, or removing
    a new untracked source input regenerates a content-distinct diagnostic
    identity; a caller-supplied false revision is rejected.

## Non-goals

- Backward compatibility with ABI 1.5 callers.
- Runtime feature fallback based on client version.
- Publishing or modifying an existing immutable release.
- Changing serialized Rive data/schema versions.
- Broad cleanup outside the Apple C boundary.
- Allowing dirty builds to qualify for release. Local builds with dirty
  runtime source must use a content-distinct diagnostic identity; packaging
  rejects any dirty release tree.
