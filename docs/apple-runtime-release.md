# Apple runtime releases

## Exact runtime identity

The Apple C boundary still has an ABI in the ordinary sense: its calling
convention, layouts, and symbols must match the client that calls it. That ABI
is no longer a separately versioned client contract, however. A client
identifies the runtime only by:

- `runtimeVersion`, the Apple runtime crate version; and
- `sourceRevision`, the full source commit revision.

Their canonical `runtimeIdentity` is:

```text
<runtimeVersion>@<sourceRevision>
```

The SDK compiles its expected version and revision independently from the
XCFramework headers. At startup it passes those primitive values to
`nux_runtime_bind`. Exact equality returns an opaque runtime binding; any
mismatch returns the typed runtime-identity-mismatch status and a null binding.
Flow-runtime context creation requires that binding and checks it before
reading the request. This makes replacing the whole XCFramework insufficient
to replace both the actual and expected identity.

There is no supported ABI-major/minor negotiation, compatible-minor range, or
feature fallback. The exact SwiftPM URL and checksum choose one immutable
XCFramework, and the identity binding proves that the linked SDK and runtime
belong to the same release.

## Artifact contract

Schema 2 of `artifact.json` records `runtimeVersion`, the full
`sourceRevision`, their canonical `runtimeIdentity`, the SwiftPM archive
checksum, the pinned toolchain provenance, and `contractFingerprint`. It
contains no client-facing ABI version fields.

`contractFingerprint` is tooling-only. It is the SHA-256 digest of the
generated public header and is used by packaging and release verification; a
client must not negotiate behavior from it. Verification also derives the
complete public `nux_*` symbol manifest from that header and requires every
device and simulator library to export exactly that set. The metadata,
fingerprint, and embedded build provenance must agree across all slices, and
`swift package compute-checksum` must reproduce the recorded
`swiftPackageChecksum`.

The XCFramework archive includes `LICENSE` and `THIRD_PARTY_NOTICES.md` at its
root. Packaging verifies those files byte-for-byte against the release source,
records the notice path and embedded Luau version in `artifact.json`, and
checks the same Luau version in every target library's build provenance.

The SDK and XCFramework are one atomic release unit. The consumer-facing SDK
must pin all of the following together:

- the independently compiled expected `runtimeVersion` and full
  `sourceRevision`;
- the immutable release asset URL; and
- the exact `swiftPackageChecksum` from that asset's sibling `artifact.json`.

Changing any one of these requires a new atomic SDK/runtime release. Published
assets and existing pins are never edited in place.

## Release workflow

The Apple runtime release workflow is intentionally tag-only. Before its first
use, a repository administrator must:

1. Enable release immutability for the repository in GitHub settings.
2. Create a fine-grained token scoped to this repository with
   **Administration: read** permission.
3. Create and protect the `apple-runtime-release` Actions environment, then
   store that token in the environment as `NUXIE_RELEASE_ADMIN_TOKEN`.
4. Protect the `apple-runtime-v*` tag pattern so only release maintainers can
   create matching tags.

The built-in `GITHUB_TOKEN` retains the narrower `contents: write` permission
used to create the release. The administration token is exposed only to steps
that read the immutable-release setting; it is not used to create or edit
releases.

To release, first bump `crates/nux-apple-runtime/Cargo.toml`, merge the clean
release source, and push the exact tag `apple-runtime-v<crate-version>`. The
workflow rejects a tag whose commit is not already reachable from
`origin/main`. It rebuilds and verifies the XCFramework using the same pinned
Xcode and Rust versions as Apple runtime CI. It then creates a draft with
exactly `NuxieRuntime.xcframework.zip` and `artifact.json`, downloads and
compares both draft assets, rechecks that release immutability is enabled, and
publishes. Finally, it downloads the public immutable assets without
credentials and verifies their bytes, exact identity, public contract, and
SwiftPM checksum.

The workflow fails rather than changing an existing release or attaching
replacement assets. If a run fails after draft creation, inspect and delete
only that unpublished draft before retrying; a published immutable release
must never be replaced.

Dirty trees are for local diagnostics only. When explicitly allowed and the
compiled runtime source differs, packaging uses a content-distinct source
revision of the form
`<full-commit>-dirty.<content-sha256>`, which also makes the resulting
`runtimeIdentity` content-distinct. The dirty digest covers tracked changes
and new files under the bounded `crates` and `vendor` source roots; generated
tool output and the workspace `target` tree are excluded. Packaging rejects
all dirty trees by default, and an explicitly allowed diagnostic artifact
never qualifies for publication even when its runtime source is unchanged.

The `0.1.x` releases and their client pins remain immutable. The exact-identity
interface removes the previous client-facing ABI negotiation and therefore
ships as the breaking `0.2.0` release; no existing `0.1.x` artifact is
rewritten or replaced.

The customer-facing SwiftPM pin uses the published asset URL and the checksum
from its sibling `artifact.json`:

```text
https://github.com/nuxieai/nuxie-runtime/releases/download/apple-runtime-v<crate-version>/NuxieRuntime.xcframework.zip
```
