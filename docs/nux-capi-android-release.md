# Nuxie Android runtime distribution

The 1.1.0 release adds the reference-counted shared command-queue executor for native SDK operations. It preserves ABI v4 and existing scene handles. Hosts can submit owned callbacks, close admission while draining accepted work, observe termination, and wait safely from other threads. See [the executor contract](nux-capi-executor.md).

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

The release cut is fixed at Rust 1.94.1, Bazel 9.3.0 with rules_rust 0.74.0
building `//crates/nux-capi:nux-capi__android__cdylib`, Android NDK
29.0.14206865, API 23, `arm64-v8a` plus `x86_64`, and the feature union
`android-vulkan,scripting,android-authored-wgsl`. Bazel supplies the pinned
Rust toolchain and both Android targets; the build does not use rustup or
cargo-ndk. `libc++_shared.so` is copied byte-for-byte from that NDK for each
ABI because the Vulkan runtime has a dynamic C++ runtime dependency.

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

Install the pinned Bazel launcher and point the build at the exact NDK:

```sh
tools/bazel/install.sh
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

## Immutable v0.4.14 release candidate

The artifact version in `tools/android_runtime_contract.py` is authoritative for
the builder and publisher. This candidate is not published by building it.

The v0.4.14 candidate contains everything on main since v0.4.13, through
`2739b89493`, including [UNIV-4921 / #1084](https://github.com/nuxieai/nuxie-runtime/pull/1084).
Color emoji and other color glyphs now draw in editable text fields, including
multi-glyph clusters and fallback fonts. Hosts can provide cached PNG glyph
pictures alongside outlines through the optional `get_picture` callback appended
to `NuxFontOutlineCallbacks`. Native color takes precedence; host answers are
cached per glyph and font variant with a bounded 4096-entry FIFO. Bitmap `sbix`
and CBDT placement follows HarfBuzz bearings and extents, with guarded `sbix`
reads and color classification sized for the font's glyph count. PNG answers
are copied synchronously, limited to 4 MiB and 1024 pixels per side, and require
finite em-space metrics. Callback threading, lifetime and cache rules are in
`nux_capi.generated.h`; hosts must follow them when adopting pictures.

ABI v4 and every existing export are retained. `NuxFontOutlineCallbacks` grows
from 32 to 40 bytes on the supported 64-bit targets, with the old prefix through
`release` unchanged; the runtime reads `get_picture` only when `struct_size`
covers it. `NuxFontPicture` and its sink are new public types. Existing hosts
using the old callback prefix remain accepted. There are no new C entrypoints.
The Rust `RawTextInput::invalidate_shape` hook lets a host refresh changed font
fallbacks without changing text or undo history.

The rest of this cut includes:

- CSS fit-content text measurement (UNIV-4743): capped hug text wraps before
  height measurement, intrinsic widths reuse cached shapes, paint dirt
  remeasures text participants, and fallback fonts preserve fit-content sizing.
  Layout continues to use stock Taffy/CSS semantics and can differ from Rive.
- Runtime and Luau performance work: fewer graph and property borrows, shared
  artboard hosts and advance inputs, inline transform/path/paint/text/dirt
  reads, contiguous keyframe searches, reused shape measurements and paint
  buffers, faster word-keyed maps and script field lookup, and skipped idle
  async, converter and binding walks. The pending-members-only update walk was
  reverted; the final code skips clean members during the ordinary walk.
- Upstream full-round cusp joins, cubic-chop early-out, cached path verb counts,
  host-canvas submission and target barriers, delayed paint-image retries,
  exact Vulkan bootstrap enumeration failure handling, angular-gradient
  objects, and text measurement and font reuse.
- In-band shaders use their file's script authority, scripted `quadTo` paths
  fill as cubics, and Bazel names the shared scene builder correctly. Probe
  and gate checks pin the Luau ABI, guard its native backend cast, and
  fingerprint the staged Silver validator; layout regression tests cover
  fit-content matrices, capped height and runtime minimums.

Release size ceilings remain unchanged pending candidate measurement. The Android
archive and every packaged file are checked against the committed release maxima;
its C ABI runtime version is 0.10.16.
The full change list is `git log android-runtime-v0.4.13..2739b89493`.

## Inherited v0.4.13 release

The inherited v0.4.13 release adds the video properties layer. Bindings, keyed
animation and Lua now control a playing video through the same native
properties. Loop (60004), muted (60005), volume (60006), rate (60007), loop
start (60014), loop end (60015), paused (60016) and position (60017) are
bindable and animatable while the video plays; writing position seeks. Duration
(60018), ended (60019) and state (60020) are read only and notify bindings as
decoder state and the media clock change. Keys 60008 to 60013 keep their
starting-only meanings, and an invalid write keeps the prior value. The direct
command path is cut: `nux_player_video_command` and
`nux_video_sync_group_command` accept only the host decisions pause (1),
suspend reason (6), reenter (7) and dispose (8) and return
`NUX_STATUS_INVALID_ARGUMENT` for every other kind, and `nux_player_video_step`
refuses decoder observation 7 (selected seek frame) the same way. For Lua
scripts, `video:playRange`, `video:scrub`, `video:cancelRequest` and
`video:requestStatus` are removed, `video:state()` returns the state number
(the key 60020 ordinal) instead of a name, and `video:duration()` returns 0
before metadata instead of nil. A host report that play was blocked now sets
paused to true and notifies bindings; writing false through the bound control
retries play. The video schema declares loop and muted as bool, marks the live
settings bindable and animatable, and adds keys 60016 to 60020. At load a
default two-way paused binding takes the video's starting value, so autoplay
wins; a file that chooses source-first for that binding lets the starting
view-model value win. SDKs must adopt this release together with their matching
source changes (Android in
[nuxie-android#196](https://github.com/nuxieai/nuxie-android/pull/196); iOS
main from 92770f81). An SDK that still sends the removed command kinds cannot
play video on this binary, and the new SDK sources need this binary.

The candidate also carries the runtime performance campaign (a data-oriented
component graph, shared property and paint cells, and reused draw lists and
text shape plans) and every upstream mirror since the previous release: the
string template converter, deferred canvas sampling, angular gradients with
regenerated Vulkan SPIR-V, data definitions and uint target guards, draw
batching order, empty binding rejection, the applicable a9239045 runtime fixes,
font codepoint coverage, ORE binding validation, reverse-state freezing,
selectable drawable traversal, draw-target validation, async poll pacing,
collapsed text runs and keyframe lookup, stateful scripted interpolators,
held-pose freezing, nested artboard mesh hosts, cubic vertex weight guards,
import slot ownership and stopping recursive script artboard inputs; WebGL
synchronization and target exposure, a browser backend change outside this
archive; and the parked guest frame release and deferred guest VM teardown,
accounted for without runtime behavior while guest execution stays parked. It
fixes single-pass draws losing their draw contents, hosts a video's mesh on the
image it draws through, and gives text input and scroll physics one monotonic
engine clock. Scripts can decode a trusted shader asset for native renderers,
and a direct GPU-canvas script now gets the shader name it asks for. The
upstream watermark pre-roll is removed; files that carry its manifest section
still import. This is the first release whose archive is compiled through the
repository's Bazel build.

Changes kept landing on main after this candidate was prepared, and the release
carries everything up to its tagged commit; `git log
android-runtime-v0.4.12..android-runtime-v0.4.13` lists every change. Those
that matter to SDKs follow. A drawing script can draw a video's frames like a
picture. Video gains `drawnByScript` (60021), a starting-only bool; a Video
with it set paints nothing itself, is not hit-tested, and is decoded while a
script that reads it is on screen
([nuxie-runtime#1051](https://github.com/nuxieai/nuxie-runtime/pull/1051)).
Missing characters can reach the runtime's process-wide font fallback hook
through the C API, and a host can supply a glyph's outline when the ordinary
outline is empty, following upstream's first CoreText outline recovery
([nuxie-runtime#1040](https://github.com/nuxieai/nuxie-runtime/pull/1040)). Ten
portable symbols are added: `nux_font_new`, `nux_font_new_with_outline`,
`nux_font_with_weight`, `nux_font_has_glyph`, `nux_font_free`,
`nux_font_probe_file`, `nux_font_probe_has_glyph`, `nux_font_probe_make_font`,
`nux_font_probe_free` and `nux_font_set_fallback_proc`. Immutable `NuxFont`
handles may be used across threads, hook reentry returns `REENTRANT_CALL`, and
caller-sized input structs accept appended fields. The Android SDK does not
call the font symbols yet. Layout is no longer a Rive parity target: the
runtime now lays out with stock Taffy 0.15, with CSS semantics, on a persistent
tree, in place of the vendored Yoga-order fork. Rive's Yoga defaults are stated
explicitly, but some layouts now resolve differently from Rive's; for example a
hugging text box can settle at a different width. Hosts that compare layout
output with upstream Rive should expect such differences. The other later
changes are upstream mirrors and fixes, among them renderer interlock
selection, artboard paint sources, cached bounds, and shared editor and nested
animation APIs in the Rust runtime (no C API change).

ABI v4 is retained. The font symbols and their layout entries are additions:
every v0.4.12 export and layout entry is unchanged, and no existing symbol,
struct or layout changes. The packaged `include/nux_capi.generated.h` gains the
font declarations; its other changes are documentation comments, which now
describe the refused command kinds and observation and the thread rule for
`NuxFont`. Existing release size ceilings remain unchanged, and the release
build's measurements are checked against them before publication.

The inherited v0.4.12 release adds checked replacement of several values at once. The
runtime checks a batch of value writes against the installed rules and groups
as one replacement and applies every write or none, reporting the first
refusing rule; single checked writes are unchanged. The value-rule script
module exposes that batch beside `set` as `setAll(writes)` and adds
`listValues(root, path, property)`, which reads one property from every item of
a list in order. `nux_capi.h` documents both, including the 4096-write and
8 MiB bounds. The candidate also carries four upstream mirrors: elastic scroll
momentum now releases when it reaches or crosses an edge; shader admission,
live shader asset lifecycle and integer gradient packing, with regenerated
Vulkan SPIR-V; WebGPU pseudo dynamic pipeline state, a browser backend change
outside this archive; and the deferred guest draw cleanup, accounted for
without runtime behavior while guest execution stays parked. It still stops
before the video properties layer. ABI v4 is retained with no symbol, struct or
layout change; the export lists and layout oracle match v0.4.11. SDKs reach the
new script functions by adopting this release, without source changes. Existing
release size ceilings remain unchanged pending measurement.

The inherited v0.4.11 release carries the upstream-parity fixes, platform layout fit,
globals, answered markers, value rules, form state, native text-field reads,
and retained list-row access and insertion. It stops before the video
properties layer. ABI v4 is retained; hosts must use matching headers for the
appended step-result and value-change fields and the Vulkan layout-scale
argument. Existing release size ceilings remain unchanged pending measurement.

The inherited v0.4.10 release adds `nux_player_video_present_android_hardware_buffer`,
which presents a decoded video frame without a GPU readback or CPU copy. Hosts
that decode into an `ImageReader` or `AImageReader` with GPU sampled usage pass
the frame's `AHardwareBuffer` with its crop, rotation, display size and Y'CbCr
matrix and range. The Vulkan renderer imports the buffer and converts it to
RGBA on the GPU before the call returns, so the host can return the buffer to
the decoder right away. The display size stretches video with non-square
pixels. Hosts should pass the matrix and range because drivers' own
suggestions are not reliable. Devices without
`VK_ANDROID_external_memory_android_hardware_buffer` and sampler Y'CbCr
conversion return `RuntimeError`. ABI v4 and the existing size ceilings are
retained; SDKs using the new symbol must adopt this release.

The inherited v0.4.5 behavior enumerates root, nested, and materialized list videos.
`NuxVideoInfo` appends `source_artboard_index` and `source_component_id` while
preserving existing field offsets and ABI v4. Hosts check `struct_size` before
reading the tail. The source coordinates identify the binary definition;
`component_id` addresses a live occurrence in this player. Repeated definitions
have independent playback owners. Reordering a list preserves surviving IDs;
removed IDs reject commands, and newly mounted instances receive new IDs.
The visitor copies no encoded video bytes and retains no removed scene objects.

The inherited v0.4.3 behavior preserves deferred font loading during catalog inspection with
an empty import configuration. Required-font validation applies when asset hooks
or an expected asset catalog are supplied, correcting the overly broad v0.4.2
check. Asset-bound imports reject missing or invalid fonts referenced by concrete
authored text styles. It applies to embedded and external font
bytes, preserves unused assets and unbound styles, and uses the existing import
error. System font sourcing
remains SDK-owned, and release publication does not activate SDK capability.

The inherited v0.4.1 behavior clears inferred accessibility labels when bound Text becomes
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
git tag android-runtime-v0.4.14 <full-source-sha>
git push origin android-runtime-v0.4.14
tools/publish-nux-capi-android-release.sh android-runtime-v0.4.14
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
