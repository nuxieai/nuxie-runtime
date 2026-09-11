# Checked text occurrence policies

This implements the metadata prerequisite for A17. It does not yet fix pre-wrap
line breaking. The established nowrap/pre alignment policy now uses explicit
Text object targets, proving the path before adding different wrapping behavior
for normal and pre-wrap text in one scene.

## Contract

Runtime requirements version 2 adds a deterministic text_policies array. Each
entry contains object_id (default-artboard-local Text index) and policy. Current
policy css-nowrap-alignment-v1 requires text-css-nowrap-alignment-v1. The compiler
emits entries only for nowrap/pre Text objects, in emission order. These IDs are
separate from source-map layout IDs and must stay with the corresponding Rive
artifact. Scenes without occurrence policies retain their version-1 envelope.

Before import, ensure_supported validates envelope version, capability support,
nonempty version-2 mappings, unique entries and matching capability declarations.
After import, ensure_text_targets uses the actual default artboard to reject
missing objects and non-Text targets. The reference host installs every mapped
policy before its first layout, then retains the occurrence through resizing.
Unknown fields/policies and invalid unsigned IDs fail deserialization.

Version 1 remains readable and keeps its former scene-wide nowrap installation.
It cannot contain occurrence overrides. Existing hosts reject version 2 rather
than silently ignoring targeted requirements. As before, raw Rive import does
not interpret the sidecar. This is not artifact authentication: substitution of
another valid Text ID or removal of an individual mapping while leaving a valid
list cannot establish source provenance. Publish all artifacts together.

No Rive wire records or baseline runtime defaults were changed by this step.
The public Rust and TypeScript interfaces expose the new mapping explicitly.
Pre-wrap still uses the existing partial mapping with no new policy declaration;
its semantic failures and wrapped-tab rejection remain open.

## Evidence

The new public contract first failed because compiler output still used version
1. It now checks explicit targets for nowrap/pre in a mixed scene, omission for
normal text, deterministic order, target validation, invalid mapping/capability
combinations and compatibility with older manifests.

The checked-host test imports a mixed normal/nowrap/pre/pre-wrap scene. Version
2 targeted installation and version 1 scene-wide installation produce exactly
the same recorded draw stream. It rejects malformed manifests before writing a
draw stream, including duplicate entries, empty mappings, undeclared policies,
version-1 overrides, layout-object targets, nonexistent targets, negative IDs,
unknown policy names and unknown fields. It also verifies unavailable capability
rejection and ordinary version-1 scenes.

- 68 module Rust tests pass (17 compiler, 39 contracts, 6 adapter, 6 raster).
- The runtime nowrap regression, five JS/WASM tests including complete accepted
  corpus byte/source-map/requirements parity, checked host, TypeScript and two
  gallery tests pass.
- Module Clippy, pure-runtime boundary and 27/27 renderer-state controls pass.
- Full native-glyph browser comparison remains 597/605: exactly the existing A09
  failure and seven pre-wrap failures. All 597 browser/native image pairs and
  their layout bounds are identical to prewrap-full; no threshold changed.
- All 117 focused vector comparisons for nowrap/pre/tabs pass at the same
  three widths. Inspected the three representative native vector images and
  their differences at 240px; residual differences follow glyph edges, without
  missing text or alignment changes.
- Reviewed nine browser/native pairs covering mixed forced-line nowrap
  alignment, preserved spaces and tabs at 240/390/768px. They retain overflowing
  left origins, correctly centered short lines, and line-relative pre tab stops.

Commands: CARGO_INCREMENTAL=0 and NUXIE_HTML_REVIEW_DIR targeting
output/playwright/html-to-riv/text-policy-full with
`bash tools/html-to-riv/validation/run.sh native-glyphs`. The full gate stops at
the retained visual failures; renderer-state controls were run separately.
The vector follow-up uses NUXIE_NATIVE_GLYPHS=0, review directory
text-policy-vector, and `npm --prefix tools/html-to-riv test -- --grep
'nowrap-|pre-|tabs-' --output=test-results-text-policy-vector`.

Logs: /tmp/html-text-policy-{red,contract,build,host,gate,clippy,boundary,state,vector}.log.
The early contract log records the old exact-envelope assertion; it was updated
to the new documented envelope and the complete gate verifies all contracts.
Artifacts: text-policy-full/gallery.html, baseline-comparison.json and
policies-{240,390,768}.png under output/playwright/html-to-riv. The vector gallery
is text-policy-vector/gallery.html.

Next: add a dedicated pre-wrap occurrence policy using original source characters
in line construction, fix trailing-space hanging and unbreakable-word overflow,
then recompute tab advances at soft-line origins. Keep the seven semantic and
two vector-only pre-wrap controls. A17 remains partial.
