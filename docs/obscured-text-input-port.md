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
