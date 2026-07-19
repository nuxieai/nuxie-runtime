# Apple runtime releases

The Apple runtime release workflow is intentionally tag-only. Before its first use, a repository administrator must:

1. Enable release immutability for the repository in GitHub settings.
2. Create a fine-grained token scoped to this repository with **Administration: read** permission.
3. Create and protect the `apple-runtime-release` Actions environment, then store that token in the environment as `NUXIE_RELEASE_ADMIN_TOKEN`.
4. Protect the `apple-runtime-v*` tag pattern so only release maintainers can create matching tags.

The built-in `GITHUB_TOKEN` retains the narrower `contents: write` permission used to create the release. The administration token is exposed only to steps that read the immutable-release setting; it is not used to create or edit releases.

To release, first bump `crates/nux-apple-runtime/Cargo.toml`, merge the release
source, and push the exact tag `apple-runtime-v<crate-version>`. The workflow
rejects a tag whose commit is not already reachable from `origin/main`. It
rebuilds and verifies the XCFramework using the same pinned Xcode and Rust
versions as Apple runtime CI. It then creates a draft with exactly
`NuxieRuntime.xcframework.zip` and `artifact.json`, downloads and compares both
draft assets, rechecks that release immutability is enabled, and publishes.
Finally, it downloads the public immutable assets without credentials and
verifies their bytes and SwiftPM checksum.

The workflow fails rather than changing an existing release or attaching
replacement assets. If a run fails after draft creation, inspect and delete
only that unpublished draft before retrying; a published immutable release
must never be replaced.

The customer-facing SwiftPM pin uses the published asset URL and the checksum
from its sibling `artifact.json`:

```text
https://github.com/nuxieai/nuxie-runtime/releases/download/apple-runtime-v<crate-version>/NuxieRuntime.xcframework.zip
```
