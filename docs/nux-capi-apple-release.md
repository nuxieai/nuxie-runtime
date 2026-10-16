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
size-report SHA-256, and every measurement. Review maxima independently against
measured artifacts, preserving unaffected limits. The original limits use 1 MiB
rounding; the v0.10.8 visibility addition uses a narrower 128 KiB increase for
the Intel iOS simulator slice. See its
[size review](evidence/apple-video-visibility-size-review.md).
Most remain frozen from the
qualified v0.6.0 authored-data build. The v0.9.9 semantic API cut updates only
the iOS arm64 device/simulator slice ceilings and iOS-only expanded ceiling;
compressed archives, other slices and representative linked ceilings remain
unchanged. See [the v0.9.9 size review](evidence/apple-semantic-v099-size-review.md)
for the measurements and comparison with published v0.9.8.

## Immutable release

The v0.10.16 candidate contains everything on main since v0.10.15, through
`2739b89493`, including [UNIV-4921 / #1084](https://github.com/nuxieai/nuxie-runtime/pull/1084).
Colour emoji and other colour glyphs now draw in editable text fields, including
multi-glyph clusters and fallback fonts. Hosts can provide cached PNG glyph
pictures alongside outlines through the optional `get_picture` callback appended
to `NuxFontOutlineCallbacks`. Native colour takes precedence; host answers are
cached per glyph and font variant with a bounded 4096-entry FIFO. Bitmap `sbix`
and CBDT placement follows HarfBuzz bearings and extents, with guarded `sbix`
reads and colour classification sized for the font's glyph count. PNG answers
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

Release size ceilings remain unchanged pending candidate measurement. Both Apple
archives are checked against the committed release maxima.
The full change list is `git log apple-runtime-v0.10.15..2739b89493`.

## Inherited v0.10.15 release

The inherited v0.10.15 release adds the video properties layer. Bindings, keyed
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
source changes (iOS main from 92770f81; Android in
[nuxie-android#196](https://github.com/nuxieai/nuxie-android/pull/196)). An SDK
that still sends the removed command kinds cannot play video on this binary,
and the new SDK sources need this binary.

The candidate also carries the runtime performance campaign (a data-oriented
component graph, shared property and paint cells, and reused draw lists and
text shape plans) and every upstream mirror since the previous release: the
string template converter, deferred canvas sampling, angular gradients with
their shader changes, data definitions and uint target guards, draw batching
order, empty binding rejection, the applicable a9239045 runtime fixes, font
codepoint coverage, ORE binding validation, reverse-state freezing, selectable
drawable traversal, draw-target validation, async poll pacing, collapsed text
runs and keyframe lookup, stateful scripted interpolators, held-pose freezing,
nested artboard mesh hosts, cubic vertex weight guards, import slot ownership
and stopping recursive script artboard inputs; WebGL synchronization and target
exposure, a browser backend change outside these archives; and the parked guest
frame release and deferred guest VM teardown, accounted for without runtime
behavior while guest execution stays parked. It fixes single-pass draws losing
their draw contents, hosts a video's mesh on the image it draws through, and
gives text input and scroll physics one monotonic engine clock. Scripts can
decode a trusted shader asset for native renderers, and a direct GPU-canvas
script now gets the shader name it asks for. The upstream watermark pre-roll is
removed; files that carry its manifest section still import. This is the first
release whose archives are compiled through the repository's Bazel build.

Changes kept landing on main after this candidate was prepared, and the release
carries everything up to its tagged commit; `git log
apple-runtime-v0.10.14..apple-runtime-v0.10.15` lists every change. Those that
matter to SDKs follow. A drawing script can draw a video's frames like a
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
caller-sized input structs accept appended fields. Layout is no longer a Rive
parity target: the runtime now lays out with stock Taffy 0.15, with CSS
semantics, on a persistent tree, in place of the vendored Yoga-order fork.
Rive's Yoga defaults are stated explicitly, but some layouts now resolve
differently from Rive's; for example a hugging text box can settle at a
different width. Hosts that compare layout output with upstream Rive should
expect such differences. The other later changes are upstream mirrors and
fixes, among them renderer interlock selection, artboard paint sources, cached
bounds, and shared editor and nested animation APIs in the Rust runtime (no C
API change).

ABI v4 is retained. The font symbols and their layout entries are additions:
every v0.10.14 export and layout entry is unchanged, and no existing symbol,
struct or layout changes. The packaged headers gain the font declarations;
their other changes are documentation comments, which now describe the refused
command kinds and observation and the thread rule for `NuxFont`. Existing
release size ceilings remain unchanged, and the release build's measurements
are checked against them before publication.

The v0.10.14 candidate adds checked replacement of several values at once. The
runtime checks a batch of value writes against the installed rules and groups
as one replacement and applies every write or none, reporting the first
refusing rule; single checked writes are unchanged. The value-rule script
module exposes that batch beside `set` as `setAll(writes)` and adds
`listValues(root, path, property)`, which reads one property from every item of
a list in order. `nux_capi.h` documents both, including the 4096-write and
8 MiB bounds. The candidate also carries four upstream mirrors: elastic scroll
momentum now releases when it reaches or crosses an edge; shader admission,
live shader asset lifecycle and integer gradient packing, with regenerated
Metal shaders; WebGPU pseudo dynamic pipeline state, a browser backend change
outside these archives; and the deferred guest draw cleanup, accounted for
without runtime behavior while guest execution stays parked. It still stops
before the video properties layer. ABI v4 is retained with no symbol, struct or
layout change; the export lists and layout oracle match v0.10.13. SDKs reach
the new script functions by adopting this release, without source changes.
Existing release size ceilings remain unchanged pending measurement.

The v0.10.13 candidate carries the upstream-parity fixes, platform layout fit,
globals, answered markers, value rules, form state, native text-field reads,
and retained list-row access and insertion. It stops before the video
properties layer. ABI v4 is retained; the step result, value change result,
and Metal render operation each grow by eight bytes. Hosts must compile
against the matching headers and follow the size-gated field contracts.
The Metal and Vulkan present corrections are host-layer changes documented
in their headers, including Vulkan's layout scale argument. Existing release
size ceilings remain unchanged pending measurement.

The v0.10.12 candidate adds `nux_player_video_present_metal_pixel_buffer`, which
presents a decoded video frame without a CPU copy. Hosts pass an
IOSurface-backed 32BGRA `CVPixelBuffer`, such as `AVPlayerItemVideoOutput` vends
when Metal compatibility is requested; the renderer samples that surface in
place and keeps the buffer until GPU work reading it completes. Other formats
and memory-backed buffers are rejected. Generation, renderer-domain and 64 MiB
rules match `nux_player_video_present_metal`, and colors are identical to that
byte path. The renderer now calls Core Video, so both module maps link
`CoreVideo` and SDK targets must link it too. ABI v4 and the existing size
ceilings are retained; SDKs using the new symbol must adopt this release.

The v0.10.11 candidate adds `nux_player_text_input_content_offset_set` for
native editors to synchronize their content displacement with an authored
TextInput's existing ScrollConstraint. The stationary field viewport remains
the geometry and accessibility basis. This presentation-only operation does
not edit text, selection, bindings, or responses. ABI v4 is retained; SDKs using
the new symbol must adopt this release rather than the v0.10.10 binary.

The former v0.10.9 request API described below is replaced by the video properties in [the current host contract](video-runtime-host-contract.md#video-properties-and-script-control); its request and hold methods are no longer available.

The v0.10.9 candidate adds script-owned video ranges, terminal hold, duration,
coalesced scrubbing and observable request completion. Browser and Apple hosts
preserve initial and same-position paused frames. A completed endpoint seek can
acknowledge its actual decoded frame even when that frame precedes the source
duration by more than the ordinary seek tolerance. ABI v4 and the existing size
ceilings remain unchanged. The SDK must adopt this release before authored
scripts use the new video methods; a local XCFramework override does not update
its public package pin. A seek into an audio-only tail can still yield no decoded
image; endpoint acknowledgement does not synthesize missing decoder output.

The v0.10.5 candidate enumerates root, nested, and materialized list videos.
`NuxVideoInfo` appends `source_artboard_index` and `source_component_id` while
preserving existing field offsets and ABI v4. Hosts check `struct_size` before
reading the tail. The source coordinates identify the binary definition;
`component_id` addresses a live occurrence in this player. Repeated definitions
have independent playback owners. Reordering a list preserves surviving IDs;
removed IDs reject commands, and newly mounted instances receive new IDs.
The visitor copies no encoded video bytes and retains no removed scene objects.

The inherited v0.10.3 behavior preserves deferred font loading during script-inert catalog
inspection with an empty import configuration. Required-font validation applies
when asset hooks or an expected asset catalog are supplied. This corrects
v0.10.2, which rejected external-font catalog inspection before SDKs could bind
authenticated bytes.

Asset-bound imports reject missing or invalid fonts referenced by concrete authored
text styles. Embedded and external bytes use the same
validation. Unused font assets and unbound styles remain admissible. This uses
the existing import error without public ABI or scene-format additions; System
font sourcing and capability activation remain SDK responsibilities.

The v0.10.1 patch separates explicit ViewModel binding from ordinary player
advancement, matching the upstream host lifecycle. New players inherit the
artboard context; later bindings and replacement models are applied once.
This prevents duplicate pending-trigger execution without changing translated
trigger semantics or the public ABI.

The v0.10.0 candidate adds the `.nux` RIV-superset video scene extension,
capability-gated import, occurrence playback and caption APIs, synchronized
playback groups, and Metal/Vulkan frame submission. ABI v4 is retained: the
import configuration uses a size-gated appended capability pointer and older
prefixes continue to reject video while accepting ordinary RIV scenes. The
host integration contract is in [video-runtime-host-contract.md](video-runtime-host-contract.md).
Native decoder adapters and device fixtures are source integrations; SDK
packaging and published media delivery remain separate adoption work.
Candidate packaging and device evidence must pass before this version ships.
The [video size review](evidence/apple-video-v010-size-review.md) raises only
the iOS-only expanded ceiling by one MiB after measuring the added runtime.

It also preserves the v0.9.15 binding-feedback fix, which settles internal view-model binding feedback within the
same player frame. Layout measurements can update dependent paint geometry
before publication; external observers remain commit-gated and host mutation
batches retain atomic publication. SDK pixel/device qualification remains a
separate adoption gate.

The v0.9.14 release additionally preserves float vertex coverage in every
Metal shader variant. Precompiled feather-capable and specialized non-feather
shaders no longer disagree because of half-precision rounding before
interpolation. A published text scene with its exact external font is compared
pixel-for-pixel across forced shader variants and repeated frames.

The v0.9.13 dither and raster-order blend fixes remain: both variants enable
the same dither helpers and use explicit fused multiply-add for color blending.
Native pixel tests also cover opaque and translucent fills and fractional
edges without relaxing equality. Other rendering backends retain their
existing coverage precision.

Optional frame capture copies into caller-owned shared Metal storage before
drawable presentation. Capture uses
the renderer's presentation queue and waits for GPU completion. It requires a
readable BGRA8Unorm drawable, a shared buffer on the same device, and a row
stride covering the pixels and aligned to 256 bytes. Rejected and skipped
frames leave the buffer untouched. Older operation prefixes disable capture;
all existing field offsets remain unchanged. The additive extension keeps
ABI v4 and the existing release size ceilings. Candidate artifact qualification
must pass before publication.

Frame-qualified geometry reads continue to require a live successful step
result matching the current occurrence and render revision; stale mutations,
foreign occurrences and ambiguous names fail closed. See
[the geometry contract](text-run-geometry.md).

Runtime publication does not enable the SDK scene-semantics capability or qualify
VoiceOver behavior. SDK adoption, publisher-generated signed fixtures and device
acceptance remain separate delivery gates.

Tag the landed commit as `apple-runtime-v<crate-version>`, then run:

```sh
tools/publish-nux-capi-release.sh apple-runtime-v<crate-version>
```

The publisher requires a clean commit that is exactly `origin/main` and whose
local and remote tag both resolve to that commit. It re-verifies both artifacts
and release size budgets, refuses an existing release, uploads all four assets
to a draft, downloads them, compares every byte, and re-verifies the downloaded
archives before making the draft public. Published assets are never replaced.
