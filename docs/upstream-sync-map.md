# Incremental upstream sync

Maintain the completed C++→Rust port by applying upstream deltas, not by
restarting a whole-runtime migration. [PARITY_WORKFLOW.md](PARITY_WORKFLOW.md)
defines the source-first translation and two separate review passes. This
document replaces the former large-cycle, scored-row, and ratchet workflow.

Keep the Nuxie Video hooks in `core_registry.rs` when syncing: `set_bool_with_completion`, `set_double_with_completion`, `get_bool`, `get_double`, `property_field_id` and `object_supports_property` route keys 60004 through 60020 to Video.

Keep the CSS scroll mode hooks in `core_registry.rs` too: `ScrollConstraint`'s `virtualize` and `infinite` arms in `set_bool_with_completion` call `ScrollConstraint::set_virtualize_value` and `set_infinite_value`, which leave a loaded CSS viewport's flags unchanged, and its `CoreCapabilities` implement `lifecycle_on_added_clean` with `ScrollConstraint::on_added_clean`, where a viewport's scroll mode is decided once (`docs/CSS_LAYOUT.md` section 3). Translated viewports keep upstream's setters and toggles.

## Known Apple font boundary gap

The host outline addition mirrors the first `CoreTextHBFont::getPath`
attempt at `daf55385:font_hb_apple.mm:468-492`: read the ordinary outline,
then ask the host only if it is empty. `nux_font_new_with_outline` adds a
plain C command sink. The iOS SDK retains its selected CTFont at size2048,
inverts y and divides by2048, matching upstream `apply_element`. The runtime
converts quadratics to cubic paths. Ordinary byte/file font creation retains
its existing behavior. As with upstream `HBFont::withOptions`, option variants
are plain HB fonts and do not retain host outline recovery. Callback retention,
validation, the bounded command sink, and C handle safety are our host machinery.

Upstream also provides CoreText tables and units-per-em at construction, and
on its second attempt (`useSystemShaper=true`) shapes with CTTypesetter and
CTRun. Those native answers include glyph IDs, string indices, advances,
forced bidi embedding level, visual reversal for RTL, and zero offsets. Its
system-shaped path is asked from CoreText before HarfBuzz. The current host
outline addition implements the first-attempt outline recovery. The second
CoreText-shaped attempt is still absent and must not be claimed as parity.
Metrics, coverage, weights, feature/axis queries and color layers inherit
HBFont in the pinned native subclass.

For the selected iOS26.5 Apple Color Emoji face, the retained probe establishes
emjc payloads in every sbix strike and nil CoreText glyph paths. The pinned
runtime uses rive-app/harfbuzz rive_13.1.1. Its sbix reference_png requests
png-space, rejects a different graphicType and returns an empty blob.
HBFont::isColorGlyph then returns false. Text::updateRenderStyles reads
getPath; both the HarfBuzz outline and CoreText outline are empty. Even if a
color command were emitted, empty color layers cause drawColorGlyph to return.
Therefore the source trace predicts no emoji ink upstream in this exact case.
This is source evidence, not a direct upstream simulator rendering verdict.
UNIV-4921 now adds the bounded host-picture boundary described below. The
runtime still does not decode emjc; the iOS SDK must rasterize it through the
platform and supply PNG pictures. Device image positioning/scale/color
qualification and SDK adoption remain separate work.

Track device qualification in [UNIV-3375](https://universe.basis.dev/issue/UNIV-3375).

### UNIV-4921 additive color glyph boundaries

- **host-glyph-pictures**: `HbFont::with_host_pictures` and the appended
  `NuxFontOutlineCallbacks::get_picture` opt a font into platform PNG answers.
  Native COLR/readable PNG wins. Rust sources declare `HostPictureScope::Glyph`
  or `HostPictureScope::Cluster`; the C callback always uses glyph scope.
  Glyph scope requests every nonzero glyph without native color, keyed by
  glyph id. Cluster code points are context only; the first context seen supplies
  the cached answer. Every component can draw; hosts return None for spaces and
  hidden glyphs without ink. Cluster scope requests only the cluster's first glyph
  in logical shaping order, keyed by (glyph id, complete cluster text); a picture
  suppresses the same font's remaining cluster glyphs. Neither scope requests
  glyph 0. Present and absent answers share a 4096-entry FIFO cache independently
  per font variant; evicted entries may be requested again. Both scopes count as
  color capability for VS16 fallback. Host-only cluster metadata is captured
  before fallback/ellipsis slices.
  Pictures compose with empty-outline recovery; option variants retain the
  picture source with a fresh answer cache, while preserving upstream's policy
  that plain HB variants do not retain host outline recovery.
  The C sink copies synchronously, converts positive-up `top` to negative
  `image_bearing_y`, and rejects repeated answers, non-finite metrics, PNGs
  over 4 MiB, and encoded/reported dimensions outside 1..1024 or inconsistent
  with each other. Context/threading rules match the outline callback.
- **text-input-color-glyphs**: RawTextInput retains native/host color glyphs
  outside both outline paths. Authored text and selected-text drawables and the
  direct editor/scripting draw route paint them after outlines. Each route
  matches its outlines' clipping: direct RawTextInput clips both; imported and
  selected-text drawables add no field overflow clip. Selection keeps their
  colors. Foreground follows upstream Text's first solid Fill (black without
  one); color glyph command opacity is always 1.0, like upstream Text. Direct
  RawTextInput paint handles are write-only and use the same black default. Metrics, cursor, hit testing and
  ordinary outline transforms remain upstream behavior.
- **raw-text-host-pictures**: only host picture layers in RawText use Text's
  decoded-image cache/path. Native RawText layer traversal remains unchanged,
  including upstream's lack of an image branch in drawColorGlyphLayer.

The sbix/CBDT extent correction is parity, not an additive host policy:
`font_hb.rs` follows pinned HarfBuzz rive_13.1.1 strike selection, bitmap-origin
semantics and integer rounding into Rive's 2048-unit scale. Oracle expectations
come from that HarfBuzz source; the existing C++ probe has no bitmap-extents
query. Runtime/device rasterization and host color-space choices remain SDK
qualification work.


## Audio renderer follow-up (2026-10-25)

The user authorized importing the previously deferred generic `AudioRenderer`
and configurable mixing blocks from `4252d730`, using current source authority
`0f4fa96fc5e9fe4bc41a398bff8275e59cfb86e2`. This is a follow-up to the
accounted checkpoint, not a new upstream commit or a rewrite of the original
scope decision below. Guest Wasm processors and execution remain excluded.

The implementation uses the existing pure-Rust audio backend and CPAL device
boundary; no new library is required. Translation, the independent source review
and correction rereviews, and the separate Rust-integration review and rereview
pass. Source corrections preserve fade scheduling/progression, actual short-read
counts and untouched sum tails. Integration corrections release singleton locks
before renderer destruction, with reentrant-destructor regressions.

Pool validation passes on the reviewed source:
- Linux audio backend: 34 tests (`74cec947-12e9-417f-835e-170edc0fef3a`).
- macOS audio backend with `audio-device`: 36 tests
  (`47a16bc0-9300-4d13-889b-ca3422b70d1e`); the existing physical-device smoke
  remains ignored, so no physical output/hardware latency is claimed.
- Runtime audio host, release/reentry, and AudioEvent: 7 tests across 3 suites
  (`de743611-ffcd-4096-bb27-394d1ec887bc`).
- Public Nuxie audio consumer: 9 tests
  (`2da738c2-01bb-4c17-b916-24f659cf42ed`); the existing expected-red decoder
  resampling case remains ignored, with no assertion/tolerance changes.
Patch hygiene and Bazel registration checks also pass. An initial consumer
invocation used a nonexistent `tools` feature and was rejected before build;
the correct default-feature invocation above passes.

Rust owns each renderer callback until explicit detachment or engine destruction;
attachment handles do not retain the engine. Detachment waits for an executing
callback. The runtime wrapper exposes the same renderer and block-size controls.
Headless reads retain whole-block look-ahead (128 frames by default), use the
engine frame clock, and preserve newest-attached-first mixing and callback
attachment/stop behavior. Existing tiny ownership/control tests select explicit
one-frame blocks rather than retaining the superseded arbitrary-pull behavior.

The approved CPAL boundary remains matching-format only: it does not add an
engine/device resampler. Requested device blocks are clamped to CPAL's reported
range; the first nonempty callback supplies the observed block size, retained
for reopening that sink. CPAL does not expose miniaudio's distinct internal
sample-rate/period fields, so internal-rate conversion and its construction-time
reinitialization are not claimed ported. No guest execution, Luau audio-thread
callbacks, framework release, or consumer migration is included.

## Current checkpoint

- LAST_SYNCED_SHA: `ed3303569554da2a13ef468c33bf405dd0f49dbc`
  This is the accounted checkpoint excluding explicitly deferred Wasm execution
  ([UNIV-3728](https://universe.basis.dev/issue/UNIV-3728)), not full-feature parity.
- Current target: `a203f5355840bbc22c4544fef9edc411c85f3c03` (refreshed 2026-10-27).
  The user authorized continuing one commit at a time through latest upstream;
  refresh the tip at catch-up and continue if it advances.
  7 upstream commits remain after this checkpoint at the fetched target.
  All 332 commits after the continuous goal's starting checkpoint `503eab63`
  through this checkpoint are accounted for (354 after `5892bb05`); this
  excludes deferred execution and is not a claim that every validation harness
  is green. Refresh upstream after landing this checkpoint and continue if it
  advances. The broad validation results below remain separate from this count.
- `ed330356`: Correct deterministic scroll velocity to store microseconds,
  preserving fractional-second intervals. The existing engine clock is already
  monotonic; preserve the approved per-drag explicit pointer-clock adaptation.
  Translate controlled timestamps and the fractional-seconds regression without
  changing Taffy/CSS layout semantics. The Silver recipe and its generator use
  the same cumulative f32 timestamps. Both independent reviews, including that
  supplement, pass. Pool `1735a9fb-e9a3-4bb2-9631-090c069f7211` passes all
  38 scroll tests (28 input, 5 velocity, 5 elastic). The changed file/Silver
  regression fails at frame 0/op 11, color 4294901760 versus 4294902015, in
  candidate job `e239ad6d-7b5d-4c54-9082-5d8c00480eb4` and clean parent
  `91005ff490` job `28233fe7-262c-470d-b6ea-65896a3a6c99`. This preexisting
  failure is not waived into a pass; expectations remain unchanged. No new
  device or broad Silver coverage is claimed.
- `cf9af36b`: Only `.rive_head` metadata advances from `a972783b` to
  `389f4a56`. No public implementation delta or absent private changes are
  claimed ported. Active source/oracle/Silver pins advance; historical evidence
  and actual shader generation provenance remain unchanged. Both independent
  disposition reviews, git diff --check and Bazel registration checks pass.
  No executable or device validation is claimed for this metadata-only delta.
- `c7204ede`: Only root-only to-target paths designate “My ViewModel” in
  listener assignment. Values sourced from another property remain in the
  bindable, avoiding accidental self-parent assignment. Add source-path length
  and use it for both listener actions and transition comparands. Translate
  both public listener-assignment regressions. Private Dart/editor mirror
  changes absent from the public delta are not claimed. Active pins advance;
  historical shader generation provenance remains unchanged. The touched Rust
  comparand lacked upstream’s pre-existing root/reference/stored/fallback
  resolution; restore it and preserve null as a valid identity for pointer
  equality, rather than treating it as a failed comparison. The native editor’s
  separately hydrated path storage has no downstream counterpart.
  Both independent reviews pass after correcting nullable identity. Pool
  `8eae7702-7890-49a9-ac01-8cb97bd2d07d` passes 19 listener/state-machine/VM
  integration tests; `ca50391f-ccd7-4fd5-b044-7780738cbcea` passes the nullable
  identity regression. Patch hygiene and Bazel registration pass. No new device
  coverage; comparand resolution branches have source-review rather than direct
  public upstream-fixture coverage.
- `4839c35a`: Bound each owner’s undelivered fetch answers and bytes, reserve
  answer capacity for in-flight requests, discard canceled outcomes, and refund
  rate starts when a request is canceled before the provider sees it. Recheck
  cancellation before each queued dispatch and drain ready batches as synchronous
  answers free slots. Preserve the poll sequence boundary and callback ordering.
  Translate all four public fetch regressions. Active source/oracle/Silver pins
  advance; historical shader generation provenance is unchanged.
  Both independent reviews pass. Explicit compiler+scriptnet pool graph passes
  all 31 fetch tests (`992ba148-ed78-44ed-8694-a1c3c7bb7f46`); the scriptnet
  policy graph passes all 12 tests (`da394969-6550-4b99-be91-8bedd5c9e13d`).
  Earlier default graphs executed zero gated tests and are not counted. Patch
  hygiene and Bazel registration pass; no new live-network/device coverage.
- `f3721644`: Exclude the sole public file, WAMR patch 0039 for wide AOT
  call argument/result staging and tail-call restrictions. It changes the guest
  compiler, not ordinary Luau execution or browser renderer Wasm compilation.
  Guest execution remains excluded under UNIV-3728. Active source/oracle/Silver
  pins advance; historical evidence and actual shader provenance are retained.
  Both independent disposition reviews, patch hygiene and Bazel registration
  checks pass. No executable or hardware validation is claimed for this exclusion.
- `b27b97fd`: Resolve animated TextStyle font IDs in the file the style was
  imported from, retain that origin through TextStyle/TextStylePaint cloning,
  and dirty global-style owners when their asset changes. Out-of-range or
  non-font IDs preserve the current asset, as upstream. Port the public keyed-font
  regression. Private editor/export changes described only in the commit message
  are absent publicly and are not claimed ported. CSS text semantics remain ours.
  The WITH_RIVE_EDITOR co-op resolver branch has no corresponding downstream
  editor resolver; imported-file font IDs use the public runtime path. A weak
  file handle preserves origin identity without a file/style ownership cycle.
  Source/oracle and Silver pins advance; shader artifact provenance is unchanged.
  Both independent reviews pass. Pool `9d1f0780-dcf5-41bd-91a5-16db0bd87185`
  passes all 17 tests across keyed-font, font-binding, global-style and shared-style
  suites. Patch hygiene and Bazel registration checks pass. No new device/pixel
  coverage is claimed.
- `ec420822`: Only `.rive_head` changes from `cae4dad5` to `a972783b`.
  No public executable delta or absent private code is claimed ported. Active
  source/oracle/Silver pins advance; shader artifacts and prior evidence retain
  actual provenance. Both independent disposition reviews, patch hygiene and
  Bazel registration checks pass. No new executable/hardware validation claimed.
- `62949d42`: Transfer the per-frame allocator through the backend recycling
  hook after logical flushes. The default resets and returns it; an override may
  retain the old allocation and return another empty allocator. Preserve the
  non-null/empty contract and expose the upstream initial block-size constant.
  Helper-backed browser renderers forward the same hook. Source/oracle and Silver
  pins advance; generated shader products retain their actual source revision.
  Source-equivalence and Rust-integration reviews pass. Linux Vulkan product
  compilation passes (pool `aa59ae71-c8ea-48c3-97bc-039da131fa9b`), as do WebGPU
  and WebGL2 wasm32-unknown-unknown builds (`049fa75c-44b2-4328-8cdb-7f8c51b944b8`).
  These are compile checks, not Android-device or browser-pixel qualification.
  Mac pool `4c22a84a-0299-4096-8a12-87524eeabe60` passes all 8 selected shared
  allocator and Metal lifecycle tests with live Metal required. The pre-existing
  cfg-disabled abandoned-frame test did not execute and is not counted. Patch
  hygiene and Bazel registration checks pass.
- `ce78b66f`: Only private `.rive_head` metadata changes from `412408d8`
  to `cae4dad5`. No public executable delta or private implementation is claimed
  ported. Source/oracle and Silver references advance; real shader artifact and
  historical validation provenance remain unchanged. Both independent disposition
  reviews, patch hygiene and Bazel registration checks pass; no new runtime or
  hardware coverage is claimed.
- `2642c18a`: Translate Solo provider-reach checks for non-CSS owning layouts,
  skipping needless recollection only when no option reaches a provider. Walk
  all options, including inactive ones, without caching. Preserve unconditional
  CSS recollection: anonymous measured CSS content need not be a layout provider.
  Collapse and notification ordering remain intact. Source/oracle and Silver
  references advance; shader artifacts retain their actual generation revision.
  CSS validation exposed a pre-existing mutable artboard reborrow while Solo
  resolves its active option. Adoption, reconciliation and queued style updates
  now measure under a shared borrow before mutating layout state; CSS content
  placement likewise releases mutation before traversing Solo children. Both independent
  reviews accepted the corrected boundaries; regression coverage includes initial
  CSS solve, post-switch settling and queued invalidation.
  Final pool job `de41d822-ed3c-4738-979a-7e4a2ca08a44` passes all 134 tests
  (26 layout-participant, 1 proxy-visibility, 107 live-move). Patch hygiene and
  Bazel registration checks pass. No new device or renderer-pixel coverage.
- `eeaf1407`: Only `.rive_head` changes from `d4406516` to `412408d8`.
  Private monorepo metadata exposes no executable delta; no private implementation
  is claimed ported. Active source/oracle and Silver pins advance; generated
  shader artifacts and historical evidence retain their actual identity.
  Both disposition reviews, patch hygiene and generated Bazel registration pass;
  no new executable or hardware validation is claimed.
- `d96b8979`: Guest WAMR interpreter/AOT/memory/signal patches, tier-cache revision,
  guest job/worker clock imports and generated web copies are excluded. The shared
  Region registry cache avoids per-atomic slot pins; our Luau native handles
  already access retained Arc regions directly, and browser descriptors use our
  separate SAB store, so there is no corresponding registry lookup/cache to port.
  Cache retirement hooks therefore do not apply. The upstream 20-microsecond
  timed-mutex spin is a backend-specific optimization, not imported into our
  Rust atomic/yield lock boundary; timeout/poison/retirement behavior remains.
  This is explicit scope accounting, not guest-engine parity. Source/oracle and
  Silver references advance; generated shader artifacts retain actual provenance.
  Both independent disposition reviews, patch hygiene and generated Bazel
  registration checks pass. No new executable or hardware coverage is claimed.
- `0f4fa96f`: public delta changes only `.rive_head` from `e6c7fdb0`
  to `d4406516`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `bda5b785`: The complete four-file public delta sets the guest runner-owned browser AudioContext audio-session type from auto to playback, with generated copy/docs/tests. Host-provided contexts remain untouched. No corresponding downstream guest runner/AudioContext exists; no native iOS audio implementation changes. Excluded guest-host infrastructure under UNIV-3728; no independent renderer or ordinary runtime change.
  Both disposition reviews pass; patch and Bazel registration checks pass. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No new build/test/device coverage is claimed.
- `eb7741e4`: public delta changes only `.rive_head` from `bd41e84a`
  to `e6c7fdb0`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `183c426d`: The complete nine-file public delta carries the guest audio processor worklet in threaded Emscripten builds, adds a blob-URL fallback, and changes its generator/docs/browser tests. This is excluded guest-host infrastructure under UNIV-3728, not browser renderer Wasm compilation. No independent renderer or Luau runtime change.
  Both disposition reviews pass; patch and Bazel registration checks pass. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No new build/test/device coverage is claimed.
- `8b3ca41b`: public delta changes only `.rive_head` from `de38f525`
  to `bd41e84a`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `1be2c7fa`: Translate the complete Metal texture-storage delta: uploadable textures on macOS devices outside the Apple GPU family use Managed storage, Apple-family devices and non-macOS targets use Shared, and render-target-only textures remain Private. Preserve upstream platform availability and allocation/error behavior. No shader or format change.
  Both independent reviews pass. Pool validation passes:143 existing Metal
  ORE tests (`e4843852-05e7-4494-8a6c-3716ff6a44bd`) and the live texture-upload
  rendering regression (`1f40f8a3-c476-415e-8609-beaed6ec75c8`). Available Apple hardware exercises Shared
  upload storage and Private targets; Intel/discrete Managed hardware remains
  untested. No iOS hardware or cross-platform GPU qualification is claimed. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No shader regeneration or framework release is needed.
- `90195332`: public delta changes only `.rive_head` from `8cd0a485`
  to `de38f525`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `6c65f385`: Translate NestedArtboard::update so dirty referenced artboards flush in the same advance regardless of pause state, preserving frozen animations. Port the complete public playing-artboard opacity regression. Account for the three updated upstream Silver streams without weakening CSS adaptation checks. No guest execution or layout-policy change.
  Both independent reviews pass. All four opacity tests pass on the pool
  (`f442a73e-ff89-4735-9039-fcd7f3716be5`). Focus compares byte-exact
  (`4233593a-fcbf-467f-9604-1d2a40f0dc06`); echo compares within unchanged
  epsilon (`b1294a1a-80ce-4dfa-9e77-7fa671570ea7`). Initial focus/echo CLI runs
  incorrectly requested an existing capture directory; corrected runs pass.
  Bankcard retains its approved first CSS difference but fails its unchanged
  stream seal `86f2d351`. A controlled old-condition pool capture produces
  `a0408c25` (`e6debe8a-a9f6-4fa8-9333-7500b387f0ba`), while the reviewed source
  produces `b131c276` (`2dff442a-da27-407b-aa46-66df42e425be`); both are184353
  bytes. Thus seal drift also exists before this gate change; UNIV-5576 tracks
  the unresolved historical cause. No seal or CSS expectation is weakened.
  Exact non-color operations, resolved draw colors, final paint states and
  color-update multisets match for old/new C++ streams
  (`3c37a92a-4d86-4a64-9f34-1ac2012e5603`) and Rust streams (`f5383484-318c-48b4-b41e-3bc2d07c0478`).
  Both comparisons cover6385 non-color operations,1163 draws,129 paints and133
  color updates. The temporary pool diagnostic is removed; no new harness or
  ledger is retained. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No live GPU or browser coverage is claimed.
- `c5a4e468`: public delta changes only `.rive_head` from `c2f51857`
  to `8cd0a485`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `57acb425`: The complete public delta contains only 68 guest .riv fixtures, a new AssemblyScript barrier fixture, and test-shell additions (70 files). Guest worker/barrier fixtures and their execution harness are excluded under UNIV-3728. The barrier implementation described by the title is absent from the public delta and is not claimed imported. No independent renderer, Luau or ordinary runtime executable change.
  Both disposition reviews pass; patch and Bazel registration checks pass. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No new build/test/device coverage is claimed.
- `48b8c0ba`: public delta changes only `.rive_head` from `59d7b7da`
  to `c2f51857`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `9eb2a6d9`: translate inherited lite-RTTI membership and intermediate-class
  downcasts, adapting the existing source-mapped roots and consumers. Translate
  the complete public hierarchy, null/const, construction/destruction and retained
  ownership regressions against the mapped helper. No unrelated Any-cast redesign.
  Independent source and separate integration reviews pass, including correction
  of the public D/E destructor test to destroy through the base-pointer callback.
  Pool validation passes:263 renderer tests with5 existing ignores
  (`05d1c339-a396-4b60-9b6d-47a53bc1f9db`),2 retained runtime ownership tests
  (`712f3d9c-f5c8-45d1-8a5f-b0028d3c0e43`), WebGPU/Wagyu library
  (`9151f0a7-371f-4079-9f7d-a608251d5257`), Vulkan library
  (`019c680f-55b0-41dd-ae01-c445d53d7184`) and native WebGL2 library
  (`ee7d1e4a-9576-45d9-ba45-ef2bd8cee7bd`). Six Metal/helper/image regression cases pass
  (`bf0bbbb0-a1df-482a-ab12-321fbf557c5e`).
  The wasm32 WebGL2 build stopped in external rules_rust CopyFile actions under
  the pool remote-only policy (`d739e7cb-6a13-45dc-8068-39c4ecb4a186`), before
  renderer compilation; tracked in UNIV-5575. No Wasm/browser/GPU coverage is
  implied by native compilation. Active source/oracle and Silver pins advance;
  production shaders retain `86594` and the GM header retains `e3b9461b`.
- `3ec7ef07`: public delta changes only `.rive_head` from `da345444`
  to `59d7b7da`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `4252d730`: translate WebGPU buffer allocation limits, four-byte backing/upload
  alignment and mutable shadow handling; port immutable triangle and unaligned
  index-update GM scenarios and oversized-buffer rejection checks. Translate
  ordinary audio nearest-millisecond scheduling, exact intra-pull start offsets,
  pre-start stop/fade behavior and applicable public timing/fade regressions.
  Our synchronous output-level monitor has no partially initialized attachable
  node; file/allocation counters already use atomics. Generic AudioRenderer,
  configurable block/device-rate APIs and their dependent callback tests remain
  deferred pending a separate host-audio API decision; not all ten public audio
  cases are claimed ported. Guest Wasm workers/processors, job-pool/threading and
  browser/build dependencies are excluded under UNIV-3728. Yoga-specific setup
  and counters are excluded under the CSS contract. Native Rust audio retains
  its existing backend rather than importing miniaudio C/device integration.
  Independent source and separate Rust-integration reviews pass. Pool validation:
  22 audio tests (`5f6c4165-f3da-48d0-bba1-cbeb105a2d01`), three runtime audio-host
  and lifetime cases (`087ba52b-0d61-4fc9-a982-2c830b7ebd1a`), and both unchanged
  Metal pixel comparisons (`1889b5f0-7b37-4613-988f-ebda1334d27f`) pass. WebGPU
  production-feature build passes (`900d3840-cad8-41a8-bd65-91af1f222710`). The first WebGPU invocation omitted
  renderer-webgpu and failed on gated imports; no source fix was needed. Native
  WebGPU oversized-buffer tests are added but not executed here (Dawn test-link
  prerequisite); no live WebGPU/browser or audio-device coverage is claimed.
  Active source/oracle and Silver
  pins advance; production shaders retain86594 and the exact GM header retains
  e3b9461b generation provenance. No editor/compiler change or release.
- `6e8050f3`: public delta changes only `.rive_head` from `e2e9eb25`
  to `da345444`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `9cb784fc`: The complete public delta adds dart pub get before compiling upstream's
  C++ core generator on a clean checkout. This development script has no Rust
  runtime owner or downstream invocation; no C++ generated-header or CI workflow
  changes are public in this commit despite its title. Account as inapplicable,
  with no private generator output or CI implementation claimed imported.
  Both disposition reviews pass; patch and Bazel registration checks pass. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No new build/test/device coverage is claimed.
- `f4da8fa4`: public delta changes only `.rive_head` from `75da10d6`
  to `e2e9eb25`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `e48d8910`: D3D11 binds retained groups through the active pipeline's registers.
  Its four implementation files and changed D3D11 test branch are inapplicable
  because this fork has no D3D backend. The shared ore_types delta only updates
  explanatory backend comments, mirrored in Rust; no common executable change.
  Both disposition reviews pass; patch and Bazel registration checks pass. Source/oracle and Silver references
  advance; production shader provenance remains86594 and GM fixture e3b.
  No new build/test/device coverage is claimed.
- `3df35b94`: public delta changes only `.rive_head` from `f018ea15`
  to `75da10d6`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `e3b9461b`: resolve reused bind-group layouts against active pipeline slots.
  Translate shared slot-space validation/tables and deferred group application,
  Metal and GL bindings, Vulkan bound-layout validation and WebGPU ignored-layout
  mode. Translate the public layout-reuse Metal scenario and updated unit calls;
  advance the exact GM shader header fixture and offsets. Unsupported D3D backend
  implementations and their test branch are inapplicable, not claimed ported.
  Both independent reviews and correction rereviews pass. Pool validation:
  125 shared ORE tests (`b916f8f7-b129-4e83-b35e-ba13815ded95`),
  603 renderer tests with six existing ignores (`5c0f3ac8-6acd-40a6-8ce3-3a07a7043433`),
  the live Metal layout-reuse pixel case (`b73e9847-7aac-4706-8b6d-90509680f5c8`),
  Vulkan build (`f1aa03d4-4e82-4fed-938e-70e700de5e44`), and
  WebGPU/Wagyu build (`836d4d79-3f66-4792-87a6-9e3f4bd1b874`). Review corrected
  stale GL record-size assertions and a private Metal enum import; validation
  corrected WebGPU snapshot byte/line assertions to exact measured upstream
  dimensions. The live Metal regression exposed a missing inherited Pipeline
  projection on PipelineMetal; expose its existing retained base through the
  ordinary recording-thread checked handle. Both correction reviews pass.
  No assertions/tolerances weakened. No live Vulkan/WebGPU/browser
  device coverage claimed. Source/oracle and Silver references advance; production shader products retain actual `86594b71`
  generation provenance. No editor/compiler format change or release is needed.
- `708d1daf`: public delta changes only `.rive_head` from `918bbc62`
  to `f018ea15`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `bcb9c95d`: queue script fetches beyond per-owner transport/rate ceilings,
  start them in order during polling, and bound queued count/body bytes. Preserve
  owner-thread delivery, provider calls outside locks, cancellation, timeouts and
  poll outcome ordering. Default per-minute cap becomes zero (unlimited); default
  queue limits are 1024 requests and 32 MiB of bodies. Translate every public
  scripting-fetch test change through the existing Luau adapter. This shared
  networking change is in scope; no guest Wasm engine is imported. Both independent
  reviews and correction rereviews pass. Pool validation passed 27 actual
  Luau fetch/queue/cancellation cases (`80888e8c-9340-40f3-a407-73c7eea6fc94`),
  48 promise cases (`4c5eb869-7d11-455a-8df0-f141e88dff1e`), and
  12 network-policy cases (`f4ca910e-e487-480a-8690-b25b3255f67a`).
  Initial compilation exposed five stale test-fixture view-model calls; pass the
  same non-null handles as Some(handle), matching the existing Option API.
  The new minute-limit case then exposed an existing promise translation error:
  Lua and/or incorrectly selected failure when fulfilled success-handler was nil.
  Restore the source's explicit conditional so absent success remains absent and
  fulfillment forwards normally. All assertions remain unchanged; production
  view-model and queue behavior required no validation-driven adjustment. Source/oracle and Silver
  references advance; shader products retain verified `86594b71` provenance.
  No new device or live-network coverage is claimed.
- `1513dd72`: public delta changes only `.rive_head` from `523433dd`
  to `918bbc62`. This private-monorepo reference exposes no implementation;
  no runtime behavior is claimed imported. Both disposition reviews pass; patch and Bazel registration checks pass.
  Source/oracle and Silver references advance. Shader products retain actual
  `86594b71` generation provenance. No executable change or new build/device
  validation is claimed; patch and registration checks apply.
- `c02b45aa`: port optional Thai, Lao, Khmer and Burmese dictionary line
  breaking, source trie segmentation, lazy compressed rebuilding and all public
  dictionary tests. Each language and compression remain individually default-off,
  matching upstream; the shared text owner lives in render-api with runtime
  forwarding. Literal dictionary tables preserve ICU licensing, every numeric
  element, size and checksum. Upstream's Python generator emits C++ tables;
  its generated data is mechanically translated, not regenerated or claimed as
  a new Rust generation tool. Premake options map to Cargo features; upstream
  editor defaults remain off. The generator cache ignore entry has no downstream
  generated cache counterpart.
  Exclude unconditional Hangul-to-AL tailoring, removal of LB26/LB27, space-only
  Korean test expectations and Hangul conformance skips under our CSS-normal
  text contract. Existing full Unicode conformance coverage remains intact.
  Both independent reviews pass. Pool validation passed 11 compressed dictionary
  cases plus 39 shared API tests (`f6f7531f-55ce-453b-a617-04d8687ef7d4`),
  9 uncompressed dictionary cases plus 39 API tests (`68a39773-277b-4388-9cfb-6f5e5c94baf1`),
  and 26 existing default-mode shaping/rule/full Unicode conformance cases
  (`0db30829-22f8-4ae2-b947-3398d6e6fadd`). The runtime with all dictionary features
  compiles (`c3c204a0-d3da-49a7-9226-22671f5c4ac7`). All assertions are preserved;
  no translation corrections were required by these checks. Shader products
  retain verified `86594b71` provenance. No new device coverage claimed.
- `41b17c32`: define USE_WEBGPU_SAMPLERS alongside TARGET_SPIRV in the
  non-GL Wagyu GLSL compilation branch. Preserve the GL/GLSLRAW branch. The
  offline WGSL generator already supplies this define, so unchanged generated
  products retain their actual `86594b71` provenance. Integration review found
  dynamic GLSL inputs were standalone readable-name products despite minified
  defines. Correct all 18 live inputs to the existing embedded header payloads,
  verified byte-for-byte against pool products (47,318 bytes), so defines select
  their intended branches. Both correction reviews pass. The WebGPU/Wagyu library
  build passes on the pool (`6f5c5c81-9b5d-49da-82e3-2ad4b1028ce5`). Native test
  execution is unavailable: linking failed on absent Dawn symbols, the previously
  documented limitation (`762e171f-f395-48ed-9d1e-43ad3776ae18`); no test pass is
  claimed. Export and sampler conditional regressions are retained for equipped
  hosts. No public upstream tests were added;
  private `.rive_head` remains unclaimed. No new device coverage is claimed.
- `bece7efb`: the runtime delta extracts the existing layout-proxy insertion
  loop into a helper without changing its behavior. Our hierarchy-stack insertion,
  needs-proxy filtering and trailing flush before dependency sorting are already
  equivalent. New editor entry point, clip initial values, journal capture and
  proxy-rebuild query belong to upstream native-editor lifecycle owners absent
  here; they are inapplicable, not rejected runtime rendering fixes. Preserve
  our runtime/CSS defaults. No public tests were added; private `.rive_head`
  and unpublished REV/button integration are unclaimed. Both disposition reviews
  are complete. Patch hygiene and Bazel registration checks pass. No executable Rust changes, with
  no new build or device coverage claimed. Shader products retain `86594b71`.
- `784e3870`: set GL texture ownership true by default so ORE-created textures
  and renderbuffers are deleted on release. External wrapping paths retain their
  explicit false ownership. Replace the downstream assertion preserving the old
  upstream defect with the new invariant. Both independent reviews and the context-scope correction rereviews are complete.
  Pool validation passed 603 renderer tests (6 ignored;
  `8dc1ade1-895f-4eee-8b67-4102eaea9cf8`), including a supplemental actual-command
  regression for owned deletion, non-owning wrapper release, and exactly-once
  external-owner deletion. Upstream added no public tests. Private `.rive_head` is not implementation
  evidence. Source/oracle and Silver references advance; shader inputs/products
  retain verified `86594b71` provenance. No new browser-device coverage is claimed.
- `f5bd36c1`: determine destination-blend barriers from retained base draw-group
  contents before batches are built; preserve separate clockwise-atomic clip
  barrier state, subpass ordering and destination-read lists. Cull empty paths
  before sorting and treat an empty clip as fully clipped. Port all three public
  empty-draw tests, including the nonempty destination-reader assertion.
  Both independent reviews and correction rereviews are complete. Pool renderer
  tests passed 602 cases (6 ignored; `08f3f91e-9801-480c-a6f4-29d96c664789`), explicitly
  including all three new cases. The native Metal context-destruction test passed
  (`7b23e90a-cd7e-49a7-86a7-941d1925a917`). Source review removed a duplicated constant;
  compilation exposed a test-only flag-wrapper API mismatch, corrected to the
  exact bit check with both rereviews. No assertions were relaxed. The existing
  destination-barrier helper now maintains its upstream tail pointer, required
  by the newly translated invariant.
  Private `.rive_head` is not implementation evidence. Active source/oracle and
  Silver pins advance; shader inputs/products remain at verified `86594b71`
  provenance. No new device-pixel coverage is claimed.
- `c51c078e`: translate gradient transform invalidation for all five transform
  properties through generated setters and LinearGradient hooks, including radial
  and angular inheritance. Port the public angular-gradient shader-transform test
  with its translation, rotation and scale sections. Private editor degree-display
  and converter-table changes described in the message are absent from the public
  delta and are not claimed imported. Both independent reviews are complete.
  Pool validation passed 893 runtime/gradient/paint/binding cases (2 ignored;
  `1771b512-491a-4ea7-a526-3400159d0b2a`), including all three new public sections.
  Callback metadata regenerated from the exact upstream headers on the pool
  (`5a322e90-2aed-4464-b04e-57f31ff3feac`) matches the reviewed schema file after rustfmt.
  Active source/oracle and Silver pins advance; unchanged
  shader inputs/products retain verified `86594b71` provenance. No new device
  pixels are claimed.
- `86594b71`: translate four-wide cubic chopping, tangents, Wang estimates,
  hardware inverse-square-root policy, and live fill/stroke/feather preparation.
  Context-owned buffers retain original curves, candidates, extra pieces,
  rotations, cusp bridges and counts across paths. Preserve existing Rust crate
  layering; the renderer gains no production runtime dependency. Rename the
  shared cusp-join flag in CPU and shader owners.
  Both independent reviews and correction rereviews are complete. Review caught
  initially unretained intermediate buffers, an authored join using a chopped
  tangent, missing final-lane padding, and overbroad Wasm floor/ceil cfg; all were
  corrected at the source owners. Port all three new public Bézier tests and the
  expanded Wang case. GM deltas only rename helpers; no authored Rust generators
  exist for those cases, so retained streams and captures are unchanged. The
  existing missing `measure_cubic_local_curvature_glsl` test is not claimed
  ported (its public delta is a helper rename and capacity assertion).
  Pool validation passed 944 runtime/math cases (2 ignored;
  `e183b4c6-5b12-44f7-bc0a-829ab2ab4f9d`), 599 renderer cases (6 ignored;
  `57335a7f-0a9e-4836-bcbe-3efa8cc73e46`), Metal compilation
  (`733815f5-3e50-479b-86b1-9df7c7766b5f`), and Vulkan compilation
  (`1ffc7674-04bb-4694-b997-5457b03b213c`). The three initial renderer failures were old
  exact-square-root count expectations. Independent pinned-C++ evaluation at O0
  and O2 confirmed scale/translation counts 5 on Linux x86_64 and macOS arm64;
  the former 34 boundary is now 33 on SSE and 34 on NEON. Exact literals and
  historical scalar-bit assertions are retained, without tolerances. C++ pool
  jobs: `0fddf053-bc8b-4e6e-a287-8b8b226488af` and
  `0c923754-9555-4153-aab8-d25e9730acbf`. Other architectures are source-derived,
  not newly hardware-validated. The temporary evaluator was removed after review.
  Shader generation succeeded on the pool (`934848e2-974c-45f6-9e8d-31c10b016f17`):
  all 66 WGSL, 96 selected SPIR-V, 24 WebGL2 and 54 WebGPU products were verified
  and imported from this revision. They are byte-identical to the prior products;
  the shader delta is a flag rename/formatting. Input manifest SHA-256
  `6095cc4c29f40b5f4f8c496b9cf0a5da6587c252bf4b441300ebf8f0f0abc213`;
  generator `59db72734d9b9f4bec8daf352040b3b3d8a6f22dbf2b4b144cb4141323c4ca66`. Historical
  evidence keeps its original provenance. Private `.rive_head` is not
  implementation evidence. No new device pixels or speedup is claimed.
- `b3360ab9`: translate the complete public artboard storage delta: cold
  instance lists use LazyVector; source-owned animations, state machines,
  rejected objects and the lazy keyframe-bind index share ArtboardSourceState;
  the source dependency-order recipe uses its own Sidecar. Preserve allocation-free
  empty reads, source sharing, ordered destruction and lazy index construction.
  Translate LazyVector predicate removal. This is memory-storage behavior, not
  CSS layout semantics; the CSS contract remains unchanged. Upstream added no
  tests. C++ `WITH_RIVE_EDITOR` clearAnimations, clearStateMachines and
  editorPrepareForDestruction rewrites have no shipped Rust/native-editor
  counterparts and are not imported. Private `.rive_head` is not implementation
  evidence.
  Both independent reviews and correction rereviews are complete. Pool passed
  873 runtime tests (2 ignored) and 71 regression cases across 14 targets
  (`34ef7da3-895b-45ad-94ce-b544b24843e1`), plus shipping C API compilation
  (`a67afa34-04c5-43c4-945f-170d5961a58c`). Integration review caught a shared-storage
  callback borrow conflict and an eager instance keyframe query. Stable boxed
  source storage with separately mutable index state preserves callback reads;
  pose checking now queries only the selected source after its serial guard.
  Two focused callback/nonallocation regressions and extended LazyVector tests
  retain all prior assertions. The source cluster uses Rust OnceCell ownership;
  the dependency recipe retains Sidecar ownership. No read clones were added.
  Source/oracle and Silver references advance; shader products retain actual
  `41a2120b` provenance. No new hardware or memory-size measurement is claimed.
- `1a15344b`: translate the complete public Vulkan warmup predicate change:
  depth/stencil permutations that seed MSAA from an offscreen texture require
  both multisampling and preserve-render-target color loading. Clear/discard
  passes must not seed previous color. Private `.rive_head` is not implementation
  evidence. Both independent reviews are complete. Pool Vulkan compilation
  passed (`b480d885-4518-46bc-9735-c529a87e5ccc`); generated build metadata and
  patch hygiene checks pass. Upstream added no tests for this predicate change.
  Source/oracle and Silver references advance; shader inputs/products retain
  actual `41a2120b` provenance. No Android hardware coverage is claimed.
- `e579560a`: account for the complete 22-file public delta. Translate the
  conservative single-entry SPIR-V splitter, reusable scratch and retained
  multi-entry shader words, per-stage temporary Vulkan modules and their cleanup.
  Translate shared GPU frame advancement and unlabeled replay pipeline diagnostic
  names. Port all four public splitter tests, including every baked GM entry.
  D3D12 descriptor/frame lifetime changes and its eight backend-only test cases
  have no shipped backend counterpart; private `.rive_head` is not implementation
  evidence. C++ premake registration is represented by Rust/Bazel registration.
  Both independent reviews and correction rereviews are complete.
  Pool validation passed 260 portable renderer tests (5 existing ignored), including
  all four new SPIR-V tests (`da6052ec-7e0a-42b1-9b8e-f821b6c8be77`), all 94
  portable ORE tests (`1435b486-3551-457f-9793-ca836b25ccb1`), and Vulkan compilation
  (`d4cfe433-1476-4865-ad03-8f14c0678d0b`).
  Initial validation exposed a stale baseline sampler test: anisotropy requires
  linear filters. Its corrected fixture preserves success/error-order assertions
  and adds exact nearest-filter rejection coverage; production behavior is unchanged.
  The initial default renderer run omitted the new tests because their parent
  module was feature-gated. Registration now lives at the unconditional root;
  the final run explicitly executed all four. The malformed-module fixture also
  required matching upstream’s exact 20-byte copy into a zeroed 24-byte buffer;
  its rejection assertion is unchanged. No test assertions were weakened.
  Shader products retain actual `41a2120b` provenance; source/oracle and Silver
  references advance. No new device pixel or Adreno hardware coverage is claimed.
- `0238f299`: account for the complete 54-file public delta. Translate global
  text-style references, per-text instances and on-demand list-item styles,
  shareability analysis/cache lifetimes, host data-context reads, geometry/paint
  ownership, style metrics and variation propagation, and targetless pointer
  listeners serving styled runs in host coordinates. Wire generated types,
  advancing/artboard-referencer dispatch, File ownership and artboard binding;
  retain the missing-view-model clone behavior. Port all 26 public test cases
  across the five global-style regression files, including listener teardown
  sections. All five binary fixtures and authored RML remain in the exact
  upstream fixture tree. Private `.rive_head` reveals no implementation to port.
  Translation, both independent reviews and correction rereviews are complete.
  Pool validation passed all 26 new global-style cases
  (`f8d46577-da1f-443d-bde1-fdc3bc68cb63`), 871 runtime unit tests (2 ignored)
  and 58 existing regression cases across 15 targets
  (`ee4ce26d-f262-46e8-9e9e-bdf2eb63242d`), plus shipping C API compilation
  (`e125b0a4-117b-44f8-9923-c35f78d6ed1a`). Validation exposed Rust callback
  borrow boundaries in property notifications and nested/list context teardown;
  corrections preserve notification stages, source identity and destruction order.
  Synthetic tests now construct the derived paint type and traverse polymorphic
  container capabilities, retaining all assertions and tolerances. Schema and
  callback generation ran on the pool. Silver advances its reference only;
  no new device pixel coverage is claimed.
  CSS layout/text behavior remains the deliberate product contract; no Yoga
  semantics or guest Wasm engine are introduced. Source/oracle and Silver pins
  advance; shader inputs/products retain actual `41a2120b` provenance.
- `73d7b1c2`: account for the complete 18-file public delta. Translate shared
  artboard animation/state-machine source reads, exact object-vector reservation,
  and nested semantic visibility without destructive tree reparenting. Preserve
  hidden-node registration, last-flattened-tree action/focus guards, boundary
  lifetimes, local-artboard parent lookup and sibling semantic owners. Include
  public sharing and nested-collapse regressions plus existing test updates.
  The authored RML/YAML and binary fixture are retained in the exact upstream
  fixture tree; private `.rive_head` exposes no implementation to port.
  Both independent reviews and correction rereviews are complete. Pool runtime
  unit tests passed 871 cases, 2 ignored (`c6c65dd9-7f00-4138-b095-b8de312baafc`);
  host accessibility/live moves passed 119 (`017ed3af-5d53-44ef-abab-deb361a8d8cf`);
  shipping C API compilation passed (`c0eb24d8-7dbf-4049-ba5d-11d3957e92c0`).
  Eleven upstream regression targets passed 90 cases in aggregate: nine targets
  passed 55 cases in `7f91b51a-8814-4dcf-b0ad-067fb07d18e8`, and the two corrected
  semantic targets passed 35 in `b7c00abc-3943-4d82-a0b3-e53d0751003b`. Initial validation
  caught a derived-method test lookup, two exact-type test casts that omitted
  NestedArtboardLeaf inheritance, and an obsolete pre-flatten action expectation.
  Corrections preserve source identity assertions and enforce the new upstream
  last-snapshot membership timing; no production behavior was invented for tests.
  The selected-handle borrowed-list correction was also rereviewed. CSS layout
  remains deliberate product behavior. Active source/oracle and exact Silver
  references advance (Silver is reference-only); shader products retain actual
  `41a2120b` provenance. No new device pixel coverage is claimed.
- `cf0f6752`: account for the complete 91-file public delta. Translate renderer
  recording/resource lifecycle and BC format guards, native GPU format guards,
  view-model/trigger/delegate lifetime fixes, lazy-vector replacement, bounded
  UTF decoding, raw-text/input and native Luau text regressions, with applicable
  public tests. Guest-Wasm tiering, frame GC/reporting, VM/JS/AssemblyScript
  changes and fixtures remain excluded under UNIV-3728; private `.rive_head`
  exposes no additional implementation. One public malformed-byte TextValueRun
  fixture is not literally executable through the existing safe-String API:
  length already uses bounded bytes, and binary import normalizes invalid UTF-8
  before storage. No lossy substitute is claimed as that upstream test.
  Active source/oracle and Silver references advance. Both independent reviews
  and correction rereviews are complete. Pool validation: runtime library 871
  passed, 2 ignored (`5e435195-bda3-46b6-9574-07b22ebbac74`); six runtime
  integration targets 96 passed (`80ed774f-c12d-4def-b0da-1aa89a301cec`);
  scripting lifetime 15 passed (`374c3a7e-8336-4e49-b431-2756b63091e8`);
  scripting text 13 passed (`aea68a1f-6b20-4a72-badd-2a6dc6b39a37`); renderer library
  592 passed, 6 ignored (`51062540-5af9-41e3-bae2-f1b2a0c9ff2d`). Vulkan
  shipping compilation passed (`076eea01-3e53-4079-b457-a5ef7a32dfda`), as did
  Metal tools compilation (`75685d4b-584a-448a-99d7-ddd6a828f67d`). Initial
  validation exposed a test-only private import and an obsolete Lua byte
  prefilter; both were corrected against source and rereviewed, with assertions
  unchanged. Generic LazyVector snapshots retain safe copy-on-write semantics;
  live delegates use shared nullable weak slots for in-flight removal. No new
  device pixels are claimed. Shader products retain actual `41a2120b` provenance.
- `4e930086`: translate tools-only image hide/reveal state and bounded hosted
  artboard settling, native Luau deterministic seeding and draw-time settling,
  and CSS-compatible image-asset layout invalidation. Preserve image identity,
  Taffy layout behavior, and the default seed of 1. The complete public delta
  has 13 files; guest-Wasm image proxies, host VM seeding and guest draw bridges
  are excluded under UNIV-3728. CLI options, capture paths and tests mentioned
  in the commit message are absent from the public delta and are not claimed.
  Private metadata exposes no additional implementation. Translation and both
  independent reviews are complete; all corrections were rereviewed. Pool results:
  render API 40 passed (`2ff76def-6917-4ee4-a2df-defeb9a2101c`); runtime 871
  passed, 2 ignored (`10bf1c8f-903d-4093-add8-da15a9e81ff9`); renderer 590 passed,
  6 ignored (`1a2188ca-cc9b-43d9-b0d5-4b0bc3d1e1a5`); four scripting targets
  43 passed (`e10e4f8f-6d7b-423d-8172-5911bbef192c`), all with zero failures.
  Tools-disabled C API check passed (`04fa7fd8-2056-4a43-b633-8e38326f3f73`),
  as did Metal tools compilation (`9dcd8251-9f49-472e-bb7c-7c679e877da9`).
  Initial test-fixture and ScriptArtboard bridge compile omissions were corrected
  and rereviewed before these passing reruns. The first macOS request was
  requeued after source changed, not a source failure. The getter-only foreign
  image bridge cannot observe a later identical raw-dimension write; native
  retained owners share visibility state and shipping builds carry zero-sized
  state. Generated build metadata also registers two already-tracked `6771fdad`
  tests, without adding new test behavior. No new browser/device pixel coverage
  is claimed. Active source/oracle and Silver references advance; shader products
  retain actual `41a2120b` provenance.
- `91a25f40`: translate the GL destination READ framebuffer rebind after
  PLS copyback, updating the mechanical implementation, raw source authority
  and existing command-order regression. The full public delta contains this
  GL implementation and private `.rive_head` provenance only; Flutter tests
  and changelog mentioned in the commit message are absent from the public
  delta and are not claimed. Active source/oracle and Silver references advance.
  Both independent reviews are complete and clean. Linux pool renderer library
  tests with `renderer-webgl2,with-rive-tools` passed 590 tests, zero failures
  and 6 ignored in `e681325a-a649-4c4a-b416-df67935f07b8`, including the exact
  copyback command-order regression. Shader products retain actual `41a2120b`
  provenance. No new live Chrome PLS or device/pixel coverage is claimed; prior
  browser-build infrastructure limitations remain historical and unresolved.
- `3fb675a0`: account for the complete two-file public delta: upstream's
  `tests/deploy_tests.py` guards the blank-command indexing check when the
  server-only command is `None`; the other file is private `.rive_head`
  provenance. The deployment helper has no live downstream counterpart or
  caller, as already established for `2a29c82d`; no runtime change applies.
  Active source/oracle and Silver references advance. Both independent reviews
  are complete and clean; exact regenerated Silver changes only its source
  reference. No compilation was run because runtime and shader inputs are unchanged.
  Shader products retain actual `41a2120b` provenance and historical evidence,
  including its supplementary macOS compilation result; no private code is claimed.
- `f16d5e66`: translate the data-converter interpolator restart timing correction
  and public regression from the header, implementation and converter test.
  The fifth public file is private `.rive_head` provenance, not implementation
  evidence; no absent Dart changes are claimed. The remaining public file is
  the authoritative `time_based_interpolation.sriv` recording, updated upstream
  from 11,979 to 11,986 bytes (SHA-256
  `d258242179539e07f6e7db878086a77758161fbc37e691b2c81a358978afa6b2`).
  Nuxie reads this expected recording from the pinned upstream fixture; no
  tracked downstream duplicate exists. Its producer remains
  `data_binding_test.cpp:1958` ("Interpolate color and number"). Existing
  operation comparisons and tolerances are unchanged. Active source/oracle and
  Silver references advance; shader products retain actual `41a2120b`
  provenance. Both independent reviews are clean. The whole converter suite
  passed 4 tests with zero failures in pool job
  `fa13ee93-6a8b-454f-84a6-894975147ebf`; the exact changed Silver replay
  `wave_b1_time_based_interpolation` passed 1 test with zero failures and 35
  filtered in `39d369a7-21c6-424e-a025-ba574e673f1f`. Both used the default
  arithmetic lane, without tolerance changes or downstream expected-byte edits.
- `2a29c82d`: account for the complete two-file public delta: upstream's
  `tests/deploy_tests.py` adds Windows Ctrl-C handling, a PowerShell blank-command
  fix, server-only operation and `.mjs` MIME support; the other file is private
  `.rive_head` provenance. Nuxie has no live counterpart or references to this
  deployment helper in tools, crates, Bazel or CI. No runtime change applies,
  and no private implementation is claimed. Active source/oracle and Silver
  references advance; both independent reviews are complete and clean. Exact
  regenerated Silver was checked to change only its source reference. No new
  pool compilation was run because runtime and shader inputs are unchanged. Shader products retain
  actual `41a2120b` provenance, with historical evidence and limitations intact.
- `3d7dcc5b`: inspect upstream removal of the two CoreGraphics renderer paths,
  related Skia/build/testing-window plumbing and the old Silver visual-diff
  source. These renderer/test-window adapters and the optional Silver visual
  diagnostic were never shipped in Nuxie; no live downstream owner is deleted.
  Apple CoreGraphics image/font facilities and framework links remain required
  and unaffected; current GM capture uses the separate bitmap-decoder/third-party
  path. Silver producers, replay and baselines are unchanged. Source-equivalence
  and Rust-integration reviews are complete and clean. Exact regenerated Silver
  metadata was verified to change only the source reference. No new pool
  compilation was run: runtime and shader inputs are unchanged. Active source/oracle and Silver references advance to this source;
  shader generation and imported products retain actual `41a2120b` provenance.
  Historical validation evidence and captures are retained. Private `.rive_head`
  provenance is not treated as implementation evidence.
- `e406d1f4`: translate per-entry texture/sampler pairs in shared ORE metadata,
  pipeline sampling and layout derivation, Luau entry-point forwarding and GL
  unpaired-before-paired sampler binding. Preserve upstream's optional pair
  block behavior: older/unknown metadata remains file-wide, truncated pair
  blocks leave all entries unknown, and an overflowing extension is omitted.
  Translate the public entry-layout regressions. Guest Wasm host from-RSTB
  creation, ABI/generated bindings, JS bridges and AssemblyScript fixture changes
  are excluded under UNIV-3728; private metadata exposes no extra source.
  The parked editor/compiler integration remains unmerged; this runtime change
  does not claim its producers emit the new optional metadata yet.
  Translation and both independent source-equivalence and Rust-integration
  reviews are complete and clean. Pool renderer library validation with
  `renderer-webgl2,with-rive-tools` passed 590 tests, zero failures and six ignored
  in job `082c3a4c-1791-441d-9e73-397035ff45b0`; both new entry-layout regressions
  and the GL sampler-order case ran. Luau `shader_asset_resolution` passed all
  18 tests, with zero failures or ignored, in
  `2104b90d-a6b6-4fa2-95fc-b642de426bf2`. The non-tools Vulkan consumer check
  passed in `4c2c6b92-4774-47c5-b093-f042339a5151`. Active source, oracle and
  Silver pins advance; no shader regeneration was needed and products retain
  actual `41a2120b` provenance. Browser and Metal hardware remain unqualified;
  the supplementary macOS check later passed at source `41a2120b`, as recorded
  below, and is not an `e406d1f4` validation result. No new device pixel
  validation is claimed.
- `41a2120b`: translate the shared angular-gradient coordinate correction so
  stops advance clockwise, matching the upstream Canvas2D/Figma direction.
  The GPU owner delta only clarifies that the gradient paint types must fit
  three packing bits; it adds no new packing behavior. Private `.rive_head`
  provenance does not expose additional implementation and is not claimed.
  Active source/oracle and Silver references advance to this source. Both
  independent reviews are clean, including rereview of the raw source-authority
  newline correction. Pool shader generation
  `c2b0de72-4cc1-4ec6-8b09-258d3eb297dd` succeeded; the imported 66 WGSL files,
  96 SPIR-V headers, 24 embedded GL strings and 54 WebGPU GLSL/export files were
  verified against those products and carry actual `41a2120b` provenance.
  Export aliases are unchanged; historical Metal captures retain their original
  provenance. WebGL2 library pool job `9b19e65a-8da3-4e2f-8553-43a09ea0328f`
  passed 574 tests with zero failures and six ignored. Vulkan consumer check
  `6104b07d-1ba3-4c95-a2f7-2cb6882503f4` passed. The supplementary macOS check later passed in
  `4ca8cef0-b210-42f0-86c0-b2c575fa7fc9`: its report records platform `macos`,
  build exit code 0 and source
  `41a2120bb572e8168ba08fd2ea514c9ddf25d270`, snapshot
  `9f1b9ad9e072dfaec78e2c5784e48e27fd1abcb99b9839b8bfc7ce203a9ca91b`.
  Its delay was source preparation, not a source failure. This is historical
  `41a2120b` compilation evidence, not an `f16d5e66` result or pixel validation. The prior wasm32 CopyFile infrastructure limitation
  is unchanged and that target was not rerun. No new browser, ANGLE or device
  pixel validation is claimed.
- `d278866b`: translate depth-stencil layer-mask support for GL/WebGL2:
  capability gating, separate misc pipeline/batch flag, destination-scaling blend,
  color-write suppression and the shared depth-stencil shader mask factor.
  Vulkan/WebGPU remain unsupported in this mode, matching upstream; raster-ordering
  support is unchanged. No DrawContents bit or sort-key width is added.
  Translation, independent source-equivalence and Rust-integration reviews are
  complete, including correction rereviews. Corrections refresh exact source
  byte/line counts and minified-name test fixtures without weakening assertions;
  the load-store fixture mismatch was already present at downstream `95dad0e6`.
  Pool shader generation succeeded in job
  `f4940aed-22d9-48b5-a07e-70d5fae11a23`; imported products carry actual
  `d278866b` provenance: 66 WGSL files, 96 SPIR-V headers, 24 embedded GL
  strings and 54 WebGPU GLSL/export files. Historical Metal captures retain
  their original provenance. Active source/oracle and Silver references also
  advance to this source. Linux/WSL pool WebGL2 library tests passed 574 tests
  with zero failures and six ignored in job
  `49d7c5d1-e3a6-457c-a240-1ec2afa1ab6b`. The initial compile found the new GL
  regression using a nonexistent default constructor; replacing it with the
  existing `PipelineState::new()` preserved all assertions, passed both
  correction rereviews, and passed this rerun. Pool Vulkan consumer check passed
  in `d3acf3cc-89d6-49a5-bcbd-ce7f3e2c95d1`; Metal consumer check passed in
  `a77d9437-d848-477f-994b-38aea3cdb3af`. Actual `wasm32-unknown-unknown` pool
  check `4d52eee5-6e5e-4480-be72-12339e614bfe` failed before renderer compilation
  on the previously observed `rules_rust` CopyFile remote-only execution-strategy
  error. That target remains unverified; these native checks do not establish
  browser pixel qualification. Existing Metal pixel cases and their
  tolerances remain unchanged, including the max-channel budget of 48. The added
  upstream ANGLE/EGL 4x-MSAA pixel lane has no Nuxie native ANGLE test window;
  that coverage is unavailable and is not claimed by Metal validation.
  Private `.rive_head` and Dart/editor changes absent from the public delta are
  not claimed imported.
- `6771fdad`: translate applicable component-list row lifecycle, default-machine
  recording/reset, queued focus/semantic and tracked-pointer cleanup, scripted-row
  eligibility and dirty-layout iteration behavior, plus public regressions.
  Include compatible redundant-dirt suppression, the top-size text-fit check,
  and bounded glyph-outline caching while preserving CSS/Taffy and host-font
  behavior. No Yoga semantics are imported. Both independent reviews and
  correction rereviews are clean: script eligibility uses polymorphic data-bind
  access, and pointer reset dispatches through the Rust listener trait without
  callbacks. CSS unit pool job `b8d94137-819c-4923-a669-5d78cdf57d18`
  passed all 422 selected tests (450 filtered). Linux pool job
  `74e01102-026f-4201-8979-ae8e0b78ba6f` passed all 83 tests across seven
  list/semantic/text integration suites, after correcting the new test's
  identity comparison to use the existing weak
  state-machine handle API; its allocation-identity assertion is unchanged and
  both correction reviews are clean. A supplementary macOS pool run,
  `a476bfac-22e7-43b4-b25b-2151edd42f70`, failed with exit 143 after losing its
  scheduler resource reservation. That infrastructure failure does not establish
  a source regression; the completed Linux 83-test result remains the evidence.
  Authoritative Silver metadata was
  regenerated (13 producer-line shifts and the source ref only); all 30 manifest
  checks pass, with existing CSS divergence annotations and tolerances unchanged.
  Ordinary/scripted goldens remain unrun because their C++ builder has no
  declared pool target. No new GPU/device qualification is claimed.
  Private `.rive_head` contents are not claimed.
  At this checkpoint, actual shader generation and artifact provenance remained `74e5d3f6`.
- `094d24e1`: account for the complete four-file public delta. Three shader
  files put ATTR declarations on single lines and add clang-format guards for
  Unreal's compiler. Tokens and supported runtime behavior are unchanged; Nuxie
  has no Unreal shader consumer. Private `.rive_head` changes are not claimed.
  Advance active source/oracle and Silver references only. Shader source/archive,
  tools and generated artifacts keep actual `74e5d3f6` provenance. Independent
  source-equivalence and separate Rust-integration reviews are clean; all 30
  Silver manifest checks pass. No new compiler or hardware validation is claimed.
- `8ea42e0e`: text-input `maxCharacters`/`maxLines` limits and the empty-line
  cursor fix are eligible. Upstream Yoga/legacy hug-layout semantics are
  excluded by the CSS/Taffy product decision. CSS-owned TextInput maxLines
  measurement uses the current laid-out max_width, never the upstream Rive Hug
  shortcut's stale measured width. Non-CSS text with an explicit existing Hug
  setting uses measured width. Upstream Yoga/legacy hug helper and engine changes
  are excluded. Both independent reviews and correction rereviews are complete.
  High-level pool validation (job prefix `3d6666b6`) passed 13 tests. The initial
  raw-input run passed 53 tests, including all 17 new upstream cases, and failed
  one existing outline-emoji case because its machine-local font was unavailable.
  That case now embeds the existing checked-in HarfBuzz Zycon fixture; its
  assertions are unchanged. Raw-input pool rerun
  `87f0956d-ef1e-4984-877a-b9abf12fc6c5` passed all 54 tests.
  Schema validation exposed two preexisting stale inventories after the live-move
  records landed: exact definitions are 373, and runtime properties are 815
  (798 + 15 live-move fields + 2 text limits), with 314 animatable fields
  (310 + 4 live-move fields). Exact exclusion tuples now include LiveSlot,
  LiveStep and LiveStepUnit. The C++ Id-dispatch parser now detects the next
  static method regardless of clang-format indentation, preserving all exact
  comparisons. Schema pool rerun `a8d891ea-5100-41b9-91d8-c69ae573fcfc`
  passed all 47 integration tests (24 header comparisons, 22 schema checks,
  1 Id-dispatch check); no assertions were weakened. All 30 Silver manifest
  checks also pass.
  Ordinary and scripted goldens were not run: their C++ builder invokes local
  premake/make and has no declared pool target. No golden or device coverage is
  claimed. No private Dart/FFI changes absent from the public delta are claimed.
  Active source/oracle and Silver provenance advance; shader products and tools
  retain actual `74e5d3f6` provenance.
- `156e959c`: translate inner-feather change notification and path rebuild,
  with the public upstream path regression. Private `.rive_head` contents are
  not claimed. Both independent reviews and correction rereviews are clean.
  Pool job `0516edac-fc39-4054-be50-fba817bf7b20` passed all 7 selected feather
  unit tests (0 failures, 0 ignored, 860 filtered), including the new upstream
  regression, existing inner-text coverage and two CSS shadow tests. All 30
  Silver manifest checks pass. This is scoped validation, not broad suite or
  device qualification. Active source/oracle pins advance; exact Silver
  regeneration shifts five path-test producer lines by 45 without changing
  expectations. Shader products and tooling retain actual `74e5d3f6`
  provenance; earlier validation evidence below remains historical.
- `47d1cd54`: translate applicable sampler validation and WebGPU feature
  support from this mixed commit. Guest Wasm/AnimaScript web test-player and
  dependent premake changes remain excluded under
  [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728); private `.rive_head`
  content is not claimed. Both independent reviews and the test-import correction
  rereview are clean. macOS pool job `b7d1d91b-20a0-4022-8ebd-9ff18ed09a11`
  passed 266 tests with 0 failures and 5 ignored, including the upstream sampler
  regression. Initial job `33850159` failed compilation because the new test
  used the wrong export; it was corrected to the existing public `script_guards`
  export without weakening assertions. All 30 Silver and 16 provenance checks
  pass. No new browser/device coverage is claimed. Active source/oracle pins
  advance; shader products and generation tooling retain actual `74e5d3f6`
  provenance. Results for earlier checkpoints below remain historical.
- `4efe82a1`: account for all nine public files in the clang-format 19.1.7
  housekeeping delta. Eight code/shader files change whitespace/comments only;
  `.rive_head` is private provenance. No unseen RML regeneration is claimed.
  Advance active source/oracle pins and exact Silver metadata without changing
  Rust behavior. Independent source and integration reviews are clean; 30 Silver
  manifest and 16 provenance tests pass. No new compiler
  validation is claimed; the `74e5d3f6` results below remain historical.
  Shader source/archive/tool pins and generated artifacts remain at their actual
  `74e5d3f6` generation provenance despite equivalent shader tokens.
- `74e5d3f6`: translate the dedicated angular-gradient fragment feature,
  modulated-image bit 4, angular-gradient bit 9, ten-feature key/mask contract,
  and 18 specialization slots across shared, Metal, Vulkan, WebGPU and WebGL2
  consumers. Import exact pool-generated shader products and refresh generated
  aliases, Metal capture ordering and namespace assertions coherently. Both
  independent reviews and correction rereviews are clean. Pool shader generation
  passed (job `8eb47d12-0906-4ce1-8828-f76a6d84f51f`); renderer library tests
  passed 265 with 5 ignored (job prefix `6eccc0f0`); Vulkan consumer compilation
  passed (job prefix `04421b57`). macOS Metal consumer compilation passed
  (job `9d082198-b890-4da4-ac12-bafec22298ca`, exit 0).
  Local metadata checks passed: 30 Silver, 16 provenance and 6 shader verifier
  checks. Active source/oracle pins and generated products now use this revision;
  Silver producer lines are unchanged. Browser consumer validation remains
  unqualified after earlier Dawn-link and rules_rust CopyFile failures: neither
  a pass nor a proven baseline failure is claimed. No new device pixel coverage
  or full-feature parity is claimed. Unreal-only preprocessing has no supported
  downstream owner; private `.rive_head` changes are not claimed. No CSS layout
  semantics or guest-engine work is included.
- `16c184ef2`: account for the complete 2,675-file formatting delta. All 2,674
  modified files retain code/preprocessor tokens except private `.rive_head`
  provenance; the added shader `.clang-format` is tooling configuration only.
  Shader generator/Makefile bytes and all 36 minifier batch token streams are
  unchanged. No Rust behavior changes or fresh compiler evidence are claimed.
  Active runtime/oracle pins advance; Silver provenance refreshes 241 producer
  line numbers only. Retained shader artifacts, exact source snapshots and their
  generation tools remained pinned to `ada892f3`, with token equivalence
  through that checkpoint. That split was superseded by the fresh `74e5d3f6`
  pool generation; historical artifacts were not relabeled as rebuilt.
- `ada892f3`: translate the five retained shared shader owners; unsupported
  D3D host/backend changes and the D3D root signature are not imported. Private
  `.rive_head` contents are not claimed. Regenerate portable shader products on
  the pool using pinned compilers; refresh minified aliases and sampled-clip
  overrides coherently. Both source and integration reviews passed, including
  rereviews of alias corrections. Pool generation passed (job
  `65d1b4db-9ee8-489d-941b-87b521a0f9c9`); renderer library tests passed
  265/0/5 ignored (job `35e28b9f-977c-4502-bfc6-92ed2033678a`). Explicit
  macOS Metal compilation passed (job `d65acac0-1ba8-4f71-842b-29609ce8ca95`),
  using the translated Metal source before browser-only artifact/alias refresh.
  Browser-feature native tests did not run: Linux linking lacked Dawn symbols.
  The wasm32 consumer check was blocked before consumer compilation by the
  pool's remote-only strategy rejecting rules_rust's CopyFile action. Neither
  is counted as a pass or asserted to be a proven baseline failure. Browser
  execution and new hardware raster comparisons remain unverified. Metadata
  checks passed (30 manifest, 16 provenance, 5 pipeline); both workflow YAML
  files and Buildkite JSON parse. Editor/compiler publication remains parked.
- `12e5327b`: replay only each canvas's last frame, discard replaced ranges
  before dependency scheduling, close stale same-canvas ranges on explicit
  begin, and reset new-frame recorder transforms without resetting nested
  resumes. All four public upstream regression cases are translated.
  Independent source-equivalence and Rust-integration reviews passed. Pool
  renderer validation (`--no-default-features --features with-rive-tools --lib`)
  against the pinned upstream fixtures passed: 265 passed, 0 failed, 5 ignored,
  including all four new regressions. Job `28028624-f5f7-4703-b8f9-8f4d929a97b0`
  completed successfully; its downloaded evidence contains no cleanup receipt,
  so cleanup is unverified. No hardware coverage is claimed. Prior checkpoint
  results below are historical.
- `8920a6b0`: per-entry shader binding metadata, narrowed automatic layouts,
  actual-layout slot validation and retained draw binding validation translated
  through shared ORE, deferred recording/replay, Luau and live renderer adapters.
  Independent source and Rust-integration reviews passed, including correction
  rereviews. All eleven upstream entry-layout tests pass. Pool validation:
  renderer 261 passed/5 ignored; runtime 806/2; scripting with authored Metal
  359/1; tools-enabled ORE 125/0; recording 9/0; render API 39/0. All 51
  provenance/manifest/pipeline metadata checks pass. Native Metal and both
  wasm32-unknown-unknown browser configurations compile on builder-mac.
  The Linux WebGPU attempt failed before renderer compilation because a
  rules_rust CopyFile action had no available remote execution strategy; no
  Linux pass is claimed. Browser checks used the unchanged pool client's
  platform constraint with the same Wasm target and panic flags, not local
  compilation or a changed execution policy. Initial test failures were Rust
  fixture setup/projection errors, corrected without weakening upstream assertions.
  Browser pixels, Vulkan/device execution and Golden/Silver execution were not
  rerun. Some earlier failed/completed jobs reported pending scratch cleanup;
  successful browser and native Metal jobs report verified cleanup.
  RSTB section 2 now carries upstream entry metadata; private authenticated
  Metal reflection moves to section 128, while Metal target 2 is unchanged.
  The authorized editor/compiler coordination and artifact recompilation are
  tracked in [UNIV-5495](https://universe.basis.dev/issue/UNIV-5495).
  By explicit user decision, the editor/compiler branch remains parked and
  unmerged because the current iOS release still uses the old format. Its rollout
  requires separate authorization under UNIV-5495; no release is claimed complete.
  Its compiler suite passes 135 tests; this does not claim private upstream
  compiler code was imported. Guest Wasm bindings remain excluded under UNIV-3728.
- `0026fac6`: outside-primary-press text-input blur, retained focus identity,
  root-space affine mapping and FocusData forwarding translated. Existing CSS
  dispatch remains intact. Independent source and Rust-integration reviews passed.
  Linux pool validation passes all three viewport/focus regressions and 806
  runtime units, with two existing ignores. Pinned fixtures were bundled by pool
  tooling. All 30 Silver-manifest checks pass after regenerating one producer-line
  reference (53 to 54); no expected behavior or tolerances changed. Private Dart
  controller/bindings mentioned in the commit message are absent from the public
  delta and are not claimed ported. Golden/Silver execution, browser compilation,
  Apple hardware and tools root-transform callback cases were not rerun.
- `80d76ee6`: C++ kit-build-only change adds `_nx_platform` to HarfBuzz's
  warning-diagnostic filter. Shipped Rust uses harfrust; no runtime translation
  is needed. Pinned native oracles inherit upstream's Premake file. `.rive_head`
  is private provenance only. Active pins advance without relabeling historical
  results. Silver manifest checks pass (30); no runtime/hardware rerun for this
  build-only change.
- Validation at `868da86c`, integrated atop downstream `50bf96810b`:
  ported per-layer view-model pending-change windows, pending-reader classification,
  layer-aware trigger comparisons and frozen-layer searched marks. Independent
  source-equivalence and subsequent Rust-integration reviews passed. All 22
  change-window cases (including four new upstream cases) and 793 runtime units
  pass, with two existing ignores, on the rebased branch via builder-mac. All 51
  provenance, Silver-manifest and pipeline metadata checks pass. Initial remote
  tests lacked fixture files; supplying the exact pinned upstream assets resolved
  that infrastructure failure. No assertions or tolerances changed. Stack grid
  semantics, layout benchmarks and layout-only fixtures are explicitly excluded
  under the October 23 CSS policy. The matrix keeps pre-change expectations for
  those stack-cell cases through `fixtures/sync/layout/matrix_overrides.json`
  under that policy. `.rive_head` is private provenance, not ported
  functionality. Renderer sources are unchanged. Golden/Silver execution, browser
  compilation and hardware qualification were not rerun for this change.
- Validation at `5551696a`, integrated atop downstream `d714f3f22c`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All five new upstream text cases and the changed component-list case
  pass. Runtime units pass 434 with two existing ignores; browser runtime
  compilation passes. The existing local paint-only test required redundant
  remeasurement, the behavior this upstream commit explicitly removes. Both
  reviewers approved exact unchanged measurement count instead, preserving
  width and zero-extra-shaping checks. Test-only trait-import/string-ownership
  fixes were also rereviewed; no production workaround was added. The 61
  metadata/provenance tests pass and build registration is current. Runtime
  Silver passes with 85 byte-exact, 119 epsilon, 10 declared divergences and
  17 unsupported cases; the updated rewards recording is consumed from the
  exact upstream pin. Fresh pinned ordinary Golden passes 363 exact entries
  and verifies one existing divergence; scripted Golden passes 361 exact
  entries and verifies three existing divergences, each across 364 entries.
  CSS/Taffy solving is unchanged; no Yoga solver or benchmark speedup is claimed.
- Validation at `16332c8e`, integrated atop downstream `78423a6f93`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean; the test-feature-gating finding was fixed and rereviewed. Runtime
  units pass 434 with two existing ignores; the new supplemental angular
  import/clone/paint test and both existing gradient-stop tests pass. Runtime
  wasm32-unknown-unknown compilation passes. Metadata checks pass 39; schema
  and Bazel registration were regenerated and the Silver manifest is current.
  Fresh pinned ordinary Golden passes 363 exact entries and verifies one
  existing divergence; scripted Golden passes 361 exact entries and verifies
  three existing divergences, each across 364 entries. Runtime Silver passes
  its existing classifications: 85 byte-exact, 119 epsilon, 10 divergences and
  17 unsupported cases. Schema validation retains 19 known header-comparison
  failures (UNIV-3729) and a stale aggregate-count assertion (expects 358;
  the pre-change schema already had 362, now 363). No assertions or tolerances
  were relaxed. The public delta provides no angular-gradient asset or tests:
  supplemental tests verify real import and factory/paint calls, not a new
  pixel comparison. Private editor/Dart/RML changes are not claimed imported.
- Validation at `269ebcab`, integrated atop downstream `0d460f0328`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All four translated loader test groups pass, including 10 instance
  failure combinations and five device failures in headless/presenting modes.
  Full renderer units pass 950 with 11 existing ignores and only known
  UNIV-4801 failing. Android Vulkan C API libraries compile; metadata checks
  pass 39 and the manifest/build registration are current. The fake loader
  exercises real production helpers but intentionally fails device creation
  after capturing extensions: no physical Vulkan startup is claimed. No
  OpenGL fallback was added; unchanged bootstrap/version negotiation is not
  newly certified by this delta. Browser-only compilation was not repeated
  for this native-only change.
- Validation at `5e3de385`, integrated atop downstream `b08d8e6eb0`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Renderer units pass 946 with 11 existing ignores and only known
  UNIV-4801 failing. Both complete upstream delayed-image tests pass using
  actual null-context renderer paints/images and the byte-identical PNG.
  Both wasm32-unknown-unknown browser backends compile; metadata checks pass
  39 and the Silver manifest/build registration are current. Private web mesh
  fixes, web sink override and browser tests are absent from the public delta;
  no browser pixel result or private host implementation is claimed imported.
- Validation at `f2aa739e`, integrated atop downstream `2f12e1d6cd`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Renderer units pass 944 with 11 existing ignores and only known
  UNIV-4801 failing; both new upstream canvas command-buffer spy tests pass.
  Both wasm32-unknown-unknown browser backends compile, as do Apple Metal and
  aarch64-linux-android Vulkan C API libraries. Metadata checks pass 39 and
  the Silver manifest is current. Supplied command buffers have an explicit
  unsafe host lifetime/submission contract. Concrete Rust hosts still return
  default resources: the private Android override is absent from the public
  delta, so Android stall elimination is not claimed. No new Android/Vulkan
  device execution or browser pixel coverage is claimed.
- Validation at `183df53f`, integrated atop downstream `f0303f3e8c`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Render API units pass 38; runtime units pass 432 with 2 existing ignores;
  translated RawPath integration tests pass 2; metadata checks pass 39. Renderer
  units pass 942 with 11 existing ignores and only known UNIV-4801 failing.
  All three stroke-position GMs pass unchanged exact pixel comparisons, including
  the two expanded 480x1920 fixtures. New pristine and approved-adapted C++
  captures have verified source, toolchain, adapter and PNG provenance at
  `/private/tmp/nuxie-oct16-183df-gms.4HE0LC`; existing FMA/coverage adaptations
  remain explicit. The new zigzag exposed a preexisting initial-cap translation
  error: Rust used the original line tangent instead of the emitted cubic's
  rounded control tangent. The source correction passed both rereviews and
  eliminated the stable 2/21-pixel differences without tolerance changes.
  Both wasm32-unknown-unknown browser backends compile on the final bytes.
  No new Android, other vendor-hardware or browser pixel coverage is claimed.
- Validation at `e7124f18`, integrated atop downstream `291a8a027c`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean after preserving the new SIMD block's separate product/reduction
  arithmetic. Runtime units pass 432 with 2 existing ignores; renderer units
  pass 942 with 11 existing ignores and only known UNIV-4801 failing. Metadata
  checks pass 39; manifest and build registrations are current. Three focused
  trickycubicstrokes/roundcaps/feather Metal comparisons pass unchanged contracts
  against fresh revision-stamped C++ on matching Apple M5 Max adapters. None
  are byte-exact: first two have max delta 1 and no pixels beyond delta 2;
  feather has max delta 4 and two pixels beyond delta 2 within the existing
  32-pixel budget. Evidence: `/private/tmp/nuxie-oct16-e712-metal`. No new
  browser pixels, Android or vendor-hardware coverage is claimed. Renderer
  formatting passes; whole runtime-owner formatting still reports the two
  preexisting compact cpp_min/max helpers, unchanged in this delta.
- Validation at `1c75867d`, integrated atop downstream `b0ab40448a`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Integration added the missing exact JOIN_TYPE_MASK constant; both
  reviewers cleared it. Native renderer units pass 942 with 11 existing ignores;
  only the known UNIV-4801 uniform-range fixture fails. The initial run also
  exposed obsolete test expectations for encoded join flags and the generated
  GLSL byte count; exact replacements were rereviewed and the full suite rerun.
  Both browser backends compile. Shader reproducibility passes all 66 WGSL
  modules and 56 upstream headers; the reviewed generated inventories include
  54 retained GLSL files, 24 WebGL payloads and 96 Vulkan headers. Metadata
  checks pass 39 and generated manifest/build registration is current.
  Fresh revision-stamped C++ ffi-metal and Rust rust-metal-atomic replays pass
  all three existing trickycubicstrokes, roundcaps and feather fixture contracts
  on the same Apple M5 Max. None is byte-exact: the first two have max delta 1
  and zero pixels beyond delta 2; feather has max delta 4 and two pixels beyond
  delta 2, within the unchanged 32-pixel budget. Binary, stream, archive and
  output provenance is retained at /private/tmp/nuxie-oct16-cusp-metal. No
  Android/vendor hardware or browser pixel validation is claimed. Existing
  scalar exceptional-angle limitations are not relabeled complete parity.
- Validation at `e8d647fd`, integrated atop downstream `ea6475e4ba`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean after explicitly excluding ARM32. Stable Rust 1.94.1 does not expose
  ARM32 neon/fp-armv8 cfgs even with those codegen features enabled; the owner
  now rejects that unshipped target rather than silently selecting wrong math.
  Its retained ARM32 branches are not executable/validated support; future
  enablement needs explicit target wiring and validation. Apple Silicon runtime
  units pass 408 with two existing ignores; translated SIMD cases pass 27 and
  strict SIMD cases pass 14, including hardware reciprocal-square-root estimates
  at the unchanged upstream tolerance. The constant bit-cast assertion compiles.
  Browser runtime compilation passes; metadata checks pass 39 and generated
  manifest/build files are current. Portable random samples retain upstream
  counts/scales but are not libc-rand-identical. GVec4x4 is safe Rust storage,
  not a generic native-vector ABI claim. x86 execution, Android devices and
  browser pixels were not rerun. The renderer tuple-only edit needs no caller
  migration here; this does not certify its preexisting scalar helper's math:
  NaN clamping differs, and FMA requires the documented compiler-contraction
  context. No full renderer or golden sweep was rerun for this delta.
- Validation at `e54bc92a`, integrated atop downstream `1ae4dd47ac`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Source review corrected the test helper to use upstream's derived-aware
  drawable access rather than a concrete downcast. Runtime units pass 408 with
  two existing ignores; scripting units pass 364 with one existing ignore.
  Both new legacy-hit fixture tests, both hidden-hit cases, 27 scroll-input
  cases and seven scripted-scroll cases pass. The first scripted-scroll run
  contained zero tests without upstream-test-seams; only the corrected seven-case
  run is execution coverage. Real Lua lookup tests cover functions, missing and
  nonfunction fields, inherited fields and errors. The exact upstream fixture
  blob is d4e97d49ca5024c2a82f91797ac3ac90ce71efc5. Metadata checks pass 39;
  generated manifest/build registration is current. Existing instance-level
  borrowing remains an approved adapter boundary; arbitrary same-instance
  callback reentry is not proven by these tests. Browser/device validation was
  not rerun for this shared host/runtime change.
- Validation at `113d1bce`, integrated atop downstream `0f7c9c5414`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean; source review corrected a historical comment's snapshot claim without
  changing behavior. Native renderer units pass 940 with 11 existing ignores;
  the only failure remains UNIV-4801's unchanged WebGL uniform-range fixture.
  All new bounds and revised area regressions pass. The bounds test retains
  200 paths, seven matrices, one-to-eight cubics and exact comparisons; its
  local portable PRNG is not a byte-identical libc-rand corpus. The obsolete
  triangle-area expectations were replaced using the exact new expression and
  pinned AABB/Mat2D headers under Apple clang 21 arm64, O0 and O2: exceptional
  payload 0x7fffffff, inverted-height area 0xa7152264, and both translated and
  untranslated thresholds 0x48800001. This isolated expression check is not a
  full C++ renderer run. Both browser backends compile; metadata checks pass 39
  and generated manifests/build files are current. An initial feature selector
  was rejected before compilation; the manifest-correct renderer feature roots
  were used for the reported result. No new Android, browser-execution or
  vendor-specific hardware qualification is claimed.
- Validation at `0f40af12`, integrated atop downstream `4f64dfd304`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All three translated font-binding tests pass, including the pinned
  Silver stream comparison and new null-base-font cache regression. The fixture
  follows upstream's no-explicit-binding flow and normal asset invalidation.
  Source/manifest/pipeline checks pass 39 after regenerating the manifest's
  shifted upstream source line; generated build files are current. Full runtime
  suites, browser execution and new hardware validation were not rerun for this
  single platform-independent cache correction.
- Validation at `8f06a91c`, integrated atop downstream `325b282ef7`:
  independent source-equivalence and subsequent fresh Rust-integration reviews
  are clean. Runtime units pass 408 with two existing ignores; editor-enabled
  runtime units pass 409 with two existing ignores; live scripting units pass
  363 with one existing ignore. All three supplied nested-animation/editor
  integration tests pass. The integration test initially called a nonexistent
  strong-handle identity method; both rereviews accepted the test-only correction
  to the existing weak-handle identity comparison. Production bytes were unchanged;
  the three integration tests were rerun, not the complete unit suites afterward.
  Live scripting and its runtime dependency compile for wasm32-unknown-unknown.
  Source/manifest/pipeline checks pass 39; generated build files are current.
  An initial manifest invocation omitted the explicit upstream directory and
  rejected another checkout's pin; the correctly scoped invocation passes.
  The translated Lua regression uses a real null-data host thread and replaces
  actual renderer/frame context roots before collection and VM destruction with
  File still alive. This tests approved Rust ownership, not native REV lifetime
  callbacks. A supplemental failed-hydration test confirms input remains unset.
  Native REV arenas, persistent-ID decoding/resolution, mutation/validation and
  borrowed editor comparator ownership remain excluded under d4fe1022; shared
  editor helpers are included. Inactive mirrored Lua modules are not claimed
  executed. No browser execution, new device or guest-Wasm coverage is claimed.
- Validation at `fdd555d6`, integrated atop downstream `5ea7a8fd55`:
  independent source-equivalence and subsequent fresh Rust-integration reviews
  and correction rereviews are clean. Source review restored the upstream
  constructor's all-slot name generation followed by forward clearing. Integration
  review restored owner-scoped reverse texture destruction so replaced rings do
  not defer deletion. Native WebGL tests pass 119, including all eight new cases;
  the sole failure is the unchanged uniform-range fixture tracked in UNIV-4801.
  Both browser renderers compile for wasm32-unknown-unknown. Source/manifest/
  pipeline checks pass 39; generated build files and four exact source snapshots
  are current. The rotation/reuse test uses actual Rust context/storage owners
  with a recording provider and zeroed records, preserving upstream counts and
  identity assertions but not drawing its rectangles on a GPU. Supplemental
  cases cover pending/failed fence replacement, ordering, SSBO exclusion,
  async modes, constructor order and owner/stale-generation destruction.
  Real browser GLsync plumbing is compiled, not browser-executed here. No actual
  ANGLE/Mali device, raster-image or new Vulkan hardware coverage is claimed.
- Validation at `ce788b3b`, integrated atop downstream `5b2837f46d`:
  independent source-equivalence and subsequent fresh Rust-integration reviews
  are clean. Runtime units pass 407 with two existing ignores; live scripting
  units pass 361 with one existing ignore. All three translated upstream cases
  pass, retaining 2,000/2,000/200,000 property counts and the strict two-second
  sweep threshold. Supplemental tests cover cleanup reentry, dying weak identity,
  moved-slot repair and stale-slot guards. The upstream unbound-number fixture
  uses test-local Lua userdata retaining real production lifecycle state because
  the host Number wrapper requires a bound model; numeric access is unexercised
  upstream, and no registration/disposal algorithm is duplicated in the fixture.
  Browser scripting and its runtime dependency compile for wasm32-unknown-unknown
  with luau/tools. An initial selector applied luau to nuxie-runtime and was
  rejected before compilation; the corrected package-scoped invocation passed.
  Source/manifest/pipeline checks pass 39; generated build files are current.
  Mirrored Lua VM modules remain inactive under the approved luaur boundary;
  no compiled execution coverage for those modules, browser execution, new
  device coverage or guest-Wasm execution is claimed.
- Validation at `ce891f52`, integrated atop downstream `afda479502`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Integration found stale internal aliases in
  the retained sampled-clip WebGPU bridge; corrected aliases restore its texture
  binding and single color output. These supplemental clockwise shaders have no
  current mechanical WebGPU consumer; this is generation-contract coverage,
  not newly enabled host behavior. Full shader reproducibility passes for all
  66 WGSL modules and 56 headers. Shared authority, retained GLSL, embedded GL
  payloads and Vulkan headers were independently checked against upstream output.
  Both browser renderers compile for wasm32-unknown-unknown. The native suite
  passed 930 tests (11 ignored), including live Metal pixel/pipeline/golden cases;
  its two failures were the unchanged UNIV-4801 fixture and a missed test-only
  minified-name literal. The exact literal was corrected, independently
  rereviewed, and its targeted rerun passes (one test); the full suite was not
  rerun after that test-only correction. No assertion was weakened. An initial
  native invocation incorrectly included WebGPU without Dawn linkage and failed
  before tests; the corrected feature selection produced the results above.
  Source/manifest/pipeline checks pass 39. No browser pixels, Android/Mali hardware,
  new Vulkan hardware qualification or guest-Wasm execution coverage is claimed.
- Validation at `6fa2cccf`, integrated atop downstream `bbfa0a9d09`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Integration review corrected owning GL
  execution-scope entry/restoration. Native WebGL2 passes 111 tests, including
  all three new precreation tests; one unchanged uniform-range fixture fails
  because it expects offset 16 despite the already-merged upstream 256-byte
  alignment rule. Source identity and ancestral history establish that baseline
  failure, tracked in [UNIV-4801](https://universe.basis.dev/issue/UNIV-4801);
  no separate baseline executable was run and no assertion was weakened.
  Both browser renderers compile for wasm32-unknown-unknown. Source/manifest/
  pipeline checks pass 39; generated build files and Silver manifest are current.
  An initial native invocation omitted the existing webgl2-test-support feature
  and failed before tests; the corrected invocation ran the suite. A stale
  snapshot byte-count assertion was corrected to the exact upstream header size
  and independently rereviewed. Final validation used frozen inputs.
  No browser pixels, real parallel GL driver, new Vulkan hardware or guest-Wasm
  execution coverage is claimed.
- Validation at `fa2cd6e3`, integrated atop downstream `ba5a8cdf71`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. All eight translated non-canvas runtime
  cases and the supplemental callback-reentry regression pass. Renderer units
  pass 740 with six existing ignores, including seven translated canvas cases,
  three live Metal pixel tests and two new golden cases. One unchanged allocator
  address-reuse test failed its allocation premise in the first full run; the
  unchanged full rerun passed. Serializer replay passes 14. Both browser
  renderers compile for wasm32-unknown-unknown. Source/manifest/pipeline checks
  pass 39, build-generator tests pass 16, schema reproducibility passes, and the
  generated build files and Silver manifest are current.
  Validation corrected TESTING feature registration, fixture API/borrow usage,
  and an existing test's transform-only update-dispatch expectation: upstream
  now also gives non-transform PaintImage an update override. The exact
  registration set and transform-only constraint assertions remain enforced.
  The final runtime unit rerun passes 399 with two existing ignores.
  New golden captures come from independently built pinned C++ with pristine
  and existing approved Metal-adapted captures retained. No tolerances changed.
  No broad Silver replay, browser pixels, Vulkan/Android hardware, performance,
  private editor behavior or guest-Wasm execution coverage is claimed.
- Validation at `30454306`, integrated atop downstream `09c3eaa7fe`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. The translated renderer's member API is accessible to its internal
  adapters; no new external Rust crate API is claimed. Source/manifest/pipeline
  checks pass 39; generated build files, corpus metadata, changed-file formatting
  and diff hygiene pass. The first native validation attempt exhausted host
  disk space before tests executed, and queued browser compilation was cancelled.
  After task-local output cleanup and capacity recovery, the reduced-parallelism
  retry passes all 728 renderer tests with six existing ignores, including live
  Metal coverage. The separate WebGPU/WebGL2 wasm32-unknown-unknown compilation
  retry also passes.
  Native-editor arena retirement remains explicitly excluded. No new pixel,
  hardware, Golden or Silver replay coverage is claimed.
- Validation at `1b887eb6`, integrated atop downstream `35e3b96ded`:
  independent source-equivalence and separate Rust-integration reviews confirm
  the public header-visibility fix is already equivalent. No runtime source or
  tests changed. Source/manifest/pipeline checks pass 39; generated build files,
  corpus metadata and diff hygiene pass. No runtime execution, editor feature
  parity, private undo behavior or new hardware coverage is claimed.
- Validation at `c63a5ba0`, integrated atop downstream `d185c5d94d`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Integration review corrected parent identity
  for arguments authored before converter arena insertion; the attachment hook
  preserves core/container ownership and a supplemental native regression
  verifies argument-originated bind registration. All 22 translated upstream
  tests pass, including exact fixture import, clone bind identity and live
  view-model updates. Runtime units pass 399 (two existing ignores), including
  that attachment regression. Browser runtime/scripting compiles for
  wasm32-unknown-unknown. Source/manifest/pipeline checks pass 39; schema
  reproducibility passes with rustfmt supplied on the test PATH. Generated
  build files, corpus metadata, changed manual-file formatting and diff hygiene
  pass. Validation corrected two test-only temporary borrow lifetimes by
  retaining the same ordered binding handles; assertions are unchanged and
  both rereviews are clean. No native formatter dependency, tolerance changes,
  consumer migration, broad Golden/Silver replay or new hardware/browser-pixel
  coverage is included. Historical shader/reference provenance is retained.
- Validation at `536a4f7a`, integrated atop downstream `388727443d`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Full feature-enabled live Metal renderer units
  pass 728 (six existing ignores), including all four new upstream regressions
  and the retained native-paint texture assertion. WebGPU/WebGL2 compilation for
  wasm32-unknown-unknown passes; source/manifest/pipeline checks pass 39 and
  generated build files, changed-file formatting and diff hygiene pass.
  Validation corrected test-only identity-matrix API spelling and the scheduling
  test host: upstream serializing paint unchecked-casts a canvas image to its
  concrete serializing image type; Rust retains its checked boundary and uses
  the existing null factory only for that order-only test. Real recording,
  resource replay and exact canvas-order assertions remain unchanged. Both
  corrections were independently rereviewed; production and tolerances were
  not changed to fit tests. Historical shader/reference provenance is retained.
  No new browser pixels, Vulkan/Android hardware or broad runtime Golden/Silver
  replay is claimed.
- Validation at `4672b88b`, integrated atop downstream `4ce0085eab`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Review corrected forwarding through existing
  scripting/C API factories and the C++ source-oracle bridge. Validation corrected
  a manual test command-name table and source-proven contracted arithmetic in the
  translated GM grid positioning; production rendering and tolerances were not
  changed to fit the test. Full feature-enabled live Metal renderer units pass
  724 (six existing ignores), including exact RGBA comparisons for both updated
  gradienttransform and new gradienttypes against fresh approved-adapted C++
  captures. Pristine C++ captures and both adaptation identities are retained
  separately; pristine/adapted differences are not called literal pixel parity.
  API/SRIV units pass 49; mechanical units pass 143 and deferred units pass 184
  (three existing ignores). WebGPU/WebGL2 wasm32 and Apple C API with scripting
  compilation pass; the modified C++ bridge passes pinned-header syntax checking.
  Shader regeneration reproduces all 66 WGSL modules and 56 upstream headers;
  Vulkan SPIR-V and embedded GLSL are regenerated from the same source. Source,
  manifest and pipeline checks pass 39; generated owners/build files and diff
  hygiene pass. No broad runtime Golden/Silver replay, new browser pixels or
  Vulkan/Android hardware coverage is claimed.
- Validation at `10d08fb6`, integrated atop downstream `5ca6b63393`:
  independent source-equivalence and subsequent Rust-integration reviews and
  correction rereviews are clean. Review corrected C-string name collisions,
  persistent registry-backed Data, the native file-registration projection and
  Data creation after sandboxing. Runtime units pass 398 (two existing ignores);
  focused scripting view-model units pass 79 (one existing ignore), including
  native identity and late-initialization regressions. Source/manifest/pipeline
  checks pass 39; regenerated Silver metadata changes only current provenance
  and upstream test locations. Browser scripting compiles for
  wasm32-unknown-unknown. Generated build files, changed-file formatting and
  diff hygiene pass.
  No broad Golden/Silver replay or hardware/browser-pixel coverage is claimed;
  no assertions or tolerances were weakened.
- Validation at `9166e9b6`, integrated atop downstream `daae4170d9`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Validation corrected test-only paint/CommandPath dispatch; both reviews
  cleared that correction without changing geometry or assertions. Feature-enabled
  mechanical renderer tests pass 143, including both new hairline batch-count
  cases; portable renderer tests pass 245 with five existing ignores. The initial
  portable-only run excluded the new feature-gated cases and is not counted as
  their validation. WebGPU and WebGL2 compile for wasm32-unknown-unknown.
  Source/manifest/pipeline checks pass 39; generated files, manifest, changed-file
  formatting and diff hygiene pass. No fresh broad Golden/Silver, browser pixels
  or hardware matrix is claimed; no assertions or tolerances were weakened.
- Validation at `9a20396e`, integrated atop downstream `52b11c7b07`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Portable ORE units pass 94; renderer units pass 245 with five existing
  ignores, including the new missing-resource refusal and updated dynamic-offset
  regressions. Source/manifest/pipeline checks pass 39; generated build files,
  corpus manifest, changed-file formatting and diff hygiene pass. No assertions
  or tolerances were weakened. No fresh broad Golden/Silver, browser-pixel,
  hardware matrix or platform compilation coverage is claimed.
- Validation at `a9239045`, integrated atop downstream `a1043b58ea`:
  independent runtime and ORE/Luau source reviews, followed by a separate
  Rust-integration review, are clean. Source review corrected attribute-count
  admission to call lua_objlen before type checking, including oversized
  strings/buffers; the correction was rereviewed. All four new upstream runtime
  regressions pass within eight focused cases. Existing lifecycle/data-binding
  suites pass 47; runtime units pass 396 with two existing ignores; ORE units
  pass 174; scripting GPU/feature cases pass 25. Browser scripting/runtime
  wasm32-unknown-unknown compilation passes. Source/manifest/pipeline checks
  pass 39, and generated test registration, formatting and diff hygiene pass.
  The preexisting direct borrowed setter cannot reborrow its owner inside a
  callback; the released-handle and transaction-deferred paths preserve the
  reviewed reentrant behavior. No assertions/tolerances were weakened. No
  fresh broad Golden/Silver, device or browser-pixel coverage is claimed.
  The 49 guest-only public files are explicitly deferred, not ported.
- Validation at `20e3e886`, integrated atop downstream `01dc19cc29`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean after corrections and rereviews. The Rust text adapter now enumerates
  the pinned HarfBuzz configuration's selected cmap, not a union or fallback
  font: source review corrected record priority, wildcard/null-offset behavior,
  terminal segments and HB_NO_LEGACY nominal mapping. Integration review
  corrected truncated-array admission, unsigned Format10 arithmetic and
  duplicate-range allocation before validation. The same nominal lookup is
  used by HbFont and FileProbe; no native HarfBuzz dependency was introduced.
  All 12 supplemental coverage cases pass within 396 runtime unit passes
  (two existing ignores). Existing raw text, font binding, selection and text
  input suites pass 91 cases. Native editor/testing/tools and
  wasm32-unknown-unknown compilation pass. Source/manifest/pipeline checks pass
  39; generated files, formatting and diff hygiene pass. Upstream added no
  public tests in this commit. No assertions/tolerances were weakened and no
  fresh broad Golden/Silver, device or browser-pixel coverage is claimed.
- Validation at `1d33ee5d`, integrated atop downstream `79efdeff3c`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including rereviews of validation corrections. Source review restored
  creation-observer notification for freshly compiled prepared shader modules.
  Validation corrected two test descriptor constructors and a test helper's
  module path. A supplemental Metal test now asserts upstream's new immediate
  rejection and exact first-binding diagnostic instead of the superseded
  skip-invalid-entries behavior; production behavior was not changed to fit it.
  Metal units pass 172; renderer units pass 245 with five existing ignores,
  including all nine new upstream WebGPU-parity cases; scripting GPU cases pass
  six, including texture-view aspects. Vulkan and wasm32-unknown-unknown WebGPU
  and WebGL2 compilation pass. Source/manifest/pipeline checks rerun pass 39;
  generated files, changed-file formatting and diff hygiene pass. Recursive
  formatting additionally reports preexisting style differences in three
  untouched deferred test files, which are left unchanged. No tolerances were
  weakened. No fresh broad Golden/Silver, browser-pixel, Android or hardware
  matrix coverage is claimed; parked guest execution remains excluded.
- Validation at `5df52be2`, integrated atop downstream `42b4fdcd16`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Source review corrected the new test to instantiate once from the
  authored artboard, matching upstream, rather than clone an instance again.
  Seven frozen-pose and nine related lifecycle/callback cases pass; the callback
  test was rerun with its required tools feature after the initial gated run.
  Runtime units pass 384 with two existing ignores; wasm32-unknown-unknown
  compilation passes. Source/manifest/pipeline checks pass 42, with generated
  files and formatting clean. No assertions or tolerances changed. No fresh
  broad differential or device/browser-pixel coverage is claimed.
- Validation at `daf55385`, integrated atop downstream `269117ce9c`:
  independent source-accounting and subsequent Rust-integration reviews confirm
  the complete public behavioral delta is guest-Wasm-only and deferred under
  UNIV-3728. No runtime source or fixture bytes change. Source/manifest/pipeline
  tests pass 42; generated inventory/build files and diff hygiene are clean.
  Earlier runtime, editor and browser results below remain at their actual
  tested revisions; this metadata update does not claim fresh execution or
  hardware coverage.
- Validation at `ca8fad85`, integrated atop downstream `c101a4c286`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Existing text-selection, native text-input and raw text-input suites
  pass all 86 cases. Native editor/testing/tools and wasm32-unknown-unknown
  builds pass. Source/manifest/pipeline tests pass 42, with generated-file and
  formatting checks clean. Upstream added no new tests in this delta. No
  assertions or tolerances changed; no fresh broad differential, hardware,
  browser-pixel or performance result is claimed.
- Validation at `785bca18`, integrated atop downstream `0241be30e9`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. The new missing-drawable regression and existing draw-order case pass;
  runtime units pass 384 with two existing ignores. Browser
  wasm32-unknown-unknown compilation passes. Source/manifest/pipeline tests pass
  42 and generated-file/formatting checks pass. Assertions and tolerances are
  unchanged. No new broad differential, hardware or browser-pixel result is
  claimed; earlier broad results retain their exact revisions below.
- Validation at `10e21383`, integrated atop downstream `45842a8ec1`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Work-pool tests pass 16 in each of synchronous and threaded modes,
  including all four new upstream cases. Six async-host and eight image-decode
  integration tests pass; wasm32-unknown-unknown compilation passes. Source,
  manifest and pipeline checks pass 42, with generated-file and formatting
  checks clean. The manifest test initially selected another local upstream
  checkout and passed when rerun with the exact pinned RIVE_RUNTIME_DIR.
  No assertions or tolerances changed. Broad Golden/Silver results remain
  recorded at their actual earlier revision below; no new hardware or browser
  pixel coverage, private GC implementation, or parked guest execution is claimed.
- Validation at `40e52541`, integrated atop downstream `dbe4d0b403`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Both new regressions and six existing keyframe-binding cases pass;
  testing-enabled runtime units pass 384 with two existing ignores, including
  the collapsed-run abort verifier. Native editor/testing/tools and
  wasm32-unknown-unknown compilation pass. Source/manifest tests pass 34,
  rebased pipeline tests pass eight, and formatting/generated-file checks pass.
  Exact-revision C++ release references and provenance-bound Rust debug runners
  pass ordinary Golden 364 (1,182 stream, 1,174 side-channel segments) and
  scripted Golden 362 plus the two unchanged divergence signatures (1,176
  stream, 1,168 side-channel segments). Silver remains 85 byte-exact, 121
  existing-epsilon, eight recorded divergent, 17 unsupported and zero runtime
  pending. Assertions and tolerances are unchanged; the hook-test fixture now
  uses upstream's actual LayoutComponent interpolator-host type instead of Node.
  The preexisting editor-only keyed-object/property alternate-list and empty
  keyframe behavior remains outside this delta; compilation is not full editor
  parity. No new hardware/browser-pixel coverage or parked guest execution is
  claimed. The CI rebase preserves main's composed pipeline and updates its
  upstream pin in the new owning `.buildkite/lanes.json` file.
- Validation at `ac0c03ae`, integrated atop downstream `8ce94f1aa4`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including rereviews of released root-layout callbacks, exact lazy
  scripted initialization, converter borrow boundaries and complete SMI
  interpolator clone/init dispatch. All six new upstream interpolator cases
  and 12 related scripting cases pass with unchanged assertions. Default-feature
  runtime unit tests pass 384 with two existing ignores. Native editor/testing/tools
  and wasm32-unknown-unknown runtime compilation pass. Source/pipeline tests
  pass 12, manifest tests pass 27, and formatting/generated-file/diff checks pass.
  Both exact-revision C++ oracles and provenance-bound Rust release runners
  were rebuilt. Ordinary Golden passes 364 comparisons (1,182 stream and
  1,174 side-channel segments); scripted Golden passes 362 and verifies the
  two existing divergence signatures (1,176 stream and 1,168 side-channel
  segments). Silver retains 85 byte-exact, 121 existing-epsilon, eight recorded
  divergent and 17 unsupported cases, with none pending in the runtime lane.
  No tolerances changed. No new device or browser-pixel coverage is claimed;
  parked guest execution remains outside this validation.
- Validation at `be93c2ce`, integrated atop downstream `50c3a8dc9d`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All six direct frozen-pose cases and nine existing quiet-row,
  settled-layer, callback-mutation and state-machine action cases pass.
  Binary unit tests pass 45; runtime unit tests pass 390 with two existing
  ignores. Native editor/testing/tools and wasm32-unknown-unknown runtime
  compilation pass. Source/pipeline tests pass 12, manifest tests pass 27,
  and generated-file/diff checks pass.
  Both exact-revision C++ oracles, both provenance-bound Rust release runners
  and the comparator were rebuilt. Ordinary Golden passes 364 comparisons
  (1,182 stream and 1,174 side-channel segments); scripted Golden passes 362
  and verifies the two existing divergence signatures (1,176 stream and 1,168
  side-channel segments). Silver retains 85 byte-exact, 121 existing-epsilon,
  eight recorded divergent and 17 unsupported cases, with none pending.
  No assertions or tolerances changed. No new device or browser-pixel coverage
  is claimed; parked guest execution remains outside this validation.
- Validation at `2a043c31`: independent source-equivalence and subsequent
  Rust-integration/accounting reviews are clean. The seven-file public delta
  is guest-Wasm-only and remains deferred under UNIV-3728; no runtime source
  changes are imported. Source/pipeline tests pass 12, manifest tests pass 27,
  and generated-file/manifest/diff checks pass. Existing runtime and renderer
  results below retain their original source revisions; no fresh runtime,
  browser-pixel or hardware validation is claimed for this metadata update.
- Validation at `3ccf0d9d`, integrated atop downstream `0ffd5e5384`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Both canvas-backing regressions and the adapted WebGL2 replay sink
  execute in the combined Metal/WebGL2 renderer suite: 820 pass, six existing
  ignores, run serially for the shared Metal lifetime instrumentation.
  The initial parallel run exposed three interfering instrumentation tests
  and the existing image-paint test's transient Bazel fixture path. The path
  now follows the existing Bazel/Cargo lookup convention; both reviews cover
  this test-only fix, and the unchanged exact pixel assertion passes.
  Both ORE failure-note tests pass. Both browser renderer products compile
  for wasm32-unknown-unknown. Source/pipeline tests pass 12, manifest tests
  pass 27, and formatting/generated-file/diff checks pass. No assertions or
  tolerances changed. Broad runtime Golden/Silver evidence remains pinned to
  the preceding `1b9b7381` result below, not relabeled as a fresh run here.
  No browser-pixel or new Android hardware execution is claimed.
- Validation at `1b9b7381`, integrated atop downstream `bc67d2413b`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including shared raster invalidation across callbacks, unwind-safe
  self-transform restoration, and synchronous nested-layout attachment without
  reborrowing the active owner. All 29 public mesh cases plus two callback
  regression tests are live; the scoped renderer run passes 64 tests and seven
  runtime targets pass 45 tests. The renderer's test-only runtime dependency
  enables tools so the upstream editor-host callback test actually executes.
  Runtime and both browser renderer products compile for wasm32-unknown-unknown.
  Source/pipeline tests pass 12, manifest tests pass 27, and formatting,
  generated-file and diff checks pass.
  Ordinary Golden passes all 364 eligible comparisons with unchanged settings
  (1,182 stream segments and 1,174 side-channel segments). Scripted Golden
  passes 362 comparisons and verifies the two unchanged divergence signatures
  (1,176 stream segments and 1,168 side-channel segments). Both C++ runners
  were rebuilt at this exact upstream revision; both Rust release runners
  were rebuilt and provenance-verified from the frozen corrected source.
  Final Silver retains 85 byte-exact, 121 existing-epsilon, eight recorded
  divergent and 17 unsupported cases, with no pending runtime cases.
  No tolerance or assertion was weakened. No browser-pixel or new Android
  hardware coverage is claimed; guest execution remains parked.
- Validation at `3f67b2f8`, integrated atop downstream `de0cace647`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Both complete live skinned-rig regressions and all eight existing
  bone/weight unit tests pass. Source/pipeline tests pass 12, manifest tests
  pass 27, and formatting/generated-file/diff checks pass. This scoped shared
  runtime guard has no new platform path; no fresh browser compile, hardware
  execution or broad Golden/Silver run is claimed for this commit.
- Validation at `1e55b86a`, integrated atop downstream `53118749fe`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All seven scripting-artboard tests pass, including the exact upstream
  cyclic-input fixture and existing instancing/origin/opacity cases. Runtime
  compilation passes for wasm32-unknown-unknown. Source/pipeline tests pass 12,
  manifest tests pass 27, and formatting/generated-file checks pass. No fresh
  broad Golden/Silver run or device/browser-pixel execution is claimed for
  this scoped guard; the immediately preceding broad results remain below.
  Guest execution remains deferred, not validated by these Luau cases.
- Validation at `439ffe0c`, integrated atop downstream `a8acdb89de`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including binary secondary-import slot accounting and trait-dispatch
  corrections. All ten translated upstream regressions, 44 binary unit tests
  and 384 runtime unit tests pass (two existing runtime tests ignored).
  Runtime compilation passes for wasm32-unknown-unknown. Source/pipeline tests
  pass 12, manifest tests pass 27, and generated-file/correspondence checks pass.
  Ordinary Golden passes all 364 eligible cases under existing verification
  settings (1,182 stream segments and 1,174 side-channel segments).
  The obsolete `db_health_tracker` divergence is promoted after both reviews;
  its existing 0.0011 tolerance is unchanged. This establishes current
  tolerance-equivalence, not byte equality or causation by this upstream delta.
  Scripted Golden passes 362 comparisons and verifies the two unchanged
  divergence signatures (`collapse_data_binds`, `editor_scripted_vector_v7`),
  with 1,176 stream segments and 1,168 side-channel segments. Both C++ runners
  were rebuilt from this exact upstream revision and Rust runner provenance
  verified against the frozen source.
  Silver executes 214 of 231 cases: 85 byte-exact, 121 within existing epsilon,
  eight existing recorded divergences and 17 unsupported cases; none pending.
  The eighth divergence was already recorded on main before this sync, not
  introduced or waived here. Runner publication now atomically replaces
  read-only Bazel outputs without changing their modes or provenance checks;
  both reviews and all 16 provenance harness tests pass. No browser-pixel or
  new hardware execution is claimed. Guest execution remains parked.
- Validation at `67142392`, integrated atop downstream `d425a708c4`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including exact source-count assertion corrections. Both browser
  products and their test targets compile for wasm32-unknown-unknown. The
  feature-enabled renderer suite passes 495 tests (six ignored), all seven
  Luau target-exposure tests pass, and two ORE failure-note tests pass.
  Validation discovered that Bazel unit-test targets did not inherit wrapped
  crate features; the emitter now supplies them explicitly for libraries and
  binaries. That blocking build fix received both reviews; all 28 Python
  generator/frontend tests and generated-file checks pass. Initial runs that
  silently excluded feature-gated tests are not counted as coverage.
  Source/pipeline tests pass 12 and manifest tests pass 27. No browser pixel,
  new hardware, native-GL product or broad Golden/Silver execution is claimed.
  Shader artifact provenance is unchanged; guest target gating remains parked.
- Validation at `acbb030b`, integrated atop downstream `9c64135638`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Both browser products and their test targets compile for
  wasm32-unknown-unknown using the current Bazel frontend. Source/pipeline tests
  pass 12 and corpus manifest tests pass 27; diff checks pass. The upstream
  change has no new public regression test. No browser pixel execution, Dawn
  native execution, new device coverage or broad runtime Golden/Silver run is
  claimed. Shader bytes and their historical provenance are unchanged.
- Validation at `6f3510dc`, integrated atop downstream `92e8c5776f`:
  independent source-equivalence accounting and subsequent Rust-integration
  reviews are clean. Source/pipeline tests pass 12 and manifest tests pass 27.
  The complete public behavior change belongs to parked guest execution; no
  runtime code, dependencies, shader bytes or historical provenance changed.
  Current source/oracle pins and manifest agree. The f40 validation below remains
  applicable evidence for unchanged runtime owners, not new guest coverage.
- Validation at `f40c9dfe`, integrated atop downstream `a49b777f7f`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including correction and fixture rereviews, are clean. Both browser products
  compile for wasm32-unknown-unknown. All 566 GL/Vulkan unit tests pass (11
  ignored); 42 shader-asset tests, two native shader lifecycle regressions,
  two sampler tests and nine signature cases under each of production-key tools
  and sample-key tools configurations pass. Four instrumented Metal resource
  tests pass. The final required-live Metal suite passes 673 tests (six ignored),
  including all four refreshed pixel cases and cross-shader-mode stability.
  Shader reproduction is exact (66 WGSL modules/56 headers); current
  SPIR-V, GLSL, Metal input inventories and source pins are regenerated.
  Source/pipeline tests pass 12 and manifest tests pass 27.
  Reviews corrected late-loaded shader owner retention, admission before cached
  artifact use, preservation after malformed envelopes, and payload identity
  after asynchronous compilation. Async replacement coverage exercises the
  actual publication predicate, not an end-to-end scheduler. The ORE visitor
  remains split across the existing runtime/artifact/scripting owners with
  approved authentication, UTF-8, entry-point, module-size and mapping limits.
  Validation restored two accidentally omitted approved Metal coverage guards
  and updated exact generated-name/count/hash assertions; no tolerance changed.
  Four older GM references were rebuilt from fresh f40 C++ with only the existing
  FMA/coverage adaptations: additive_blend, additive_advanced_blend,
  gradienttransform and image_paint each match corrected Rust at zero differing
  pixels. Pristine captures and historical provenance remain separate. No new
  browser-pixel, Vulkan/Android hardware or broad runtime Golden/Silver result
  is claimed; guest execution remains deferred under UNIV-3728.
- Validation at `71713c9f`, integrated atop downstream `cb39d02fc5`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All 27 native scroll-input and four velocity tests pass with the
  upstream wall-clock semantics, including both new edge-release cases.
  Runtime compilation passes for wasm32-unknown-unknown. Source/pipeline tests
  pass 12 and manifest tests pass 27. The in-range velocity assertion changes
  from zero to nonpositive exactly as upstream does; no locally invented
  tolerance or event timing was introduced. No new hardware, browser pixels
  or broad Golden/Silver run is claimed.
- Validation at `9f47f076`, integrated atop downstream `236a383cde`:
  independent source and subsequent Rust-integration reviews, including
  correction rereviews, are clean. Both browser products compile for
  wasm32-unknown-unknown; WebGPU test targets and WebGPU-plus-Wagyu compile.
  All 14 focused shared/GL dynamic-state tests pass. Native WebGPU test
  execution cannot link without the external Dawn library; no browser pixels
  or new hardware coverage is claimed. Compilation corrected the exact retained
  C++ snapshot length to 206,657 after byte comparison, and the test-only null
  backend's missing WebGPU trait cfg. The initial Wagyu check omitted its
  product root; the corrected feature combination passes. Source/pipeline tests
  pass 12, manifest tests pass 27, and inventory/structural correspondence are
  current. No shader source changed or new performance result is claimed.
- Validation at `dce52972`, integrated atop downstream `268b8015c0`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including the required CoreType trait-import correction. All 33 native
  text-input tests pass, including the exact new caret didChange regression;
  runtime compilation passes for wasm32-unknown-unknown. Source/pipeline tests
  pass 12 and manifest tests pass 27 with the pinned RIVE_RUNTIME_DIR (the first
  invocation used the stale default checkout and was corrected). Structural
  correspondence and generated inventory are current. No assertions changed;
  no new hardware, browser pixels or broad Golden/Silver execution is claimed.
- Validation at `c73593c3`, integrated atop downstream `15d3d3c5bf`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including correction rereviews, are clean. Required-live Metal passes 670
  tests (six ignored), including all 57 pixel GMs without shader instrumentation;
  four instrumented resource-shader tests pass. Three focused WebGL/Vulkan
  dispatch/specialization tests pass; Vulkan and both wasm32-unknown-unknown
  browser renderer products compile. Current shader reproduction is exact:
  66 WGSL modules/56 headers, 96 SPIR-V headers, current GLSL/Metal inputs and
  all 83 authority owners. Historical Metal combinations remain historical;
  production builds regenerate current combinations, not those archived inputs.
  Source/pipeline tests pass 12, manifest tests pass 27, inventory is current.
  Integration corrected a misplaced/defaulted bounds argument; test compilation
  corrected five unchanged-coordinate path calls. One exact expected resolve key
  changed from 9 to upstream's 10 after the hairline-key insertion; assertions
  and pixel tolerances remain unchanged. No new Vulkan/Android device coverage,
  browser pixels, broad runtime Golden/Silver or upstream-only retrofit GM
  execution is claimed; Metal still rejects unsupported depth/stencil modes.
- Follow-up correction at `c73593c3`: the translation added upstream's
  default `Draw::countSubpasses` pass-contents copy to `default_count_subpasses`,
  but `base_draw` replaced that slot with a separate single-pass counter that
  never filled pass 0. Image rect, image mesh and clip-reset draws therefore
  pushed and sorted with empty pass contents, and in depthStencil every nested
  clip's reset cleared the outer clip instead of intersecting it. Those draws
  now keep the default slot, as upstream's inherit the default; the separate
  counter is removed. A focused regression covers both clip-reset stencil
  choices and image rect and mesh counting. Three sites still choose
  differently from upstream: the depthStencil nested clip-update draw type and
  clockwiseAtomic nestedClipUpdateOnly read combined contents where upstream
  reads the pass's, and the non-depthStencil tessellation pushes pass the pass's
  contents where upstream passes combined. All three are equivalent: pass
  contents differ from combined only by opaquePaint on the depthStencil hairline
  pass, which is never a clip update. Work:
  [UNIV-4202](https://universe.basis.dev/issue/UNIV-4202).
- Validation at `c4d2c6cb`, integrated atop downstream `2045e6af85`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Tools-enabled artboard replacement tests pass 12, including both new
  lifetime regressions with real Luau execution. Runtime/change-window/audio/
  core suites pass 249 (two ignored), including all 18 change-window cases.
  That existing suite requires the testing observation feature; the initial
  command omitted it and was corrected without source/assertion changes.
  Runtime with tools compiles for wasm32-unknown-unknown. Source/pipeline tests
  pass 12, manifest tests pass 27, and inventory/structural correspondence are
  current. No new hardware, broad Golden/Silver or browser-pixel execution is
  claimed; private editor code absent from the public delta is not imported.
- Validation at `7fb595be`, integrated atop downstream `e738e272df`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including correction rereviews, are clean. Isolated current-build runtime
  suites pass 239 tests (two ignored), including both upstream hosted-audio
  regressions and exhaustive borrowed type queries for local video extensions.
  Schema/dispatch tests pass 22; callback metadata reproduces from published
  headers. Isolated parent and current builds both reproduce 19 existing
  schema-header failures and two codegen count failures; no assertions were
  weakened. Shared-target baseline artifacts initially proved unreliable, so
  current and parent validation were repeated in separate build directories.
  Three focused GL scanner/source-snapshot tests pass in the isolated build.
  Both wasm32-unknown-unknown browser renderers compile; source/pipeline tests
  pass 12, manifest tests pass 27, and inventory is current. Structural coverage
  is 1,066 mirrored/13 adapted/15 deferred owners, not behavioral proof.
  Integration corrections preserve explicit video ancestry and wire callback
  regeneration into make schema. Compilation found 32 residual calls to removed
  empty hooks; only those calls were removed. The new audio test needed inherited
  NestedArtboard access rather than an exact Rust downcast; swap/engine identity
  assertions remain intact. No new hardware, browser pixels, broad Golden/Silver
  execution or guest execution coverage is claimed. Whole-workspace formatting
  has unrelated existing differences; changed Rust files are formatted.
- Validation at `b2c28dde`, integrated atop downstream `c14997d541`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All ten scripting/layout tests pass, including the exact upstream file
  display-scale regression through the approved live-VM import seam. All 25
  WebGL2 ORE tests pass, including padded allocation/upload and binding-range
  cases; both wasm32-unknown-unknown browser renderers compile. Source/pipeline
  tests pass 12, manifest tests pass 27, and generated inventory is current (one
  producer line shifted by ten). Compilation caught an old source-byte count;
  both rereviews confirmed the new snapshot exactly matches upstream at 52,622
  bytes before correcting the expected length. No assertion/classification was
  weakened. No new browser pixels, native-device runs or broad Golden/Silver
  execution are claimed; guest execution remains parked.
- Validation at `05d690ee`, integrated atop downstream `9c5626ff47`:
  independent source-accounting and subsequent Rust-integration reviews are
  clean. Source/pipeline tests pass 12, manifest tests pass 27, and generated
  inventory/structural correspondence remain current. The public build-helper
  delta has no runtime/shader counterpart; no executable Rust behavior changed
  and no new runtime, guest or hardware validation is claimed.
- Validation at `74dda58f`, integrated atop downstream `a5def1244b`:
  independent source-accounting and subsequent Rust-integration reviews are
  clean. The guest-only public delta remains deferred. Source/pipeline tests
  pass 12, manifest tests pass 27, generated inventory is current, and structural
  correspondence remains 1,065 mirrored/13 adapted/15 deferred owners. Only
  current pins/accounting changed; no runtime, shader, fixture or assertion
  changes, and no new execution coverage is claimed.
- Validation at `625454e3`, integrated atop downstream `5199217ddc`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including correction rereviews. Required-live Metal passes 668 tests
  (six ignored) without shader instrumentation; all 57 pixel GMs pass exactly.
  Instrumented resource-shader tests pass four, WebGL2 dynamic depth/stencil
  command coverage passes, and Vulkan plus both wasm32-unknown-unknown browser
  renderers compile. Shader reproduction is exact (66 WGSL modules/56 upstream
  headers); all shader-authority owners, 12 source/pipeline tests, 27 manifest
  tests and generated inventory pass. Validation caught stale test references,
  expected preload keys and current shader metadata; these were corrected from
  upstream and generated bytes without changing assertions or historical
  captures. Four shader-instrumented pixel failures reproduce the previously
  recorded exact differences and all pass without instrumentation; tolerances
  remain unchanged. Browser pixel execution, Vulkan/Android hardware, broad
  runtime Golden/Silver and new C++ pixel captures were not run. Metal still
  rejects depth/stencil/inkbleed modes. Guest execution remains deferred.
- Validation at `0dd067f1`, integrated atop downstream `cbf0fba095`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Runtime/text/input/scroll/focus suites pass 323 (two ignored), including
  the new authored single-line/multiline viewport regression. Compilation found
  two test-only borrowed-matrix dereferences, corrected without changing math.
  The older hidden-flag test clicked the text center at y=108.896484 outside its
  50pt viewport. Its setup now clicks the viewport center; all three visibility
  assertions remain unchanged. Both corrections were rereviewed. Source/pipeline
  tests pass 12, manifest tests pass 27, and generated inventory is current after
  refreshing one producer line shifted by an upstream include. No classification
  or tolerance changed. No broad Golden/Silver, browser pixels or physical-device
  rerun is claimed; prior limitations and guest-execution deferral remain.
- Validation at `8427a425`, integrated atop downstream `a09ff19c7a`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. The public executable delta is confined to the deferred guest AOT
  cache owner on Nintendo NX. Source/pipeline tests pass 12, manifest tests
  pass 27 and generated inventory is current. Structural correspondence is
  1,065 mirrored/13 adapted/15 deferred owners, not semantic proof. No runtime,
  dependency, fixture or assertion changed; no guest or hardware run is claimed.
- Validation at `04104ce2`, integrated atop downstream `3856efcd95`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Rotation limits now compare within a turn centered on the configured
  limits, preserving upstream f32 grouping and max-before-min order. Runtime,
  rotation and IK-stacking tests pass 239 (two ignored), including all three
  upstream rotation cases and all eight programmatic rows. Source/pipeline
  tests pass 12, manifest tests pass 27 and generated inventory is current.
  No tolerance changed. No broad corpus/platform rerun is claimed for this
  bounded arithmetic change; prior limitations remain explicit.
- Validation at `8092a195`, integrated atop downstream `55ffba4e9b`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including correction rereviews, are clean. Translate animation pre-apply
  polling and continuation, current-file VM pending queries, foreign-file
  identity and recursive hosted-artboard checks. Rust decoder callbacks queue
  results for Lua-thread delivery, so the root also drains reachable foreign
  VM queues. Integration review caught cross-VM reordering; delivery sequences
  now preserve work-pool callback order instead of grouping callbacks by VM.
  All five upstream host cases and a supplemental real two-VM decode-order
  regression pass, both non-threaded and threaded (six cases in each lane).
  Runtime/affected suites pass 258 (two ignored); scripting/host suites pass
  378 (one ignored). WebGPU/WebGL2 wasm32 and Android arm64 Vulkan with
  scripting/scriptnet compile. Source/pipeline tests pass 12, manifest tests
  27 and generated inventory is current. No broad Golden/Silver/Metal or
  browser-pixel/physical Android rerun is claimed; prior limitations remain.
  Guest Wasm held-outcome registration remains deferred under UNIV-3728.
- Validation at `84a3719f`, integrated atop downstream `cf8edf978d`:
  independent source-scope and subsequent Rust-integration reviews are clean.
  All public executable changes are deferred guest AOT boot/cache/Windows and
  tier-transplant removal work. Remove only the deleted source owner's deferred
  inventory entry; no Rust implementation existed to delete. Source/pipeline
  tests pass 12, manifest tests pass 27, generated inventory is current, and
  structural coverage is 1,065 mirrored/13 adapted/15 deferred owners. No Rust
  behavior, dependency or assertion changed; no guest AOT or broad runtime
  validation is claimed. Historical evidence and UNIV-3728 remain intact.
- Validation at `f9d7e648`, integrated atop downstream `1ab231ff46`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. Eight existing live Lua image-decoder tests pass; source/pipeline tests
  pass 12, manifest tests pass 27, and generated inventory is current. Structural
  coverage is 1,065 mirrored/13 adapted owners. The extracted browser decoder
  maps to the approved Rust codec/work-pool owner: asynchronous settlement,
  cancellation, canonical error and premultiplication already match the shared
  contract. This is not a claim of browser-native codec equivalence; JS startup
  exception and malloc guards concern absent Emscripten glue. Guest browser VM,
  typed calls, ABI/IDL/generated bridges, deferred File startup and guest tests
  remain parked under UNIV-3728. No browser pixels, broad corpus or platform
  rerun is claimed for this metadata-only change; earlier limitations remain.
- Validation at `a68b6dab`, integrated atop downstream `1e53c494d0`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean. All three script-module asset tests pass, source/pipeline tests pass
  12, manifest tests pass 27, and the generated inventory is current. The shared
  AnimaScript module-name constant is translated; this does not enable guest
  execution. Guest VM adoption, registration, mixed-backend Luau guards, WAMR
  and guest-browser bindings remain deferred under UNIV-3728. Existing omitted
  VM-presence assertions were not removed or weakened by this update. Private
  editor implementation is absent from the public delta and is not claimed
  imported. Active pins advance together; historical evidence is unchanged.
  No broad corpus/platform rerun is claimed for this constant-only change.
- Validation at `ae4e1776`, integrated atop downstream `0af32351b6`:
  independent source-equivalence and subsequent Rust-integration reviews are
  clean, including missing VM-handle forwarding and receiver-coercion correction
  rereviews. Runtime/work-pool/watermark/network tests pass 265 (two ignored);
  scripting passes 375 (one ignored), including all three new host-loop cases.
  Both new owner-pool cases pass in the non-threaded lane. Threaded validation
  initially passed 14 and failed one new cancelled-owner case; the unchanged
  serial repeat passes all 15. Both upstream and Rust have a preexisting
  false-idle window between queue removal and in-flight publication, and this
  upstream test drains through that older global query. This is a source-based
  explanation, not a traced interleaving or reproduced C++ failure. No assertion
  or algorithm was changed. Threaded host tests with scriptnet disabled pass
  all three. WebGPU/WebGL2 wasm32 and Android arm64 Vulkan with scripting/scriptnet
  compile. Source/pipeline tests pass 12, manifest tests 27 and generated inventory
  is current; structural correspondence remains 1,065 mirrored/12 adapted owners.
  No new full Golden/Silver/Metal, browser-pixel or physical Android run is claimed.
  Guest Wasm execution remains deferred; broader historical failures stay below.
- Validation at `de3e8609`, integrated atop downstream `f465a95aba`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including focused correction rereviews, are clean. Source review corrected
  the ScrollEvent phase constructor to match upstream's C-string boundary;
  a supplemental NUL/invalid-UTF8-suffix regression passes. The first scripting
  run exposed three constructor fixtures missing upstream's Rive-global
  registration; the setup was corrected through the existing public API and
  rereviewed without changing production behavior or assertions.
  Final runtime/input suites pass 290 (two ignored), scripting suites pass 398
  (one ignored), including all six new upstream scripting cases and the new
  runtime wheel-cancellation regression. WebGPU/WebGL2 wasm32-unknown-unknown,
  Android arm64 Vulkan with scripting/scriptnet, and no-default runtime compile.
  Source/pipeline tests pass 12 and manifest tests 27; the generated manifest
  is current. The initial manifest invocation used its default checkout rather
  than the pinned upstream; the explicit pinned-directory rerun passes.
  Structural correspondence remains 1,065 mirrored owners, not semantic proof.
  No renderer code changed. Broad Golden/Silver/Metal results at `7aa93402`
  below were not rerun for this bounded input change; existing binary/probe
  failures remain explicit. No browser pixels or physical Android execution
  is claimed. Wasm guest execution and dependent bindings remain deferred.
- Validation at `7aa93402`, integrated atop downstream `b8a3b3574a`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including focused correction rereviews, are clean. Native runtime/affected
  tests pass 656 (three ignored), with the unchanged converter Silver failure;
  all 15 new change-window tests pass. Scripting passes 376 (one ignored), and
  render API/SRIV/C ABI tests pass 93. Native Metal passes 666 (six ignored)
  on Apple M5 Max, macOS 26.6.2, Xcode 27.0, using exact pixel comparisons and
  the C++ capture's non-instrumented shader execution. The instrumented run
  exhibited four small pixel differences which disappear under matching
  execution settings; no pixel tolerance changed. Fresh pinned C++ captures
  cover gradient transforms, expanded empty paths and stroke-position shapes.
  All four instrumented Metal resource-shader integration tests pass after
  refreshing only the regenerated fixture identity and retaining old captures.
  WebGPU/WebGL2 on wasm32-unknown-unknown, Android arm64 Vulkan with
  scripting/scriptnet, and no-default runtime compile.
  Fresh pinned ordinary/scripted Golden comparisons each cover 364 entries
  with zero failures and the existing one/two recorded divergences.
  All 231 runtime Silver cases ran: 85 byte-exact, 121 within unchanged epsilon,
  seven recorded divergences, 17 unsupported cases, and only
  `data_converter_to_number` (frame 41, op 2120, expected 1850 fields, got 1443).
  Broader binary/probe suites are not all green: binary tests pass 169 with
  15 failures (older C++ source-shape assertions, malformed contents, overwide
  LEB128 and artboard referencers); old-pin/source-baseline checks reproduce
  their causes. The new requestSourceWrite source assertion is translated,
  not removed. Three native probe targets pass 112 with 12 random-function
  failures: the unchanged release C++ oracle inlines upstream rand() and
  bypasses its old out-of-line deterministic hook. Those cases provide no
  valid random differential evidence and are not counted as parity passes.
  Source/pipeline checks pass 12, manifest tests 27, boundary helper tests 168;
  the broad boundary scan retains the same 49 diagnostics as untouched
  `b8a3b3574a`. Source correspondence covers 1,065 mirrored owners structurally,
  not semantically. Shader authority (81 owners/163 artifacts), 98 SPIR-V
  hashes, and C ABI v4 layout (63 types/447 fields) pass. Historical v3 and
  prior capture provenance are retained. No browser-pixel or physical Android
  execution is claimed; guest execution remains deferred.
- Validation at `160085c6`, integrated atop downstream `97e52f7db0`:
  independent source-equivalence and subsequent Rust-integration reviews,
  including correction rereviews, are clean. The new contribution matrix
  exposed two existing layout differences: definite item sizes were capped by
  fixed tracks, and unresolved percentage minima were omitted from intrinsic
  sizing. Both now follow the pinned public Yoga source, with unchanged test
  expectations. Native runtime/affected integration tests pass 612 (three
  ignored), with only the recorded converter Silver assertion failing;
  scripting passes 377 (one ignored). WebGPU/WebGL2 on wasm32-unknown-unknown,
  Android arm64 Vulkan with scripting/scriptnet, and no-default runtime compile.
  Fresh ordinary/scripted Golden comparisons each cover 364 entries with zero
  failures and the existing one/two divergences; the scripted wrapper was rebuilt
  and its comparison repeated after main's audio-ABI fix. All 231 runtime Silver
  cases ran: 85 byte-exact, 121 within unchanged epsilon, seven recorded
  divergences, 17 unsupported skips, and only `data_converter_to_number`
  (frame 41, op 2120, expected 1850 path fields, got 1443). Structural/helper
  checks pass 16 and manifest tests 27; 1,065 mirrored owners are structural
  coverage, not semantic proof. Boundary-checker unit tests pass 168; its broad
  scan has the same 49 diagnostics on an untouched `97e52f7db0` snapshot and
  this branch, identical after normalizing the root path. No browser-pixel or
  physical-device execution is claimed; deferred guest execution is not a pass.
- Validation at `dc75beed`: independent source-equivalence and separate
  Rust-integration reviews, including correction rereviews, are clean. Native
  runtime/affected integration tests pass 316 (two ignored); scripting passes
  377 (one ignored). wasm32-unknown-unknown WebGPU/WebGL2, Android arm64 Vulkan
  with scripting/scriptnet, and runtime without default features compile.
  Fresh pinned ordinary/scripted Golden comparisons each cover 364 entries
  with zero failures and the existing one/two recorded divergences. All 231
  runtime Silver entries were checked individually: 85 byte-exact, 121 within
  unchanged epsilon, seven unchanged divergences, 17 unsupported skips, and
  only the recorded `data_converter_to_number` failure (frame 41, op 2120,
  expected 1850 path fields, got 1443). No assertions or tolerances changed.
  Source/pipeline checks pass 12, manifest tests 27, and public Yoga-ref resolver
  tests four. Structural coverage is 1,065 mirrored owners, not semantic proof.
  No browser-pixel or physical-device execution is claimed.
- Validation at `7c9291ae`, atop downstream `01baaeafb6`: independent source
  and separate Rust-integration reviews are clean after adding the omitted
  enum `values()` attribution and replacing an allocating VM identity probe
  with a non-executing opaque-token accessor. Runtime library tests pass 203
  (two existing ignored); scripting with tools/scriptnet passes 377 (one
  existing ignored). The non-tools runtime compiles, as do browser WebGPU and
  WebGL2 with scripting/scriptnet/tools on wasm32-unknown-unknown. Source/pipeline
  tests pass 12, with 1,064 mirrored owners (structural coverage only); manifest
  tests pass 27. No new full-corpus or physical-device coverage is claimed for
  this tools-only delta; deferred guest execution is not a validation pass.
- Validation at `cd04cd33`, atop downstream `0b8a891205`: source-equivalence
  and separate Rust-integration reviews are clean after correcting deferred
  resource destruction order and rejecting malformed Rust UBO descriptor
  counts before range recording. ORE passes 195 tests with live Metal required;
  deferred ORE passes 33, including both translated upstream regressions;
  scripting passes 376 (one existing ignored test). The added Lua test fixture
  was corrected to match its zero-color pipeline and now asserts exact deferred
  command ordering; runtime validation was not weakened. Browser
  wasm32-unknown-unknown WebGPU/WebGL2 and Android arm64 Vulkan compile with
  scripting/scriptnet. Source/pipeline tests pass 12 and manifest tests pass 27.
  No new full-corpus, browser-pixel or physical Android execution claim is made;
  the prior Silver failure and guest-execution deferral remain separate.
- Validation at `1c54a97c`: independent source-equivalence and subsequent
  Rust-integration reviews are clean. Scripting library tests pass 374 (one
  existing ignored test), including all three new upstream mesh regressions;
  four renderer-binding tests, one gated upstream renderer test and three
  deferred-instance tests pass. wasm32-unknown-unknown with WebGPU, WebGL2,
  scripting and scriptnet compiles. Source/pipeline tests pass 12 and manifest
  tests pass 27. The preceding `81a73b81` vector suite passes 14 (one existing
  ignored benchmark). No new full-corpus or hardware execution claim is made;
  the broader sweep and its known Silver failure remain recorded below.
- Validation at `6cd5d108`, integrated atop downstream `0d5fd220` without
  discarding its paint/Text callback fixes: both independent review passes and
  correction rereviews are clean. Native validation passes 256 tests (two
  ignored), including five real Apple loopback transport tests and the new
  downstream paint/Text regressions; scripting passes 371 (one ignored), and
  binary-reader/import tests pass 41. The runtime also compiles without default
  features. wasm32-unknown-unknown WebGPU/WebGL2/scripting/scriptnet and Android
  arm64 Vulkan/scripting/scriptnet compilation pass; these are not browser-pixel
  or physical Android execution claims. Fresh pinned ordinary and scripted
  Golden runs each cover 364 entries with zero failures and the existing one
  ordinary/two scripted recorded divergences. All 231 runtime Silver cases were
  executed individually after the bulk runner stopped at the known converter
  failure: 85 byte-exact, 121 within unchanged epsilon, seven recorded
  divergences, 17 unsupported skips, and the unchanged `data_converter_to_number`
  failure (frame 41, op 2120, expected 1850 path fields, got 1443). No assertion
  or tolerance was relaxed. Manifest tests pass 27; source/pipeline checks pass
  12, with 1,063 mirrored and 12 adapted owners (structural coverage only).
  Networking remains opt-in with no provider installed; Wasm guest execution
  remains deferred under UNIV-3728.
- `3330baec` was integrated atop downstream `ca4c9ebb` without discarding its
  drawable-dispatch preparation. Focused source-level integration review is
  clean; the combined tree passes 109 runtime library tests (two ignored) and
  all six newly translated tests. The full Golden/Silver sweeps recorded in
  its row ran before this downstream-only rebase; they are not claimed as a
  second full sweep of the combined tree.
- Catch-up baseline validation at `bbf3c4d8`: 832 renderer tests pass (11 ignored), with
  live Metal required and the documented serial test configuration; 330 scripting
  tests pass (one ignored), and 11 Apple host tests pass. wasm32-unknown-unknown
  WebGPU/WebGL2/scripting and Android arm64 Vulkan/scripting compilation pass.
  Runtime library tests have 90 passes, two ignored, and the previously reproduced
  native-hydration failure [UNIV-3788](https://universe.basis.dev/issue/UNIV-3788).
  Fresh pinned ordinary and scripted C++/Rust Golden runs each cover 364 entries:
  ordinary retains the three [UNIV-3793](https://universe.basis.dev/issue/UNIV-3793)
  failures; scripted also retains the converter teardown abort
  [UNIV-3795](https://universe.basis.dev/issue/UNIV-3795). Normalized failure and
  summary lines match the `75a22f94` runs. Runtime Silver covers 231 cases:
  92 byte-exact, 111 within existing epsilon, seven recorded divergences,
  20 unsupported and the same `stateful_multi_property` failure (frame 16,
  operation 1061, expected `drawPath`, got `makeRenderPath`). All case exit codes
  match the prior sweep. `fill_trim_path`, `layout_paint` and
  `stacked_path_effects` now reproducibly pass within existing epsilon rather
  than byte-exactly; this is consistent with intervening upstream `43aa1025`
  trim arithmetic and unchanged stored fixtures, not a proven causal attribution.
  No assertion or tolerance was relaxed. Manifest tests pass 26 and structural
  correspondence tests seven; the source check counts 1,055 mirrored and ten
  adapted owners (structural coverage, not proof of behavior). Android hardware,
  browser pixels and native Dawn execution are not claimed by compilation.
- Follow-up correctness audit at the same upstream pin corrects nonrecursive
  clipping dirt, semantic registration/queued dispatch, and native semantic
  bounds. Golden loading now uses File's public instantiation path and mirrors
  C++ destruction order; the hydration regression models failed-init cleanup
  without prematurely disposing tracked properties. Each correction received
  source-equivalence and separate Rust-integration review. Native/C API semantic
  bounds intentionally exclude rendered clipping and root frame-origin
  projection; those remain separate host geometry helpers, not native policy.
  Three stale Silver exclusions (`data_viz_demo`, `db_health_tracker`, and
  `echo_show_demo`) now replay literal upstream actions and pass unchanged
  epsilon; the exact-case ratchet increases by three. In particular, the data-viz
  producer runs 12 frames after mutation, not the dormant recipe's 30.
  Final code at `89f28829a3`: 92 runtime library tests pass (two ignored), all
  66 affected native/host semantic tests pass, and browser wasm32 WebGPU/WebGL2
  with scripting compiles. Ordinary and scripted Golden each process 364 entries
  without failures, retaining one and two recorded divergences respectively.
  Runtime Silver: 92 byte-exact, 115 within unchanged epsilon, seven recorded
  divergences, 17 unsupported skips, zero failures. Generator tests pass 27 and
  structural correspondence tests seven; 1,055 mirrored owners remain covered.
  Remaining exclusions are not approved adaptations merely because they are
  recorded: 17 runtime Silver skips, seven recorded divergences, 41 pending
  scripted (Luau, not parked Wasm) producers, and three unknown-provenance cases
  still need adjudication. The grid-participant exclusion now executes but
  differs at frame 0/op 169, so it was not promoted. Structural file pairing
  and a fully accounted commit sequence do not prove these behaviors correct.
- Performance baseline after these corrections is diagnostic, not a speed-parity
  claim. The existing release benchmark runs 25 fixtures for 100 frames at 60 Hz,
  five samples, median aggregation, in both runner orders. Longer-frame output
  comparison matches 23/25; `zombie_skins` retains a transform mismatch and
  `virtualize_blendmode` a color mismatch. Across those 23 matching fixtures,
  summed Rust/C++ median hot-loop times are 6.20x (C++ first) and 6.05x (Rust first).
  All 25 aggregate to 6.58x and 6.67x, but the two mismatching fixtures cannot
  support a performance-parity claim. These measurements exclude import,
  actual GPU rendering and audio-device work. Our builds/validation were stopped,
  but unrelated host load remained; a quiet-host rerun is required before
  publishing performance conclusions. Existing ceilings are historical
  regression limits, not an "as fast as upstream" criterion.
  Both unchanged ceiling checks pass 19/25 and fail six: the two script-dependency
  fixtures, image scripting, `audio_script`, Spotify and multi-listeners. The
  nominal audio fixture does not measure audio-device execution.
  Raw [C++-first](evidence/runtime-closeout-2026-09-30/post-harness-fix-cpp-first.json)
  and [Rust-first](evidence/runtime-closeout-2026-09-30/post-harness-fix-rust-first.json)
  reports retain source/toolchain/binary identity; the adjacent correctness log
  records the two mismatches. Pre-harness-fix reports are historical diagnostics,
  not a valid optimization baseline. Next, resolve the two longer-frame output
  discrepancies and profile measured Rust overhead (starting with data-viz,
  Spotify and car-widget fixtures); keep upstream algorithms unchanged and
  optimize only proven translation overhead or explicit Rust adaptation seams.
- Longer-frame follow-up at the same pin restores the production C++ contracted
  arithmetic at the proven `KeyFrameDouble`, `Mat2D::decompose`, and IK sites;
  this is not a blanket floating-point policy change. The 100-frame
  `virtualize_blendmode` comparison now passes, bringing the longer-frame sweep
  to 24/25 matching fixtures. `zombie_skins` still differs in
  its accumulated replay at 0.9 seconds; a single jump to that time passes, so
  this remains unresolved rather than a tolerance waiver. Source-equivalence
  and separate Rust-integration reviews covered each correction. The unchanged
  one-iteration and 1,000-iteration upstream IK assertions pass; their test
  target needed only a stale trigger caller migrated to the existing
  callback-safe handle API. Focused runtime/math/interpolation/host tests pass
  183 with two existing ignores, the C API clip-budget recovery test passes,
  and scripted Golden processes 364 entries without failures, retaining two
  recorded divergences. WebGPU/WebGL2 with scripting compiles for
  `wasm32-unknown-unknown`. In the remaining zombie trace, the first four IK
  solves agree bit-for-bit; the next receives different bone-world positions
  while parent inverse and target still agree. Continue at animated bone/local
  pose composition before `solve2`, not by changing its cosine equations or
  loosening assertions.
  Profiling also identified a semantic subtree invalidation in
  `ClippingShape` absent from upstream. Removing it preserves the separate
  live host geometry checks and passes clipping/semantic/video tests. Its
  contribution was visible in the advancing Spotify profile; no post-change
  release timing or speedup is claimed yet. Handle upgrades and borrow checks
  seen in other profiles are not, by themselves, evidence of removable work.
  The initial candidate was not cleared for merge: full runtime Silver reported 85
  byte-exact, 116 epsilon matches, seven recorded divergences, 17 unsupported
  skips, and six failures. A controlled build reverting only the keyframe
  fallback to separate multiplication/addition restores all six:
  `artboard_list_overrides_horizontal`, `artboard_list_overrides_vertical`,
  `computed_values_test`, `data_converter_to_number`, `layout_paint`, and
  `transition_artboard_condition_test`. Upstream's unit-test build requests
  `-ffp-model=strict` and `--no_ffp_contract`, unlike the production archive
  used for Golden. The converter difference changes path field count, not
  merely a coordinate; numeric formatting may explain it, but that mechanism
  has not been traced directly. Validation therefore used the existing C++
  producers under identified arithmetic configurations, as described below. Do not
  downgrade these cases, relax tolerances, or regenerate stored outputs to
  conceal the disagreement. The diagnostic unfused build is not the candidate.
  Follow-up source review found that the Rust SRIV comparator accidentally
  required both vector coordinates to be bit-exact when either was negative
  zero. The correction preserves per-coordinate signed-zero identity and the
  existing nonfinite policy, while applying the unchanged upstream vector
  distance/tolerance to a finite neighboring coordinate. Both review passes
  are clean and all ten comparator tests pass. Full runtime Silver now reports
  85 byte-exact, 121 epsilon matches, seven recorded divergences, 17 unsupported
  skips, and only the unchanged converter failure. No manifest or tolerance
  changed. A live strict pinned C++ replay of the six original producer tests
  passes all 41 assertions against copied stored baselines. A second isolated
  build of those same producers changes only generated floating-point flags
  from strict to precise/contract-on (same debug optimization, source, and
  `TESTING` defines). Five cases still pass; C++ itself fails the stored
  converter assertion and emits a 1,338,671-byte stream instead of 1,338,818.
  Rust matches that emitted stream within the unchanged comparator tolerance.
  Thus the remaining stored-Silver failure is a demonstrated arithmetic-build
  mismatch, not evidence of a Rust-only converter regression. This controlled
  debug-build result is not a claim that it is the production archive.
  The checked-in strict fixture and its failing classification remain intact;
  only an isolated temporary copy was used for the matched-build comparison.
  For reproduction: upstream `bbf3c4d8`, Apple Clang 21.0.0; strict binary SHA256
  `f8ea004a6867ada79f2e50238d4c0013a65157263aa0a71bf53e43ec2f653b0b`,
  contract-on binary `d915b362f9941d451a440acf1538953710dda25379ca046a99e9e3ee4c674693`,
  emitted converter SRIV `eb895fd2c33cf1bb55fe3022ef61595bfe7535c7aa7fb8ec3d995cb80f1c4cba`.
  The subsequent zombie trace narrowed further: all preceding numeric property
  writes agree; the local bone pose already differs before solve 4. That led
  to IK pose-history detection (`holdsOurSolve`) and local rebuild, rather
  than assuming the later solver algebra caused the input drift.
- Zombie follow-up restores a missing virtual dispatch rather than changing
  IK arithmetic. C++ `Constraint::onDirty` reaches
  `IKConstraint::markConstraintDirty`, which invalidates the tip and then the
  ancestor FK bones. The Rust handle path invoked only the base operation,
  leaving an ancestor's previous solved local pose in place. Disabling only
  the C++ ancestor loop in the debugger reproduced Rust's pose-history
  decisions and matrices. The correction restores ordinary and active
  Shape/Layout dirt dispatch, concrete IK callbacks, and borrow-released
  strength/inversion setters. Source-equivalence and independent Rust
  integration reviews are clean; ordinary constraints gain no heap allocation.
  All 25 fixtures now match the unchanged 100-frame C++ comparison. Scripted
  Golden processes 364 entries with zero failures and its two existing recorded
  divergences. Runtime library tests pass 93 with two existing ignores; four
  IK stacking/dispatch tests and the unchanged one-iteration and 1,000-iteration
  IK regressions pass. WebGPU/WebGL2 with scripting compiles for
  `wasm32-unknown-unknown`. No arithmetic, tolerance, manifest, or fixture changed.
  The full 231-case runtime Silver sweep is unchanged: 85 byte-exact, 121
  epsilon matches, seven recorded divergences, 17 unsupported skips, and the
  one documented strict-baseline converter mismatch. The converter continues
  to pass against matched contracted C++ output. There are no new failures;
  25/25 longer-frame agreement is not a whole-runtime or speed-parity claim.
  Resolved 2026-10-09 at pin `6f3510dc`: the default lane mirrors production
  contraction by design, so `data_converter_to_number` is now classified with
  the other producer-build-mode (`--no_ffp_contract`) divergences rather than
  left as a red exact entry. The validator pins its first difference (frame
  41, op 2120, addRawPath 1850 vs 1443 fields), and the strict lane must
  still replay the stored silver exactly: `make silver-corpus-strict-fp`, run
  as its own step in the CI silver lane. No silver, tolerance, or arithmetic
  changed.
- The 2026-09-28 audit accounts for all 58 upstream commits in
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5..5892bb05be7ae966b751625b4ee12239e6860dc1`.
  The structural source-correspondence check covers 1,032 owners; it does not
  prove behavioral parity. The three existing ignored serialized-output
  comparisons remain expected-red under
  [UNIV-3015](https://universe.basis.dev/issue/UNIV-3015); they were not rerun
  during this audit.
- Runtime Silver validation at `8e8492f8` also found `animated_clipping-layout`
  failing at frame 6, operation 114 (expected `drawPath`, got `rewind`). A clean
  `c22b630257` baseline reproduces the same failure; this is not an 8e regression.
  Nine targeted layout Silver cases passed then. At `955d6a05`, this clipping
  case now compares byte-exact; two grid/stack cases also become exact and their
  checks are strengthened accordingly. The full 231-case runtime Silver sweep
  retains one failure, `stateful_multi_property`
  ([UNIV-3751](https://universe.basis.dev/issue/UNIV-3751)); its first differing
  command changes with the new upstream baseline. No tolerance was relaxed.
- The user authorized manual, one-commit-at-a-time work on 2026-08-31.
  The preceding accounted change, upstream's Rive 7.3 layout translation,
  anchor, constraint, scroll virtualization-buffer, and Luau 0.733 update, was ported
  mechanically and reviewed as one source commit. Upstream:
  `2bbd8820878f38398b6ea6e722cb88310588c8e8`. Work:
  [UNIV-1880](https://universe.basis.dev/issue/UNIV-1880).

| Upstream SHA | Applicable translated slices | Work |
| --- | --- | --- |
| `ed3303569554da2a13ef468c33bf405dd0f49dbc` | Correct deterministic scroll microseconds and translate controlled-clock regressions; retain monotonic engine clock and approved explicit pointer-time adaptation. No CSS layout semantics change. | — |
| `cf9af36b5ef0cd713c013ed55f8ebe3ef8564518` | Private .rive_head metadata only (a972783b to 389f4a56); no public executable delta. Active pins advance; historical and shader provenance retained. | — |
| `c7204edefed9162b66bc847646717d1f3a7065dd` | Translate root-only My ViewModel discrimination for listener assignments and shared source-path length for transition comparands; port property-sourced and nested-root assignment regressions. Absent private Dart/editor code not claimed. | — |
| `4839c35a2c641b1f128a1b04f063d1a0873da7e1` | Translate undelivered answer count/byte budgets, in-flight reservations, canceled-response cleanup, unsent rate-start refunds, queued-dispatch cancellation and synchronous batching; port all four public fetch regressions. | — |
| `f37216442e55be2e018ffc61ecf333e219b1c3f4` | Exclude WAMR AOT wide-call argument/result staging patch and dependent tail-call changes; no ordinary runtime/Luau/browser-renderer delta. Active pins advance; real artifact provenance retained. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `b27b97fdad7c7922b31163e8daa1d48ea3d161b9` | Translate keyed font-asset resolution, imported file origin retained through style/paint clones, owner dirtying and public keyed-font regression. Preserve approved CSS/native-text boundaries; absent private editor/export code is not claimed. | — |
| `ec4208228304441a78a092de4108cebbe19c5501` | Private .rive_head reference only (cae4dad5 to a972783b); no public executable delta. Active pins advance; shader provenance retained. | — |
| `62949d42cdc06d3c62ea0cabe649551cd7e5c348` | Translate owned per-frame allocator recycling through the backend virtual hook, default reset, empty-result contract, public initial-size constant and helper forwarding. No shader delta. | — |
| `ce78b66fee30fb483de0e4d0231107c7fd2cb744` | Private .rive_head reference only (412408d8 to cae4dad5); no public executable delta. Active pins advance; actual shader provenance retained. | — |
| `2642c18a51bc9fbabb3fa490d8afcc2450571a5f` | Translate Solo provider-reach recollection optimization for non-CSS layouts. Preserve CSS anonymous-content recollection and all existing collapse/notification behavior. Shader products retain real provenance. | — |
| `eeaf140769620842d24cf85ef1dab57ba355be78` | Private .rive_head reference only (d4406516 to 412408d8); no public executable delta or new hardware coverage. Active source/oracle and Silver pins advance; actual shader provenance retained. | — |
| `d96b8979439397d5d79ec5ea08357d738c5e801c` | Exclude guest WAMR/compiled-lane changes and job/worker clock imports. Shared Region registry caching has no counterpart in retained Luau handles; timed-mutex spin optimization remains unimported at the approved Rust lock boundary. No executable delta; shader artifact provenance unchanged. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `0f4fa96fc5e9fe4bc41a398bff8275e59cfb86e2` | Private .rive_head reference only (e6c7fdb0 to d4406516); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `bda5b785f0ecc889848498a44ee115b2b679c595` | The complete four-file public delta sets the guest runner-owned browser AudioContext audio-session type from auto to playback, with generated copy/docs/tests. Host-provided contexts remain untouched. No corresponding downstream guest runner/AudioContext exists; no native iOS audio implementation changes. Excluded guest-host infrastructure under UNIV-3728; no independent renderer or ordinary runtime change. Both disposition reviews and patch/Bazel registration checks pass; pins advance, production shader86594 and GMe3b provenance retained. No new executable validation. | — |
| `eb7741e449615abc735e30e5165fa29ac232918e` | Private .rive_head reference only (bd41e84a to e6c7fdb0); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `183c426d8939c7dd922a28c924ad97fda8b072b0` | The complete nine-file public delta carries the guest audio processor worklet in threaded Emscripten builds, adds a blob-URL fallback, and changes its generator/docs/browser tests. This is excluded guest-host infrastructure under UNIV-3728, not browser renderer Wasm compilation. No independent renderer or Luau runtime change. Both disposition reviews and patch/Bazel registration checks pass; pins advance, production shader86594 and GMe3b provenance retained. No new executable validation. | — |
| `8b3ca41b12bfd8ac9454444102c1fb130b3e7e11` | Private .rive_head reference only (de38f525 to bd41e84a); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `1be2c7fa490ad2aad636f063a3f760cadc7e5a10` | Translate the complete Metal texture-storage delta: uploadable textures on macOS devices outside the Apple GPU family use Managed storage, Apple-family devices and non-macOS targets use Shared, and render-target-only textures remain Private. Preserve upstream platform availability and allocation/error behavior. No shader or format change. Both reviews pass;143 Metal ORE tests and live texture-upload regression pass. Intel/discrete Managed hardware untested; pins advance, production shader86594 and GMe3b provenance retained. No shader regeneration or release. | — |
| `90195332fa88f1cb45d688dd1c4b373578785490` | Private .rive_head reference only (8cd0a485 to de38f525); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `6c65f385a1bf546c53ae3e69da17b40de5b9eac4` | Translate NestedArtboard::update so dirty referenced artboards flush in the same advance regardless of pause state, preserving frozen animations. Port the complete public playing-artboard opacity regression. Account for the three updated upstream Silver streams without weakening CSS adaptation checks. No guest execution or layout-policy change. Both reviews pass;4 opacity cases, exact focus and epsilon echo pass. Bankcard old/new C++ and Rust streams differ only by color reordering; unchanged CSS seal already fails in old-condition control, tracked in UNIV-5576; pins advance, production shader86594 and GMe3b provenance retained. No assertion weakened; no live GPU/browser coverage. | — |
| `c5a4e46827f76326cc1a73fe02ce345e2e33e1f9` | Private .rive_head reference only (c2f51857 to 8cd0a485); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `57acb425f40aa64d226be54a91d5a7cb19875b4b` | The complete public delta contains only 68 guest .riv fixtures, a new AssemblyScript barrier fixture, and test-shell additions (70 files). Guest worker/barrier fixtures and their execution harness are excluded under UNIV-3728. The barrier implementation described by the title is absent from the public delta and is not claimed imported. No independent renderer, Luau or ordinary runtime executable change. Both disposition reviews and patch/Bazel registration checks pass; pins advance, production shader86594 and GMe3b provenance retained. No new executable validation. | — |
| `48b8c0ba160586078c9e40eaa2e9ef2f17ac496f` | Private .rive_head reference only (59d7b7da to c2f51857); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `9eb2a6d99bf36d8833155779efd6e0dd9cccad5e` | Inherited lite-RTTI intermediate downcasts, source-mapped consumers and complete public regression translation. Both reviews pass; renderer/ownership tests and native backend builds pass, Metal regressions pass. Wasm toolchain CopyFile limitation tracked in UNIV-5575; no live browser coverage claimed. Fixed shader artifacts retain original provenance. | — |
| `3ec7ef079f569a5f378f964008794ac30e348a56` | Private .rive_head reference only (da345444 to 59d7b7da); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `4252d73093af032d8c59b01ab3a7aa40d3c24add` | WebGPU aligned uploads/allocation limits and ordinary audio timing/fade translation; applicable public regressions. Guest engine/thread dependencies and Yoga semantics excluded; new generic audio-renderer/block APIs deferred. Both reviews pass;22audio,3runtime audio and2Metal pixels pass; WebGPU production-feature build passes. No live WebGPU/browser/audio-device coverage; fixed shader artifacts retain original provenance. | — |
| `6e8050f36681cfedede0089bcb9730f300b0e204` | Private .rive_head reference only (e2e9eb25 to da345444); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `9cb784fc266bbc337ce4b3a1338f1fbdf7bdd168` | The complete public delta adds dart pub get before compiling upstream's   C++ core generator on a clean checkout. This development script has no Rust   runtime owner or downstream invocation; no C++ generated-header or CI workflow   changes are public in this commit despite its title. Account as inapplicable,   with no private generator output or CI implementation claimed imported. Both disposition reviews and patch/Bazel registration checks pass; pins advance, production shader86594 and GMe3b provenance retained. No new executable validation. | — |
| `f4da8fa42ed2c256d3d672e3828d18a328528f50` | Private .rive_head reference only (75da10d6 to e2e9eb25); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `e48d891044875935510c6aab5ea03d3960150fcc` | D3D11 binds retained groups through the active pipeline's registers.   Its four implementation files and changed D3D11 test branch are inapplicable   because this fork has no D3D backend. The shared ore_types delta only updates   explanatory backend comments, mirrored in Rust; no common executable change. Both disposition reviews and patch/Bazel registration checks pass; pins advance, production shader86594 and GMe3b provenance retained. No new executable validation. | — |
| `3df35b948202d3fcb0beb4b8286e4e998266f455` | Private .rive_head reference only (f018ea15 to 75da10d6); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `e3b9461b1800dc5d227b1815663eb222e1934db7` | Resolve shared layouts at active pipeline slots across Metal/GL; validate Vulkan bound slots, ignore layout-native slots on WebGPU. Port public Metal regression and exact GM shader header; unsupported D3D backend/test branches inapplicable. Both reviews/rereviews pass; pool125 ORE and603renderer tests (6existing ignores), liveMetal layout-reuse case, Vulkan/WebGPU builds pass. Exactrecord/snapshot assertions, enumimport and missing Metal Pipeline base projection corrected; no liveVulkan/WebGPU/browser coverage. Productionshader provenance remains86594. | — |
| `708d1daf41c45a8aa7af3bf768392ab3e39a2a7e` | Private .rive_head reference only (918bbc62 to f018ea15); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `bcb9c95d1d11a3f050d2f68aba047c827bd0ef8a` | Queue per-owner script fetches past active/rate ceilings with bounded waiting count/body bytes; translate public Luau fetch regressions and exact defaults/order/cancellation. Both reviews/rereviews pass; pool 27 fetch +48 promise +12 policy tests pass. Correct source-proven catch-only fulfilled dispatch and stale fixture Option calls without weakening assertions. Shader products retain86594; no guest Wasm engine or live-network coverage. | — |
| `1513dd72459d8f2dbe4500408097c0a7cbe1059b` | Private .rive_head reference only (523433dd to 918bbc62); no public executable delta. Both disposition reviews and patch/Bazel registration checks pass; source/oracle and Silver advance, shader products retain actual86594 provenance. No new build/device coverage. | — |
| `c02b45aad6b32b3eb60c25396c5669e4ff5fd746` | Optional Southeast Asian dictionary breaking and exact public tests; all language/compression flags default off. Preserve CSS-normal Korean/UAX rules and full conformance instead of global upstream tailoring/skips. Both reviews pass; pool50 compressed/API +48 uncompressed/API +26 default shaping/rule/conformance tests pass, runtime all-feature compile passes. C++ generator output translated as literal data; shader products retain86594. | — |
| `41b17c32e5ea3865a10c44469f23a9cf0696016f` | Add WebGPU sampler define in dynamic non-GL Wagyu GLSL branch; offline generator already equivalent. Correct dynamic input payload/define mismatch; both reviews and pool library build pass. Native tests cannot link absent Dawn (documented limitation), no test pass claimed. Private metadata unclaimed, generated products retain86594. | — |
| `bece7efb68a0cbf9809135236c4793bd6feb6a73` | Runtime proxy insertion already equivalent; native-editor defaults/journal/arena hooks inapplicable. Both disposition reviews complete; patch and Bazel registration checks pass, no executable change or new runtime validation claimed. Private metadata unclaimed; shader products retain86594. | — |
| `784e3870a80f586950030187cb4df843b4bb9bdf` | Own newly created GL textures/renderbuffers so release deletes them; wrapped external textures remain caller-owned. Translate current factory invariant. Both reviews complete; pool 603 renderer tests (6 ignored), including actual ownership/deletion regression, pass; private metadata unclaimed, shaders retain verified86594 provenance. | — |
| `f5bd36c1c7addd83d07304c3d37d2be05a321df6` | Schedule destination-blend barriers before batch construction; retain base-group contents, cull empty paths and fully clip empty clip paths. Port three public empty-draw cases. Both reviews/rereviews complete; pool 602 renderer tests (6 ignored) and focused native Metal destruction pass; private metadata unclaimed, shaders retain verified86594 provenance. | — |
| `c51c078efba52f369cccbbbfd26c99bc2421d613` | Gradient transform setters invalidate shaders through inherited LinearGradient hooks; port all three public regression sections. Both reviews complete; pool 893 tests (2 ignored) and callback regeneration pass. Private editor tables absent from public delta and unclaimed; shaders retain actual86594 provenance. | — |
| `86594b7144ceeabe4261afeb0082fd9ee75bf9ca` | Four-wide cubic preparation with retained context buffers, exact platform arithmetic and cusp-join flag. Both reviews/correction rereviews complete; pool 944 runtime/math and 599 renderer cases pass, plus Metal/Vulkan compilation and regenerated shader verification. GM rename-only deltas and pre-existing omitted curvature test explicitly accounted above; private metadata unclaimed. | — |
| `b3360ab942e4f187f6b9fb74f5ef0617745e8ce9` | Artboard cold-list LazyVectors, source-state and dependency-recipe sidecars, lazy keyframe index and predicate removal. Both reviews and correction rereviews complete; pool 873 runtime tests (2 ignored), 71 regressions and shipping C API compilation pass. C++ editor-only hooks are unshipped. CSS behavior retained; private metadata unclaimed; actual41a shader provenance retained. | — |
| `1a15344bdc16c1cdac83fa9007fc801777fe1470` | Exclude invalid depth/stencil Vulkan warmup permutations: offscreen MSAA seeding requires MSAA and preserved color. Both reviews complete; pool Vulkan compilation passes. No new upstream tests or Android hardware claim. Private metadata unclaimed; actual41a shader provenance retained. | — |
| `e579560ae8bace3074a9a91670cc1573d4cf68d7` | Vulkan single-entry SPIR-V splitting, reusable scratch and temporary stage modules; shared frame advancement and replay diagnostics. Four public splitter tests. Both reviews and correction rereviews complete; pool 260 renderer tests (5 ignored), 94 ORE tests and Vulkan compilation pass. D3D12-only changes are unshipped; private metadata unclaimed; actual41a shader provenance retained. | — |
| `0238f2995df485168a87801e4dbf92478a3317d2` | Global text-style references, instances/sharing, host data binding and run pointer listeners; all 26 public regressions. Both reviews and correction rereviews complete. Pool: 26 new tests, 871 runtime tests (2 ignored), 58 existing regressions and shipping C API compilation pass. Full54-file public delta accounted, private metadata unclaimed; CSS contract and actual41a shader provenance retained. | — |
| `73d7b1c208b6f044c3679e73b864fc55d47516b0` | Shared artboard source lists and nested semantic visibility/lifetime, public regression tests. Both reviews/correction rereviews complete; pool 871 runtime (2 ignored), 90 upstream regression, 119 host/live-move tests pass and shipping C API compiles. Full18-file public delta accounted, private metadata unclaimed; CSS behavior and actual41a shader provenance retained. | — |
| `cf0f67522988e0297cb83056c63f8265cacd3e16` | Renderer recording/resource lifecycle and BC/native GPU format guards; view-model/trigger/delegate lifetime; bounded UTF decoding and raw-text/input/native Luau tests. Full91-file delta accounted; guest engine/fixtures excluded and private metadata unclaimed. Invalid-byte TextValueRun fixture not literally executable at safe-String boundary. Both reviews and correction rereviews clean; pool 871 runtime +96 runtime integration +15 lifetime +13 scripting text +592 renderer tests pass (8 ignored total), Vulkan and Metal compile pass. No device pixel claim; actual41a shader provenance retained. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `4e930086e8739bb1e2fe0dccced3b66ca5e3f583` | Tools image hide/reveal identity and bounded artboard settling, native Luau deterministic seed and draw settling, CSS-compatible image layout invalidation. Both independent reviews and correction rereviews clean; pool API40/runtime871+2ignored/renderer590+6ignored/scripting43 passed, tools-disabled C API and Metal tools checks passed. No device pixels. Guest-Wasm proxy/VM work excluded; private CLI/test changes unclaimed. Actual41a shader products retained. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `91a25f40d81f5f84d34434def13b1d59cc892b35` | GL destination READ framebuffer rebind after PLS copyback, mechanical source authority and command-order regression. Both independent reviews clean; Linux pool renderer-webgl2/with-rive-tools library passed 590/0/6 ignored including exact copyback command order (e681325a-a649-4c4a-b416-df67935f07b8). No new live Chrome PLS/device pixels. Active refs advance; actual41a shader provenance retained. Private provenance and absent Flutter tests/changelog unclaimed. | — |
| `3fb675a02566925f7108d1c5f6ffc5abcf768d2c` | Upstream test-deployment helper guards a None command before its blank-command indexing check; no live downstream counterpart or runtime delta. Both independent reviews clean; exact regenerated Silver verified ref-only. No new compilation because runtime/shaders are unchanged. Active refs advance, actual41a shader provenance and historical evidence retained; private metadata unclaimed. | — |
| `f16d5e66c91902200e2bddba626c34623ea5f2a1` | Interpolator restart timing fix and public regression; exact upstream time_based_interpolation Silver baseline consumed from the pinned fixture, no downstream expected-output regeneration or tolerance changes. Both independent reviews clean; pool converter suite 4/0 and exact changed Silver replay 1/0 (35 filtered) passed on the default arithmetic lane. All five public files accounted; private provenance and absent Dart changes unclaimed. Shader products retain actual41a provenance. | — |
| `2a29c82dc10ee801b7466235e4f4c38f06b29481` | Upstream test-deployment helper changes (Windows Ctrl-C, PowerShell blank command, server-only operation, mjs MIME); no live downstream counterpart or runtime delta. Both independent reviews clean; exact regenerated Silver checked ref-only. No new compilation because runtime/shaders are unchanged. Active refs advance, actual41a shader provenance and historical evidence retained; private metadata unclaimed. | — |
| `3d7dcc5be99d8ba0b3c2cb838c1f48f79954c150` | Upstream removes two unshipped CoreGraphics renderer paths and related build/test/visual-diff plumbing; no live downstream owner deletion. Retain unrelated Apple image decoding/framework links. Both independent reviews clean; exact regenerated Silver verified ref-only. No new pool compilation: runtime/shader inputs unchanged. Active source/oracle and Silver refs advance; shader products retain actual 41a provenance. Historical evidence retained; private metadata unclaimed. | — |
| `e406d1f4231d309f52e4a02ff68d1bf980d3bee1` | Per-entry sampling metadata, pipeline/layout selection, Luau entry names and GL unpaired-first sampler order with public entry-layout regressions. Translation and both reviews clean; renderer WebGL2/tools pool tests passed 590/0/6 ignored, Luau shader asset resolution passed 18/0/0, and non-tools Vulkan check passed. No new browser/Metal hardware qualification. Guest Wasm from-RSTB host/ABI/JS/AssemblyScript work excluded; private metadata unclaimed. Existing compiler integration remains parked. Shader artifacts retain actual 41a provenance. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `41a2120bb572e8168ba08fd2ea514c9ddf25d270` | Correct shared angular-gradient coordinates to clockwise stop progression; GPU comment clarification only, no packing change. Both reviews and newline-correction rereview clean; exact 41a pool artifacts imported with unchanged aliases. WebGL2 library tests passed 574/0/6 ignored; Vulkan check passed. Supplementary macOS check passed in 4ca8cef0-b210-42f0-86c0-b2c575fa7fc9 at exact source41a; this is historical compilation evidence, not a later-checkpoint result. Prior wasm32 infrastructure limitation unchanged and not rerun. No new pixels; historical captures preserved. Private metadata unclaimed. | — |
| `d278866b98ed71118da6cf4126751f2a92ccf70c` | GL/WebGL2 depth-stencil layer masks: capability, misc key/batching flag, destination scaling and shader sentinel/factor; Vulkan/WebGPU remain unsupported in this mode. Both reviews and correction rereviews complete; exact d278 pool artifacts imported. Linux/WSL WebGL2 library tests passed 574/0/6 ignored; Vulkan and Metal consumer checks passed. Actual wasm32 check failed before renderer compilation on rules_rust CopyFile remote-only strategy; no browser pixel qualification. Exact source counts and minified fixture expectations corrected; load-store mismatch preexisted this commit. Existing Metal pixel assertions unchanged; added upstream ANGLE/EGL 4x-MSAA coverage unavailable in Nuxie. Private metadata and absent Dart changes unclaimed. | — |
| `6771fdad560d6d027ba8bdd891fbb95bdd8f1d11` | Row lifecycle/default-state recording, reset/event/pointer cleanup, scripting eligibility and pending-layout behavior with public regressions. Includes CSS-compatible dirt/text/cache optimizations. Both reviews/correction rereviews clean; pool 83 integration and 422 CSS tests pass, plus 30 Silver metadata checks. Exact Silver provenance refreshed without changing CSS divergence annotations; private metadata unclaimed. | — |
| `094d24e1744141102962dae1e82f8536900a22d9` | Account for three shader ATTR formatting/comment changes for the unsupported Unreal compiler and private provenance. No supported runtime semantic delta; advance active source/oracle and Silver references while retaining actual 74e5d3f6 shader provenance. Independent source-equivalence and Rust-integration reviews completed; 30 Silver manifest checks passed, as recorded in [PR #1144](https://github.com/nuxieai/nuxie-runtime/pull/1144). | — |
| `8ea42e0ee49bb5d9f1887643a1042226790772f6` | Text-input character/line limits and empty-line cursor correction are eligible; preserve CSS/Taffy semantics and exclude Yoga/legacy hug-layout behavior. CSS-owned maxLines uses current laid-out max_width; only non-CSS explicit existing Hug uses measured width. Upstream Yoga/legacy hug helper and engine changes are excluded. Both reviews and correction rereviews complete; 13 high-level tests pass and all 17 new raw cases passed initially. Full raw/schema reruns pending after bounded fixture/parser/inventory corrections. Goldens lack a declared pool C++ builder and were not run. No private Dart/FFI changes absent from the public delta claimed. | — |
| `156e959c7a71d96f2ffc16515b483ee464c38863` | Translate inner-feather change notification/path rebuild and the public path regression. Refresh exact Silver producer-line provenance and active source/oracle pins; no private metadata claims or shader relabeling. Both independent reviews and correction rereviews clean; all 7 scoped feather tests and 30 Silver manifest checks pass. No broad/device qualification claimed. | — |
| `47d1cd54ccefd468db7cb26ecc4699ae1723dd85` | Translate applicable sampler validation and WebGPU feature support. Exclude guest Wasm/AnimaScript web test-player and dependent premake changes; no private metadata claims. Active source/oracle pins advance, shader products retain actual 74e5d3f6 provenance. Both independent reviews and correction rereview clean; macOS pool tests pass 266/0/5 ignored, with 30 Silver and 16 provenance checks passing. No new browser/device coverage. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `4efe82a106e0223056c97c137d1ae735aed4f0c2` | Account for all nine public housekeeping files: eight whitespace/comment-only changes and private provenance. No Rust behavior delta or unseen RML regeneration claimed. Advance active source/oracle and Silver references; retain actual 74e5d3f6 shader generation provenance. Both independent reviews clean; 30 Silver and 16 provenance checks pass. | — |
| `74e5d3f635c3c463fe10e4327261705549ee5919` | Translate angular-gradient shader specialization and shifted feature bits/keys across supported shared and backend owners. Regenerate portable products on the pool; update all aliases and exact Metal capture. Both reviews and correction rereviews clean; renderer tests and Vulkan compilation pass, with Metal/browser/device limitations recorded above. Exclude unsupported Unreal-only preprocessing and private metadata claims. | — |
| `16c184ef2c17fa8615a5e9366f7800e0c4d79fae` | Formatting-only public source: full lexical/preprocessor audit and shader-generator token comparison found no production semantic delta. Account for new shader clang-format configuration and private provenance without claiming private changes. Advance active source/oracle references and exact Silver line metadata; preserve actual ada892f3 shader artifact provenance. | — |
| `ada892f37808e289e716d29010b7e79d0656f79d` | Translate five retained shared shader owners and regenerate WGSL/GLSL/SPIR-V with pinned pool tools; keep generated exports, Rust aliases and sampled-clip adaptation coherent. D3D-only host/backend/root-signature work is unsupported; private metadata is not claimed. Both review passes and scoped renderer tests/Metal compilation pass; browser consumer validation limitations are recorded above. | — |
| `12e5327ba8cb4346155010d34584cb4dab640d51` | Translate the complete public deferred-canvas delta: mark frame-starting ranges, retain only each canvas's last frame before grouping and dependency scheduling, close stale same-canvas ranges on begin, prohibit self-nesting, and reset retained recorder transforms for new frames while preserving nested resume state. Port all four public canvas-reframe regressions. The `.rive_head` update is provenance only; no private source changes are claimed. Both independent reviews pass; pool renderer suite passes 265 tests with 5 ignored, including all four new cases. | — |
| `8920a6b03e202086408f596fc4a317df07763440` | Translate public per-entry binding wire metadata, shader/module retention and physical-name mapping, selected-entry layout derivation, actual native-slot collision checks, draw-group validation and 136-byte deferred shader descriptors. Port all eleven entry-layout tests and changed recording/layout/asset cases. Update Luau and live Metal/WebGPU/WebGL2/Vulkan adapter owners; reserve section 2 for upstream metadata and move private Metal reflection to 128 with coordinated editor compilation. Guest Wasm IDL/VM/web proxy changes are excluded; private `.rive_head` compiler implementation is not claimed. | [UNIV-5495](https://universe.basis.dev/issue/UNIV-5495), [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `5551696a826ff546775fffe46ca78824b1f1dc51` | Translate measurement/shape keys with exact bit packing, wrapping arithmetic, initial inequality and font identity; avoid redundant post-shape layout dirt while preserving paragraph-spacing/origin invalidation. Retain variable fonts only for identical base identity and ordered options/coordinates. Preserve stable coordinate-map iteration across Rust styling passes without sorting. Port all five text regressions and the changed component-list assertion; pin the new variation fixture and consume the updated rewards Silver recording. CSS/Taffy layout semantics remain unchanged; private .rive_head contents and reported upstream benchmark speedups are not claimed. | — |
| `16332c8ef763b118b3e2b53d73ce29ca3b1218e1` | Translate AngularGradient type 1089 with complete LinearGradient inheritance, registry, clone/import and paint dispatch. Add transform keys 1103–1107 with exact defaults and notification-only equality-guarded setters; assign the angular shader followed by left-associated translation × rotation × scale. Linear/radial rendering stays unchanged. Reconcile publicly evidenced schema metadata and preserve Nuxie Video hooks. Add labeled supplemental coverage; no public tests/assets or private editor/Dart/protocol/RML contents are claimed imported. | — |
| `269ebcab3016e68340bb1427b720712213066eaf` | Use exact two-call SUCCESS-only instance extension/layer enumeration and stop before using failed counts; treat INCOMPLETE as failure. Device enumeration failures log and continue with an empty list, retaining headless creation and rejecting required missing swapchains. Preserve existing offscreen startup and optional Android presentation. Port all public loader cases through production helpers with scoped thread-local fake dispatch and initialized Rust counts. Stack poisoning/environment mutation are unnecessary at this Rust boundary. No new fallback or private .rive_head contents imported. | — |
| `5e3de3857d6ae9e019f94da507f6385b96fbf3c3` | Prepare decoded paint images through the frame-sink hook; track pending paints and retry before resource replay creates version aliases. Preserve ordered removal, generation-safe destruction, canvas exclusion and stored sampler/matrix. Port both complete lazy-image regressions. Existing direct Rust decoded-image owners retain default preparation. Private web mesh-buffer changes, web sink override, browser tests and .rive_head contents are absent from the public delta and not claimed imported. | — |
| `f2aa739e3ec4922c22ea34e4b6bce1653ea21106` | Forward host canvas command buffers and frame serials without local submission, preserving own-buffer fallback and no-active-canvas ordering. Add an explicit unsafe Rust submission/lifetime contract and port both upstream spy tests. Flush GL before state scrubbing. Acquire wrapped Vulkan targets through Rive's access tracker with exact load preservation and remove all pending sampled transitions for that texture identity. Concrete Rust hosts retain default resources; private Android override and .rive_head contents are absent from the public delta and not claimed imported. | — |
| `183df53fbdfa36441b90a06130cd1d68a87c3ce4` | Cache per-verb counts through every mutation and ownership path in both RawPath owners; carry counts through borrowed views, remove mutable verb access, and use cached counts for renderer reservations with the existing combined-record representation and implicit closes. Port the complete mutation-count test and expanded long-zigzag GM fixtures with fresh provenance-bound C++ captures. Correct the initial cap's preexisting tangent translation error exposed by the new fixture. Skia accessor edits have no live Rust backend and are inapplicable; private .rive_head contents are not claimed imported. | — |
| `e7124f18b96e11fde64a14c48138c8b4c80d2acc` | Translate the cubic convex-under-180-degree and nondegenerate-line early-out in both runtime and renderer owners. Preserve separate SIMD products/reduction, ordered C++ min/max NaN and tie behavior, exact EPSILON, sequential zero-edge substitutions and untouched output slots on early return. The existing solver remains unchanged. No new upstream tests occur in the public delta; private .rive_head contents are not claimed imported. | — |
| `1c75867de17587c541ad2d7ca88b10577b033bd2` | Collapse cusp slivers onto exact shared endpoints and emit n+1 cubics with full-round cusp joins. Retain chop parameters and re-chop at emission; preserve adjacent tangent pairing, polar counts, shared spokes, cap ordering and retained Rust scratch ownership. Translate the new join encoding and full-round shader branch across all supported generated backends. Update existing cusp expectations and label supplemental collapse coverage; no new upstream unit tests occur in the public delta. Private .rive_head contents are not claimed imported. | — |
| `e8d647fd5acd66574d0a8a9614955ece7e38fd3c` | Translate bitmask, fast min/max, integer-conversion rounding, hardware-only reciprocal-square-root estimates, named four-column matrix loads/transposes and const bit-cast; migrate existing Rust tuple callers and port the complete public test delta. The renderer's tuple-only syntax delta has no corresponding live tuple caller. ARM32 source branches are retained but explicitly unsupported under the pinned stable toolchain; no ARM32 performance or execution claim. Supplemental mask/transpose/const checks are labeled separately from upstream tests. Private .rive_head contents are not claimed imported. | — |
| `e54bc92a625cee04f88d903822c4503cb44f3714` | Check the exact legacy serialized method sentinel once after successful script self creation, clear only absent down/move/up/exit handlers in order, recopy corrected bits to the occurrence, and retain them for later instances. Reuse live Lua field/function introspection and default unknown backends to present. Typed lookup failures preserve pending status and uncommitted bits. Gate pointer hits on current script lifetime and pointer handlers; retain independent scroll bounds and the broader upstream listener-registration mask. Port both exact legacy-hit fixture cases and add Lua lookup coverage. Private .rive_head contents are not claimed imported. | — |
| `113d1bcedcc387c92ee000f64a5d7d947b4229dc` | Use cached RiveRenderPath bounds for scale/translation and retain raw-point mapping for rotation/skew. Translate transformed area as abs(determinant) times width times height in source order; update stale triangle-arithmetic expectations from pinned C++ expression evidence. Port the 200-path/seven-matrix exact bounds regression with documented portable PRNG sampling. Historical retained comments are labeled separately from the live owner revision. Private .rive_head contents are not claimed imported. | — |
| `0f40af120fc3fec35add319d3334c09af6231457` | Clear TextStyle's cached variable font before returning when its base font is absent. Port the exact public font-asset-clearing regression, preserving normal invalidation and no explicit view-model binding. Regenerate the shifted Silver producer line. Private .rive_head contents are not claimed imported. | — |
| `8f06a91c7dad29a8cb1345535c0855de3ace4411` | Translate supplied nested-animation definition/instance APIs with focus-before-input ordering, count/accessor transition traversal, standalone editor animation/keyboard helpers, paint identity deduplication and cloned view-model property metadata. Preserve exact artboard instantiation error and port the host-thread/context-replacement regression at the approved Rust ownership boundary; failed host hydration already constructs before publishing input. Missing-File completion is already weak-upgrade guarded; ordinary state-machine object iteration and no-arena composer/layout paths are equivalent. Native REV arena rebuilding/mounting, persistent (client,object) paths, mutation envelopes, coop validation, arena-only path-effect hydration, borrowed editor comparator/listener ownership and Lua instantiate/release/lifetime hooks remain excluded under d4fe1022. No dummy validation flags or native editor host were introduced. Private editor contents referenced by .rive_head are not claimed imported. | — |
| `fdd555d62280db2f1d659b0d8fd821ed6a4d11f3` | Translate Mali storage-texture rotation: lazy ring slots, shared per-flush slot selection, replacement for pending or failed fences, zero-timeout polling, fence-before-flush advancement, and SSBO exclusion. Wire real browser sync objects with owning-context/generation-safe deletion; retain upstream constructor and reverse destruction order. Pending ubershaders advance synchronously by feature equality, specialized pipelines asynchronously; Vulkan has no override of the unchanged default no-op advancement. Port upstream rotation/reuse assertions at the documented recording-provider boundary and add lifetime/failure regressions. Private .rive_head contents are not claimed imported. | — |
| `ce788b3b6b20aa641c20f320cd9ccce920648786` | Translate constant-time scripted-property untracking in live luaur and corresponding source owners: retained slot index, bounds/identity-guarded swap removal, moved-slot repair, owner/orphan registration and exact tag filtering. Preserve non-owning stable Rust identity and untrack-before-cleanup order. Port all three public stress/lifecycle tests without changing their counts or timing threshold. Private Dart editor wrapper-cache changes described in the message are absent from the public delta and are not claimed imported; existing runtime caching is unchanged. | — |
| `ce891f526a3297f9498a3e524fce6385cf0f2f25` | Translate six shared shader owners: source-over blending, guarded UAV destination reads/writes and retained RHI declarations. Regenerate supported-backend shader inventories and exact minified-name consumers; restore the existing supplemental sampled-clip bridge's renamed aliases without enabling a new host path. Unreal-only wrappers, D3D testing aliases, Unreal deployment options and upstream-only golden launcher/filter changes have no shipped counterpart. The private .rive_head revision does not expose an additional implementation. | — |
| `6fa2cccf37a5909824dc98311d8cf2d8327abbd4` | Translate GL async ubershader precreation: require KHR_parallel_shader_compile, preserve permutation order, exclude renderPassResolve outside atomics, queue without synchronous completion, and expose the existing generic queue operation in both Rust specializations. Enter the owning GL execution domain and restore the caller domain at the approved host boundary. Retain exact GL header/implementation snapshots. The .rive_head private revision is provenance, not an additional public implementation. | — |
| `fa2cd6e38c6ab109a7feb6c814b55e4fa1f3d33c` | Translate artboard-backed PaintImage sources: per-host instancing and cycle/nesting limits, default playback, ordered advancement/update, data-context propagation, raster sizing/hysteresis and failure retry, renderer rerouting, lifetime and canvas identity. Register artboardId 487 and all advancing/referencer/host/update dispatch. Preserve editor-feature paint discovery. Rust occurrence entrypoints release parent borrows across child callbacks while retaining child host visibility. Translate all public runtime/canvas/pixel/GM/replay cases and retain the exact binding fixture and independent C++ golden provenance. No private editor implementation or guest execution is claimed. | — |
| `30454306ffcd0f360a122af1afa6e2295182abc5` | Expose RenderContext::selectInterlockMode and route beginFrame through it, preserving capability checks and selection order. ViewModelInstance null initialization is already equivalent to Rust's default None. Native-editor ObjectArena retirement, diagnostic and reference-count hooks remain outside the admitted editor-arena profile established at d4fe1022; their retirement behavior is not claimed implemented. Duplicate includes and ownership comments require no runtime change. Private editor code absent from the public delta is not claimed imported. | — |
| `1b887eb61219d549bd59eb4cb879b957384fcc64` | Already equivalent: the public change only exposes LayerState and StateTransition headers to C++ editor builds for the existing addScriptedInterpolators body. Both Rust modules and that method are unconditional; no runtime semantic change is needed. Private script undo/collaboration changes and tests mentioned in the commit message are absent from the public delta and are not claimed imported. | — |
| `c63a5ba0486e6f6cc4f88dbb98b535f9db4ab382` | Translate the string-template converter, named/positional/sequential placeholders, configurable delimiter parsing, validated number formatting and shared thousands helper. Preserve argument import attachment, bind ownership/clone retargeting, dirty propagation, generated type/property dispatch and the testing converter accessor; retain the already-null Rust parent-bind initialization. Port the public unit tests and exact string_template.riv fixture. Formatting remains pure Rust at the approved host boundary. Private editor/RML/rive_native changes referenced by the commit message but absent from the public delta are not claimed imported. | — |
| `536a4f7a3c2e5817d91fdf80f933fa0018750921` | Back a deferred canvas before resolving its image during paint mutations. Record canvasSample beside each canvas drawPath that samples a foreign image, capturing the handle at draw time; schedule its dependency without walking screen draws or tracking later paint state. Preserve the draw-filtered no-op replay and translate all four upstream regressions. The existing generic command decoder accounts for the redundant-command analysis addition; private .rive_head contents are not claimed imported. | — |
| `4672b88bf62eee54746854440a914ed20499fe8a` | Translate angular gradients through renderer factories, deferred commands, append-only SRIV opcode 45, paint packing, inverse transforms and always-complex texture ramps. Preserve upstream first-stop default only for factories without an angular implementation. Regenerate supported shader artifacts and translate both gradient GMs. The upstream-only counting benchmark and redundant-command analysis remain source-oracle tools; the shared downstream decoder understands the new payload. Private .rive_head contents are not claimed imported. | — |
| `10d08fb6eb7d1b19ceebfba38cbfec7ef9d7d45c` | Keep the first view model carrying each Data name, preserving its retained constructor identity through the Rust scripting projection. Refuse a generic uint target sync into a missing or non-integer data value, while retaining derived integer asset values. Translate both public regressions. CLI/RML/editor changes described in the commit message have no public implementation in this delta; private .rive_head code is not claimed imported. | — |
| `9166e9b62debca7482e3cd13229ec5a8c35af603` | Tie-break equal draw sort keys by signed draw index, negative for prepasses, in native and browser-specific sorters. Preserve image/shader/scissor batch admission and merge both append- and prepend-contiguous tessellation ranges. Translate both upstream hairline batch-count regressions using actual flushed draws. Private .rive_head contents are not claimed imported. | — |
| `9a20396e3ccdb89365b6fefa95ce8166946e3418` | Reject bind-group entries containing null buffers, texture views or samplers with upstream error order/text. Preserve wrapped canvas views and undescribed sampler behavior. Translate the null-sampler regression and update the dynamic-offset test to supply real buffers. Shader entry-point comment handling mentioned in the title is absent from the public delta; private .rive_head compiler changes are not claimed imported. | — |
| `a92390456361a349705a59105567e7a8271946d3` | Translate empty styled-text append suppression, list-pop parent removal, delegate mutation/reentrancy ordering, and the four new public runtime regressions. Share sample-count, sampler and vertex-layout guards with live Luau bindings, including count admission and exact errors. Forty-nine public files remain guest-execution-only and parked: Wasm headers/VMs, WAMR tools/patches, AOT cache/security/tiering, guest renderer/import/browser-runner changes and guest GC/browser fixtures. These browser files are the guest scripting lane, not the wasm32 browser renderer. Private compiler changes behind .rive_head are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `20e3e88639492e3e80344de4954842898f6c3b35` | Add the public Font codepoints API and Rust-native font coverage enumeration: ascending nominal Unicode scalars with glyphs, without fallback. Preserve the approved Rust text backend; upstream's HarfBuzz C build flag removal has no corresponding native dependency here. The font-viewer UI and other private .rive_head contents are absent from the public delta and are not claimed imported. | — |
| `1d33ee5d92348dc958424bd207ef58ee2510a3b5` | Translate shared WebGPU binding validation, per-stage limits, texture/sampler pairing inference and layout identity salts; record sampler descriptions across shipped backends and deferred recording, remove Vulkan/WebGPU fixed binding-array ceilings, and preserve creation-observer callback ordering. Add live Luau texture-view aspect parsing and the public renderer/scripting regressions. Native D3D11/D3D12 adapters are unshipped. Guest module sampler recording remains parked; private .rive_head contents are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `5df52be209e0c0388c484266f7696b8d4d0b7b4a` | Use the animation state's speed when deciding whether an applied pose is finished and can freeze. Translate the exact backwards-state regression and upstream fixture: the reverse state must play from its forward twin's end pose and freeze only after reaching its start. Private .rive_head contents are not claimed imported. | — |
| `daf55385b71fb861b2a1ae2930ae4492832d5db5` | Deferred: the complete public behavior delta adds WasmScriptingVM frameRelease export detection and calls __riveFrameRelease after frame-minor collection/debug verification. This depends solely on parked guest execution, not Luau or browser renderer Wasm. Private module queue/pre-collect-hook implementation described in prose and .rive_head contents are not claimed imported. Advance source/oracle pins and inventory only, retaining historical validation provenance. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `ca8fad85f2b85e192705606c684c5a44dde3a30a` | Traverse the artboard drawable inventory instead of all objects when collecting selectable Text; retain nested/list-artboard recursion and relative Text order. Make the drawable accessor available in runtime builds as well as editor builds. Private .rive_head contents are not claimed imported. | — |
| `785bca185218ef4b8eec3e525199f27a8f23d49f` | Translate DrawTarget validation: retain superclass validation and require drawableId to resolve to a Drawable before initialization. Translate the public missing-drawable regression with the exact upstream fixture. Private editor export changes described only in commit prose and .rive_head contents are not claimed imported. | — |
| `10e2138336df07cca4888f6a5246be39aeae67d9` | Pace nonthreaded work by the poll's captured submission boundary and threaded delivery by the initial completed-queue count. Preserve nested polls and callback-submitted work with released internal queue locks and shared pool APIs; update existing scripting callers and translate all four public regressions. Private buffer/GC changes mentioned only in commit prose and .rive_head contents are not claimed imported. | — |
| `40e52541238037bab60088a675dc29676584bdfd` | Translate collapsed-component run skips from the Collapsed dirt bit, capped u16 distances, stale-run notifications, dependency-order invalidation and exact mid-pass fallback to single steps. Preserve editor clearing notifications and TESTING verification/counters. Resolve keyed-object interpolator hosts once and lazily cache the absence of keyframe binds outside editor builds, preserving clone state. Translate both public keyframe-bind and Solo regressions. Private .rive_head contents are not claimed imported. | — |
| `ac0c03ae4d5cdd5d5e84b18ddafd6738b26e629e` | Translate stateful scripted-interpolator holders with ordered input-bind teardown; per-keyframe, per-artboard style, per-state-machine transition and per-converter ownership. Layout inheritance borrows the defining artboard's clone; bound converters keep input binds private and apply them before reinitialization on every bind. Add scripted layout interpolation and all six public regression cases against exact upstream fixtures. Private editor UI/exporter/docs/dispose changes mentioned only in commit prose and private .rive_head contents are not claimed imported. | — |
| `be93c2ced22636f2e1db2186630c3bc5d61cfeb3` | Translate callback-presence import tracking, static-pose/keyframe endpoint bounds, serial-based held-pose freezing and exact testing verification. Preserve quiet-row wake ordering for component, color and self-transform changes, keyframe-bind and transition-interpolator exclusions, and per-advance serial capture. Runtime and binary import reject machines above 65535 layers. Translate all six frozen-pose tests against the two exact upstream fixtures. Private .rive_head contents are not claimed imported. | — |
| `2a043c31ccbda27628a2c80fd105c16ea35ed12a` | Deferred: the complete public behavior delta releases Wasm guest VMs before file artboards, including Emscripten deferred-script release; its AnimaScript keyboard/artboard-input fixture and tools-only teardown test depend on parked guest execution. Existing Rust Luau cleanup already precedes artboard teardown and is unchanged. Account for all seven public files without claiming guest execution or private .rive_head content. Advance active pins/inventory only, preserving historical shader and validation provenance. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `3ccf0d9d7c2cf8bee0553245acaffe21ae3314b6` | Require every deferred replay sink to supply an explicit render context or explicitly choose none. Preserve replay-time canvas backing and content dropping without a device, add the throttled missing-device warning, and record missing ORE canvas views as a named unregistered/unbacked failure rather than unresolved churn. Translate both canvas-backing regressions and explicit no-device test sinks. Private .rive_head contents are not claimed imported. | — |
| `1b9b7381fd92e2de41f9c936f2073d1e0d984717` | Translate MeshHost and lazily allocated Node-mode NestedArtboard mesh/N-slice state, fixed display-scaled mesh rasters, draw-time UV refresh, clipping/visitor vector paths, and immediate raster release on source swaps. Bitmap Cache retains on-screen sizing; self transforms move to compositing and independent content revisions prevent stale multi-path caches. Translate public renderer-host capability, all mesh/cache regression cases and hosted semantic-rotation coverage. Private editor/agent/WebGL changes described only in commit prose are not claimed imported. | — |
| `3f67b2f86d5c62e5767f3f5b93edfb4b26f68853` | Translate runtime cubic-vertex missing-weight guard and Weight::fitsVertex registration rule, rejecting base Weight for cubic vertices. Port both complete live skinned-rig regressions. Private editor parent-change and agent add_vertices work described in the message is absent from the public delta and is not claimed imported. | — |
| `1e55b86a79b97d57065e892ac8fc3b219596e768` | Guard script artboard-input instancing with a thread-local stack of source-artboard identities, leaving only repeated inputs unset and removing the guard on unwind. Translate the public Host/Holder cycle regression with the exact upstream fixture. Shared runtime/Luau behavior is in scope; guest execution remains parked and private .rive_head changes are not claimed. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `439ffe0c1c79c6c20b7c9e553f8f0586bbd9e2f6` | Claim artboard slots before object import and retain empty slots on failure; match script inputs to their owner's parent ID, including unreadable owners. Translate the immutable binary reader counterpart, initial-valid orphan pruning, and failed-super view-model import return. Component-list null-row iteration/pooling is already represented by non-null Rust map entries; preserve it and port both upstream regressions alongside all new malformed-import cases. No private .rive_head/editor changes are claimed. | — |
| `671423924df5b251d0f9b3c9f929062bbd27eef6` | Translate the shipped WebGL branch's removal of per-pass finish/liveness probes and exact successful-framebuffer attachment-key caching; the upstream native-GL branch is unshipped. Add shared target-exposure capability (base false, deferred true with pre-init setter), gate Luau gpuTarget, and preserve unresolved canvas/backend wrapping diagnostics. Translate the new upstream host-never-exposes-target case. Guest Wasm target gating remains deferred; private .rive_head contents are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `acbb030b2966249cd15a34a7d5b72abf86f0b391` | Retain the browser WebGPU render target across frames, creating it only when absent and invalidating on actual resize; assert acquired surface dimensions and continue refreshing each frame's target view. Native offscreen targets already retain this lifetime. Upstream testing-window overflow handling has no corresponding canvas-product path; private .rive_head contents are not claimed imported. | — |
| `6f3510dcc545bc8b2a78f1004a06929d17cd022b` | Public delta only adds WasmScriptingVM callDraw save accounting and restoration before renderer-handle release, preserving outer visit state. This guest-execution fix remains deferred; it does not change the Luau or browser-renderer Wasm lanes. The commit message describes private Dart/Flutter frameBoundary integration and a test absent from the public delta; those are not claimed imported. Advance active source/oracle pins and inventory without relabeling unchanged shader artifacts or historical evidence. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `f40c9dfe8a0c4accf3e963f48429e893798854a5` | Translate shader signature admission before indexing/reflection, signed-content group separation and signature verification helpers through live Rust import/scripting owners; retain approved host authentication and resource limits. Translate integer gradient coordinates, exact additive complement, uniform/instance packing, repeat gradient sampling and all supported-backend shader consumers; regenerate current shader artifacts. D3D and Unreal-specific HarfBuzz flags are unshipped. Guest unsigned-module execution, SIMD admission, WAMR cache/AOT and guest shader APIs remain deferred; import-shape tests do not imply execution support. Private signer/editor/export changes absent from the public delta are not claimed. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `71713c9f086723a44df580db4678a3ae5d68c6c2` | Track elastic platform momentum through dragView while it can stretch and consume the delta; release at tracked coast speed when crossing or landing exactly on an edge. Preserve primed-clock sampling and absorption of the remaining tail. Translate both edge regressions and the upstream velocity expectation update. Private .rive_head contents are absent and not claimed. | — |
| `9f47f07639009b16097985fe7b794c3fe09287dc` | Translate WebGPU pseudo-dynamic fill state as three retained pipeline passes, framebuffer/dynamic cache-key separation, per-pass stencil reference and color-write-disable vertex flags; enable the upstream dynamic-state capability. Shared dynamic stencil masks include the union of all three pass masks. Preserve the wasm32-unknown-unknown browser boundary; upstream private .rive_head implementation is not present or claimed imported. | — |
| `dce52972ec2c8c9e4a03b0f49a59b4e5e0f43c97` | Dirty cursor drawables with Paint only when a blink phase changes effective caret visibility, preserving the blink clock and scrolled input position. Translate the upstream artboard didChange regression covering mid-phase, hide/show, even phases and selection-hidden carets. Public .rive_head metadata does not expose or establish any additional private implementation. | — |
| `c73593c3a868f87b5dcd328432255db33e3e2266` | Translate clockwise-fill inkbleed outer-hairline subpasses, per-pass versus combined draw contents, signed pass sorting, expanded bounds, revised stroke index/AA-band order, GPU state/key routing and 17 specialization constants across supported backends. Regenerate current shader artifacts from pinned public source; preserve historical captures. Metal keeps upstream's unsupported depth/stencil rejection; D3D is unshipped. WebGPU's wgpucwib testing alias maps to the existing clockwise-inkbleed lane. The upstream-only retrofitcubictristrips GM has no existing Rust owner; no new Rust port of that handwritten GM or private implementation is claimed. | — |
| `c4d2c6cbdeefd249a04eb7ade06cb7cf07a732f7` | Translate tools-only retirement of replaced source artboards while direct or copied instances remain alive, and cleanup on later replacement; translate thread-local advancing change-window inheritance and root-host bind tracking so newly made rows see pending fires without replaying earlier frames. Port both replacement and all three list-trigger regressions against the pinned public fixtures. Preserve safe Rust ownership and luaur; private editor invalidation/remount logic represented only by .rive_head is not claimed imported. | — |
| `7fb595be56a899b4fc6eceb22a4222206383fe93` | Translate late-hosted audio-engine propagation with both authored-fixture regressions; shared core ancestry/property-owner dispatch and filtered generated change hooks; exact GL version scanning and full-recorder registry/bind ordering; direct UUID/color hex appends, fixed-number formatting and stateful contour comparators. Preserve existing Rust text/audio backends and pure-Rust numeric boundary. C++ HarfBuzz language-table flags and Emscripten-only AAT removal are build-specific: the approved Rust/unknown-unknown backend retains accepted AAT fonts, not parity with that support reduction. GL string assembly and ORE diagnostic formatting are already equivalent. Canvas2D/no-ORE recorder specialization is not a supported product configuration; private implementation absent from the public delta is not claimed imported. | — |
| `b2c28dde1f8c51872a81ae4f9b4fbbceac1d23ed` | Translate WebGL2 ORE std140 uniform allocation/range padding and exact source-byte upload, plus File display-scale forwarding to the existing Luau VM and its upstream regression. The guest-Wasm VM forwarding loop remains parked; D3D11 allocation padding is unshipped. Private implementation represented only by .rive_head is not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `05d690ee5a8879d738679f8ebad17c4a61af50ac` | Build-only: the public delta adds optional named shallow submodule initialization to the upstream Premake dependency helper. Rust/Cargo has no corresponding live helper; C++ source-oracle builds consume the helper directly from the advanced pin. No runtime or shader behavior changes. The coop_file service named in the message is private implementation absent from the public delta and is not claimed imported. | — |
| `74dda58f80efd7310959305c284a136e5c5087ff` | Deferred: guest Wasm host-object release, value/asset lookup factoring and browser guest host-call table generation/binding. All public implementation changes are in the parked guest scripting owners and glue, not native Luau, shared runtime or browser rendering. Npm package-build and benchmark changes described in the message are absent from the public delta and are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `625454e362a27bb3168f00cd87e9487f54d39338` | Translate inkbleed strokes in single-sample depth/stencil mode: shared draw/index/chunking contracts, shader sources and generated artifacts, and supported Vulkan/WebGPU/WebGL2 execution. Preserve native Metal's unsupported depth-stroke boundary. Carry the frame policy through screen/canvas and replay-oracle interfaces; translate updated range-chunker cases. Upstream path-fiddle/testing-window controls map to existing replay tools; D3D execution and Dawn Windows Premake linkage are unshipped. Private editor implementation is absent from the public delta and is not claimed imported. | — |
| `0dd067f18bc14139720ee348dfc222af0c744e4e` | Translate live scroll-viewport resolution and full-viewport text-input hit testing; preserve slot-expanded fallback and parent clipping order. Translate the single-line/multiline regression using the exact pinned upstream authored fixture. RML remains upstream fixture provenance; private Dart/editor implementation is absent from the public delta and is not claimed imported. | — |
| `8427a425fd61d2adb4652eb12fdd74d0f7f3396b` | Deferred: the only public executable delta selects mkdir on Nintendo NX for the guest Wasm AOT cache directory. This owner remains parked with guest execution; no shared runtime, native Luau or renderer behavior changes. The private libhydrogen build change mentioned in the message is represented only by .rive_head, not public implementation to translate. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `04104ce23473313d181489440c98fb087b991b68` | Translate limit-centered rotation wrapping across the half turn, preserving f32 expression grouping and max/min clamp order. Port all three upstream rotation cases, including the new authored fixture and eight programmatic rows; consume the pinned fixture without fallback. Both reviews and focused runtime/IK validation pass; no assertion or tolerance weakened. Private editor metadata is not implementation evidence. | — |
| `8092a195ec9cd456e5199164565adb88887f737a` | Translate finished-animation async continuation/pre-apply polling, current-file and foreign-bound VM pending checks, recursive hosted-artboard traversal and shared foreign-file identity guard. Preserve global pool completion order across the approved Rust per-VM decoder handoff queues. Move the three existing host tests to upstream's new test owner and add both new upstream cases plus a real cross-VM callback-order regression. File's immutable getter was already equivalent; standalone C++ player polling removal is unshipped. Guest held-outcome registry remains deferred. Both reviews, correction rereviews and applicable validation pass as recorded above. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `84a3719f80a439ac5f3a7cc7bdc34b785544d5ac` | Guest AOT boot-time O3/O0 selection, synchronized caching, compiler/Windows support and removal of live tier transplantation remain deferred. Remove the exact obsolete deferred owner because upstream deletes its header/source; preserve historical provenance. No shared runtime/Luau/renderer changes or private editor implementation are claimed. Both reviews and inventory validation pass. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `f9d7e648a957e9683dc1d5ebe56b4f2311409a8f` | Account for the shared browser-image decoder extraction through the approved live Rust codec/work-pool adaptation, not as a deferred owner. Guest browser VM, typed calls, ABI/IDL, generated bridges, guarded File/decodedFile/command-server startup and guest harnesses remain deferred. No private editor implementation is claimed. Both reviews and focused validation pass as recorded above. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `a68b6dabe1a30f15c1c36f79b7c66eee8b079b6e` | Translate the shared AnimaScript module-name constant. Guest VM adoption/registration, mixed-backend Luau guards, WAMR and guest-browser binding changes remain parked. Existing omitted VM-presence assertions are not newly weakened; private editor code absent from the public delta is not claimed imported. Both independent reviews and focused validation pass as recorded above. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `ae4e1776466164bfabd64ecedecf2d8f347fa0fc` | Translate host SMI completion polling before advance, per-file VM pending continuation and watermark internal-overload use; owner-scoped pool counts and all-thread network request lookup. Preserve Lua-thread completion handoff and count its undelivered queue along with the existing per-VM decoder/network owners. Port all five new upstream regressions. Browser decoding uses the approved Rust codec/pool; Emscripten symbol-glue fixes have no product counterpart and the common failure text already matches. Guest Wasm pending-work query remains parked. Private CLI changes absent from the public delta are not claimed. Both review passes and correction rereviews are clean; validation and the unchanged threaded-test race are recorded above. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `de3e86090892b68072e7d4505e8d979386fc9a30` | Translate scripted scroll targeting, local position/delta dispatch, pointer IDs through nested/list and scripted-artboard routes, handler-decline fallthrough, claimed-gesture latching, quiet timeout and collapse/cancel behavior. Add the live Lua ScrollEvent constructor, readonly fields, hit claim, global atoms and artboard method; anchor pointer/scroll userdata across protected calls. Preserve the legacy 21-bit serialized method default while adding opt-in bit 21. Wheel-disabled constraints now decline inertia cancellation. Port all six scripting tests and the runtime scroll regression; add a supplemental constructor C-string-boundary regression after source review. Source-equivalence and separate Rust-integration reviews are clean. The approved live luaur implementation owns Lua changes; excluded historical Lua mirrors are not a second execution backend. Guest Wasm VM calls, ABI/IDL/generated imports and browser guest forwarding remain parked; private editor-preview changes absent from the public delta are not claimed imported. Validation is recorded above and in the PR. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `7aa93402a27c800db8a36acc8672612c100ea9b1` | Translate monotonic trigger/change sequences, per-state-machine windows and per-layer consumption, listener baselines and inactive ancestor markers; remove frame-tail view-model resets and detached Lua tracking. Translate component-trigger pending state, text-input caret/hit/undo corrections, component-list replacement/layout invalidation and clone changes. Add paint shader transforms across runtime/render API, GPU matrices, deferred commands, SRIV opcode 44, SVG, Lua, renderer FFI and C ABI; preserve optional callback prefix handling. Translate stroke-cap full-width spokes and regenerate affected backend shaders. Port all 15 new trigger-window tests and revised text/list/render tests; capture fresh C++ Metal GMs. C++ CoreGraphics/Skia/Canvas2D/test-host adapters are unshipped. Guest Wasm console/timer/render bindings remain parked; private .rive_head metadata is not implementation evidence. Reviews and validation are recorded above and in the PR. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `160085c654874d35ad654a750e782d7c78e050d9` | Translate layout-owned virtual grid contributions from every item, solved row/column lines and gaps, cached virtual inputs/item versions, original grid-cell pinning and sparse realized-item traversal. Remove the superseded runtime track-sizing model. Preserve Taffy through native contribution sizing at the approved layout boundary; the public Yoga dependency is the behavioral reference, not a runtime C dependency. Translate linear-time list updates, inline first-parent/LazyVector dependency storage, ordered symbol entries, append/reserve creation and nesting-stack changes. Core emptyId is already an associated constant. Port revised/new upstream tests; the upstream hidden scroll benchmark remains ignored and is not a performance campaign. Private .rive_head metadata is not claimed. Reviews and applicable validation are recorded in the PR. | — |
| `dc75beed2defe9bfe53dfbc30b8649649905b602` | Translate the shared virtual layout model, wrapping lines, grid row/column windows, two-axis scrolling and carousel cycles, anchoring, per-axis physics sizes/shifts, grid placement/track observation, list realization and layout/style/text-input integration. Port the new model, sweep, alignment, component-list, anchor and physics tests and consume the new grid fixture from this pin. Retain the approved Taffy engine, exposing solved grid line offsets for the same runtime observations. C++ oracle dependency resolution follows the new public yoga.ref; private packages/yoga source is not imported. Private metadata is not claimed. Reviews and applicable validation are recorded in the PR. | — |
| `de0417920b48a6e8540314c1ec2db091ae9f23fc` | C++ header dependency fix only: write attribution now includes cstddef explicitly for size_t. The paired Rust owner already uses the built-in usize for depth and needs no import or behavior change. Advance active pins after independent source and separate integration review; private .rive_head metadata is not imported. | — |
| `7c9291ae1e49d6adc2c3e38074ba27051fe677ac` | Translate tools-only write attribution: enabled switch, thread-local ordered non-owning source stack, depth-restoring scopes, state-machine layer/listener and data-bind instrumentation, and live Luau protected calls/property writes/common methods. Preserve source identity through approved Rust ownership and delayed observer delivery; non-tools builds omit instrumentation. Wasm execution-scope instrumentation remains parked under UNIV-3728. Private CLI debugger/editor/script-asset changes described in metadata are absent publicly and not claimed. Reviews and applicable validation are recorded in the PR. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `7acbdfecbb78854c0ebaf8b7fe15bd29ebc2f3c6` | Entire public executable delta remains parked under UNIV-3728: WAMR compiler patch for FP/min-max lowering, cold trap paths and CPU scheduling; ModuleTierLadder compiler-capability probing, cache revision/tuning identity, optimized guard-page tier and supported Apple CPU tuning. No shared native/Luau/renderer owner or test changes. Private AnimaScript compiler/library/REPL/editor-completion changes described in metadata are absent publicly and not claimed. Pins advance through a reviewed deferral, not AOT execution parity. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `cd04cd3326c5b760e592079ef5dea1a81dfcd86e` | Translate ORE dynamic UBO ranges and shared setBindGroup validation (group index/identity, count, device alignment and buffer bounds), default/query alignment features, and retained pre-pipeline bind groups flushed in slot order. Wire ranges through Metal, Vulkan, WebGPU, WebGL2 and deferred groups; expose alignment and backend bind errors through live Luau. Port both upstream bookkeeping regressions and focused binding coverage. Existing sizeless backend ranges already span the remaining buffer. D3D11 shadow-buffer/11.1 offset handling and D3D12 hookup are inapplicable to unshipped backends. Wasm guest feature ABI, GPU proxy and execution bindings remain deferred under UNIV-3728. Private benchmark tier/CLI changes described in metadata are absent publicly and not claimed. Independent source and separate Rust-integration reviews and applicable validation are recorded in the PR. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `1c54a97c8411a92b824b903fe083d3ba45a204ab` | Translate Luau MeshInstances binding-owned staged data, unchanged-size resize no-op, preserved entries on resize, dirty writes and one whole-array commit immediately before a valid instanced draw; a redraw without writes does not commit. Preserve staged partial writes/error order and retained renderer data between draws. Port all three upstream regressions for commit counts/data, incremental growth and out-of-range errors. Private metadata is not imported; no Wasm guest work is introduced. Independent source and separate Rust-integration reviews and applicable validation are recorded in the PR. | — |
| `f6d5441de24fd0a3a190f4026a69a02ee62761a2` | Entire executable delta is confined to the parked WasmScriptingVM: tracked view-model/artboard handle indices, constant-time tracked-slot removal, and retained-instance indexed detached-view-model advancement safe across callback handle mutation. These guest-engine changes remain deferred under UNIV-3728. No Luau, shared native view-model, renderer or test source changes occur in the public delta. Private metadata is not imported. Pins advance through an explicitly accounted deferral after independent source and separate integration review, not an execution-parity claim. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `81a73b8108814672748bfa4b1b19fde951776ba1` | Source-verify existing zero-z construction for Vector.xy/origin, Vec2 transport, node position/scale, pointer positions and gamepad sticks. The live luaur adapter already pushes fully initialized three-component vectors; preserve that equivalent behavior and leave xyz/vector math and the unused low-level fork primitive unchanged. Port upstream's stale-stack regression with the approved typed-vector transport adaptation. Private metadata is not imported. Independent source and separate Rust-integration review and applicable validation are recorded in the PR; no new full-corpus or hardware claim. | — |
| `6cd5d108fdae8fa2d00b56eeb857754adb9b67be` | Translate optional script networking (`fetch`, byte-oriented HTTP values, request policy/limits, owner-thread settlement, cancellation/watchdog and Apple platform transport through Rust bindings) and `decodeFile` with strict signed-script admission, a lazy private VM, retained decoded files/artboards and their own view-model bindings. Networking is compiled only with `scriptnet` and no provider is installed by default; no consumer app is enabled. Translate bounded binary readers, manifest-path failure propagation, import cycle/depth/work limits, nested-artboard lifetime/stateful handling, coroutine-safe Lua property access, and shared async polling budget/pending query. Port applicable policy, fetch, Apple-provider, decoded-file, properties and malformed-import tests and pin six new fixtures. Wasm guest network/file ABI, NetWire tests, held async outcomes, write-only guest memory staging, compiler-path access and iOS prelinked AOT remain parked under UNIV-3728. Private CLI/device/emulator/build work mentioned in metadata is not present in the public delta and is not claimed. | — |
| `53419065f067f9e32ede9f4f5e28e41e3ec89f29` | Translate opt-in ORE GPU profiling state, environment seed and per-label reporting; Vulkan timestamp-query allocation, three-slot safe-frame resolution and pass begin/end writes; and render-pass label ownership across Lua, recording and replay, including the new upstream recording test. Other backends retain the toggle but publish no timing rows, as upstream. Wasm label ABI/proxy changes and WAMR linear-memory growth patch remain parked under UNIV-3728. Standalone player --gpu-profile and console-only benchmark NO_GETENV guard have no shipped Rust host counterpart; no player CLI, console support or performance campaign is claimed. Independent reviews and applicable validation are recorded in PR, with hardware limitations explicit. | — |
| `45b2f609457274d064928d2dcffe64fb6712129f` | Entire public executable delta remains deferred under UNIV-3728: Android WAMR AOT target/artifact naming, module hashes, cross-compilation cache/cancellation and rejected-artifact fallback/tier-pin preservation. Add the new exact wasm/aot_artifact owner to the existing deferred correspondence scope, without a directory-wide exemption. No shared/native Luau owner or test changes. Private linked-device CLI/publisher work mentioned in metadata is absent publicly and not claimed. Pins advance through this explicitly accounted deferral, not AOT execution parity. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `115c48623dc5de51cda9850f1ffa9263ba1dc6e7` | Translate separate print-separator dispatch (stdout-only for discrete console spans, tab in line-oriented logging); synchronize state-machine data binds once after matching listener actions; reintroduce owned font allocation decoding, mapped face selection and one-shot coverage probes; and discard abandoned/detached screen-target segments without dropping resource stream bytes. Port the new font, state-action and deferred attachment regressions. Retain COLRv1 corrections and approved Rust text/luaur boundaries. Private Flutter/Android/Dart changes described in metadata but absent from the public delta are not claimed. Reviews and validation recorded in PR. | — |
| `cdeabe75935085cea27ae93392b6af5d350d8971` | Translate the explicit revert of upstream's stale mirror: restore masked color channels, shared transition drawing, Lua print separators and nested timed callback restoration, button-aware synchronized pointer dispatch/query, pre-delete focus detachment, stroke property 470/outset/three GM scenes, SRIV opcode 43, audio release and COLRv1 corrections. Remove the owned-font/probe/collection-face APIs and associated tests removed upstream. Preserve all intervening downstream source corrections, including shared color arithmetic, root dirt, Stroke identity/EffectsContainer access, and shape ordering. Standalone player refresh selection remains inapplicable; Wasm guest-lane changes remain deferred under UNIV-3728. The persistent luaur host-budget callback remains an explicit approved adaptation. Both reviews and applicable validation are recorded in the PR; no private metadata, full-corpus or hardware claim. | — |
| `820cc19bd8405f84a8f85f9f388a133d6a2c7511` | Translate owned-buffer font decoding without copying its allocation; mapped file probes with collection face selection, nominal glyph coverage and one-shot promotion retaining the mapping; Factory/proxy and CommandServer ownership paths; and upstream font regressions. Apply the actual public reversals of low-byte color coercion, shared transition owner, Lua print separators/timed-context restoration, synchronized pointer query/button forwarding, focus detachment, stroke property/outset/GM additions, SRIV opcode 43, audio release and COLRv1 changes. Synchronized pointer calls again use the upstream primary-button default; queued dispatch still forwards buttons. Preserve existing renderer stroke-position support/opcode 42 and the approved persistent luaur host-budget boundary (its supplemental test is not C++ interrupt-restoration equivalence). Reversed standalone player refresh settings remain inapplicable; Wasm guest-lane reversals remain parked under UNIV-3728. Private changes absent from the public delta are not claimed. Independent source and separate Rust-integration reviews and applicable validation are recorded in the PR; no full-corpus/device claim. | — |
| `caeae0e723f646ee853461b61e18e04a3081fc1d` | Translate stroke position property/registry/paint propagation and outset sizing; the three source GM scenes and upstream stroke/outset tests; SRIV opcode 43 writer, replay, exact integer comparison and regressions; color-glyph gradient/clip transforms, foreground-stop refresh and clamp/sort behavior through the approved Rust text backend, with upstream tests; and audio singleton release without stopping retained engines. Independent source and separate Rust-integration reviews precede validation, recorded in the PR. Private editor/Dart viewer/SystemEnum and Windows host teardown changes, and GM golden PNGs, are absent from the public delta and are not claimed imported. No new device/full-corpus claim. | — |
| `aaa91101383b6e1aadc18fcd556f61016b46b600` | Translate FocusData's node-only detach operation and invoke it during Artboard's pre-deletion pass over valid and invalid objects, before any component can be deleted. Preserve adopted-manager lifetime and the subsequent blur/manager cleanup sequence. Port the upstream regression holding nested-artboard focus in an external manager and destroying machine before artboard. Independent source and separate Rust-integration reviews precede validation; results are recorded in the PR. Private metadata is excluded. No new hardware or full-corpus claim. | — |
| `7f45ad14e45da6320361810113a23da73207ab0a` | Add CommandServer's synchronized button-interest query, preserving retained-wrapper lookup, access-lock release, coordinate conversion, instance locking, exact button forwarding and false for missing handles. Synchronized down/up already forward event.button from the earlier pointer port and are source-verified equivalent. Supplemental existing command-queue coverage exercises matching/nonmatching buttons, outside/missing/deleted targets and actual synchronized click dispatch. Independent source and separate Rust-integration reviews precede validation. The public delta contains no new Unreal host implementation or fixtures; private metadata is not imported. Validation is recorded in the PR; no new device/full-corpus claim. | — |
| `07311aeba0d2f57b60de97f000c92cc5a798fbbc` | Inapplicable test-player/display-host configuration: add fixed60/fixed120/variable refresh options to upstream TestingWindow backend parameters and parse/apply the Player CLI's --refresh flag. The Rust offscreen GM TestingWindow owns command-flush lifetime, not that display-selection interface; no corresponding Player CLI is shipped. No runtime, renderer, schema or fixture behavior changes, and no public platform refresh implementation is present. Active pins advance after source and separate integration scope review; shader and historical fixture identities remain unchanged. No 120 Hz/VRR support or new device coverage is claimed. | — |
| `3b4af2e20bf50e222c2cfd5a22b5d9a4e70d2b8b` | Translate Luau print separators in both correspondence and live owners: convert each argument first, then append the tab for nonfirst arguments, preserving reentrant context-buffer behavior. Port the exact upstream print regression and update existing source-derived logging assertions. The archival CPPRuntimeScriptingContext correspondence now captures/restores the outer interrupt with nested depth; its opaque LuaState is not a newly compiled execution backend. The approved live luaur VM already retains its persistent host interrupt across protected calls, verified by a supplemental nested success/caught-error regression instead of introducing another timeout engine. Independent source-equivalence and separate Rust-integration reviews cover the changed owners. Wasm thrown-message, budget/trap classification, boot-time error handling and IDL/native/web wrappers remain parked. Private compiler/test-runner changes absent from the public delta are not claimed imported. Validation is recorded in the PR; no new device or full-corpus claim. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `93d8161dacd3031f49b676d3192c038f3870e33d` | Translate low-byte masking for DataValueColor channel setters and route view-model ARGB composition through the upstream color helper. Extract the shared transition-child draw/dimension owner and delegate both mechanical and production Luau wrappers to it without changing weak ownership, renderer validation or scoped invalidation. Integration review found and corrected the separate live Lua DataValue channel setter, with the upstream three-write regression exercised through both Rust and Lua. The existing Lua coercion assertion now expects the new masked result rather than the previous channel spill. Independent source-equivalence and separate exchanged-owner Rust-integration reviews, including corrections, are clean. Eight Wasm guest/wire/IDL/browser-import owners remain deferred, including scoped path-effect nodes, pointer stamps, gamepads, artboard paths, stage-union GPU layouts, guest transitions and test-blob lookup. Compiler/LSP/docs changes mentioned only in private metadata are not claimed imported. No renderer source changed; existing shader artifact identities remain historical and unchanged. Validation results are recorded in the PR; no full-corpus or new device execution claim. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `0aadd4c65084a38dbeae3bd05814ead3f743ed77` | Remove the gamma-correction helper and conditional branches from all five shared shader owners, retaining the previously approved Metal FMA adaptation. Regenerate 70 WGSL modules, 98 Vulkan headers, 24 WebGL2 embedded inputs and 54 WebGPU GLSL/export snapshots with the pinned toolchain. Update exact artifact hashes and minified-name consumers, including the clockwise clip-texture bridge and inline WebGL feature defines. Correct a pre-existing direct translation error in the touched WebGL shader assembler: upstream uses OPTIONALLY_FLAT, not the triangle-attribute alias. Independent source-equivalence and separate Rust-integration reviews cover all corrections. Shader regeneration/reproducibility and source-authority checks pass; all 98 Vulkan modules retain identical non-debug instruction words. Serial live-Metal/Vulkan/WebGL owner tests: 827 pass, 11 ignored, four historical-PNG failures; separate image-paint and four Metal resource-shader tests pass. Clean downstream `1ca8775d` with upstream `2dbbfe18`, identical features and Metal validation flags reproduces all four failure metrics: additive blend 685 pixels/max 1, instanced mesh 1,359/max 1, stroke position 1,387/max 2, clipped stroke position 277/max 1. No pixel fixture/assertion/tolerance changed. The obsolete WGPU shader-boundary checker also fails identically on clean main, expecting removed shader-catalog creation sites. Browser wasm32 WebGPU/WebGL2/scripting compilation passes; 27 manifest and 12 correspondence/pipeline-tool tests pass, with 1,058 mirrored/ten adapted owners (structural evidence only). Runtime Golden/Silver results remain the preceding commit's runs, not reruns of this shader-only delta. Unreal-only shader environment and Android deployment-helper changes are outside shipped hosts; private engine changes mentioned by the commit message are absent from the public delta and not claimed ported. No new Android-device, browser-pixel or native-Dawn execution coverage is claimed. | — |
| `2dbbfe18a01c33139bf94e530f0812666ebaf59c` | Translate pointer-button enum/storage and listener type 155/property 468, button filtering and retained press/drag identity, cancellation pairing, nested/list dispatch and interest/occlusion queries, primary-only text/scroll interaction, and queued/synchronized server forwarding. Existing callers explicitly supply upstream's primary default; native invocation retains button while the unchanged upstream Lua event intentionally does not expose it. Add the missing animation-to-Scene projection required by the literal upstream test, preserving weak ownership and constructor borrow behavior. All 16 upstream cases/sections, the command-queue case, and scripting hit-call updates are ported. Frozen translation received independent source review and separate exchanged-owner integration review; test-only API corrections were rereviewed. Validation: 109 runtime library tests pass (two ignored), 17 pointer/wire tests, 74 existing runtime tests, 146 command-queue tests, six scripted wake/pointer tests, 21 schema/CPP-ID tests, 27 manifest tests and seven correspondence tests pass. Browser wasm32 WebGPU/WebGL2/scripting compilation passes. Both fresh pinned Golden runs cover 364 entries with no failures and existing one/two recorded divergences. Testing-enabled Silver remains 85 byte-exact, 121 unchanged-epsilon, seven recorded divergences, 17 unsupported and one known converter-fixture failure across 231 cases. Broad schema-header comparison retains 19 UNIV-3729 failures: a clean downstream `0600b1ce`/upstream `3330baec` archive reproduces identical assertions. No assertion/tolerance was weakened. Structural coverage is 1,058 mirrored/ten adapted, not behavioral proof. Generator path-separator normalization was already equivalent. Private metadata and new device/pixel coverage are not claimed. | [UNIV-3729](https://universe.basis.dev/issue/UNIV-3729) |
| `3330baec48641c87330b24268e403349f02c38fe` | Translate settled/frozen state-machine layers, exact transition eligibility, wake notifications and destruction suppression; converter advance predicates and data-context notification ordering; quiet-row classification, all five skip/verification traversals, bitset reset and current-host/generation-aware row identity, layout wake behavior and pool destruction order. Port all five settled-layer tests and the quiet-row resize test. Translation froze before independent source-equivalence and separate Rust-integration reviews; proven callback/cfg/test-invocation corrections were rereviewed. Six new tests, 108 runtime library tests (two ignored), 45 existing focused integration tests, wasm32-unknown-unknown WebGPU/WebGL2/scripting compilation, 27 manifest tests and seven structural tests pass. Both fresh pinned Golden comparisons cover 364 entries with zero failures and the existing one ordinary/two scripted divergences. Testing-enabled runtime Silver covers 231 cases: 85 byte-exact, 121 unchanged-epsilon, seven recorded divergences, 17 unsupported and the same strict-versus-contracted `data_converter_to_number` fixture failure; no new failures or weakened assertions. Structural correspondence remains 1,055 mirrored/ten adapted, not behavior proof. The public delta has no new renderer/device changes; no new hardware or pixel coverage is claimed. Private `.rive_head` metadata is not imported. | — |
| `bbf3c4d8ba7971ab3b5b453d148b5bba9ddf6eb6` | Translate four-vertex triangle-group allocation/alignment and zero the unwritten mapped tail without advancing the cursor; preserve cleanup behavior for partial mapping failures. Prefer Vulkan identity presentation transform when supported and otherwise use currentTransform, removing the downstream transfer guard that would reject that newly accepted fallback. Advertise half-float color buffers in Vulkan and WebGPU. Independent source review and separate Rust-integration review, including the integration correction, are clean. Full results and known failures are recorded above. Active oracle pins and Silver provenance advance to this source; checked-in backend snapshots match upstream. D3D backends are absent and not introduced. Private WGSL compiler bounds and Android surface-polling changes mentioned by the commit message are absent from the public delta and are not claimed imported. | — |
| `69f70e1afdd777d2af0ed97e987f46454a5fbe97` | Rename the shared ScriptModuleAsset language enum to AnimaScript while preserving serialized value one, Luau/default zero and the generated language property. Update the deserialization assertion without retaining an old-name alias. Independent source review and separate Rust-integration review are clean. All three asset tests, wasm32 WebGPU/WebGL2/scripting compilation and 26 manifest tests pass; structural correspondence remains 1,055 mirrored/ten adapted. Guest ABI-generator paths and WAMR build-help renaming belong to the parked execution lane. Private compiler/editor/repl changes are absent from the public delta and not claimed imported. Historical schema text and renderer capture provenance remain unchanged. Full Golden/Silver and hardware/browser pixels were not rerun for this naming-only change. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `6a2e3ab71656709cbc2b1c21a8fa92f130c37464` | Translate script GPU passes into the host target: target descriptors/views, hidden and resized/stale lifecycle, sampling refusal, per-frame wrapping/release, clear/preserve decisions, native Luau GPUTarget and Canvas fields, Metal/Vulkan/WebGPU/top-down-GL wrapping, and Apple/Android host publication. The shipped default-framebuffer WebGL2 host correctly remains hidden. Parallel translation was globally frozen before independent source reviews and separate Rust-integration reviews; corrections and validation metadata were rereviewed. All eight shared target and six Lua target cases are ported. Validation: 832 renderer tests pass (11 ignored) with Metal/Vulkan/WebGL2/tools serial configuration; 330 scripting tests pass (one existing ignored); 11 Apple host tests pass; Apple/Vulkan host checks, Android arm64 compilation and wasm32 WebGPU/WebGL2/scripting compilation pass. A fresh pinned C++ Metal target GM and immediate/deferred parity pass. Rust immediate/deferred bytes match exactly; the strict C++ fixture matches exactly with the two previously approved Metal FMA/coverage adaptations. Preserve the pristine capture's 864-pixel/max-one difference and both binaries' provenance; no tolerance changed. Manifest tests pass 26 and structural source tests seven (1,055 mirrored/ten adapted). Initial parallel Metal ownership-audit tests interleaved global records and failed three cases; the documented serial configuration passes. An additional combined native WebGPU test build lacks optional Dawn link symbols, so no native Dawn execution is claimed. Full Golden/Silver and Android hardware/browser pixels were not rerun for this commit. Guest Wasm/AS ABI and execution changes remain parked; absent D3D backends and private host/compiler changes are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `7e450bdcb03addcfc144c4f46a393f6a9b11f7d6` | Translate wheel/trackpad ScrollEvent dispatch, nested/list hit routing and occlusion, stable gesture latches, content-space delta mapping, wheel/precise/momentum physics and velocity priming, idle/cancel/overlapping-drag behavior, and scrollFlags/wheelInteractive properties 466/467. Add narrow forwarding through the existing Rust host state-machine boundary. Port all 25 upstream cases. Parallel translation was globally frozen before independent source reviews, followed by separate Rust-integration reviews; all clean after re-reviewed test-only Approx and replacement-physics lifecycle corrections. All 33 scroll/physics tests and 20 schema/CPP-ID checks pass; 15 scroll Silver cases pass (ten byte-exact, five existing-epsilon); wasm32 WebGPU/WebGL2/scripting compilation and 26 manifest tests pass. Structural correspondence is 1,055 mirrored/ten adapted. Stale schema inventory counts were corrected against independent HEAD/current/reconciled-def inventories without weakening assertions. Two pre-existing owner observations are tracked separately in UNIV-3806, not treated as wheel regressions. Full Golden/Silver and hardware suites were not rerun for this delta. | [UNIV-3806](https://universe.basis.dev/issue/UNIV-3806) |
| `35e4193429a1eb6340ff9081e7e260b75523f6b1` | Add explicit alpha fallback for unknown layer-mask modes and mask GPU mode bits to prevent clip-ID corruption. Artboard already forwards the sanitized mode; generated two-bit storage remains source-equivalent. Port runtime mode assertions and GPU clip-ID regression, exercising raw value 8 through the production private packing helper without constructing an invalid Rust enum. Independent source and separate Rust-integration reviews are clean. All 36 layer-mask tests pass with renderer-metal, including both regressions and ten Metal pixel comparisons; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted; corpus classifications unchanged. Private editor enum changes are absent from the public delta and not claimed imported. Full Golden/Silver and other hardware suites were not rerun. | — |
| `02989e8f70ffebff16977de5ce281327debbb45b` | Preserve eligible infinite font sizes, retain source NaN selection behavior, and cap the integer fitting search at 2^24 to prevent overflow/nontermination. Port all three regressions including nonfinite shaped-run results, capped huge fitting, and sequential infinity/NaN/huge layout measurement. Independent source and separate Rust-integration reviews are clean. All 11 text-fitting tests pass; five fitting/vertical-trim/path-trim Silver cases compare operation-exact within existing epsilon; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted. Corpus changes only pin/source-line metadata. Full Golden/Silver and GPU/hardware suites were not rerun for this delta. | — |
| `9aea75996f4d199e97a1bdbb33bcbcec6d4ded6f` | Add StateMachineInstance::selected_text through the active focus manager, returning empty without a manager. Port the full unfocused/empty/full/partial/obscured/cleared-selection regression. Independent source and separate Rust-integration reviews are clean. All 71 text-input/raw-input/focus-lifetime/semantic-focus tests pass; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted; corpus classifications unchanged. The JS overlay/proxy implementation described in the commit message is absent from the public C++ delta and is not claimed imported. Full Golden/Silver and GPU/hardware suites were not rerun for this accessor delta. | — |
| `43aa10256cc06a3e2183cfddd1cd22ee16984e17` | Preserve coincident controls during trimming with endpoint-stable exact interpolation and upstream's two-cut cubic helper; remove superseded public subdivision APIs. Port all 999 head/tail iterations of the upstream regression. Independent source and separate Rust-integration reviews are clean, including the source-backed update of two stale generic-lerp expected bits (confirmed against C++; assertions remain bit-exact). All 12 contour/trim/raw-path integration tests pass. Library tests: 90 pass, two ignored, known native-hydration failure UNIV-3788 remains. wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests 26 and correspondence tests seven pass; structural correspondence remains 1,054 mirrored/ten adapted. Corpus classifications unchanged. Full Golden/Silver and GPU/hardware suites were not rerun for this delta. | — |
| `d0a12883db7bf49918be4ab5fbc21c6bcead930b` | Share exact analytic winding measurement across authored bind vertices and deformed positions/controls; apply bind orientation at reference initialization and remove the coarse RawPath fallback. Port the complete small-curved-pose regression for both winding directions. Independent source and separate Rust-integration reviews are clean. All 21 winding/paint/trim/nested-opacity tests pass; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted; corpus classifications unchanged. Full Golden/Silver and GPU/hardware suites were not rerun. C++ test-player --present parsing and BackendParams presentation modes remain upstream oracle-only: there is no shipped Rust Artemis/player CLI counterpart, and headless recording/offscreen Metal harnesses do not present a window. | — |
| `3f612e3e2180cd74d2ecd2deec4fc86acd858a18` | **DEFERRED:** WAMR iOS/Darwin build selection, universal per-slice invoke/relocation shims and ModuleTierLadder popen/pclose compiler probe remain with parked execution. **NOT APPLICABLE TO RUST:** the universal Premake archive freshness rule belongs to upstream C++ builds; our Apple framework builds use per-target Cargo archives then separate lipo outputs, not in-place ar on a fat archive. Native Luau/shared runtime/renderers have no public semantic delta. EVI touch, rotation, simulator and Android page-alignment changes described by the message have no corresponding player source in this public delta and are not claimed imported. Active source/validation pins remain ea030d1b; no new executable validation claim. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `ea030d1b10a1ff3fa153c976d59729dd25959d38` | Rename Skin's accessor to bind_transform without an alias; share the source orientation test and measure exact cubic area in local bind space with determinant orientation. Preserve source arithmetic order, unordered min/max behavior and pose fallback. Port the lens/CubicWeight/size fixture and both new curved-path regressions, retaining both winding directions and existing assertions. Independent source and separate Rust-integration reviews are clean. All 20 winding/paint/trim/nested-opacity tests pass; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted; corpus classifications unchanged. Full Golden/Silver and GPU/hardware suites were not rerun. Private Dart/editor changes mentioned in the message are absent from the public delta and are not claimed imported. | — |
| `167932b72bc6e08791057dfa04d3c18aa51c7f32` | Carry actual listener type and host timestamp through the scripted drawable/native Luau pointer dispatch; force Exit for an occluded target while preserving zero previous-position, pointer narrowing, hit result and wake behavior. Port the complete upstream script/four-event regression through the live owner route, retaining the exact pointer-only method mask. Independent source and separate Rust-integration reviews are clean. Scripting library: 324 pass/one existing ignored; 17 listener/wake integration tests pass including the new case; wasm32 WebGPU/WebGL2/scripting compilation passes. Manifest tests pass 26; structural correspondence remains 1,054 mirrored/ten adapted; no corpus classification changes. Full Golden/Silver and hardware/pixel suites were not rerun. Wasm VM signature acceptance (upstream still discards the new fields at its guest ABI) remains deferred with execution. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `8ff564a3fcd233a316ac8c827b8aaf82674c5b80` | Measure skinned path reference winding from authored vertices/cubic controls through the bind-world transform before following bone mirroring. Preserve collapsed/mixed-pose fallback, area threshold, invalidation and the pinned fold-without-mirroring limitation. Add the complete folded-first-frame regression for both authored orientations. Independent source and separate Rust-integration reviews are clean. All 18 skinned-winding/paint/trim/nested-opacity tests pass; wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26. Structural correspondence remains 1,054 mirrored/ten adapted; no Silver classification changes. Full Golden/Silver, GPU pixels and hardware coverage were not rerun for this narrow source fix. | — |
| `9b958d4703c905c90cb6f311c2a7365d9877d230` | **DEFERRED WASM EXECUTION:** Remove AOT memory pregrowth at instantiation/tier swap and install WAMR on-demand memory-growth failure diagnostics. The complete public delta touches only the Wasm VM pair and private editor revision metadata; no shared runtime, native Luau or browser-renderer changes. Active source/validation pins remain at 75a22f94; this accounted checkpoint is not implemented Wasm parity. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `75a22f94f7d744020868662d9bdbeeae97c40319` | Translate BlendAccumulator frame-tagged mixing/reset seeding; component-list indexed rows, duplicate identity/pooling and focus synchronization; cached layout display-none and paint image/fill classification; indexed view-model advancement with tools retention. Port all five upstream regressions. Source and separate Rust-integration reviews correct synchronous trigger/nested-property callback borrow boundaries and all live advance/fire routes; five supplemental mutation cases preserve upstream shifted-successor behavior and self-bound removal. Reviews and correction rereviews are clean. Validation: 45 tools tests, 40 non-tools tests and 19 related tests pass; wasm32 WebGPU/WebGL2/scripting compilation passes. Manifest tests pass 26, source-correspondence tests seven; 1,054 mirrored and ten adapted owners. Full runtime Silver: 95 byte-exact, 108 operation-exact within unchanged epsilon, seven recorded divergences, 20 unsupported, and the unchanged stateful_multi_property failure at frame 16/op 1061 (UNIV-3751). Three changed blend recordings are byte-exact; rewards_demo retains its recorded frame 0/op 1461 divergence. Ordinary/scripted Golden each process 364 entries with identical previous failure lines: three focus/semantic failures (UNIV-3793), plus scripted converter teardown abort (UNIV-3795). No corpus assertion weakened. Upstream ArtboardFrame benchmarks remain oracle-only; no timing gain or new hardware/browser-pixel coverage is claimed. | [UNIV-3751](https://universe.basis.dev/issue/UNIV-3751) |
| `02e99cf4da135f7451a3cb87a8b0f935a5e9ad31` | Remember effects skipped by transparent, unclipped paints and re-add nonrecursive Path dirt when they show; reset the transient flag when effects run and on fresh/clone construction. Port the complete new electrified-button skinned-stroke hover regression against its new upstream fixture. Independent source and separate Rust-integration reviews are clean. All 17 fade/trim/skinned-winding/nested-opacity tests pass, including the new regression; wasm32 WebGPU/WebGL2/scripting compilation passes. Manifest tests pass 26; structural source coverage remains complete with no Silver classification changes. Golden, full Silver, browser pixels and hardware suites were not rerun for this narrow change. WAMR v128 register-marshalling, SIMD handler-dispatch and shuffle-table patches remain parked; rasc/compiler/sample/docs changes described only in the message are absent from the public delta and not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `db31ea87714cf6a4cba56dd5de030f941f7e35f8` | Translate native Luau Font/Text, retained line/glyph views, visual-order iteration, caret/hit-test/selection APIs, scoped typed image/blob/font lookup, shared RawText layout/revision/style changes and safe RawPath self-append. Preserve native Rust font/paint identity and source-less synthetic host adapters only at that existing boundary. Port all ten native scripting cases, three RawText cases and the RawPath regression. Source review corrects embedded-NUL/permissive-byte decoding, strict Lua field/namecall errors, unsigned conversion and tag-validation ordering; separate integration review corrects enum dispatch and VM callback retention. Correction re-reviews are clean. Validation: 102 related runtime tests; 12 text and four path tests; 324 scripting library tests (one existing ignored), plus six renderer/path regressions pass. All three new native Luau text SRIV comparisons and draw-balance assertions pass; promote their existing corpus classifications without claiming generic replay or AS execution. Browser wasm32 WebGPU/WebGL2/scripting compilation passes; manifest tests pass 26 and source-correspondence tests seven (1,053 mirrored, ten adapted owners). Ordinary/scripted Golden each process 364 entries with exactly the prior three focus/semantic failures (UNIV-3793), plus the prior scripted converter teardown abort (UNIV-3795). Full runtime Silver, new browser pixels and hardware renderer coverage were not rerun for this text-binding change. Wasm font/text ABI, VM, generated bridges and AS execution remain parked. Private editor/sample/compiler changes described only in the commit message are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `955d6a05f960be0b6003d6856656231973eafa94` | Translate single-pass layout transforms/constraints, radius-only path invalidation, unchanged local-path retention, Artboard-only tween solve invalidation, opacity-only geometry gates, shared scroll children and clipping-list/path copy removal. Text already borrows all four changed run slices. Translate the flat bindable-clone table using stable arena-slot addresses with generation-aware equality. Port all nine new regression cases; both independent review passes and correction re-reviews are clean. All nine cases, 79 related regressions and the feature-gated layout matrix pass. Runtime library: 90 pass, two ignored, prior hydration failure UNIV-3788 remains. Renderer: 816 pass/11 ignored; required-live Metal: 211 pass/three ignored. Native/default and wasm32 WebGPU/WebGL2 checks pass. Final Silver sweep: 95 byte-exact, 108 operation-exact within unchanged epsilon, seven recorded divergences, 20 unsupported, one failure. Promote both newly exact grid/stack cases with stricter Rust/Python assertions; generator tests pass 26, grid tests pass five/one existing ignored. `animated_clipping-layout` is now byte-exact. `stateful_multi_property` still fails on the first clipping frame (16), now at operation 1061, expected drawPath/got makeRenderPath; focused review finds no new source mismatch but identical root cause is not proven. Ordinary/scripted Golden each process 364 entries, retaining the same three focus/semantic failures (UNIV-3793), plus the scripted converter teardown abort (UNIV-3795). Correct the structural check's missing mapping to the existing shared layer-mask enum; 1,054 mirrored owners and bounded exceptions pass. Upstream-only ArtboardFrame benchmark extensions remain in the oracle; no upstream timing gains are claimed measured here. Private editor revision metadata is not imported. No new browser pixel or Vulkan/Android hardware result is claimed. | [UNIV-3751](https://universe.basis.dev/issue/UNIV-3751) |
| `8398db3199cea4cd3eba53747aac562b5c0df3da` | Translate alpha/inverted-alpha/luminance/inverted-luminance layer masks, source suppression and nested/stacked brackets, invalidation, measured/custom painted bounds, shared offscreen sizing/composition and allocation hysteresis. Add type 154 and properties 455–465, shared renderer capability/paint packing, nested deferred canvas routing, image dependencies, SRIV operation 42 and native/oracle forwarding. Preserve upstream raster-ordering-only support and unmasked degradation in unsupported modes. Regenerate supported shaders, retaining existing Metal precision adaptations. Both independent review passes and correction re-reviews are clean. Validation: 816 renderer tests pass (11 ignored), including all 24 mask runtime cases and ten pixel cases; required-live Metal passes 211 tests (3 ignored); all 29 new bounds/raster cases pass; default, wasm32 WebGPU/WebGL2 and host C API checks pass; shader regeneration is exact (70 WGSL/60 headers); schema metadata and ID-dispatch checks pass. All 231 runtime Silver case invocations match the clean parent's outcome, with the same two failures (UNIV-3734/3751). Ordinary Golden processes 364 entries with three parent-reproduced focus/semantic failures (UNIV-3793); scripted Golden adds a converter teardown panic also reproduced in parent Rust (UNIV-3795). Runtime unit tests retain the prior hydration failure (UNIV-3788); broad schema-header checks retain the same 19 parent failures (UNIV-3729). No assertion or corpus classification was relaxed. Browser pixels and Vulkan/Android hardware were not run. The upstream-only Golden redundancy analyzer remains in the source oracle; the Rust generic command decoder understands the new payload. Private editor changes are not claimed imported. | [UNIV-3793](https://universe.basis.dev/issue/UNIV-3793), [UNIV-3795](https://universe.basis.dev/issue/UNIV-3795) |
| `c14cb2510071bd4cfa08d52ba5cd44d98c362237` | Replace depth/stencil fill instancing with repeated uint16 index patterns and vertex-ID-derived attributes in Vulkan, WebGPU and WebGL2. Mirror shared range chunking, signed base-vertex flags, cached Vulkan buffer bindings, attribute-free fill pipelines and the 15-slot shader specialization layout; preserve instancing for strokes and existing Metal precision adaptations. Translate all twelve range-chunker cases and StackVector empty coverage. Regenerate supported shader artifacts from the pinned source. Propagate the new flag/key layout and generated entry-point names through the native Metal adapter. Both independent review passes and correction re-reviews are clean. Validation: 782 renderer tests pass (11 ignored), all twelve range cases pass, required live Metal passes 177 tests including 39 GMs (3 ignored), default and wasm32 WebGPU/WebGL2 checks pass, and shader regeneration reproduces all 70 WGSL modules and 60 upstream headers exactly. Vulkan/Android hardware and browser pixel parity were not run for this commit. D3D11/D3D12, Unreal and the standalone shader-hotload host do not ship here; supported Vulkan hotload ordering is translated. The retrofitcubictristrips GM symbol rename remains in the upstream oracle because no corresponding Rust GM host exists. | — |
| `0d504972edf5253a9bd6e172facc9483719496f1` | **DEFERRED WASM EXECUTION:** Wasm host-string/name caches, missing-data initialization retry, zero-backed guest canvases, matrix natives, WAMR direct/leaf calls and interpreter fusion, bounded-module watchdog/trap handling and AOT interrupt capability/cache updates. All public changes are in guest execution, its ABI or engine tooling; shared runtime, Luau and browser renderer owners are unchanged. Private rasc compiler/tests mentioned in the message are absent from the public delta and are not claimed imported. | UNIV-3728 |
| `abf676e79e616e003893dc941471c90ccbaf5c6e` | Feed listener invocation values into the action's typed bindable property before the existing source-binding/converter path. Preserve pointer positions/deltas, key/text/focus values, complete gamepad snapshots, zero for missing indices, button-mask bounds, changed-value selection and skip-on-type/event mismatch. Add serialized uint8 properties 453/454 with zero defaults and schema/registry dispatch. Port all seven upstream input-value regressions and retain the exact binary fixture and RML source provenance. | — |
| `6916d763f75dfb7a297c4ad9624943839f7263b0` | **DEFERRED WASM EXECUTION:** distinguish byte counts from element counts in the guest ABI generator, validate the full native element-counted buffer through WAMR, and correct six web guest-buffer staging sizes. The complete public delta changes no shared runtime, Luau or browser renderer implementation. Do not claim these guest-execution safety fixes as implemented in the Rust runtime. | UNIV-3728 |
| `73d678e2f61cf0182eb2faef18d5a44728fd0480` | Avoid re-solving participant layout when rebuilt paths leave intrinsic bounds unchanged: invalidate cached host bounds before measuring, compare against the last reported bounds with upstream's 16-ULP-scaled tolerance, and dirty only world bounds for rigid movement. Retain Taffy and carry this decision through Rust's existing borrow-release handoff. Defer paint-path bounds computation until a PaintImage child exists; preserve existing reference-based straight-vertex access. Port the five frame-probe cases, including two hidden reporting sweeps, with test-only layout-pass counting. The new standalone C++ allocation-counting frame benchmark and beforeRun host hook remain available in the upstream oracle, not claimed as a newly shipped Rust benchmark runner. | — |
| `0515bfec7724f34ebc674e63ad3a64b40755d4da` | Route recording Image:view by canvas provenance first, deferred image identity second, and registered foreign image last. Register sampled canvases even when not opened in the current frame; preserve the flag on foreign image handles, resolve through the registry at replay and invoke the host preparation hook before wrapping. Translate both upstream deferred-import regressions and changed callers. Existing Rust canvas constructors already retain source-canvas provenance and are preserved. **DEFERRED:** Wasm module and scripting-VM dispatch changes remain parked with guest execution; shared ORE/deferred and Luau behavior is included. Shader artifacts remain unchanged with their actual prior provenance. | UNIV-3728 (Wasm slice only) |
| `3b615b829a58b67379f9304a161b3e87119bbf04` | Pack tangent angle and miter ratio into tessellation output and unpack them in the shared vertex shader, replacing the extra join texture fetch and trigonometric recalculation. Preserve upstream's distinct 65536 cyclic-angle and 65535 ratio quantization, rounding and join branches. Regenerate supported backend shader artifacts. The public delta also disables two OOM-prone D3D12 factories in upstream's standalone rendering-test host; that host is not shipped, and product WebGPU/Dawn backends are not disabled. | — |
| `4921ab8169d2d8bfc4d2d25761b99689bc85a72d` | Translate mutable image-mesh instances, per-instance transform/opacity/additiveness/UV data, retained resource lifecycle, efficient Rive draws and atomic-mode expansion. Carry instances through deferred snapshots/replay, SRIV opcodes 39–41, Luau and live renderer wrappers; retain upstream's UV limitation only for generic non-Rive fallback. Add the upstream deferred, batching, serialized replay and mesh GM cases. Regenerate supported shader artifacts and update the Metal indexed instance count. **DEFERRED:** Wasm guest bindings, IDL and execution proxies remain parked; browser renderer compilation is not deferred. Standalone C++ benchmark command walkers remain in the upstream oracle, not claimed as a new Rust benchmark host. | UNIV-3728 (Wasm slice only) |
| `10b048fba81b6218101ae6fda7cdc03e110a7b32` | Pass the host's explicit sample count, raster-ordering disable and clockwise-fill selection into deferred canvas frames, with a per-canvas override; retain default mode for direct canvas calls. Wire the shared exact-source and Metal adapters plus existing Metal GM/Apple and Android Vulkan replay hosts, including canvas-first replay. Port both null-device mode regressions. Warn once when an unspecified mode falls back to depth/stencil with four samples, using Android WARN logcat or stderr elsewhere. Standalone upstream test-window implementations are not shipped; their selected-mode propagation is applied at the corresponding live Rust host boundaries, not claimed as new host support. The private Artemis Vulkan 1.3 device change named in the title is absent from the public delta. | — |
| `d59ea0d564d11c5eb1685b3e719a94e12a605881` | **INAPPLICABLE TEST-PLAYER HOST:** adds an explicit --fit option, defaults styled artboards to layout fit, resizes them to the rotated window dimensions with a zero-time advance, and uses the selected fit for drawing and pointer-coordinate alignment. The Rust render-only GM/window adapters do not ship this interactive Player. No shared runtime/layout algorithm, fixture or assertion changes; do not impose this test-host fit policy on product APIs or claim new player functionality. | — |
| `f415694b7737e03b8b1aa93f9756645c30b50a6d` | **INAPPLICABLE UPSTREAM IMAGE-DIFF REPORT HOST:** classify image-size mismatches before generic OpenCV failures, carry both dimensions into a dedicated HTML section, and update status parsing, sorting, CSV totals, cleanup and failure accounting. The Rust pixel/golden comparison tools have independent contracts and neither ship nor invoke this Python reporter; no shared runtime, renderer, fixture or assertion changes. Upstream reporting remains available through the advanced pin, not claimed implemented in Rust. | — |
| `57dddb3727306e284773ec20c653cf686c45abee` | Pack 15-bit z indices and 8-bit coverage into normalized depth, pass z indices explicitly through sorted draw emission, and use greater-than depth testing with zero clears. Translate shader/constants and SIMD changes, new exhaustive depth-packing tests and swizzle assertions; regenerate supported backend artifacts. Add shared closeable deferred-canvas retirement, carry retired backings in snapshots, drain inline/idle sessions and route Luau allocation through the deferred host. Existing Rust deferred frames remain non-Send; lifetime behavior is preserved without claiming newly supported cross-thread frame replay. Wasm execution callers remain parked. C++ advanced-blend constructor/common-source harness scaffolding and the signature-only retrofitcubictristrips GM change have no corresponding Rust test host; they remain in the upstream oracle, not newly ported Rust tests. | UNIV-3728 (Wasm slice only) |
| `696345630f860112d0496e0d1844e9d7087250d8` | Preserve a separating space before an identifier after a closing parenthesis in the shader minifier, matching upstream for compact and human-readable output. Update the executable Rust generator and pinned Python source authority, regenerate affected backend artifacts and retain build-time generation for Metal. This is generator behavior, not a runtime shader compiler dependency. | — |
| `836a74261a9e843f37ccb4a29a9d8567915d4598` | Resolve non-virtualized list-row artboards directly and return empty bounds when absent, preserving the existing Rust null-safe behavior while matching upstream lookup order. Port the complete orphan-row regression: replace Buttons with three unbound view-model items, retain three layout/scroll rows with empty bounds, and resolve a scroll index without moving either offset. | — |
| `27e2adac5ddc23422f7a9ba4ebea187198b218c8` | **DEFERRED WASM EXECUTION:** adds modulate_opacity to the Wasm scripting ABI/IDL, native and web wrappers, and guest import table, forwarding resolved renderer handles to existing modulation. The complete public delta changes no native Luau or shared renderer implementation. Private translator/gamepad/custom-property changes named in the message are absent from the public delta; neither those nor the parked execution bindings are claimed implemented. | UNIV-3728 |
| `a7efbbe733cf9975fec6e614818ea80e02f7e234` | **INAPPLICABLE TEST-PLAYER HOST:** introduces a one-frame render-thread slot, draining/joining and gated --threaded option in upstream's interactive Player, plus a default-false TestingWindow capability. Rust render-only GM/window adapters do not ship that Player or input/presentation scheduler. The complete public delta changes no shared deferred replay/runtime algorithm, fixture or assertion. Do not impose this test-host scheduling policy on application-owned renderers or claim new threaded player functionality. | — |
| `330e78b69a7b5b68e7235aa9da2e932433fff492` | Extract the public GL fixup parser and encoder, preserving append-on-partial-failure, raw name/kind bytes, little-endian u16 casts and complete encoded payloads. Descriptor application delegates to the parser with checked Rust slice bounds. The public delta contains no editor stage-aggregation callers or row-count rejection implementation; those private changes are not claimed imported. | — |
| `3d0d3f56ba21caa2ae13ef64451526639d793aee` | Add authored additive blend mode 12 and byte strengths on Drawable/ShapePaint (source keys 450/452), preserving inheritance, property callbacks, animation/binding and GPU srcOver resolution. Carry additive paint/image state through deferred rendering, Luau, SRIV operations 36–38 and replay; retain plain-operation bytes at zero strength. Port the runtime/GPU/scripted paint cases, serialized replay additions and the complete 64-draw Silver producer. Private editor/native/Unity changes described in the message are absent from the public delta; unshipped CoreGraphics/Skia renderers are not imported. | — |
| `1a360ac87346e89cff18335a140bc8c686a54896` | **INAPPLICABLE TEST-PLAYER HOST:** the public delta adds gamepad event payloads and dispatch/encoding to upstream's interactive TestingWindow/Player. The Rust GM/window adapters are render-only and have no matching input loop. Keep the existing runtime wire receiver and Silver encoder unchanged; this is not a claim of new device polling or product gamepad integration. The private Artemis polling implementation named in the message is absent from the public delta. | — |
| `1988fdd490cc7a0b88992bd7bc9b27f7c567ba62` | Always initialize the atomic image-rectangle gradient varying to zero when no gradient is present. Translate the exact shared shader branch and regenerate its supported backend artifacts; retain existing shader/compiler boundaries and a Naga-free runtime. The public delta contains no Unity host implementation change. | — |
| `36c52f14b18c20c44c45b96b6ea13218f874bf3c` | Route GM-host canvas flushes through the host-owned Metal command buffer, preserving lazy creation before a screen frame, asynchronous per-flush commit and successor ownership across frame begins. Keep generic canvas completion and ORE frame behavior unchanged. Terminal readback retains the existing Rust shared-texture completion/getBytes adapter, not upstream's private-texture blit implementation; the host still commits and releases its successor. The private Artemis frame-number/swapchain changes described in the message are absent from the public delta and are not claimed imported. | — |
| `ed2c2dcb81d03ecfaef83962b6bdbf4572d1356c` | **ALREADY-EQUIVALENT PARITY TESTS:** upstream applies deterministic triangulation thresholds to immediate and deferred test frames. Both Rust paths already reach the shared observing NullBackend frame descriptor with an infinite frame budget and matching default geometry guards. No production policy, assertions or fixtures change; interactive rendering keeps its own budgeting policy. | — |
| `794f432ad65116b50d585ad4bff8a6813e3e38ee` | Add synchronized queue/server focus traversal with resulting focus state, key input and directional focus. Capture traversal and state under one retained machine lock, preserve FIFO callback dispatch and missing-target defaults, and make the existing server boolean traversal methods delegate to the result methods. Translate all nine upstream regression cases and every section/direction variant using a real server thread; keep Rust focus nodes on their owning thread. | — |
| `f3e99df8ea3c9460ca1fd8f64e26115f1979742a` | Subtract the probe shape's vertical trim from content height when fitting into a non-fixed text box, preserving authored fixed-box sizing and the existing integer-size search. Translate the full short-run hug regression, checking actual fitted glyph-run size and both layout heights against upstream expectations. Existing Rust shaping/Taffy boundaries are unchanged; private editor changes are absent from the public delta. | — |
| `8f3690df4346c8b8b9072bc5c3de6f3ab0c37203` | Skip hidden nested artboards and component lists in all three positional hit/event paths, retaining collapsed/paused gates and unchanged gamepad behavior. Translate both overlapping-target regressions with the exact upstream fixture and lifecycle order. The public delta contains no Dart editor implementation; no private editor change is claimed imported. | — |
| `fa3f6f307179dcfff9ff34142e1697c837ca1704` | **ALREADY-EQUIVALENT RUNTIME / NEW REGRESSIONS:** the four directional StateMachineInstance focus methods already forward to the active manager and return false without one in both Rust runtime and host owners. Port all three added upstream cases, including the three independent position-test sections, edge retention, external-manager identity and absent-manager behavior. The public delta contains no JS binding or key/text-input implementation change despite the title; no absent private change is claimed imported. | — |
| `3ae88a8257e44c5d6239e6181240560f9137c681` | **DEFERRED WASM EXECUTION:** restores WAMR libc-wasi sources, include paths and feature flag so guest Luau constructors remain host-controlled instead of also running at instantiation. This reverses the parked 8daed0a3 build change. Only the WAMR build owner and private revision pointer change; no native Rust Luau, shared runtime or browser renderer change is imported. | UNIV-3728 |
| `80e2e555349e88f21977c429d5e29c6e57454302` | **UPSTREAM PATHFIDDLE HOST ONLY:** adds Shift+R context recreation, resets cached window dimensions, releases/waits on the D3D12 host at teardown, and makes its GPU-based validation opt-in. No PathFiddle or D3D12 host ships in the Rust workspace; the full delta changes no shared renderer algorithms, supported Rust backend owners, shaders, tests or fixtures. Do not transplant test-host context reset policy into application-owned renderers. | — |
| `17a44957bcdbc471e96fd979d8770271da2ffb43` | **DEFERRED WASM EXECUTION:** supplies environ_sizes_get, environ_get and fd_close imports to the WAMR host, validating both output pointers before writing environment counts. The Luau blob here runs inside the guest Wasm module; this does not change the native Rust Luau backend, shared runtime or browser renderers. The complete public delta remains within parked execution. | UNIV-3728 |
| `f01e9e26284d27090840e6c247c4955f3cf9cf71` | Retain composed local paths across rigid transforms by comparing ordered geometry versions, visibility and relative transforms against the last built snapshot, using upstream's separate linear/translation tolerances. Keep world-path rebuilding unchanged. Port all three path-composer regressions, including the twelve-file, 120-frame comparison; release Rust occurrence borrows across synchronous forced-rebuild callbacks without suppressing dirt. Consume all 71 modified upstream Silver baselines (the commit message says 69). The unchanged comparator now accepts juice; promote it to exact. Two removed commands shift the existing data-binding failure from operation 582 to 580. The known stateful_multi_property clipping failure remains red under UNIV-3751, without relaxing its exact classification. The macOS-only CoreGraphics visual-diff diagnostic remains in the pinned C++ test tooling, not the shipped Rust runtime. | [UNIV-3751](https://universe.basis.dev/issue/UNIV-3751) |
| `af39db759ecd48fe78dfee4fff59a918368181c6` | **UPSTREAM TEST-DEPLOYMENT ONLY:** adds Unreal iOS package/device launch support, relocates Unreal installation, and avoids forcing window resolution on the Unreal player. No Unreal host or these deployment scripts ship in the Rust workspace, and no runtime, renderer, shader, fixture or assertion changed. Existing native Apple host APIs are unaffected. | — |
| `8daed0a3dfc437bf18980244bb95222f2a32a9aa` | **DEFERRED WASM EXECUTION:** disables WAMR libc-wasi and removes its source/include paths. The full public delta is confined to the parked WAMR scripting backend and private source pointer; no Rust Luau, shared runtime or browser-renderer change is imported. | UNIV-3728 |
| `93e4ce468f0aadff5622e960baa16e6c87e0d5fb` | Cache skinned-path and layout-participant lookups at registration, coalesce path/composer shape notifications per dirt cycle, and skip hidden stroke-effect measurement except for clip sources. Preserve active Rust owner borrows through explicit registration witnesses and notification extraction. Port the complete hidden-stroke visibility regression. Upstream private-editor rescans are not enabled by the distinct Rust tools feature. | — |
| `af45192d7335c988bb7fe34ef2e9f8413e9b7a85` | Compute transformed precise RawPath bounds directly by mapping points before solving curve extrema; preserve identity-call behavior and transform-before-interpolation order for quadratics. Use the new method for built and provisional shape paths without copying them. Port the complete upstream raw-path bounds case, including its transformed-vs-copy regression. | — |
| `074bfb139e5390760e04a9fd953d410f71fa6384` | Collapse a leftward text selection from its leading edge and suppress all nested/list data-bind work while the host is collapsed, retaining queued source dirt. Port the full cursor and nested-binding regressions and three-draw collapsed-binding Silver producer. The inventory scripted reference uses upstream's updated bytes; its preexisting pending classification is unchanged. Private RML exporter changes described in the message are absent from the public delta. | — |
| `7133967b25572642eb228f29254e64ed0b626469` | Measure shape control-point hulls directly without copying paths, compose the optional transform after each path transform, and memoize local bounds with geometry, transform and collapse invalidation. Preserve separate precise intrinsic measurement. Port the complete upstream local-bounds mutation regression. | — |
| `f0ec99212cc7bbc1d4755c22794eff81d103319b` | Translate hugUnbounded sizing property 451, per-hug-axis available-space unbounding, min-content probe widening and bounded participant shape measurement. Retain the approved Taffy layout boundary and distinguish actual min-content probes from real zero available space. Port all four new hug cases and updated upstream layout-matrix fixtures. The Yoga dependency update is reference-oracle provenance, not a new Rust runtime dependency. | — |
| `912312dc744a989ea561064bcd7291e7659e02ef` | Drain queued 2D and ORE resource destroys from idle deferred sessions and replay them without a frame, factory or renderer. Return only newly drained command tails while retaining those bytes in their streams; preserve generation-safe repeated replay. Port all four upstream idle-session/pending-destroy cases. Private editor texture-owner changes described by the commit are absent from the public delta and are not claimed imported. | — |
| `5210ce97ecc67a4358908a472007b5fa63d8fdd2` | **TEST-PLAYER INFRASTRUCTURE / ALREADY EQUIVALENT:** upstream adds a UIKit app, touch/lifecycle queue, CAMetalLayer TestingWindow and Xcode player wiring, none of which are shipped Rust runtime owners. The cross-platform player fallback (default state machine, first state machine, then first animation) already exists in nux-capi. Rust Apple hosts retain caller-owned layer/drawable/scheduling; the native Metal product-root harness already supplies an autorelease pool. No shared runtime/renderer algorithms, shaders, fixtures or regression assertions changed. | — |
| `a637bc5ea397ca33a74b367db3bc09e1e6effcfc` | Cache Lua context/data-context wrappers by target identity, release them with their context, preserve resolved properties' owning instances across reference swaps, and distinguish untagged orphan cleanup from owner-tag and full teardown. Re-mint disposed nested view-model properties. Port all three context-cache and five property-lifetime cases. The instance-value owner getter is already available without tools. Private Dart lazy-init/transactional-claim/editor entry-point changes described in the message are absent from the public delta and are not claimed imported. | — |
| `cb8aa75d06b0341deaf5a55171844f37bf1ac882` | Blend IK from the pose currently on its chain rather than authored bone angles. Track each link's base rotation and solved local transform, distinguish our own previous solve under the current parent from intervening constraint work, and rebuild only from the first self-owned link. Preserve link history on repeated clean lifecycle; correct the touched solver's cosine clamp NaN selection to match C++ min/max. Port both upstream stacked-IK and rotation-constraint regressions with exact fixtures. | — |
| `2b570b9e4f519c8105fce3d3fe69558926b1aa7d` | Wait for prior vertex/fragment shader and transfer reads before Vulkan ORE attachment clears, and include late fragment tests in both attachment-stage dependencies. Preserve access masks and render-pass ownership. The live Rust hosts already separate ORE teardown from screen-frame submission/presentation; the three changed upstream fiddle/testing-window classes have no direct Rust counterpart. | — |
| `5f2c56090986c9ca8c079413ed317257bf60b3cf` | **ALREADY EQUIVALENT:** the public delta introduces a U8 macro to reinterpret C++20 char8_t test literals as char on MSVC. Rust String/str literals in the translated command-queue tests already preserve the same UTF-8 payload and expected bytes (`héllo 日本 😀`, `é`, `hellé`); no runtime or test assertion change is needed. The only other public change is the private editor revision pointer. | — |
| `ead028268df05ee29a6703eeee7a0c2ed0dc245b` | Expose existing state-machine keyboard and committed-text dispatch through the command queue. Preserve owned text payloads, key modifiers/press/repeat flags, request IDs, invalid/deleted-handle errors and asynchronous handled results delivered to global then per-state-machine listeners. Port all four upstream input cases, including both invalid-handle variants and real text-field editing, within the approved Rust queue/server execution boundary. | — |
| `df0cc777d8211f81824943a7665884f5565aecb8` | Tag drawables with custom properties using generated name key 449; carry file-scoped property lookup, reentrant draw visitors and floating nested modulation levels through native runtime and Luau. Translate renderer color modulation, gradient cache identity, image/mesh packing, deferred commands and serialized opcode 35; forward through native/browser adapters and recording/replay tools. Port all three runtime cases, all three renderer cases and the complete scripting visitor case with both exact fixtures. Wasm execution bindings, borrowed drawable handles, trap-save cleanup, guarded IDL generation and web imports remain parked; private editor changes named in the message are absent from the public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `1f7efd6c629383d00ab6daf4923e37e501901f48` | Measure skinned path winding, cache only noncollapsed measurements with a consistent bone orientation, and preserve the upstream folded-without-mirroring limitation. Reset on vertex changes, not bone moves. Add opt-in nearby-origin coarse area with the same zero-origin default; correct the touched owner's scalarized cubic sample guard/reciprocal order to match the source SIMD lanes. Port the raw-path regression and all eight skinned-winding cases. WITH_RIVE_EDITOR binding-reset hooks remain outside the existing shipped runtime profile; tools is not that editor profile. | — |
| `9b9cd7b185398629f10ae0131e2a53322ed4a9b4` | Focus can land on any eligible node, including containers. Translate stable pre-order traversal, independent parent flags, focus-versus-traversal eligibility, subtree edge behavior and detached-node re-entry; remove automatic leaf descent. Port the changed/new focus tests, exact new traversal fixture, two new Silver producers and updated source baselines. Wasm UTF-16 string ABI/transcoding and WAMR AOT compilation changes remain parked; private Dart generator/FFI changes described in the message are absent from the public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `ca3a5070113801702937d24f852b7cb2c8765673` | **DEFERRED:** WAMR patch 0019 and the Wasm VM add quick native/AOT dispatch, nested entries, export lookup caching and outermost-only canvas/pass cleanup. All execution changes belong to the parked Wasm lane; upstream Luau and shared renderer owners are unchanged and receive no speculative adaptation. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `a3a7a317accdba678db7522615264e0403d6f76e` | **DEFERRED:** only WAMR patch 0018 and the private editor revision change. The AArch64 AOT far-call stub tail-branches through x16 to preserve return-stack prediction; it is wholly within parked Wasm execution, with no shared runtime, renderer or Luau delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `7f4354cd0775aaabc59bfcb9892d974fd53f9f0c` | Add trusted, stable local font-file mapping on Apple/Linux/Android, retaining mapped bytes through the font and derived font lifetimes without changing ordinary byte decoding. Keep the approved Rust shaping/outline backends and their parser validation; expose the mapped-file lifetime requirement as an unsafe Rust API contract. Port all three upstream font-file tests. Private Dart/FFI editor callers described in the message are absent from the public delta. | — |
| `9e6a97ff93dff788d8e78d0d1b82da377e8c9377` | Add non-owning deferred-session attachments, idempotent registration, detach and destruction notification from a moved-out list, preserving the shared Rust session lifetime. Check the recording thread before returning a stable screen renderer. Port all four upstream host/session lifetime tests. Private native/Windows host-export changes named in the message are absent from the public delta. | — |
| `50ba2f5a581089e93592393d00b61da4ba46473e` | Preserve complete compiler diagnostics and labeled replay failure notes, clearing notes on resource reuse/destruction. Carry depth-only non-filtering sampler flags through reflected layouts, per-binding intern IDs, native Lua and WebGPU. Existing live Rust hosts already carry full frame descriptors; upstream's HostFrameSink convenience class is unshipped. Import the regenerated upstream GM shader fixture and port the complete depth-sampling scene and both new layout scenarios. D3D-only implementations and the upstream-only golden CLI's optional allow-new flag are outside the shipped profile; Wasm trap-detail and layout-POD changes remain deferred. Private shader compiler and Unreal CI changes named in the message are absent from the public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `6a717fa59880458141ac955bc00f86e80da54189` | Re-sync borrowed Rive Vulkan textures each frame, retaining their source texture and unregistering/detaching weak context links at destruction. Advertise only host-enabled Vulkan capabilities; enable supported anisotropy and depth-bias clamp in the Rust device owner. Save both valid-sized images on GM parity failure. Wasm image views, console time-zone handling, WAMR AOT extraction and Windows WAMR links remain deferred. Private Artemis window/host changes described in the message are absent from the public delta and are not claimed imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `14bdbfaf603065435cd8393cef3ca5a47b4cf80b` | Centralize texture upload and view-range validation, normalize zero extents/counts, and carry explicit upload byte sizes through native Lua and deferred recording/replay. Render into nonzero mips using view extents and Vulkan subresource ranges with pre-batch layout tracking. Port all six GPU-free range cases and the complete mip-render-target scene. The live Rust Vulkan swapchain already handles successful suboptimal results. Unshipped D3D implementations are not imported; Wasm scripting GPU proxy/VM changes remain deferred. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `bb2903fafe88ce68976eb5a4448c5542d6823e5d` | Retain failed-import occurrences through importer resolution, with cleanup on every return path, and register NestedArtboard/ScriptInputArtboard referencers only after successful import. Expose the tools registrar getter; the Rust backboard importer already initializes its file reference to None. Share the upstream RivBytes test builder and port both malformed-stream regressions. Private rive_native registrar-map ownership changes are absent from the public delta; no native binding or C++ sanitizer result is claimed imported. | — |
| `407f9a359256420d922badcbe492c76c5b4a4ba5` | Add command-server runtime messages with opaque tags and owned byte payloads, non-owning global listener registration, delivery-time listener selection, discard without a listener, and next-pass delivery for messages posted during callbacks. Port all four upstream cases, including both discard sections, through the approved Rust same-thread queue/server pump. The subscription getter is already correctly spelled; typed Rust POD storage does not reinterpret signed bytes and needs no char-to-uint8 storage change. | — |
| `2914b0817b6e6749f4f183bb7b03df38ea04dc77` | Release nested linear animation instances before their mounted artboard, including data-bound keyframe cleanup. Validate script-authored converter and path-effect return values: accept only the four DataValue kinds and Path/PathData respectively; invalid converter returns produce a successful no-value backend result (empty initial converter value; existing cache preserved), while invalid path returns retain the failure branch and contribute no geometry. Port the two nested-lifetime cases and all three scripting-return cases. Private Dart/WebAssembly BigInt binding changes described in the message are absent from the public delta and are not claimed imported. | — |
| `1371150d929cf8d282d782cb9e5b7fc7783799b1` | Reconcile ScriptedInterpolator's owned ScriptInput disposal: the existing Rust destructor already clears backlinks and retires only owned ScriptInput occurrences; expose the explicit source-shaped disposal method and route Drop through it. Preserve per-animation clone ownership and avoid a wider disposal refactor. Add the exact upstream data_bound_keyframe_test fixture retrieval and import regression, with supplemental Rust ownership checks. Upstream's C++ sanitizer/leak measurements are not claimed as local results. | — |
| `def2e241b80f6118a1d3bda8d3689e3f8bfcd430` | **OUTSIDE THE ENABLED RUNTIME PROFILE:** resolve the Luau dependency bump from rive_0_734 (`fb6ff089`) to rive_0_734_ice (`c1c57894`). The complete dependency-tree delta changes only Analysis/src/Frontend.cpp, giving concurrent type-check operations local internal-error reporters. No VM, parser, bytecode compiler or bytecode format changes occur. Static analysis is not in the shipped luaur dependency graph; record the editor-tools relevance in luau-fork.md without claiming that analyzer fix implemented. | — |
| `576282493a805993d4ca6379e425f105fe0d03d5` | Guard missing view-model IDs/properties, visit authored property graphs once, and memoize active instance copies as null before completing them: active back-edges are dropped, while completed copies remain shared. Route root/default/runtime creation through the copy entry point. Reject null import factories before parsing and missing paint-mutator artboards/factories before paint initialization. Live Rust File factories remain nonnull retained handles. Port all four cycle/ID tests and the null-factory import regression. Private editor pickers, Dart/native plugin and Windows wrapper changes described in the message are absent from the public delta and are not claimed imported. | — |
| `d7d5fdd8419b5e5932f56e8a51a62eb83c5a2d71` | Translate host-owned selection across regular Text objects, shared cursor layout views, lazy selection geometry, pointer/keyboard selection and copy ordering, nested selectable discovery and stable target tokens. Add ordered file SelectionStyle resources and generated type/property dispatch. Preserve weak Rust object registration and the approved shaping boundary; clipboard, focus and pointer capture remain host-owned. Port all 16 public upstream selection tests. Private editor inspector/RML changes described in the message are absent from the public delta and are not claimed imported. | — |
| `ee60b7014f1a28fa6bb5f2588cb274c273080f32` | Translate inside/center/outside stroke positioning, paint cloning/state, forced contour closure, positioned clip recursion and complete clip-cache identity. Carry stroke position through public and deferred APIs, preserving bitwise stroke-parameter equality and deterministic command padding. Port the three complete new GPU scenes and existing clip-stroke helper refactor. Shader sources are unchanged; unsupported renderers retain upstream's default no-op, and no serialized SRIV or foreign callback opcode is invented. The interactive path-fiddle keyboard control has no shipped Rust counterpart. | — |
| `30742b4ced290af534e00a784a4a36d4cdb1d45b` | Add command-queue font-property setting, replacement and clearing; preserve the property and report an error for an invalid nonnull font handle. Carry font metadata through subscriptions/messages and mirror the updated asset-property error text. Port all five upstream font-binding cases using existing decoded-font handles and the approved Rust same-thread queue/server pump. | — |
| `7db8b61f747b24553bc4783df50cff997610e17c` | Translate author-controlled text word breaking: historical break-word, overflowing whole words, and cluster-boundary break-all. Wire serialized/bindable property 446 through generated dispatch, shaping, fitting, measurement and invalidation; retain the approved Rust shaping/layout boundaries. Port all eight line-break regressions, the Text-object propagation case, and the 21 added layout-matrix cases. Private Dart/editor/RML/compiler wrapper changes mentioned in the message are absent from the public delta and are not claimed imported. | — |
| `5705446d6aeb0dad34a63d8ddadbb79fbe327a37` | Apply retained vertex bindings under the current ORE pipeline at draw time, release bound resources on finish, surface GL framebuffer errors, and share native Lua draw guard messages. Remove the obsolete Chrome 136 float tessellation texture workaround and regenerate supported shaders. Port the three complete vertex-order/pipeline-switch/buffer-offset scenes. Wasm VM draw guards and init-trap retention remain deferred; unshipped D3D backends and private CLI tests are not imported. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `09e85e1f63df628c0828ae0c740c1248032cadd8` | Gate elastic release velocity and snapping on a nonempty scroll range, preserving pointer-driven overscroll and settle behavior; port all three upstream release regression sections. Authoring-only threshold defaults described in the message are absent from the public delta; the runtime deserialization default remains unchanged. | — |
| `ce5a0eefbea8d75eac98dad68261f14e9d041fb0` | **DEFERRED:** Wasm module calendar/time-zone imports and native calendar/strftime bridges for AssemblyScript and the Wasm-hosted Luau blob. All executable changes belong to the parked Wasm execution lane, not our native Rust Luau engine or browser renderer. Private compiler/library/test changes named in the message are absent from this public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `69f43681303ec4e17c131c9f8d584c19f1c34290` | Translate the tessellated path vertex single-exit control flow: retain computation after discarded stroke/fill vertices and return the discard predicate at the end. Regenerate supported shader artifacts; do not claim upstream vendor performance measurements as local validation. | — |
| `b86b7ecb0256842cc37823f63c8699d5bffe081e` | Translate additive blending for supported renderers: paint/image draw inputs, clamped per-draw state, srcOver-only alpha modulation, gradient row/fraction packing and gradient-coordinate uniforms, and corresponding shader decoding. Preserve upstream defaults for unsupported renderer overloads and advanced blend modes. Port both complete additive rendering scenes; no new serialized opcode or runtime shader compiler is introduced. | — |
| `1e979c628717de902a1f3920c9d55a18bd3955ef` | Replay source dependency-order recipes when instance object/helper slots match, retaining upstream's guarded sort fallback and unsorted sentinel. Add tools-only artboard byte ranges and single-artboard replacement through the shared object reader, seeded global import state and repaired referencers. Port the complete public artboard range/replacement/order test file. Replacement retains the approved arena lifetime for externally retained view-model objects; its root keeps its host identity, but replacement-local whole-file authored IDs (and scroll snapshots requiring them) are unavailable because a partial stream cannot establish those offsets. Private editor regeneration/export changes described in the message are not present in the public delta. | — |
| `d732510cacdd4f2af470199d7fefb5177d9c118a` | Make same-instance view-model replacement a successful no-op before reparenting, dirtying, callbacks or dependent invalidation; route the by-name API through the shared by-property sequence after its type check. Port all four public regression cases, including tools callback counts and real-swap relinking. Private Dart editor/export/RML changes described in the message are absent from the public runtime delta. | — |
| `1af8ccbefdf906ea5c33e80845360c62b43bafb3` | **DEFERRED:** Wasm/AssemblyScript audio source/sound APIs, f64 frame clocks, paused-tools playback gating, node parent/children handles, asset-lookup sharing, binding generators and native/browser Wasm import bridges belong to the parked execution lane. No native Luau, shared audio/runtime or browser renderer implementation changes occur in the public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `83a90f226194b04a3a2d3344f35b9cb47168b55b` | Reject missing/non-Bone tendon references with InvalidObject in runtime loading, and preserve the bind position of a vertex lacking a Weight during deformation. The upstream native-editor recovery branch is not a shipped Rust runtime feature; private CLI diagnostics described by the commit message are absent from the published delta and are not claimed imported. | — |
| `1cfd492e4041ff7d1e35f3697bf47d9d97e6a13e` | Register list-child listeners against the newly allocated child handle rather than the parent handle. Port both upstream child-listener destruction cases (nested property and list element), retaining parent subscription callbacks across child teardown and the approved weak Rust listener lifetime. | — |
| `1cdd96e32f8a7366ed682e834a4d84def51498ed` | Translate keyboard listener claims after matched actions run, and dispatch to every peer on a FocusData before reporting the accumulated claim upward. Preserve text-input handling and the existing Rust callback-safe listener snapshot. Port both upstream peer-claim tests. Private CLI/RML changes described in the message are absent from the published runtime delta and are not claimed implemented. | — |
| `aeb29dde6845b6f31f8bb986dee0857408271922` | Preserve upstream's documentation that keyboard modifier bits are a raw RML bitmask and key enum values mirror the editor's keyboard-input definition. The published runtime delta is comments plus private editor revision metadata; no executable CLI/runtime behavior or numeric values changed. | — |
| `be0935d3ce4998dd1216ddaf83638901732a4c80` | Translate dependency sorting's append-then-reverse ordering and borrowed per-component dependent traversal. Preserve root ordering, cycle detection and partial output on failure, including the Rust active-root adapter. No change to authored dependency semantics or graph identity. | — |
| `0211d9ee727fd66473d2f073fec9c788539607c4` | Translate artboard teardown's fallback to the existing runtime audio engine when no explicit engine is assigned, matching AudioEvent playback resolution without allocating an engine. Port the upstream regression that plays two sounds without assigning an engine and requires both to stop when the artboard is destroyed. Preserve the approved Rust audio backend. | — |
| `02bea09bc68eb923498a3fe77da257a96e48d2e9` | Translate Unicode line breaking and exact generated Unicode 17 property data, glyph-cluster-safe line fitting, whitespace and shaped-run annotation changes, upstream rule tests and conformance data. Share the translated rule owner beneath the existing render-api/runtime crate boundary without replacing the approved Rust shaping engine. Active Silver inputs use this checkpoint's four changed upstream captures; historical renderer artifacts retain their true provenance. | — |
| `d85630c59d7657c336d09bf902a10595dbf6539d` | Reconcile failed typed property reads: existing Rust owner emits the lookup error and returns before value serialization, while its caller preserves later commands and the draw epilogue. Port upstream's full five-type regression covering missing/nested/wrong-type paths, valid reads, exact callback IDs, later commands and queued draws. Rust's initialized value enum already avoids the C++ uninitialized scalar payload. | — |
| `51c02b50420ceb0520f0b3193116c374c9f2b9a0` | Translate stroked clipping: shared defaulted StrokeParams, ordered paint setters, live renderer clipping and adapter forwarding, deferred command insertion/payload/replay, and depth/stencil clip-stroke masks and color-write behavior. Translate all five upstream clip-stroke GMs. Unsupported renderer implementations retain upstream's default no-op; no new serialized SRIV opcode is invented. Shader sources are unchanged and retain their actual prior generation provenance. | — |
| `2579994c59cff57ac04d3a38401fa37ad1315425` | Translate hybrid premultiplication: rounded packed solid colors, per-draw unmultiplied selection for advanced blends except the depth/stencil KHR path, and corresponding path/mesh/atomic fragment coverage and blending semantics. Regenerate affected supported-backend shader artifacts while retaining the approved Metal feather-precision adaptation and historical capture provenance. | — |
| `a4dbc3ffa50fa4e9c0346c5fdeddb4a664911cec` | Reconcile the ahead-ported BitmapCache, artboard raster/composite lifecycle, renderer state queries and deferred canvas hosts against the complete reached commit. Add opt-in serialized cache capture, canvas declaration/content brackets and replay routing, with the upstream serialized-replay and silver-factory regressions. Move the reached BitmapCache definition from forward overlay to runtime reconciliation. C++ player/deploy UI harness changes are not shipped Rust product implementations. | [UNIV-3544](https://universe.basis.dev/issue/UNIV-3544) |
| `d97479357d83d10e0b440827c7905a6994263205` | Image paints with image assets, sampler and transform properties, deferred-command and serialized replay support. Scripted transitions select among design children and lazily instanced view-model-list artboards, with scoped Lua child wrappers and changed/draw protocol callbacks. Wire generated type/property registration, data binding, lifecycle and list index propagation. **DEFERRED:** transition execution methods in WasmScriptingVM remain part of parked Wasm execution; shared protocol and native Luau are in scope. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `00943a6b29b55467e2595fe244b555293fcade68` | **DEFERRED:** WasmScriptingVM probes the major-collection counter, requests collection before warning on host-handle growth, waits for sliced collection completion, and suppresses warnings for collected modules without the counter. The complete executable delta is confined to the parked Wasm scripting VM; no ordinary Luau/shared runtime/browser renderer changes are present. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `f7c3d1f9975c4993b68fd5f0fbf568bb4607e77c` | Track Vulkan texture contents per mip/layer, including pending uploads and resolve targets. Convert load to clear for never-written attachment subresources, mark only the framebuffer's base layer, and preserve canvas/Rive texture contents through target access tracking. **DEFERRED:** the Wasm-only player frame-boundary nursery scavenging call belongs to parked Wasm execution. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `6a8cf4a3adad93a20fb496ce3910e418d47a7d02` | Add bounded state-machine hit testing for host-owned input decisions. Scripted layouts count only inside their last-known layout box after world-to-local conversion; unboxed scripted drawables retain everywhere-hit behavior. Propagate bounded queries through nested state machines and reverse-ordered artboard lists without changing ordinary pointer hit testing. Private macOS window chrome is not present in the public delta. | — |
| `19c0fad37c35405ae08bda3779be49affc532b32` | Keep Vulkan depth testing enabled when a depth attachment is present and depth writes are requested, including always-compare writes. Preserve the attachment/compare/write predicates and translate the three-overlapping-quads regression: the nearest always-write quad must occlude later lessEqual draws. | — |
| `5d7ab77e6a0fc9f91e69fd08c8b470c7d072d555` | Remove the clockwise-atomic unmultiplied-paint workaround. Return incremental coverage from stroke/fill helpers, scale the complete premultiplied color for fixed-function output, and retain alpha-only coverage followed by RGB premultiplication for the programmable blend path. Remove obsolete defines from all entry points and regenerate affected supported shader artifacts, retaining historical capture provenance and the approved Metal feather ABI adaptation. | — |
| `45d4d01dfd1fe70d3f9e73764538c16f63a04d07` | Measure fitted text at its fitted size behind both the 7.4 file feature and authored fitFontSizeResizesBox flag. Use participant-authored axis ownership before the first layout solve, retain older-file measurement, narrow generated text storage and expose property 315 throughout decoding/schema/registry/copy wiring. Translate all four upstream regressions and refresh exact layout matrix fixtures. Preserve the approved Taffy exact-width returned-slot boundary, not an override of upstream shaping width. | — |
| `814f89ac5314a72c07df3b9c998c58ece33573a8` | **DEFERRED:** Wasm debugger enter/line/leave ABI and native/web bindings, module-call/boot/detach hooks, thread-local boot context and debug tier pinning. All executable changes belong to the parked Wasm scripting VM/binding lane; no native Luau, shared runtime or browser renderer change is present. The private CLI preview debugger named in the title is not part of the public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `7732f41ef93e4cb74286934ee596e1041d0a0ba7` | Render GL canvases top-down through per-target orientation, carrying the target direction through viewport, scissor, blit, gradient/image/clip coordinates, winding and shaders. Remove the superseded GL canvas-mirror textures/registry and ORE/scripting flip paths; retain direct imported views. Translate new canvas mesh/winding regressions and affected existing DAG cases. Preserve Vulkan's compiled-atomics admission and distinct unsupported-build error. Previously unported C++ GM/test-window baseline plumbing is not claimed as a Rust implementation. | — |
| `1cc2396f0d0d3f6d9c0b16809904e85265f617eb` | Translate single-sample depth/stencil rendering across Vulkan, WebGPU and WebGL2, including direct target attachments, sample-specific resources, shader destination reads, compact shader/pipeline keys and the upstream key-collision/GL retarget regressions. Expose clockwise single-sample mode through the Rust product and replay hosts; preserve Metal's upstream-supported modes. Regenerate affected shader inputs/artifacts. C++ GLFW/EGL test-window plumbing, D3D and deployment-only changes are not Rust product implementations. | — |
| `c5ac0a130a6ff425e38721183eb1f8f05af8e54a` | **ALREADY SATISFIED AT THE RUST HOST BOUNDARY:** upstream's shared `HostFrameSink` now supplies `oreCommandBuffer()` at ORE frame begin. Rust uses backend-specific hosts: Android Vulkan opens its native frame before replay and passes its recording command buffer through `begin_frame_external`; WebGPU installs its retained command encoder via `beginFrameExternal` for normal and standalone canvas frames. The C++ shared-host class itself is not shipped. Windows viewer/D3D12 and HiDPI changes described by the title are not present in this public delta or claimed imported. | — |
| `70f3c4e63930d0e876ef24a61dbd3b891bd4ea63` | Admit `emulateDynamicColorWriteDisable` in the valid ubershader miscellaneous-flag mask for draw types that use pipeline dynamic state. Preserve capability filtering and permutation validation, fixing the upstream dynamic-state assertion without changing shader code or generated artifacts. | — |
| `d158f80d326be040d2b83d2ec04122ebcb58d9f8` | Translate `[[nodiscard]]` to `#[must_use]` on all six float-AABB transformation/rounding methods in both runtime and renderer API owners, preserving their return-value semantics. Upstream also corrects discarded offsets/insets in the unported C++ `strokes`/`xfermodes2` GMs; no corresponding Rust GM implementation or changed runtime arithmetic is present. Existing captured GM streams retain their historical provenance and are not claimed regenerated at this checkpoint. | — |
| `83acdadc2e528848066d28f771d961a7eec6dad8` | After an ORE replay resource fails to resolve, suppress all eight subsequent pipeline/buffer/bind-group and dynamic-state commands as well as draws until the next pass; retain command decoding, resource resolution and pass completion. The image-paint GM already uses the matching f32 PI constant and arithmetic. C++ check-golds/deployment/Unreal packaging and testing-window external-backend aliases have no Rust product counterpart and are not imported. | — |
| `890ed9a48355b487be471b59b2363d51f9a429ee` | **DEFERRED:** Nintendo WAMR platform/shims, prelinked-AOT registry/baker/loading and fallback, tier-ladder restrictions, and Wasm GC benchmark verification/tuning. Add the exact new `wasm/prelinked_aot` owner to the deferred inventory. **NOT APPLICABLE:** Nintendo-only Vulkan build selection and upstream C++ test-player dump/async-pump changes; this repository does not ship that test player or Nintendo platform. No shared runtime, supported renderer or native Luau implementation changes are present, and no such behavior is claimed translated. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `edcf7d9d0c6042b8882311733e5103992c3d4d84` | Carry reflected uniform-block minimum sizes through binding-map serialization, layout identity and stage merging; reject undersized/out-of-bounds uniform ranges before native bind-group creation and while recording deferred commands. Default zero-sized ranges to bytes remaining after the offset. Translate shared and deferred regressions across supported backends. **DEFERRED:** Wasm guest GPU proxy validation, factory-error traps and locale import remain with guest execution; unsupported D3D backends are not introduced. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `c5faa1fad8e95a6f8639696e08d646f3b2968e48` | Expose module-loaded and module-error host hooks at the upstream boundaries: the loaded closure is available before execution, and failed/yielded module frames remain available before result transfer. Default hooks are no-ops. The public delta contains these runtime hooks, not the VS Code debugger or CLI integration described by its title. | — |
| `df5d96b7deb6af1822383171d566bc1e4949ab51` | **NO RUST BEHAVIOR DELTA:** the sole executable change replaces Emscripten `Module.HEAPU8` with a fresh view of `wasmMemory.buffer` after allocating decoded image pixels. The live Rust scripting decoder uses the approved Rust image codec and owned pixel buffers on native and `wasm32-unknown-unknown`, with no Emscripten heap-view bridge. This is not deferred Wasm script execution, and no Emscripten dependency is introduced. | — |
| `85d7f952b9266e3dc184effc5974d40437ef3fe6` | Import and clone bindings authored on file-level view-model instance values, reconcile two-way defaults source-first, and install per-artboard clones when binding list instances. Preserve context-change notifications, target removal and deferred binding deletion/resynchronization; translate all six authored instance-value-binding regressions. | — |
| `25a2dc10786955df879ebc295e7c67923b8bde90` | **DEFERRED:** Wasm/AssemblyScript 2D canvas ABI, native/web guest bindings, canvas/image handle ownership and resize behavior, and nested call-scope cleanup of guest render passes and canvas frames. The complete executable delta is in the parked Wasm script VM/binding lane; native Luau, shared runtime/renderer owners and admitted browser renderer compilation are unchanged. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `edddc609a6595b3a4bc9df637d47b4ae127a2a40` | Skip registry-typed bool properties rejected by an object without desynchronizing the stream; propagate untyped-property failure as malformed through import and asset stripping while preserving unknown-object forward compatibility. Translate all four hand-assembled import regressions and clarify typed semantic-listener matching. CLI authoring, editor and generator changes described by the commit message are absent from the public delta and are not claimed imported. | — |
| `65638e57ff92bbdd658fa571b1ddbb1ad5c6a71c` | Record nested script render passes and settle each inner pass ahead of its enclosing pass while retaining resource lifecycle command order. Track open passes by registration token, finish nested passes innermost-first, detach on context destruction, and close passes before frame reset. Translate native Luau cleanup/wrapper changes, supported backend changes, renderer/scripting regressions and all three nested-pass GM modes. **DEFERRED:** the Wasm scripting VM/guest GPU wrapper slice remains parked with script execution; unsupported D3D backends are not introduced. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `54ce53ddddb5daae38514e62a626f2bbccf3c062` | Scope protected Luau call cleanup to canvas frames and the render pass opened by that call: track monotonic frame registration tokens, snapshot inherited pass identity, and preserve enclosing calls' GPU work. Translate the nested-frame and render-pass regressions through the live native scripting boundary. | — |
| `6951a4b3d1344bf965c9975a8670730542fbada2` | Consolidate each binding container's owning data context, remove duplicate artboard/state-machine/scripted-converter fields, and preserve context installation before callbacks and after clone bind construction. Delegate formula/group unbind to the base, releasing occurrence borrows before child observer cleanup; translate all three upstream ownership regressions and cover the Rust callback boundary. Preserve nullable formula parent semantics; operation converters do not register upstream's unsafe null dependent. Retire the obsolete local C++ nested-artboard semantic-UAF oracle patch: upstream already incorporates its replacement in `081f85a6`, so reference builds use upstream's implementation. | — |
| `a2e27b91df9c59a0608ac198ccc7bc4a0d983ddb` | Add the Focusable text-acceptance contract, TextInput/FocusData overrides and primary-focus ancestor query, preserving false for ordinary focusables and no focus. Translate the mock focus-tree and authored TextInput regressions. The public delta contains these runtime APIs/tests, not the editor playback/layout implementation described by its title. | — |
| `8551ef2b1f0fd288dea9ad99182b61c3537f8523` | Select all text on keyboard traversal, or on any newly granted focus when selectAllOnFocus is enabled. Translate the nested-restoring traversal flag, boolean property 1099 with false default, registry/schema/copy/deserialization wiring and all three upstream regressions, retaining ordinary pointer caret behavior. TransformComponent additions are editor-only copy/deserialization hooks, outside the admitted runtime profile. | — |
| `69f887bb9279d13e1d72126186676526fcfed537` | **DEFERRED:** route Wasm artboard/view-model inputs through the cached slot-or-legacy-name ABI and release only owned guest names. The public delta is limited to those two Wasm VM setters; native Luau, shared input owners and browser renderers are unchanged. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `e50a67032b1e9c092b9485bafe52e508d8e2ea1d` | **DEFERRED:** Wasm guest path/contour measurement handles and lifetime management, generated native/web ABI imports, typed AssemblyScript wrapper generation, and element-to-byte sizing corrections in the guest-memory web bridge. These are Wasm script execution bindings, not the admitted browser renderer; no shared runtime geometry, native Luau or renderer owner changes. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `95110280a85b4fcbf9917cfa87a432052f000e33` | **DEFERRED:** pin Wasm debug boots to the O0 tier, suppress background compilation/upgrades, and cache per-instance input slots while preserving the legacy guest input ABI. All executable changes are confined to the parked Wasm scripting VM; no native Luau, shared runtime or browser renderer delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `2d3eb846aa4e7ed0bb4e3a6303cf6aff8a0fbc02` | **DEFERRED:** synchronous Wasm AOT boot compilation, tier-specific module cache identity, same-hash compile-job retention and unique compiler temporary files. The complete executable delta is confined to the parked Wasm VM/tier ladder; no shared runtime, native Luau or browser renderer change. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `ea15876025689f4d7a8b20b03ab636a78c250763` | Exclude Vulkan atomic/clockwise-atomic shader data and execution branches on Android by default, with the explicit android-vulkan-atomics opt-in matching upstream's build option. Preserve non-Android behavior and hotload stream order, including removal of the duplicate atlas vertex read. Remove atomic GLSL inclusion from the admitted WebGL profile; no shader bytecode regeneration is required. | — |
| `621f2a2e295f79a52c5fcf8c617327803f28d3c7` | Move layout tween and inherited interpolation state into a lazy retained owner, seed both animation buffers from current layout, preserve default-cascade/no-allocation and interruption behavior, and translate the interrupted-animation Silver regression. Scroll physics, range-mapper interpolator and layout-style interpolator already initialize to Rust None, matching the three explicit upstream null initializers. | — |
| `3b2c51e2dd957722fd3061112667d884b1ec60c3` | Add NestedArtboardLeaf fitToLayoutParent boolean property 1098 with upstream false default, byte-sized fit storage, registry dispatch and cloning/deserialization. Preserve legacy direct-parent sizing unless opted into layout reach through transparent containers. Translate solo/opt-in Silver scenarios and updated matrix fixtures without changing authored expectations. | — |
| `d90dfa396913a89472ca5f14aab3045dfa5101f5` | **DEFERRED:** Wasm VM AOT boot imports, C++ exception trapping, and pristine module-byte retention for the tier compiler/cache. All executable changes are confined to the parked Wasm scripting owner; no shared runtime, native Luau, or browser renderer delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `eb41943a0c2127193709977ec353254f6bab6ca5` | **DEFERRED:** WAMR AOT compiler patch reserves the platform x18 register for Apple/Windows AArch64 artifacts. Entire executable delta belongs to the parked Wasm script execution/compiler lane; no shared runtime, native Luau, or browser renderer changes. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `293eaf002cdda1ab6ec20316ee2d67ebd2854775` | Compare all seven semantic content fields during full re-flattening and invalidate derived labels when explicit labels change or clear. Translate the three diff-field regressions and four authored zero-area fixture scenarios. Flutter widget/platform changes described in the commit message are absent from the public delta and are not claimed imported. | — |
| `3f7495c6eb29f73885374682bea8d589dc94c3b2` | Reset all four GL color-write masks immediately before each ORE color-attachment clear so state inherited from another renderer cannot suppress clearing. | — |
| `0d8bb5a342f84a53119a6817c46ad1739cb7b696` | Support stroked and feathered image paints in atomic mode through internal clip stroke/feather state, shared path pixel-bound calculations and bounds outsets. Move feather-radius conversion to the shared GPU owner and preserve clip-paint construction order. This upstream commit does not expose a public clipStroke API. | — |
| `39afeca44449b12d41c91aaf78f1ed96913ed69a` | Do not consume deferred ORE commands until a screen frame opens successfully; translate the exact failed-screen regression. Add GL scratch-framebuffer invalidation and depth-only draw/read-buffer disabling. Thread the default-enabled Vulkan debug-name option through context construction and avoid loading the optional function when disabled. Bootstrap edits only add trailing newlines; private Android host changes mentioned by the title are not present in the public delta or claimed imported. | — |
| `9463ff7b5b9a1452d0c32e41390a99cd39b6c946` | Translate atomic-mode image paint modulation and gradient packing across shared renderer and supported backends, preserving the upstream stroke/feather limitation at this checkpoint. Regenerate shader artifacts. Treat singular artboard self-transforms as pointer misses, cancel all tracked gestures before dispatching drag ends, and retain per-pointer drag ownership and the upstream regression cases. D3D is not shipped or claimed. | — |
| `d8727299e08ba8517887a8a9f0fae80819208294` | Decode/write watermark manifest sections without narrowing section IDs; attach pre-rolls only to top-level instances, preserve timed state-machine handover and opaque fitted drawing, and port manifest/playback tests. Native/browser clocks are monotonic; import-free Wasm hosts explicitly provide a monotonic clock before instancing watermarked files. | — |
| `25db4792fadc1d26b9deed73e6e865122ff1ef95` | Track eligible out-of-band image, font and audio assets per loaded file; apply global registration/replacement/removal to existing and future files, clear every alias when resources are deleted, and preserve embedded/internal-loader ownership and invalid-handle errors. Translate the corresponding command-queue scenarios. | — |
| `1610277371ebe63cc0c76513e7a803f73b7f8bb0` | Read, retain and expose the runtime header file ID at full unsigned 64-bit width in both the runtime and editor/export binary decoder; preserve version/property decoding and overflow handling. | — |
| `7ed4209dad9d62c86e6a35a478ba1e6fee149e5a` | Move linear-animation binding/interpolator caches into one lazy extras owner, preserve fresh-on-copy caches and explicit bind/holder/interpolator teardown, and initialize didLoop to false. Derive per-layer owner/artboard context from the state-machine call, retain the layer definition, create Any-state instances only when transitions need them, and seed the process-global RNG once per state-machine instance rather than once per layer. Remove the unused blend-state temporary. | — |
| `899b20b23841de3834ab56c76351b3358eb590cd` | **NO RUST BEHAVIOR DELTA:** upstream's golden executable/deployment scripts decouple deferred recording from scripting, expose `--deferred`, warn on immediate fallback, and fix a Prospero C++ RTTI flag. Our deferred session/replay and runtime-file import GM are not gated on scripting and explicitly select deferred versus immediate execution; these upstream launcher/benchmark/console-build owners have no downstream counterpart. No new launcher, fallback or console support is introduced or claimed. | — |
| `7e0b60b042abc7e66357c84e6e90919c7d1b1f50` | Translate tools-only shader compilation-mode switching, preserving the prior mode and GL/Vulkan forwarding. Translate the ubershader/specialized comparison scenes and frame-scoped mode restoration. Metal/WebGPU retain upstream's no-switch default; unsupported D3D backends are not introduced or claimed. | — |
| `d619bc2a83f3c592a57eb58b9c83315142bcfbfc` | Translate the Vulkan frame-sync coordinator and its synchronizer dependency, including frame/fence state, completion caching, readback and resource cleanup. Preserve the four upstream coordinator scenarios and the approved Ash/Vulkan context boundary. The upstream commit adds utility APIs without switching production callsites; the product host's existing scheduling is not replaced by this delta. | — |
| `947107bc5971acb208b68135437aa5e33023d619` | Add queued main view-model lookup and report successful main/global lookups through state-machine listeners. Route lookup failures to the state-machine error channel, preserving request IDs, listener association, existing-instance identity, locking and callback order; translate the bound/unbound/invalid lookup and cleared-global regressions. | — |
| `4217c6378d0934c784a467abcaac9e9684c98411` | **DEFERRED:** tools-only Wasm tier-ladder/state-transplant guards, disabled-engine inline singleton surface, and WAMR internal-header build gating belong to the parked Wasm execution lane. No native Luau, shared runtime, browser renderer or fixture behavior changes occur in this public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `4ca3b88a34e02d534bb800a803e81217ee11cc60` | Gate layout-controlled text box sizing and overflow at file version 7.3; preserve content-sized legacy boxes, in-memory defaults and clone state while leaving effective text dimensions unchanged. Translate the four exact five-frame text-layout Silver scenarios and consume the updated upstream assets/recordings through the pinned harness. Preserve approved Rust text and Taffy boundaries. | — |
| `2210ed8799c0128504dd664a7179f4f8f299e85a` | Split image rectangle and mesh instance streams, resource counts, allocation/mapping and backend bindings; translate the seven-attribute vertex layout and shared shader changes across Metal, Vulkan, WebGPU and WebGL2. Regenerate shader artifacts and preserve the existing Metal and WebGPU host adaptations. Translate resource-count and map-failure regressions. D3D backends are not shipped; ORE GM namespace qualifications require no Rust behavior change. | — |
| `6cd8fccc90dc3c30dff0f71175648b4417246212` | **DEFERRED:** Wasm frame-collector hooks, leak reporting, guest decode/error/sampler handling, WAMR AOT cache/tiering and memory-growth fixes, Android engine build support and the Wasm-only GC benchmark remain with the parked execution lane. No native Luau, shared runtime, supported renderer or fixture changes occur in this public delta. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `1fac60f87be3639918f06a1b8e23908b1621b1a1` | Permit state-machine layers without Any or Exit states while retaining required Entry validation in both live runtime and binary import; preserve null-safe Any-state instance creation, transition handling and initialization boundaries. The editor features referenced by the commit message are not part of the public runtime delta. | — |
| `78b07a3fdc72fa7f64bf464d871ae80474a5a39a` | Translate the generated layout conformance runner and its real collection/item-position inspection helpers. Preserve all four exact fixture/expectation pairs, referenced image/font assets, positive/negative checks and explicit upstream known-defect checks. Correct existing LayoutComponent/Artboard/Text computed-size virtual dispatch, Text local-bounds accessors, and the omitted layout-participant undefined-to-zero read conversion exposed by the matrix; retain the approved Taffy and Rust text boundaries. | — |
| `bb7200ef181d72f0e83e737636a41b1e43601d7a` | **DEFER/SKIP:** Windows WAMR build/platform fixes, tier-ladder process layout, and Wasm execution clock portability remain deferred with UNIV-3728. The only renderer edit adds an ORE header to the unsupported D3D12 path-fiddle host. No shared/native-Luau runtime or supported renderer behavior changes; no Windows Wasm execution or D3D12 support is claimed. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `9d2e7d04d1bd5ee5863c7155d059b1e7b5810148` | Re-home hidden focus within its ancestor chain and descend renewed scopes to eligible leaves after the owning root's update. Preserve root-scoped visibility checks, unattributed host-node behavior, and all-roots entry points; reuse filtered children in leaf selection. Translate the focus regressions and same-frame collapsing assertion, using upstream's changed focus_collapsing Silver recording. | — |
| `a6b6723ba291f6f00888c09a1f975c6c23936095` | Apply nested-artboard layout placement after constraints, using the mounted instance's origin and the parent's linear transform; remove the superseded world-composition override. Translate all six constraint/follow-path/origin-override regressions with the three exact upstream fixtures and original tolerances. Retain the approved Taffy boundary. | — |
| `bc97fa18fdd658f7c0c80f935121276d3b28ffaf` | Carry backend display scale into Luau layout-resize callbacks, with upstream default/equality behavior, snapshot-based registered-object notifications, and known-size guarding before redispatch. Preserve the separate local context pixel-ratio extension. The Wasm execution owner's v2/legacy export dispatch remains deferred under UNIV-3728; shared runtime and Luau behavior are not deferred. | — |
| `2f3cf360a4637718d1f2fcab301666b2664b7cc8` | Mirror the RHI shader's four-sample destination-fetch branch and retain coherent shared shader-generation inputs; this does not introduce or qualify an Unreal renderer. The `RIVE_NO_FILESYSTEM` guard affects only upstream's optional tarnished-Silver file writer: Rust's `SerializingFactory` exposes in-memory bytes and has no corresponding file-writing method or filesystem dependency to guard. Serialization bytes and runtime playback are unchanged. | — |
| `9ed5b5168d95aab07e873db341fb65613d317cfc` | **SKIP:** exact 128-bit fractional-index comparison is entirely inside `WITH_RIVE_EDITOR`, in the native editor-only sibling-order owner already excluded at d4fe1022. Runtime `.riv` deserialization and playback do not use this type. No native editor ordering API is introduced or claimed; the public delta otherwise changes only `.rive_head`. | — |
| `27a2876815e282c9f235def8b7c529c77d8bb105` | Mirror the Adreno 530 shader-preprocessor workaround: explicitly parameterize the clockwise fragment-main forwarding macro, restrict the image-buffer swizzle helper to its consuming modes, and remove the non-ASCII comment character. Preserve existing Metal adaptations; physical Adreno 530 qualification is not claimed. | — |
| `ad34081b740a3071ca92f2e73223404a600199e3` | Track the last semantics viewport transform per synchronized state machine and republish current geometry with root/version/frame metadata when that transform changes, even without a state-machine advance. Add root-ID and bounds-snapshot accessors and preserve the exact queued viewport-change regression. The private Android accessibility UI implementation described by the commit title is not in the public delta and is not claimed as imported. | — |
| `c55840a8f4a400b3572875a186e0dda0708e7718` | Move focus-manager ownership from state machines to root artboards; preserve lazy root creation, adopted-manager identity, cleanup/fallback and destruction ordering, nullable state-machine forwarding, and file/scripted root initialization. Translate the focus lifetime and text-input/component-list regression changes. The parallel Wasm-script HostArtboard initialization remains deferred with its execution owner under UNIV-3728. | — |
| `1f1a7c593e154a7856b70cbad95033ca516ee296` | Restrict Metal Depth24Unorm_Stencil8 to non-ARM64 macOS and include tvOS 16.4 in compressed-texture availability. Preserve the new command-server scripting-context routing regression through the actual queue/import boundary. The public delta does not contain the Apple Concurrency API wrapper changes described by the title; those are not claimed as imported. | — |
| `a81a46debcec6110d7866ccbf123e935db1f4f5d` | Translate the scoped autorelease-pool helper with Rust's closure-scoped Objective-C pool boundary and the non-Apple no-op branch; preserve the bare-thread and nested-pool retain-count regressions. The public delta contains no playback callsite edits, so the commit message's private CLI/native changes are not claimed as imported. | — |
| `f1ec753f1ad9e64f0844f845ffc8d68d16d15bdb` | Translate the SVG factory and renderer utility owners, including path/gradient/image serialization, significant-digit precision, save/restore, opacity, blend grouping and deduplicated clip groups; preserve all upstream SVG regression cases. The upstream Emscripten-only SVG testing-window adapter and Canvas2D harness changes have no downstream testing-window counterpart; no Emscripten or Canvas2D backend is introduced or claimed. | — |
| `bec99be4e4fecee71d0db012edeffdc561da319a` | Reconcile ahead-ported obscured text input with the complete upstream commit: bullet shaping, intact editing, word navigation, cursor/journal re-resolution, and handled-empty selection export that blocks ancestor clipboard fallback. Move reached property 1095 into runtime schema reconciliation and retire its forward overlay; preserve the full upstream regression cases. | [UNIV-2852](https://universe.basis.dev/issue/UNIV-2852) |
| `c18b32511bfeaeee6b7c54e35152aea3fdbb5964` | Rename the depth/stencil interlock mechanism and its draw/shader vocabulary across shared renderer and supported backends, retaining MSAA sample-count and resolve concepts where upstream does. Preserve numeric enum and shader behavior; regenerate GLSL and update bindings, retain equivalent SPIR-V/WGSL payloads with renamed symbols, and record Metal's macro-only adaptation separately from historical captures. Enumerate build inputs from current translated owners so stale pre-rename cached files cannot enter the shader batch. Correct GL render-pass feature gates to match their owning context; no legacy aliases or new D3D backend. | — |
| `503eab633367d00ad3627770e7898d22379cc2e2` | Add queued and synchronized focus traversal, focus clearing, focus-node/state queries and listener responses; preserve request IDs, error routing and lock boundaries. Correct the existing font-sharing adaptation to atomic shared ownership and a synchronized color-layer cache so the queue can cross threads without recreating font identity. Translate all five upstream command-queue regressions, including the actual threaded synchronized case. | — |
| `0d8ca59dac1b67c35d03a79cb29c595ccddab8d9` | Replace independent checked/mixed booleans with the upstream two-bit integer check-state field, integer binding dispatch and tri-state decoder. Remove the superseded mixed property and flag APIs while preserving packed wire bits; translate all five upstream check-state tests. | — |
| `7d59acedbf37270e538a3093ec777f5b66b7ffb9` | Deduplicate and sort the shader minifier’s reserved-name set in the owned Rust tool and exact Python source snapshot. All 345 reserved names are unchanged; shader output semantics are unchanged. | — |
| `ff8e6c4847c1686bbacbe4da5cf67b420769ff5f` | Enable GL pipeline dynamic state and execute dynamic midpoint-fan/outer-cubic batches as borrowed coverage, main fill and stencil reset with one program, unchanged draw arguments and preserved scissor. Mirror the shared color-write-emulation commentary. | — |
| `8c7bb1ea1b502361e086324fc0187ad3ccfd9afd` | Replace the defective high-surrogate bitwise shortcut in UTF::ToUTF16 with upstream arithmetic. Preserve BMP handling and low-surrogate behavior; translate the exact BMP, supplementary-pair and exhaustive supplementary round-trip tests. | — |
| `73a477e2fb564886efeab870ee0884d0b20e8179` | Translate the shared lite_rtti_cast_or_assert helper. Canvas2D TestingWindow/JavaScript wrappers and Emscripten test-build options target an upstream test backend we do not ship; no Canvas2D renderer parity is claimed. Renderer fixes described outside this public Git delta are not treated as imported. | — |
| `5ab9af03eb23e978abccc2dbd047bcde30e3cdbb` | Translate ORE GL scratch framebuffer/vertex-array loans, attachment scrubbing, resolve reuse and cleanup through the safe Rust context lifetime boundary; mirror Windows link inputs and removal of the standalone webgpu_player build root. Translate upstream regression scenarios. GLFW foreground behavior, testing-window EGL vsync, and deleted demo assets have no Rust product counterpart. | — |
| `7098a7c86220fefe6e0620d83b53d906f6fe8dae` | Reconcile ahead-ported text-input horizontal/vertical alignment with the complete upstream delta; add caret blink timing/visibility and linked-corner-radius invalidation. Translate the upstream raw/native input and layout regression cases. Preserve later obscured-input behavior and its overlay while recording reached alignment metadata in runtime reconciliation. | — |
| `60ddf34e5155728ad9a307ed9b09c239872904e4` | C++ incomplete-type build fix: move unchanged FiddleContext frame methods into a source file with the complete ORE context include. Upstream tools_common includes the new file; our custom harness source lists do not compile FiddleContext. No Rust behavior delta. | — |
| `8fb7e2da583c7a2313f2a966b89e3d43d722489f` | Upstream adds `RIVE_NO_AUDIO_DEVICE` to suppress automatic miniaudio device creation. No Rust production delta: our engine constructors always create a device-free mixer. Explicit host-owned CPAL sinks are separate APIs and retain their contract. | — |
| `d4fe10229b3c148f315ba309924fa837135e7a76` | Translate runtime ID vocabulary/registry dispatch while retaining the single-varuint u32 wire representation; remove Folder runtime registration, preserve semantic bitmask getters and pending text/cache overlays, return zero for unrooted computed coordinates, and move the fire-event owner to its upstream handwritten path. Retain explicitly pinned last-public definitions for schema generation and reconcile runtime metadata against current generated headers; upstream no longer publishes the canonical JSON. Native C++ editor-only arena, fractional ordering and mutation/journal extensions are outside the runtime build, not claimed as implemented. | — |
| `9b3319623a210449145097e9f7c7447cf2e50f3b` | Invalidate text-input layout after every reshape, including single-line auto-width measurement. Keep the already-present shape/layout dirt helper and remove the duplicate text-change invalidation; preserve separately audited ahead-ported text properties. | — |
| `df50e6245122a3ebc2cbc8fad728b5cf497e8877` | Share transparent-container and list-participation predicates between layout collection and ArtboardComponentList parent resolution. Direct layouts still win; otherwise only flagged lists walk through transparent groups/Solos to the nearest layout. Translate all strengthened upstream assertions, including unflagged free-form placement and exact stack identity. | — |
| `e16f92461a8f3857b3b249946bc176818fe77efb` | Move cold state-machine reporting, bindables, non-pointer input and scripting state into lazy clusters; move six cold data-binding work queues behind a sidecar while keeping the hot dirty queue inline. Preserve callback ordering and the Rust shared-ownership boundary; borrowed binding access snapshots explicitly only where ownership or re-entrant callbacks require it. | — |
| `1c8e554f621dc088719b0ea31c8ccc89ac25d974` | **DEFERRED:** Wasm VM print callback installation before module instantiation and thread-local startup output routing, plus producer metadata. No shared/native Luau behavior changed; the existing native log sink is already installed before script execution. Do not introduce a dummy Wasm API. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `86fc70a74e0db7b747e04471bd4372cd38f6a0e7` | Add separate content-sizing and owning-layout ancestor walks. Nested layout axis queries and swaps resolve through containers, retaining the owning layout across the swap; nested leaves obtain fit bounds through the content-sizing walk. Port the Solo/column regression and complete enum-driven layout_solos Silver producer with original fixture bytes. Editor-only controls/fit-type changes are outside this runtime delta. | — |
| `58c2f5e80bd7398496561197037f93e7fbe35d6f` | Retain sticky focus-backing history, reject defunct focus stops while traversing their children, restore subtree manager identities, and detach destroyed FocusData through the nearest live manager. Port all seven upstream regressions. Correct nine-slice Y analysis to use height. Adopt upstream's magnitude-relative finite scalar/vector SRIV tolerance; the existing stricter special-float bit checks remain unchanged. Opcode rejection, render-buffer type decoding and varuint indices already match. | — |
| `89c04af7d13e4c72ab5fa95628d6a3e03dd1506d` | **DEFERRED:** Wasm-only handle generation checks, arena-epoch reaping, cleanup and leak reporting, plus Wasm-guarded `File::frameBoundary()` and producer metadata. No native Luau/shared behavior or fixtures changed. Do not add a no-op frame-boundary API or count this execution behavior as implemented. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `8e8492f8312c67ac54558adce2f0798baabcdce3` | Honor nested-artboard effective fill/hug overrides in grid and stack item sizing. Translate all seven upstream layout assertions and pin both original fixtures. Retain Taffy and adapt its grid min-content measurement to the pinned Yoga dependency behavior. | — |
| `fea5ac511579cb40ae3d8c8baa39b0125918b546` | **SKIP for current runtime:** `.rive_head` and the generated AssemblyScript binding output path move from `assemblyscript/std/rive/bindings.as` to `rasc/assemblyscript/std/rive/bindings.as`; no ABI or binding-content change. Retain this producer-location change with the parked execution work. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `370e5926631d5790276b21bf86d73ffda22dbe27` | **SKIP:** only `.rive_head` and an Unreal-only raster-order atlas-blit shader wrapper change. No shared shader, supported backend, fixture, or runtime delta. | — |
| `845a82a9b67136e0b37272170504659a8ab4ca67` | Port shared ScriptModuleAsset metadata/decoding, FileAsset signature verification, DataContext global lookup, shader `rstb` access, and native Luau trigger/input dirt and keyboard/text wake behavior. Existing Rust backend-neutral traits retain the extracted Luau implementation. Wasm execution, module-VM routing, host bindings, browser engine and tiering are explicitly deferred; importing module bytes is not execution support. | [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728) |
| `5892bb05be7ae966b751625b4ee12239e6860dc1` | Content-size non-participating children through Solo containers, keeping plain groups as barriers and the layout-provider walk unchanged. Port the original 200x200 rectangle regression and exact fixture; preserve Taffy. | — |
| `d7fff883fd802b46c92d5bf479a70712fd73ecba` | Merge actual vertex/fragment binding maps for split-file ORE pipelines; validate stage agreement and backend-specific slot collisions. Port allocator v2 admission, shared layout builders, Lua fragment selection and auto layouts, original binding-map/split-stage tests, and the exact generated GM shader header. Unsupported D3D backends and C++ test-build infrastructure are not added. Editor producer followup: UNIV-2958. | — |
| `05058845faa261dace9eb0804e36cfa0a3676b05` | Enumerate dynamic-state MSAA draw types for ubershaders only when the platform supports them; share the exhaustive GPU predicate with Vulkan. Port the upstream permutation membership test. Upstream deploy_tests.py skip/window handling has no corresponding downstream tool; no shader source or artifact bytes changed. | — |
| `9cb2205f06aa5e599554e7382889f4cf5bec382e` | Queue failed focus-action requests on the shared manager, bounded and tagged by root; retry after component updates and discard remaining requests at frame end. Preserve immediate eligible focus, guard gamepad listener targets, and detach orphaned focus nodes without a dead manager. Translate gamepad navigation and uncollapse/focus Silver producers. | — |
| `a343300260bd18776f3a556d67a9322b01d9d70f` | Include the upstream unique asset registration name in FileAssetData and the command-server/queue string stream. Extend the original hosted image/font asset-list assertions. | — |
| `61f00897013eba8c57ee2002c33e6ee47d476eff` | Pack layout flags and narrow source fields; allocate layout render paths and proxies lazily. Mark dynamic clips and pointer/scroll targets before conditional proxy injection, carry source-only marks to clones, and balance clip saves. Translate overlapping layout pointer-order Silver regression. | — |
| `ed92313af34f5a8928647d021ee497b6d95d949a` | Expose selected text through RawTextInput, TextInput, FocusData and Focusable; bubble the first nonempty selection through FocusManager. Share exact code-point encoding and exclude the trailing sentinel. System clipboard access remains a host responsibility. | — |
| `cd7655b5dd634ee63f56f11a3eb7e0f1990292d3` | Already satisfied: canvas-DAG GMs are included under the Metal canvas feature without scripting. The chain test executes both forward and reversed recordings; the cycle test compares against its immediate schedule reference. | — |
| `081f85a690f26a4e8cceead05bd9c3f86707eae2` | Remove outgoing nested semantic trees and attach replacement data-bound artboards to the host manager and enclosing semantic node, including the no-state-machine branch. Port early-enable/bind and alternating-swap manager identity and node-count regressions. | — |
| `e289232b776cf315863ad995ca987982e50445ab` | Compose nested-artboard layout slot/origin inside the parent's transform before constraints, using the composeWorldTransform override. Translate the three rotated-parent assertions against actual solved layout values, preserving Taffy. | — |
| `34f6df47431ec17762a5764ae3055375f09aace1` | Create deferred canvases as device-free shells with stable image identity; allocate backing on the replay context for content and ORE. Port supported backend allocation hooks, host projections, tests and GMs; unsupported D3D backends are not added. | — |
| `dee5342ad10517f65bb2bf8fa6effd810f1ba3d9` | Port command-server import-routing regression with distinct recording-session/device identities, deferred host and ORE identity checks. Rust golden/player paths already omit upstream's removed post-import render-context overrides. | — |
| `d6107a91f7a2798893356605fa96cff3cea41c8f` | Feather text backgrounds through clockwise outer/counterclockwise hole contours, background dependency ordering and paint invalidation. Port ring-area/dependency assertions, changed fill-rule assertion, and the exact one-draw feather Silver producer. | — |
| `746e3063dab40dfb79f0db9ed8d41bc3bf419366` | Already satisfied: command-server LoadFile invokes the host VM factory and passes its returned VM directly to import, without a server-side render-context override. Do not remove host factory initialization or introduce an Unreal backend. | — |
| `4ad6fcf47526b033e5cbe16275e9219365551d76` | Carry shared push-constant shader declaration macros and retained RHI source updates. Supported-backend expanded tokens are unchanged; retain coherent compiled bundles and correct the regenerated WebGPU wrapper's renamed coordinate. Unreal deployment/MSAA defaults, C++ stacktrace construction, Visual Studio bootstrap, and Unreal-specific NDK handling do not apply to Rust product hosts. No Unreal backend added. | — |
| `ecfc9746dc5bc72f1311cdd561c979b65bc11d8a` | Recoverable Vulkan startup allocation scope, null-safe failed creation handles, fallible pipeline factories and context admission; preserve abort behavior outside scope. Translate upstream fault-injection regressions. Rust host propagates admission failure without introducing a GL fallback. | — |
| `155b22d887c79e121d1e032c60ef59903c06be1a` | Rename Sidecar allocation accessor to ensure_allocated and update all generated setters/deserializers and storage tests. Runtime diff is an API rename, not the broader Unreal changes described by its title. No compatibility alias. | — |
| `3f4047a85f11fecfde8c4d906c0c1654aa12b015` | Signed grid fields through binding reads/writes; separate placement application phase and sibling-placement registration during resync, retaining Taffy. Port all three upstream bound-grid assertions. Editor-only export fixes are not runtime code. | — |
| `afda7a160cd72ea079e2cd25580518b22330646e` | Move bound-keyframe holders from state machines to lazy per-animation ownership, with source-artboard bind lookup, read-time flushing, teardown, and joystick playback contexts. Port updated standalone assertion and joystick Silver. | — |
| `73f94edc539c1fbc79827e5940814125035f8c77` | Uniform fit-font scaling for custom line height, letter spacing, and paragraph gaps; preserve negative line-height sentinel and invalidate shape for fitting/overflow changes. Port three metric tests and varying-size Silver producer. | — |
| `1cdecb8ed9ea8310400405d20d728d4cb9924602` | Binding-map v3 baked layout IDs, context-owned layout interning, dynamic-offset cache exclusion, full required-count reporting and wide-group spill. Update shader admission and fixtures to v3; reject v2 like upstream. Editor producer followup: UNIV-2958. | — |
| `9f1d2ed110a6eda762ecb4716ceb5fecf6772699` | Move shader-derived bind-group layout population/construction into shared ORE; route Lua and GM callers through that owner, preserving reflection metadata, visibility, dynamic UBO selection, native slots, and entry limits. | — |
| `64186dc048c7b81aee83a5290a2bb520fdffaa1e` | Port Luau dependency-tree delta f4a8c732→fb6ff089 into retained luaur: unconditional GC/userdata fixes, bounded parser/bytecode-library corrections. Retain disabled experimental classes/JIT and Rust hash storage; no new scripting backend. See luau-fork.md for precise profile limits. | — |
| `47862791523acad123ef02e3d6d7476c9b615159` | Yoga-only style-storage optimization is inapplicable to retained Taffy. Rust already has one authoritative LayoutData style (moved out/restored for safe applier borrowing), not duplicate YGNode/YGStyle state. Preserve three-phase applier ordering, dirt propagation, child synchronization, and solver-cache projection. | — |
| `f35441b52a95e542a766debd9d2b3dc21905cdbb` | Already satisfied within approved browser boundaries: WebGL2 PLS passes owned Uint32Array values from Rust slices; scripted image decoding uses the Rust codec with owned bytes. Neither uses Emscripten Module heap-view exports. Do not introduce Emscripten for its heap-view fix. | — |
| `463c59fd91c08538ba37f9cccc14fa3413a0da23` | Triangulator stability: disconnected-edge distance/rewind guards, active-list zombie removal, merge-call cap and resets, vertex-visit rejection, and upstream null-edge regression. Rust already uses scoped Side enum variants. | — |
| `f7c22102d5aa505f70ebc03e9f83feb7f52301ab` | MSAA interior triangulation via outer-cubic patches, shared-edge triangle strips, source stack-vector/triangulator tests, and Metal/Vulkan/WebGPU/WebGL2 wiring. Unsupported D3D backends are not introduced; Dawn host string/backend reporting is already equivalent. | — |
| `d97f3547db4db1b5d3c3a0b56189c30fee441248` | Translate JSON pointer-log parser/replay cadence and all three new serialized script cases. Correct existing constraint/scrollbar callback borrows and root-artboard animation-reset reads exposed by replay. Replays complete; original output assertions remain explicitly expected-red, not exact. Upstream deletions are commented-out tests only. | [UNIV-3015](https://universe.basis.dev/issue/UNIV-3015) |
| `694c0879279e956e928baa182df6c15e534f0635` | Command-server asset loader quietly delegates TextAsset (including scripts/shaders), BlobAsset, and ManifestAsset to in-band import, matching upstream's type hierarchy. | — |
| `526625850eaf34fc1263d181808ffca10cae6ac1` | Already satisfied: upstream corrects the new deferred geometry test's stale pointer constructor to empty replay capabilities, exactly as the preceding Rust translation already does. No production change. | — |
| `498419c45ef2ec48676730d9949b76164b12f4d0` | Prune empty scripted/dash/trim segments at upstream boundaries; filter empty per-verb deferred geometry and its query mirror; translate both new geometry tests. The new C++ benchmark registration is performance harness infrastructure and is not imported; no runtime behavior is excluded. | — |
| `8a494cedae2db53f15bf85871406aa67b2429448` | Capability-only `DeferredSession` construction; removed device constructor and `bind_real_ore`; migrated translated tests, GMs, and Apple C-API import caller to explicit replay caps. The `.rive_head` pointer is editor-owned. Shader and decoder sources/artifacts are unchanged from the preceding checkpoint; their historical provenance is preserved. | — |
| `675703b9fd71e982eaf97c034b313eba9bde63f4` | ORE test-only hardening that exhaustively accounts for every `ShaderModuleDesc` field and verifies exact record/replay propagation, plus the exact regenerated `ore_gm_shaders.rstb.hpp` fixture and consumer offsets. The sibling `.rstb` bytes are unchanged. The `.rive_head` update (Naga 30, shader-bake diagnostics/problem reporting, parser readback, wasm-nightly, emsdk, and compiler changes) is editor/tooling-only and intentionally excluded; the runtime remains Naga-free. No checked-in production runtime or renderer source changed. | [UNIV-2414](https://universe.basis.dev/issue/UNIV-2414) |
| `b36aa3d0085d7e30e7d43f422db89146d95a5c18` | Live root-space focus bounds for `FocusData`, live-first bounds and position lookup in `FocusManager`, the directly corresponding focus tests, and the moving-host fixture. Editor overlay and host-wrapper changes referenced through `.rive_head` are outside this runtime repository. | [UNIV-2413](https://universe.basis.dev/issue/UNIV-2413) |
| `6d6ab6f8102ffdd200f0c8147d339688a91fe867` | Interpolatable uint property metadata for color channels, uint-keyframe rounding/clamping/mix behavior, and the directly corresponding interpolation tests. All eight changed upstream fixtures bump only the generated file header from 7.2 to 7.3; the vendored `text_style_background.riv` fixture and its fuzz mirrors carry the exact new bytes because the runtime observes the file minor version, while the other seven fixtures are not vendored here. The commit message's bitmask-notification behavior is not present in the checked-in runtime sources at this SHA and is not invented downstream. Editor-only inspector/timeline UI is outside this runtime repository. | [UNIV-2412](https://universe.basis.dev/issue/UNIV-2412) |
| `bb949ddf5ca66d9c999eb9547c977940af5d27ec` | **SKIP:** only `.rive_head` and the C++ `build_rive.sh` Emscripten default changed; Nuxie does not use that build script and its browser renderers target `wasm32-unknown-unknown`. | — |
| `e3c5dec2873840d09ee1ea54f78e64e805ca22f7` | Deferred replay capabilities carried as plain data, recording detached from device ownership, per-frame replay-capability verification, and the directly corresponding device-state test. | [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `966499fffe2aadcbcd1fe4388160e4e7d5c0d967` | ORE command and resource wire PODs reordered widest-first with explicit trailing padding, gapless recorded vertex/depth-stencil/bind-group-layout structs, and directly corresponding recording/silver tests. The Emscripten build-only slice is not applicable because the browser renderers target `wasm32-unknown-unknown`. | [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `707c4f60f2433b32d34597045b2f43460e6cd8fb` | Layout-stack parent propagation for components and artboard component lists; state-machine key/text input ownership; disposed Lua property reconstruction and view-model-list correction; ORE deferred-session bookkeeping and directly corresponding runtime/renderer tests. | [UNIV-2408](https://universe.basis.dev/issue/UNIV-2408), [UNIV-2409](https://universe.basis.dev/issue/UNIV-2409), [UNIV-2410](https://universe.basis.dev/issue/UNIV-2410), [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `1db281b3e82baf850635fd7aa2092920a80b6a2c` | Command-queue view-model clearing and reference confirmations; scripting import-factory routing; IK-over-distance constraint behavior and fixture; shape-paint clipping; triangulation caching and interior-budget control; ORE sampler/anisotropy, deferred shader replay, canvas import, backend shader, and directly corresponding runtime/renderer tests. Android test-host and deployment-harness changes are outside this repository. | [UNIV-2402](https://universe.basis.dev/issue/UNIV-2402), [UNIV-2403](https://universe.basis.dev/issue/UNIV-2403), [UNIV-2404](https://universe.basis.dev/issue/UNIV-2404), [UNIV-2406](https://universe.basis.dev/issue/UNIV-2406), [UNIV-2407](https://universe.basis.dev/issue/UNIV-2407), [UNIV-2907](https://universe.basis.dev/issue/UNIV-2907), [UNIV-2908](https://universe.basis.dev/issue/UNIV-2908) |

- The preceding upstream deferred host/player refactor is a
  source-based **SKIP** for this Rust runtime. Its complete diff only adds or
  changes C++ test/player harness and deferred-host files; it changes no
  runtime/core implementation, schema, fixture, or supported product behavior.
  The renderer additions are test/player-host infrastructure only.
  The existing Rust `DeferredSession`/`DeferredReplayer` surface already covers
  the product-side deferred path, so no speculative shared host abstraction was
  added. Upstream: `11217a528b34966eca3765dc88c4ec0c8417d09c`.
  Work: [UNIV-2906](https://universe.basis.dev/issue/UNIV-2906).
- The preceding applicable port at `2cfa84e8103aeeeff4c2bfee92839ab580521660`
  is complete in [UNIV-1878](https://universe.basis.dev/issue/UNIV-1878).
- Intervening `1de56230e9ea062a2da2e25eee00942eafe3bdb4` and
  `0a8499b87a7d722b982d9c444172cab94d8320f2` need no Rust translation: they
  change upstream coverage and GMS test-host infrastructure, plus C++-only
  macro guards in `0a8499b8`. Those guards require no Rust equivalent; neither
  commit changes supported product behavior, format, or fixture content.
- Historical shader-provenance note for the preceding checkpoint:
  `3ed35ee0ded0d58fb8d380930a156041a4624a2f` remained the artifact source at
  that point because the intervening change did not modify renderer sources
  or generated artifacts. This is not the current provenance of every shader;
  later entries above account for subsequent changes.
- Intervening `8efe18ec7b52a02139844ffe71438c00de13037e` needs no Rust
  translation: Apple products and verification links already explicitly set
  macOS 12. The current upstream oracle source retains its Premake macOS 11
  linker-floor fix. No runtime behavior or future dependency was excluded.
- This source checkpoint is not Adreno/PowerVR hardware qualification. The
  existing MoltenVK C-API content checks fail identically on the previous main
  and this update; follow-up: [UNIV-2875](https://universe.basis.dev/issue/UNIV-2875).
- Apart from the explicitly deferred Wasm execution lane below, the established
  Rust adaptations remain in force. The user authorized merging each reviewed
  manual update as it is completed on 2026-08-31. This is not standing automation
  authorization.
- On 2026-08-31 the user approved the full deferred-rendering redesign in
  `e949498e05483a852c10fbbdad2cd1941c15aebc`: upstream is authoritative,
  including removed APIs. Translate the complete applicable delta, then run
  separate source-equivalence and Rust-integration reviews before validation
  and merge. [UNIV-1678](https://universe.basis.dev/issue/UNIV-1678) records
  the implementation, review, and validation evidence.
  The user also approved migrating the Apple C API caller before merge,
  including API changes where needed: import and replay through an explicit
  deferred session so scripted GPUCanvas follows the new upstream contract.
  Do not restore immediate-context GPUCanvas as a compatibility fallback.
- On 2026-09-28 the user explicitly deferred Wasm/AssemblyScript execution
  beginning with `845a82a9b67136e0b37272170504659a8ab4ca67` and authorized
  continuing all other applicable upstream changes in order. Follow-up:
  [UNIV-3728](https://universe.basis.dev/issue/UNIV-3728). Port shared and
  native Luau behavior from mixed commits; record later Wasm-only dependencies
  as deferred rather than treating their absence as parity. Importing module
  metadata or bytes does not imply execution support. `LAST_SYNCED_SHA` may
  advance through accounted commits under this explicit narrower scope, not
  as a claim of full upstream feature parity.
  The future execution lane must remain pure Rust; Wasmi is only a candidate,
  and WAMR-specific AOT artifacts/live tier transfer are not promised. Its real
  module qualification is also parked: producer packages `luau_wasm` and
  `assemblyscript` at `.rive_head` `e6598b59dbdf470aa686aa8852b6913ff6e7c3b9`
  and suitable generated guest modules were unavailable locally. Do not make
  engine qualification a gate on this non-Wasm catch-up.

## Out-of-order ports

These upstream commits were ported ahead of the incremental sync with the
user's approval. `LAST_SYNCED_SHA` does not move for them. When the sync
reaches one of these commits, confirm the Rust owners match the upstream diff
at that point and account for any intervening upstream changes to the same
files. Retire overlay definitions by covered source properties, not by file
name: preserve properties from later commits in shared files. After upstream
removed `dev/defs` at `d4fe1022`, schema generation uses the pinned public seed
in `defs/upstream-runtime` plus runtime reconciliation and forward overlays.
Move covered properties into reconciliation before retiring an overlay.

The cache-as-bitmap commit has now been reached and reconciled chronologically;
its complete accounting appears above. Its original ahead-port scope is retained
in [cache-as-bitmap-port.md](cache-as-bitmap-port.md). There are no remaining
ahead-port rows in this inventory.

The alignment and obscured-input properties are now both reached and live in
`defs/upstream-reconciliation/text/text_input.json`. The former shared forward
overlay is retired. See [obscured-text-input-port.md](obscured-text-input-port.md)
for the original ahead port and its chronological reconciliation.

## One upstream commit at a time

1. Fetch upstream metadata and enumerate `LAST_SYNCED_SHA..target` oldest first
   in dependency order. Keep the normal upstream and pinned checkouts untouched;
   use a separate clean candidate worktree at the commit being translated.
2. Inspect the complete diff and surrounding upstream owners, including tests,
   fixtures, definitions, generators, dependencies, and shared backend flags.
   Classify **port**, **skip**, or **mixed** from source, never from the title.
   Check later dependencies before excluding a change. An existing equivalent
   Rust implementation is a skip only after source comparison confirms it.
3. Mechanically translate the applicable delta into the corresponding Rust
   owners. Include the same commit's relevant regression tests, fixtures, schema
   changes, and regenerated assets. Do not retranslate untouched files or add
   compatibility fallbacks. Keep translation separate from review.
4. Perform the source-equivalence adversarial review against that upstream
   commit: defaults, arithmetic, control flow, state, lifetime, and failure paths.
5. Perform a separate Rust-integration adversarial review using the approved
   adaptation boundaries below. Review any semantic corrections from either
   review or validation before considering the change complete.
6. Run affected tests, compilation, and applicable focused differential or
   renderer checks against the matching upstream checkpoint. Do not compare a
   partially updated Rust port against the final catch-up target. Passing corpus
   samples and the structural source-correspondence check are evidence, not
   proof that every changed branch is correct.
7. Commit the reviewed change with a Conventional Commit subject and
   `Upstream-Commit: <full upstream SHA>` trailer. Generally one applicable
   upstream commit becomes one Rust commit; preserve identity when a tightly
   coupled upstream series must land together.

Parallel agents may inspect upcoming commits, translate disjoint owners within
a change, and conduct reviews after translation. Keep one integration writer
and preserve upstream order; do not let parallel work merge dependent changes
out of order. Do not load implement or TDD skills for this workflow.

## Scope decisions

On 2026-10-03 the user approved porting the optional script networking and
`decodeFile` capabilities introduced by `6cd5d108`, including the platform
transport through Rust bindings. Network access remains disabled until a host
explicitly installs a provider; enabling it in consumer apps is a separate
decision. Preserve upstream's signed-script requirement for decoded files and
its policy behavior, including the documented private-address DNS gap; this is
not a security qualification for app enablement. The Wasm guest-execution
deferral remains unchanged.

**SKIP:** on 2026-10-14 the user removed the watermark feature (the manifest
section 2 decoder, the artboard pre-roll and its draw/advance hooks) as an
intentional divergence; Nuxie never ships watermarked files. Skip upstream
watermark commits and the watermark slices of mixed commits. The manifest
parser skips section 2 like any unknown section. The engine clock it shared
stays for text input and scroll physics. Historical rows above that mention
the watermark are left as written.

Port fixes and compatible features on supported runtime and renderer surfaces,
even when they lack a corpus signal or appear in an "editor" commit. For mixed
commits, retain applicable shared changes and explain the omitted portion.

Keep Taffy, Rust-native audio/text, luaur scripting, Rust ownership, and both
browser renderers on `wasm32-unknown-unknown`. Do not introduce Yoga, an
Emscripten product, runtime Naga, or legacy WGPU. These boundaries do not excuse
omitting upstream lifecycle or observable behavior around an adapted library.

Layout and CSS-oriented text are product divergences, not translation targets
(confirmed 2026-10-23; see PARITY_WORKFLOW.md).
Nuxie lays out with stock Taffy and CSS semantics; upstream Rive's Yoga fork is
no longer an oracle for layout numbers. Treat an upstream change to Yoga, to
`LayoutComponent`'s solve, or to layout-only fixtures as excluded by default:
record upstream-specific semantics as skipped and continue without asking again.
Small optimizations and clear fixes may be considered only when compatible with
our CSS contract. Preserve CSS-oriented text differences; unrelated text fixes
remain eligible. Wasm/AssemblyScript/AnimaScript guest-engine work is excluded;
Luau remains our scripting direction. Browser renderer Wasm remains in scope.
Split mixed commits and port applicable portions without blocking on exclusions.
Upstream 868da86c's stack-layout portion is explicitly skipped by this decision;
its independent state-machine portion is translated and reviewed at the checkpoint above.
The matrix keeps pre-change expectations for those stack-cell cases through
`fixtures/sync/layout/matrix_overrides.json` under that policy.
Never patch Taffy toward Yoga. Non-layout behavior around layout (dirt,
callbacks, participant lifecycle, scroll/virtualization models) stays a normal
translation. When a translated non-layout change moves a golden/Silver/probe
entry already classified as a layout divergence, restate the entry's pinned
first difference and keep its reason; new layout-caused differences get a new
classification with a reason, never a global tolerance.

Ask before adding a substantial new architecture/backend or changing accepted
payload support. Inspect format, bytecode, binding-map, generator, and dependency
changes as producer/consumer contracts; do not update only one side or silently
replace an approved backend. If a relevant change is deferred, keep the last
complete checkpoint or explicitly document the narrower supported scope. Never
describe a deferred behavior as parity with the newer upstream revision.

## Small checkpoint PRs

Several small upstream commits may share a PR, preserving their Git identity.
Run the applicable broader translated tests, Golden/Silver, rendering, lifecycle,
and platform harnesses at that boundary, plus required repository PR checks.
Report exact commands, results, skipped checks, and unavailable hardware honestly.
Do not run the entire platform matrix for every tiny commit or require another
whole-port certification. Record skip reasons and dependencies briefly in the
PR; use Git history and this checkpoint, not new ledgers or certification rows.

Advance LAST_SYNCED_SHA only through a fully accounted prefix. Keep active source,
oracle, and artifact provenance coherent at that checkpoint; preserve fail-closed
identity checks. Rebuild affected reference/generated products as needed.
Current pin locations to inspect together:

- `Makefile`: `RIVE_RUNTIME_REF` and `PERF_EXPECTED_RIVE_RUNTIME_REF`.
- `.github/workflows/{ci,_trusted-macos}.yml` and `.buildkite/lanes.json`:
  `RIVE_RUNTIME_REF`.
- `tools/fetch-test-assets.sh`: default revision, not fixed asset provenance.
- `tools/{check-renderer-decoder-provenance,generate-renderer-shaders,renderer-dawn-live-reference-bootstrap}.sh`.
- `tools/golden-runner/runtime-provenance.sh`.
- `tools/silver-corpus/{generate_manifest.py,src/lib.rs}` and `silver-corpus.toml`:
  current inventory reference and new upstream cases; preserve existing result
  classifications unless the translated upstream delta changes their evidence.
- `tools/backend-port/build-{webgpu,webgl2}-source-oracle.sh` and shader
  importers: source revisions and regenerated artifact hashes.

Historical source citations, fixture hashes/revisions, and past oracle evidence
are not current pins. Never blanket-replace old SHAs or relabel stale artifacts.

## Authorization

This guide does not activate or modify scheduled jobs. A scheduled inspection is
read-only unless separately authorized; it must not infer write or merge
permission from this manual update. Follow the repository's PR/merge policy.

On 2026-09-30 the user separately authorized the daily upstream sync in this
chat. That job follows the translation, independent source review, separate
Rust-integration review and applicable validation sequence above, then opens
and merges narrow PRs without waiting for CI. It continues oldest-first until
the refreshed upstream tip is accounted for. This replaces neither the scope
boundaries nor the parked guest-execution decision, and does not authorize
unrelated optimization, consumer migration, releases or deployment. The old
Phase S worker and drift-scout jobs remain paused.
