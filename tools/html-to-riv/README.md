# HTML/CSS to Rive

`nuxie-html-to-riv` is a separate Rust authoring module. It compiles a deliberately
small HTML/CSS language into real `.riv` bytes with embedded assets and an
element-to-object source map. The generated scene uses native layout rules and
can reflow when the runtime artboard is resized.

This is an initial compiler, not an arbitrary web-page importer. Read the
[supported language](SUPPORT.md), [validation strategy](VALIDATION.md), and
[prioritized expansion backlog](BACKLOG.md).
Each browser run produces a portable `test-results/gallery.html` with reference,
native and difference images, overlay controls and failure details. Follow the
feature acceptance workflow in VALIDATION.md before expanding support.
Browser equivalence is defined against the exported `BROWSER_RESET_CSS`, which
makes containers flex columns and defaults flex growth/shrink to zero. Supported
explicit flex declarations enable growth/shrink; see SUPPORT.md for the current
runtime representation limits. Authored styles are scoped to the fragment. Browser
defaults are not the authoring contract.

## API

```rust
use nuxie_html_to_riv::{compile, CompileInput};

let scene = compile(&CompileInput {
    html: "<div id=card></div>".into(),
    css: "#card { width:100%; height:80px; background-color:#369; border-radius:12px }".into(),
    width: 390.0,
    height: 320.0,
    ..Default::default()
})?;
// Check scene.runtime_requirements before importing scene.riv; install required policies.
// scene.source_map: authored identity -> artboard-local layout/text-run IDs.
# Ok::<(), Vec<nuxie_html_to_riv::Diagnostic>>(())
```

The library has no filesystem, network, browser, renderer or runtime dependency.
Assets arrive as bytes in a `BTreeMap<String, Asset>`; fonts carry an explicit
family and weight. Images reference those keys with `src="asset:photo"`.
All assets are embedded, including supplied assets not referenced by an element.
The runtime and renderer are dev dependencies used to validate output.
The repository's dependency boundary checker prevents this authoring module
from entering production runtime dependency closures.

`compile` returns diagnostics with `code`, `source` and `message`, currently
stopping at the first error. It never returns a partially accepted scene.
CSS diagnostics carry rule/declaration locations; HTML diagnostics identify
structural paths or attributes, not byte-accurate source spans.

## CLI

From the repository root:

```sh
cargo run -p nuxie-html-to-riv --bin html-to-riv -- \
  tools/html-to-riv/examples/box.json /tmp/box.riv
```

The input is a serialized `CompileInput`. Asset `bytes` are JSON byte arrays,
with `kind: "font"` or `kind: "image"`. The CLI writes `box.riv` and
`box.map.json` and `box.requirements.json`; publish these as one artifact. Compiler diagnostics are JSON on stderr with a nonzero exit
status. CLI argument, JSON decoding and I/O errors are plain text.

## JavaScript, browsers and Node

The module includes an ESM package, `@nuxie/html-to-riv`, with TypeScript
declarations and a standalone WASM binary. It requires no wasm-bindgen glue,
Node built-ins, filesystem access or network imports. Build before consuming it
as a local package dependency. `npm pack` also builds the WASM through its
`prepack` hook, so a clean checkout cannot silently pack a missing compiler.
After building, `npm run test:package` checks all exported files and runs a
compiled gradient through the tarball in an isolated installation. The package
remains private; these commands do not publish it.

Build locally:

```sh
rustup target add wasm32-unknown-unknown
cd tools/html-to-riv
npm ci
npm run build
npm run typecheck
```

```ts
import {createCompiler, LANGUAGE_VERSION} from '@nuxie/html-to-riv';

// Your bundler/server supplies the URL for @nuxie/html-to-riv/compiler.wasm.
const bytes = await fetch('/assets/html-to-riv.wasm').then(r => {
  if (!r.ok) throw new Error(`Compiler download failed: ${r.status}`);
  return r.arrayBuffer();
});
const compiler = await createCompiler(bytes);
const result = compiler.compile({
  languageVersion: LANGUAGE_VERSION,
  html: '<div id=card></div>',
  css: '#card { width:100%; height:80px; background-color:#369 }',
  width:390,
  height:320,
});
if (result.ok) {
  // Check result.runtimeRequirements, then import result.riv and retain result.sourceMap.
} else {
  // Display result.diagnostics beside the authored source.
}
```

In Node, pass `readFile` bytes for the exported `compiler.wasm` instead of
fetching. `createCompiler` also accepts an already compiled `WebAssembly.Module`.
Fonts/images accept `Uint8Array`, Node `Buffer`, or JSON byte arrays. Returned
bytes and source maps are owned copies and remain valid after later calls.
Unsupported versions, invalid requests and authoring errors return `ok:false`
without an artifact. Invalid WASM, ABI mismatches and unexpected WASM traps throw;
discard a trapped compiler instance before retrying.

Reuse one compiler per worker. Compilation is synchronous: run it in an editor
Web Worker and discard stale responses using your edit revision IDs. Persist
only the canonical design; compiled bytes are derived. The wrapper frees its
request/result buffers after each call, while the instance's linear memory
retains its high-water capacity for reuse. Initialization, I/O and worker
scheduling belong to the host. Package export `reset.css` provides the browser
reference defaults; the fragment-scoping contract still applies.

## Host integration contract

Persist HTML, CSS, exact asset bytes/references and `LANGUAGE_VERSION`
(`nuxie-html-v1`) as the canonical design. The current API implements that one
version; the JavaScript client rejects unknown stored versions, and direct Rust
callers must check the stored version before calling it. Use the
same compiler build for editor preview, on-demand compilation and publishing.
Compile on source/asset changes, import the result, and let normal playback
advance/draw the compiled scene. Viewport changes do not require recompilation.

Use `data-nuxie-id`, or `id`, for stable editor identity. Structural paths are a
fallback and change when siblings move. Object IDs belong to this specific
compiled artboard and must be refreshed on recompilation. `text_run_ids` lists
all generated text runs in logical order. `text_run_id` is populated only when
one run represents the entire element. Word spacing can split an element into
several runs; recompile authored text changes to regenerate that partition.
Viewport changes still use the same scene. `text_breaks` records each explicit
line break's identity, path and Unicode-scalar offset; breaks have no separate
layout object. These IDs are not a binding language.
The map does not yet contain per-property source spans or all generated paints.

Output is deterministic for identical input and compiler/schema versions. The
writer targets Rive binary 7.3 and the repository's schema/importer. Cache keys
must include the compiler/schema revision, language version, viewport and assets.
An arbitrary third-party Rive runtime version is not a compatibility guarantee.

This module does not yet replace the editor's existing project-snapshot export
path. Editor source editing, undo, incremental scene replacement,
action/binding syntax and product publishing integration remain
separate integration work. There is no hidden RML or browser process in the
compiler.

## Extending the language

Add a documented semantic mapping, rejection tests, runtime import/reflow tests,
and Chromium geometry/pixel cases before accepting new syntax. Keep parsing,
cascade/style resolution and binary lowering separate. CSS Grid is intentionally
rejected; it can be added as a later layout mapping without changing the public
source-to-scene boundary.

The experimental macOS glyph renderer is available through
`bash validation/run.sh native-glyphs`. This lane uses live native layout and
shaped glyphs with a bounded CoreText mask cache; its latest qualification results and remaining failures are recorded in
VALIDATION.md. `native` continues to select vector rendering. The generated gallery
identifies the renderer profile. See [the integration receipt](validation/live-glyph-review.md)
for host installation, exact fallback conditions and remaining qualification.

## Runtime requirements

The current manifest schema accepts versions 1–26. Version 26 adds retained
linear-gradient policies; a host must validate and install **every** requested
capability, including earlier layout, text, clipping, border and opacity
policies. The checked [reference host](examples/probe.rs) demonstrates the
complete installation order, and [requirements.rs](src/requirements.rs) defines
payload validation. Raw `.riv` import alone does not apply these policies.
This compiler module does not add editor integration.

The version-by-version notes below describe earlier capability contracts and
are not an exhaustive installation checklist. Use the current schema and
reference host when implementing an adapter.

Every successful result includes versioned runtime requirements. The CLI writes
`<name>.requirements.json`; the WASM/TypeScript result exposes
`runtimeRequirements`. Keep this metadata with the Rive bytes through publishing.
Every emitted Text object requests `text-css-shaping-precision-v1`. Install
`ShapingPrecision::CssExperimental` through
`FontAsset::set_shaping_precision_occurrence` on its imported font assets; the
host must retain the policy through font replacement. Hosts lacking the
capability must reject the scene before drawing. Text-free scenes need no
precision capability, and legacy manifests retain ordinary Rive shaping.
A scene with nonzero letter spacing requests `text-css-letter-spacing-v1`, even if
its current text is ASCII. Text containing preserved Unicode spaces also
requests `text-preserved-space-breaks-v1`, independently of letter spacing.
The checked host retains both policies through font replacement. Empty
requirements use ordinary Rive semantics.

Call `RuntimeRequirements::ensure_supported` before import, then install each
accepted capability. Version 2 adds text_policies: deterministic entries containing
an object_id and policy for Text objects in the default artboard. After import,
call ensure_text_targets against actual Text objects, then install listed policies
before the first layout. Source-map layout IDs are not Text IDs. Policy
css-nowrap-alignment-v1 requires capability text-css-nowrap-alignment-v1 and is
emitted only for nowrap/pre text. Pre-wrap uses css-pre-wrap-v1 with capability
text-css-pre-wrap-v1; install Text::set_css_pre_wrap(true) for its listed Text
objects. This capability requires version 2 and cannot use the version-1
scene-wide fallback. Pre-wrap text containing tabs also requires
text-css-wrapped-tabs-v1 and text-css-tabs-v1: the host must implement
line-relative tab advances in pre-wrap layout as well as enable the font policy.
New hosts retain version-1 scene-wide behavior;
other scenes still emit version 1. Older hosts reject version 2.
Pre-line uses css-pre-line-v1 with capability text-css-pre-line-v1; install
Text::set_css_pre_line(true) on its listed Text objects. It collapses tabs before
shaping and does not request tab-stop capabilities.

The checked reference host in examples/probe.rs applies
CSS letter-spacing mode to font assets using `FontAsset::set_letter_spacing_mode_occurrence`;
that policy survives font replacement and remains local to the file's assets.
Missing metadata or unsupported requirements fail in this host before drawing.
Importing raw `.riv` bytes directly does not inspect the sidecar or imply this
policy. This metadata is a capability contract, not an authenticated container.
The CSS policy spaces shaping clusters and suppresses optional ligatures at
nonzero spacing. The older `text-cluster-spacing-v1` capability retains its
cluster-only behavior; a host supporting only that capability must reject the
new requirement. See validation/optional-ligatures-review.md for qualification
and validation/runtime-requirements-review.md for the envelope contract.

Explicit line breaks use a text-only block container, for example
`<p>Hello<br>world</p>` with `p { display:block }`. The default reset stays flex;
see SUPPORT.md for the deliberate block-text and break-styling boundaries.


Uppercase/lowercase text transforms retain normalized source and rendered text in
an optional source-map text_transform record. Its scalar_offsets map source
Unicode scalar boundaries to rendered boundaries, including expanding casing.
Break offsets refer to rendered text. Raw HTML/CSS remains the authoring source.
Casing uses pinned Unicode 17.0 standard-library tables; unsupported table versions
fail compilation until requalified. See SUPPORT.md for current transform scope.

Capitalization also retains this mapping and follows the documented untagged
Chromium behavior using pinned ICU4X case/word data. The validation workflow
checks 62 independent browser casing references in addition to rendered fixtures.
Locale-sensitive casing remains in progress; see SUPPORT.md.

Version 5 adds `layout_pixel_bounds`, a nonempty list of artboard-local
LayoutComponent IDs, with capability `layout-css-pixel-bounds-v1`. The compiler
emits it for painted or clipped layouts, including layouts whose positions become
fractional only after resizing. After import, validate both `ensure_text_targets`
and `ensure_layout_targets` against actual object types before installing any
object policy. Then call `LayoutComponent::set_css_pixel_bounds(true)` on every
listed layout before first layout/draw. This snaps paint/clip bounds in CSS scene
space for unit-scale axis-aligned placement; logical layout stays fractional.
Hosts must preserve the requirements across publication/reimport, and reject the
capability if unavailable. Version 5 can contain the existing text policies,
underlines and strikethroughs. Versions 1–4 retain their previous contracts.

`layout-css-paint-order-v1` is an artboard-wide capability, valid with requirements
versions 1–7; it needs no per-object payload or schema version bump. The compiler
emits it whenever a parent has multiple layout children, including default order.
After validating capabilities and importing each instance, call
`Artboard::set_css_paint_order(true)` before the first draw. It keeps logical
layout children unchanged and reverses complete sibling groups in Rive's backwards
paint list, preserving nested background/clip pairs and text content. Reverse
flex directions retain the opposite sibling sequence, matching the pinned
Chromium overlap oracle; each nested parent is evaluated independently. Apply it to
every imported or cloned instance; do not infer activation from file extension,
environment settings, source-map order or the mere presence of the method. Hosts
without the capability must reject before drawing. Raw Rive defaults remain off.

Version 6 adds nonempty `layout_align_self` records `{object_id, alignment}` and
capability `layout-css-align-self-v1`. Alignment is auto, flex-start, center,
flex-end, stretch or baseline. Targets must be unique imported LayoutComponents;
validate every target before applying any policy. The compiler omits auto targets
and preserves auto through the existing parent-derived sizing behavior. Version6
can also carry earlier text, decoration and layout pixel-bounds requirements;
later painted children must not downgrade a scene's version. Versions1–5 reject
nonempty alignment payloads. Missing capabilities fail before draw.

For each alignment target, map the typed value to runtime CssAlignSelf and call
LayoutComponent::set_css_align_self_occurrence(&object, Some(alignment)); None
clears the override. This synchronizes retained layout state after releasing the
object borrow. Clones preserve the occurrence override. The accepted compiler
syntax is qualified for the native glyph profile; vector text raster limitations
remain explicit in validation/align-self-investigation.md.

Version7 adds nonempty `layout_align_content` records `{object_id, alignment}`
and capability `layout-css-align-content-v1`. Values are flex-start, center,
flex-end, stretch, space-between and space-around. Validate schema, capability
and every imported LayoutComponent target before applying any occurrence policy.
For each target call `LayoutComponent::set_css_align_content_occurrence` with
the corresponding `CssAlignContent`. None restores legacy line alignment; clones
preserve the override. Version7 may also carry all older payloads. Earlier
versions reject nonempty line-alignment payloads.

Wrapped compiler containers always carry this explicit policy, including the
authoring reset's flex-start. This separates line distribution from item
alignment. CSS initial/unset resolve to stretch; omission follows the reset.
The syntax and transport are implemented but visual qualification is pending;
see validation/align-content-investigation.md.

Distributed spacing uses runtime requirements version8 and capability
`layout-css-distributed-spacing-v1`. Hosts must validate all requirements and
imported target types before installing policies or drawing. Install
`layout_justify_content` entries (`space-around`/`space-evenly`) with
LayoutComponent::set_css_justify_content_occurrence. The existing
`layout_align_content` field may carry `space-evenly` only in version8 with the
new capability; its existing line-policy capability remains required too.
Keep policies through instance cloning and scene lifetime; unsupported hosts
must reject rather than rendering placeholder Rive alignment. Old manifests
retain their previous versions when no new distribution value is present.

`flex-wrap: wrap-reverse` uses the existing Rive flexWrapValue2. Like ordinary
wrap it emits an explicit line-alignment requirement (version7, or version8
when combined with new distributed spacing). Hosts install that line policy
and preserve the Rive wrap value through import, cloning and resize. Pixel
qualification is tracked separately in validation/wrap-reverse-investigation.md.
