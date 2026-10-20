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

The v0.10.17 candidate contains everything on main since v0.10.16, through the import
model and schema follow-ups [#1096](https://github.com/nuxieai/nuxie-runtime/pull/1096), [#1097](https://github.com/nuxieai/nuxie-runtime/pull/1097) and [#1098](https://github.com/nuxieai/nuxie-runtime/pull/1098). Its headline is
browser-like CSS layout ([#1090](https://github.com/nuxieai/nuxie-runtime/pull/1090), [UNIV-5012](https://universe.basis.dev/issue/UNIV-5012)); the contract is
`docs/CSS_LAYOUT.md`. Published Experiences that use CSS layout need this release.

A layout box, artboard or layout participant that carries a `LayoutCssStyle`
record is in CSS mode, and stock Taffy 0.15 lays it out as a browser lays out
the same CSS: block, flex and grid flow, the browser's automatic minimum,
padding-box clips and Taffy's scroll ranges, including right-to-left viewports.
Such a box also paints its own background, borders, corner radii and shadows as
CSS does, through `BoxBorder`, `BoxShadow`, `BoxGradient` and `BoxPaint`, in CSS
painting order, with dashed and dotted borders drawn as Chrome draws them.
Pointer events in CSS content bubble to the CSS boxes around a hit, and a drag
goes to its innermost drag handler. The five records are Nuxie-owned types 61000
to 61004.

Rive files without the record are translated into CSS and laid out by the same
rules. **Translated Rive files now lay out by CSS rules**, so they can differ from
earlier releases and from the Rive editor:

- Fill shares the free space by fraction (`flex: <fraction> 1 0%`), whatever each
  box's content, where earlier releases sized each Fill from its content first.
- Hug and Fixed items shrink (`flex-shrink: 1`) when their row or column is too
  small, down to the browser's automatic minimum, where earlier releases kept
  their size and let them overflow.
- A box's automatic minimum includes its definite children, such as mounted
  artboards.
- A `ScrollConstraint`'s content box that Fills its viewport keeps a zero
  minimum on its scroll axis, so it stays the viewport's size while its items
  scroll.

Each difference from the C++ runtime on upstream's test files is reviewed and
recorded, with its reason, in `corpus.toml` and `silver-corpus.toml`.

The rest of this cut includes:

- Text decorations ([#1089](https://github.com/nuxieai/nuxie-runtime/pull/1089), [UNIV-5028](https://universe.basis.dev/issue/UNIV-5028)): underline and line-through from the
  new `TextStyleDecoration` (type 60002, `line` 60022), drawn in the glyphs' own
  paint at the font's `post` and `OS/2` metrics. Text without a decoration builds
  upstream's paths unchanged.
- Non-clockwise feathered fills draw in Metal's raster-ordering mode and on the
  MSAA backends ([#1091](https://github.com/nuxieai/nuxie-runtime/pull/1091), [UNIV-5018](https://universe.basis.dev/issue/UNIV-5018)), so CSS box shadows and such Rive
  fills are visible on iOS devices, Apple-silicon Macs and Android Vulkan.
- A bare `StringTemplateArgument` (type 156) imports as a skipped object, as C++
  leaves it ([#1096](https://github.com/nuxieai/nuxie-runtime/pull/1096)). The binary import model follows C++ at the pinned
  upstream: draw targets without a drawable and paint images outside a shape
  paint are dropped, string template arguments import only after their template,
  and paint images resolve their image and artboard ([#1097](https://github.com/nuxieai/nuxie-runtime/pull/1097), [#1098](https://github.com/nuxieai/nuxie-runtime/pull/1098)).
- Gate and test work with no runtime behaviour change: main's own checks pass
  ([#1092](https://github.com/nuxieai/nuxie-runtime/pull/1092)), the microbenchmark mirror follows the runtime's upstream pin
  ([#1093](https://github.com/nuxieai/nuxie-runtime/pull/1093)), and the C++ oracle workspace tests link and pass ([#1094](https://github.com/nuxieai/nuxie-runtime/pull/1094),
  [#1095](https://github.com/nuxieai/nuxie-runtime/pull/1095)).
- [#1087](https://github.com/nuxieai/nuxie-runtime/pull/1087)'s row shrink and its revert ([#1088](https://github.com/nuxieai/nuxie-runtime/pull/1088)) cancel out; row shrink returns
  as part of CSS layout.

ABI v4 and every existing export are retained. The C headers are unchanged and
there are no new C entrypoints.

Release size ceilings remain unchanged pending candidate measurement. Both Apple
archives are checked against the committed release maxima.
The full change list is `git log apple-runtime-v0.10.16..apple-runtime-v0.10.17`.

## Inherited v0.10.16 release

The inherited v0.10.16 release contains everything on main since v0.10.15, through
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
