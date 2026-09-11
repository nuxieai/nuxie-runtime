# HTML/CSS to Rive compiler design

Status: initial standalone compiler implemented, 2026-09-07. See
[`nuxie-html-to-riv`](../tools/html-to-riv/README.md), the authoritative
[v1 support contract](../tools/html-to-riv/SUPPORT.md), and
[validation gates](../tools/html-to-riv/VALIDATION.md).

The longer-term product design below includes work outside the initial module.
The implemented slice provides Rust/CLI compilation, embedded assets, source
identities, native responsive flex layout and Chromium comparisons. It does not
implement bindings, editor source mutation, incremental compilation, or product
publisher integration. Grid and border strokes are intentionally unsupported.
The module now includes an ESM/TypeScript client and a standalone
wasm32-unknown-unknown binary, tested in Node and Chromium against native output.

## Product contract

Persist designs as HTML and CSS. Visual editing changes those sources. Compile
them into a Rive scene for editor preview, on-demand scene creation, and
publication. Published `.riv` files are derived artifacts and can be rebuilt.

User direction: start simple and progressively add complexity. Use a documented,
versioned HTML/CSS subset for designs authored in our editor. Expand it through
working examples and compatibility tests.

## Compiler placement and reuse

The compiler belongs in the editor/authoring layer, following the existing
[runtime dependency contract](pure-runtime-boundary.md). It should be callable
from the editor, publishing, and any product host that needs on-demand
compilation. HTML/CSS parsing must not become a dependency of the baseline
runtime's import/advance/draw path.

The local `nuxie-dev` checkout already has a candidate export stage:
`packages/view-compiler/src/compiler-backends/rive.ts` calls
`buildEditorRiveViaWasm` in `editor-publisher-wasm.ts`. That interface currently
accepts a project snapshot and resolves compiled resources. Reusing its scene
lowering/export implementation needs investigation; an HTML tree is not already
a valid input to that interface. Avoid making the existing snapshot format a
second durable representation of an HTML-authored design.

Proposed flow:

```text
HTML + CSS + assets + binding/behavior declarations
    -> parse with source locations and stable element identities
    -> resolve selectors, cascade, inheritance, variables and value types
    -> normalized scene description, retaining layout and dynamic rules
    -> lower into Rive objects, bindings, animations and asset references
    -> export .riv + source map + diagnostics + asset requirements
    -> import through the normal runtime for preview or playback
```

RML is not needed as an intermediate format. Its producer is unavailable in the
public upstream, and it would add another translation before reaching the same
runtime object model. See [RML investigation](rml-research.md).

## Three compilation uses

* **Editor:** compile changes and display the resulting scene using Nuxie.
  Begin with whole-document compilation and scene replacement. Preserve source
  identities now so incremental compilation and runtime-state reconciliation
  can be added without redesigning identity later.
* **Publish:** use the same compiler and language version, resolve exact asset
  versions, emit `.riv`, and validate it by importing through the target runtime.
* **On demand:** a product host may compile downloaded or changed source once,
  cache the result, and import it. Normal frame updates use the compiled scene.
  Runtime data and viewport changes should use compiled bindings/layout where
  supported; source edits invalidate the compiled artifact.

Whether published clients must receive HTML/CSS and compile it themselves is
still open. The architecture permits that as an optional authoring module, but
it adds compiler distribution and startup costs.

## Preserve rules that must remain dynamic

Lower supported sizing, padding, gaps, alignment, wrapping and positioning into
Rive layout rules. Rive supports responsive layouts, but its layout containers
have specific participation rules; an HTML element may produce a layout wrapper
plus separate paint/text objects. [Rive layout documentation](https://rive.app/docs/editor/layouts/layouts-overview).

Do not use browser-measured rectangles as the sole representation of a
responsive design. That freezes one viewport, font result and data state.
Text changes and container resizes must still reflow at playback time.

CSS breakpoints, interaction states and custom-property changes require
explicit lowering rules. Some can become bindings or state machines; any rule
that cannot be represented must receive a diagnostic or have an explicitly
specified host evaluation contract. Do not assume all CSS behavior translates.

## Editing the source

Give editable elements stable authored IDs independent of their location in the
tree. Keep a map from each source element/property to its generated Rive
objects and source spans. A single element can generate several objects.

The editor mutates the parsed source and writes HTML/CSS back. Selection,
inspector edits and undo refer to authored IDs rather than generated object
indices. Shared class edits must distinguish changing the rule for all matching
elements from adding an instance-specific override. Use syntax trees that retain
comments and formatting if preserving hand-written source is a requirement.

HTML/CSS defines structure and appearance. Data binding, actions, reusable
components and animation controls also need an authoring representation.
Choose explicit attributes and/or source sidecars before promising interactive
HTML elements. Styling a button alone does not define its application action.

## Initial slice and subsequent integration acceptance

One responsive card containing an image, heading, body text and button-like
styled container:

* HTML: containers, paragraphs, headings, spans and images.
* CSS: a versioned selector/cascade subset; explicit default styles; text
  inheritance; flex rows/columns; width/height; padding/gap; solid backgrounds;
  rounded corners; font size, weight and color. Border strokes are deferred.
* Assets: explicit image and font inputs with stable identities.
* Output: `.riv`, element-to-object map, asset requirements and source-located
  diagnostics. Unsupported behavior fails clearly instead of being discarded.

Initially reject unsupported normal/inline formatting contexts, grid, filters,
pseudo-elements, arbitrary scripts and interactions. Each addition requires a
defined mapping and visual/runtime evidence. The detailed v1 contract above is authoritative; this scope is not a claim
that the target runtime lacks the deferred capabilities.

Acceptance: import the generated bytes, render narrow and wide layouts, change
text through a supported runtime binding, and demonstrate reflow. Test Unicode
text and exact font inputs. Preview and published bytes must produce equivalent
scenes. A visual editor property change must update the right HTML/CSS source
and recompile successfully. Compare supported static cases to browser rendering
as a diagnostic reference, with documented tolerances and semantic differences.

## Remaining product integration decisions

1. Whether on-demand compilation runs only in the editor/server or also ships
   inside published clients.
2. First-class behavior/binding syntax and visual-edit CSS override policy.
3. The reusable scene/export interface in the editor repo, verified against
   its current source and tests rather than its historical architecture notes.
