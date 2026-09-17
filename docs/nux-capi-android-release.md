# Nuxie Android runtime distribution

`NuxieRuntimeAndroid.zip` is the immutable native runtime input consumed by
`nuxie-android`. It is built directly from `nux-capi`; no JNI or Kotlin SDK
code is part of this archive.

The archive contains exactly five regular files and no directory entries:

```text
include/nux_capi.generated.h
jniLibs/arm64-v8a/libc++_shared.so
jniLibs/arm64-v8a/libnux_capi.so
jniLibs/x86_64/libc++_shared.so
jniLibs/x86_64/libnux_capi.so
```

The release cut is fixed at Rust 1.94.1, cargo-ndk 4.1.2, Android NDK
29.0.14206865, API 23, `arm64-v8a` plus `x86_64`, and the feature union
`android-vulkan,scripting,android-authored-wgsl`. `libc++_shared.so` is copied
byte-for-byte from that NDK for each ABI because the Vulkan runtime has a
dynamic C++ runtime dependency.

The ZIP writer fixes entry order, timestamps, Unix modes, compression level,
and metadata. The release evidence records the full 40-character source SHA,
the audited source and toolchain inputs, the ABI-v4 contract fingerprint, the
archive SHA-256, and each of the five file SHA-256 values. The verifier checks
the committed ABI-v4 layout oracle, the full header inventory, the selected
Android header inventory, both shared libraries' exact `nux_*` export union,
ELF64 architecture, exact `DT_NEEDED` sets, at least 16 KiB alignment for every
`LOAD` segment in every shipped shared library, embedded schema-6 provenance,
NDK `libc++_shared.so` identity, and the committed aggregate and per-file size
budget.

## Candidate qualification

Install the exact toolchain and both Rust Android targets:

```sh
rustup toolchain install 1.94.1
rustup target add --toolchain 1.94.1 aarch64-linux-android x86_64-linux-android
cargo install --locked cargo-ndk --version 4.1.2
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
```

From a clean checkout of the intended source commit:

```sh
make nux-capi-android-contract-test
make nux-capi-android
```

The build writes these four release assets under
`target/nux-capi-android/`:

- `NuxieRuntimeAndroid.zip`
- `NuxieRuntimeAndroid.json`
- `NuxieRuntimeAndroid-BUILD_INPUTS.json`
- `NuxieRuntimeAndroid-SIZE_REPORT.json`

The build refuses dirty source, unpinned compiler tools, unexpected compiler
overrides, missing ABI-v4 inventories, a noncanonical archive tree, or any
failed qualification check. The budget in
`tools/android-runtime-size-budget-v4.json` is a release ceiling, not a target;
lower measurements do not require padding or other byte changes.

## Immutable v0.4.1 release candidate

The artifact version in `tools/android_runtime_contract.py` is authoritative for
the builder and publisher. This candidate is not published by building it.

The v0.4.1 candidate clears inferred accessibility labels when bound Text becomes
empty and avoids rebinding player view models on frames without explicit binding
changes. It also includes the corrected video crop mesh composition. It retains
ABI v4, the existing size ceilings, and v0.4.0's capability-gated `.nux` video
import, occurrence playback/caption APIs, synchronized groups and Vulkan frame
submission. Decoder adapters are source
integrations; Java packaging, lifecycle dispatch and asset acquisition in SDKs
remain subsequent adoption work. See the [video host contract](video-runtime-host-contract.md).
The frame-qualified text/layout geometry API from v0.3.13 is preserved.

It also preserves the v0.3.14 binding-feedback fix, which settles internal view-model binding feedback within the
same player frame, while retaining commit-only external observer publication
and host batch atomicity. This fixes delayed dependent paint geometry after
layout measurements change.

The release retains the same frame-qualified text/layout geometry API as
Apple v0.9.11, including the first logical baseline. Exact root-run ownership,
creator-lane/lifetime rules and current occurrence/revision validation apply;
no text contents are copied. It keeps ABI v4 and the existing size ceilings.
See [the geometry contract](text-run-geometry.md). Public native artifacts do
not establish native editor pixel/caret alignment or IME behavior; SDK adoption
and platform device qualification remain separate gates.

After the qualified commit has landed as `origin/main`, create and push the tag
at that exact commit:

```sh
git tag android-runtime-v0.4.1 <full-source-sha>
git push origin android-runtime-v0.4.1
tools/publish-nux-capi-android-release.sh android-runtime-v0.4.1
```

The publisher requires a clean checkout whose `HEAD`, `origin/main`, local
tag, remote tag, metadata source revision, and embedded binary provenance all
identify the same commit. It refuses an existing release. It uploads the four
qualified assets to a draft, downloads every asset, compares every byte,
re-runs full verification on the downloaded copy, and only then publishes the
draft. A failed post-upload check intentionally leaves the draft unpublished;
published assets are never replaced.

After publication, `nuxie-android/runtime/artifact.json` should pin the new tag,
full source commit, three-feature union, and SHA-256 of the downloaded ZIP.
