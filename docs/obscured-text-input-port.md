# Obscured TextInput parity

Tracks [UNIV-2852](https://universe.basis.dev/issue/UNIV-2852).

Authority: upstream Rive commit
[`bec99be4e4fecee71d0db012edeffdc561da319a`](https://github.com/rive-app/rive-runtime/commit/bec99be4e4fecee71d0db012edeffdc561da319a).

The port adds TextInput property 1095 (`obscured`, default false), its schema,
registry, deserialization, copy, and property-change callback. RawTextInput masks
at shaping and measurement, leaving its editable buffer and undo history intact.
Word boundaries use the displayed bullets. A masking toggle invalidates resolved
cursor line indices, including journal snapshots, and layout/measurement caches.

Upstream's boolean selected-text result plus output string is represented by
`Option<String>` at the Focusable boundary: `None` searches ancestors;
`Some("")` consumes the request without returning a secret. The host-facing
FocusManager still returns a string. RawTextInput itself is not a confidentiality
boundary; its selected-text method still exposes actual text, as upstream does.

Focused qualification:

```sh
cargo test -p nuxie-runtime --features tools --test upstream_raw_text_input_native --test upstream_text_input_native
cargo test -p nuxie-runtime --lib selected_text --features tools
```

The existing fixture convention uses `RIVE_RUNTIME_DIR` (or the local upstream
checkout) for the TextInput Rive file and fonts. The first command passed 35 tests;
the second passed four focus tests. Added coverage checks actual shaped glyphs
against a bullet-only oracle, measurement, Unicode, clearing, newline masking,
word navigation, edit history, toggling, and blocked selection export. Parent
authoring's native endpoint probe also passes with the property present.

This does not qualify native OS editing, converter propagation, published inputs,
or SDK integration. Those require the separate authoring/host-adapter proof.

## Host value adapter qualification

The existing occurrence-scoped `nux_player_field_string_copy/set` adapter now
accepts native TextInput's generated `text` property as well as the existing
CustomPropertyString endpoint. Both use CoreRegistry's existing callback path;
there is no new ABI function, converter evaluator, or runtime editing session.
Explicit reads still return the editable value, including for secure fields;
they are execution APIs, not diagnostic capture APIs.

`cargo test -p nux-capi --lib` passes all 35 tests. The added native-input case
was first observed failing with `NotFound`, then passes for plain and obscured
inputs in two repeated nested occurrences. It covers edits, Unicode, clearing,
preserving the other occurrence, and absence of editable values from semantic
captures. This fixture has no font and makes no claim about glyph rendering,
geometry, platform IME, or SDK integration.

`nux_player_text_input_geometry` adds the approved host-adapter geometry access.
It shares presented-field resolution with the value functions, returns native
text bounds separately from the semantic field's container bounds, and exposes
the complete input-to-root affine transform. A small `root_transform_point`
accessor exposes the existing semantic drawing transform; it does not add new
layout or nested-artboard traversal semantics. The geometry includes masking,
multiline, and first-baseline metadata, never editable text or glyph ids.

All 35 C API unit tests still pass after this addition. Repeated plain/secure
inputs include rotation, nonuniform scale, child offsets, unchanged sibling
values, and stale-capture rejection. These fontless fixtures do not qualify
baseline placement or actual native editing sessions. The root text-run
geometry API remains unchanged for existing SDK consumers during qualification.

The stronger four-field fixture (two separately owned inputs named `editable`
inside one template, instantiated twice) initially failed with ambiguous lookup.
Native TextInput lookup now checks the semantic owner's subtree, not just its
artboard. All four field geometries resolve independently and editing the first
preserves the other three. Existing CustomPropertyString lookup remains unchanged
during qualification. All 36 C API unit tests pass after this correction.

Shaped geometry is now covered with the reproducible `roboto-input.ttf` fixture
(provenance/license in `fixtures/fonts/README.md`). Plain, secure, multiline,
and empty fields report a first baseline matching the font-table oracle at
24 px, with the correct parent translation. The initial test failure was an
incorrect fixture asset reference (asset ID versus file asset index), not a
runtime defect. All 37 C API unit tests pass. This verifies the geometry returned
to a host, not UIKit/Android/browser overlay alignment or native composition.
