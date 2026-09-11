# RML in the C++ Rive upstream

Investigated 2026-09-07. Local upstream: `/Users/levi/dev/oss/rive-runtime`, remote `https://github.com/rive-app/rive-runtime.git`. The unchanged checkout is `9ed5b5168d95aab07e873db341fb65613d317cfc` (2026-08-28); fetched `origin/main` is `25a2dc10786955df879ebc295e7c67923b8bde90`. Local links below refer to the checkout; commit-pinned GitHub links refer to the fetched revision. No upstream files were modified.

## Main finding

RML is an upstream authoring/export format for the Rive object graph. The public C++ runtime consumes its exported `.riv` binary, not RML text. This is directly supported by a test saying its `.riv` fixture was exported from `packages/rml/tests/rmls/vm_listener_fire_event.rml` via the RML exporter. The fetched source also explicitly describes a shared manifest writer as usable from `rml`, which does not always link the runtime. The actual `packages/rml` producer is absent from this checkout and fetched runtime tree. [Exported fixture test](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/state_machine_event_test.cpp:379), [latest shared writer](https://github.com/rive-app/rive-runtime/blob/25a2dc10786955df879ebc295e7c67923b8bde90/include/rive/manifest_sections.hpp#L25).

Observed pipeline:

```text
RML object markup
    -> external RML exporter (implementation not in this runtime repository)
    -> .riv binary: header + typed objects/properties + assets
    -> File::import -> generated CoreRegistry/deserializers
    -> importers resolve and initialize the object graph
    -> artboard/state machine runtime
```

The first arrow's implementation and CLI remain unverified. The rest is implemented in [File::import](/Users/levi/dev/oss/rive-runtime/src/file.cpp:299), [object deserialization](/Users/levi/dev/oss/rive-runtime/src/file.cpp:171), and [ArtboardImporter::resolve](/Users/levi/dev/oss/rive-runtime/src/importers/artboard_importer.cpp:34). The latest import entry point retains the binary-only header requirement: [latest File::import](https://github.com/rive-app/rive-runtime/blob/25a2dc10786955df879ebc295e7c67923b8bde90/src/file.cpp#L315).

## Syntax visible in primary-source examples

The useful example is [focus_bounds_moving_host.rml](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/assets/rml/focus_bounds_moving_host.rml:31). Its header explicitly says the associated `.riv` was exported from the editor and that this text is documentation, **not a build input**. Consequently this is evidence of intended markup shape, not proof that every line compiles with a released tool.

* Elements name object types: `Artboard`, `NestedArtboard`, `StateMachine`, `LinearAnimation`, `LayoutComponent`, `Fill`, `SolidColor`.
* Attributes name properties: `width="500"`, `name="Main"`, `loopValue="pingPong"`, `interpolationType="linear"`, `colorValue="FF57A5E0"`.
* Objects are nested, e.g. `StateMachine > StateMachineLayer > EntryState > StateTransition`, and `Fill > SolidColor`.
* Explicit identities and references use pairs, e.g. `id="0:30"`, `artboardId="0:30"`, `stateToId="0:12"`. Some nodes omit IDs. The producer's auto-ID behavior, scoping rules, duplicate-ID diagnostics, and full reference-remapping algorithm cannot be established here.
* The example contains multiple top-level artboards followed by a `Backboard`; it is an XML-like fragment rather than a single-root XML document.
* Animation keys identify the target object and numeric property: `KeyedObject objectId="0:2"`, `KeyedProperty propertyKey="13"`, and `KeyFrameDouble value="300" frame="60"`. Property 13 is `Node.x`; 14 is `Node.y`. [Animation example](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/assets/rml/focus_bounds_moving_host.rml:49), [generated property keys](/Users/levi/dev/oss/rive-runtime/include/rive/generated/node_base.hpp:37).

This is close to an explicit serialization of scene and behavior objects. There is no evidence here of HTML/CSS compatibility or a higher-level UI component syntax.

## Schema, references, and binary representation

Generated runtime classes carry stable numeric type and property keys. `Shape` has type key 3, while inherited `Component.name` and `Component.parentId` have property keys 4 and 5. JSON definitions feed the Dart core generator; its configuration says the canonical definitions live in the editor repository's root `dev/defs`, with runtime-only and editor-only output separated. Thus the generated C++ files are useful for understanding the consumer schema but do not constitute a full RML grammar or exporter. [ShapeBase](/Users/levi/dev/oss/rive-runtime/include/rive/generated/shapes/shape_base.hpp:13), [ComponentBase](/Users/levi/dev/oss/rive-runtime/include/rive/generated/component_base.hpp:45), [generator](/Users/levi/dev/oss/rive-runtime/dev/core_generator/lib/main.dart:7), [generator configuration](/Users/levi/dev/oss/rive-runtime/dev/core_generator/lib/src/configuration.dart:1).

Runtime references differ from editor identities: the runtime `Id` is a `uint32_t` index in the containing artboard's object array; editor builds use `{client, object}` pairs and wrap runtime indices as `{0, index}`. Export therefore must translate authoring identities into runtime reference spaces; the exact RML translation is outside this repository. [Id contract](/Users/levi/dev/oss/rive-runtime/include/rive/core/id.hpp:8).

The binary starts with `RIVE`, version numbers, a file ID, and a property-type table. Objects then encode a variable-length type key followed by property-key/value pairs terminated by property key 0. `CoreRegistry` constructs objects, and generated deserializers consume their fields. Unknown fields can be skipped when their type is known through the registry or header table; absent type information prevents decoding. Object ordering also matters: the importer tracks enclosing import contexts, and a `DataBind` attaches to the last bindable object. [Header](/Users/levi/dev/oss/rive-runtime/include/rive/runtime_header.hpp:47), [objects](/Users/levi/dev/oss/rive-runtime/src/file.cpp:171), [binding/import context](/Users/levi/dev/oss/rive-runtime/src/file.cpp:352). Latest upstream reads the file ID as a 64-bit varuint; do not copy the older checkout's narrower file-ID implementation. [Latest header](https://github.com/rive-app/rive-runtime/blob/25a2dc10786955df879ebc295e7c67923b8bde90/include/rive/runtime_header.hpp#L69).

## Assets and scripts

Assets can have inline `FileAssetContents` or be supplied through a host `FileAssetLoader`. The loader gets the first opportunity to claim an asset, then inline bytes are decoded if present; unresolved/asynchronous assets are permitted. This describes the `.riv` consumer contract, not RML asset path or embedding syntax. [Asset importer](/Users/levi/dev/oss/rive-runtime/src/importers/file_asset_importer.cpp:28), [contents attachment](/Users/levi/dev/oss/rive-runtime/src/assets/file_asset_contents.cpp:8).

Scripts are compiled asset content at this boundary. `ScriptAsset::decode` reads a signed-content header and stores bytecode; script backends are selected under scripting build flags, with Luau and WASM branches. Nothing found here establishes an RML inline-script syntax or its compilation/signing workflow. [Script content](/Users/levi/dev/oss/rive-runtime/src/assets/script_asset.cpp:181), [backend selection](/Users/levi/dev/oss/rive-runtime/src/assets/script_asset.cpp:74).

## What the tests actually prove

1. An RML-exported binary can drive a view-model trigger, fire a named event, and report it once to the host after advancing. This test reads `.riv`, never `.rml`. [Event test](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/state_machine_event_test.cpp:379).
2. `data_enum_roundtrip.rml` is loaded into a `BlobAsset` as opaque bytes and assigned to a view-model blob property. The test checks asset identity and byte count. It does **not** demonstrate parsing RML into an artboard. [Blob test](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/runtime/data_binding_blobs_test.cpp:139).
3. The focus fixture documents two concrete authoring constraints: layout position comes from layout style/Yoga rather than node `x/y`, and a nested artboard needs its own state machine/listener for a focus action targeting its local objects. [Fixture explanation](/Users/levi/dev/oss/rive-runtime/tests/unit_tests/assets/rml/focus_bounds_moving_host.rml:18).

## Additional producer details from upstream commit descriptions

The following is first-party design/history evidence, **not inspected producer implementation or a verified local compiler run**:

* The initial RML work describes a compiler, XML property filtering, ID stripping/clash fixes, sorting, parenting definitions, bit flags, keyframes, interpolator children, a CLI, and serialization to editor `.rev` files. [Initial RML commit](https://github.com/rive-app/rive-runtime/commit/03d74fb282b2961fa8ebc5156a0f3c0b716ee61a).
* The Rive CLI commit describes a watch/compile/package pipeline, a live viewer, `.rev` export, Luau optimization and headless script tests, imported project libraries, and emitting `.riv` through RML's writer. It also describes generated RML core mirrors, nonnumeric colon IDs treated as labels, XML parse diagnostics, synthesized ID namespaces, dangling unresolved labels, scanned scripts/shaders registered as modules, and script asset dependency ordering. [Rive CLI commit](https://github.com/rive-app/rive-runtime/commit/2cfa84e8103aeeeff4c2bfee92839ab580521660).
* Semantics authoring required generated RML mirrors to fold `is*` attributes into bitmask fields at parse time: `isHidden="true"` and `stateFlags="256"` should produce the same bytes. The same change describes inferred role traits and `FocusData`, `--verify` treating failed binary re-import as an error, an `inspect` command for graph mistakes, and runtime accessibility-tree checks. This demonstrates why a generic XML-to-property conversion is insufficient. [Semantics authoring commit](https://github.com/rive-app/rive-runtime/commit/edddc609a6595b3a4bc9df637d47b4ae127a2a40).
* The exporter assembles manifest sections, including name/path tables and watermark metadata, into one `ManifestAsset`. A reported bug involving two manifests caused name-based binding to lose its tables. The description also says exporter errors prevent output. [Manifest/exporter commit](https://github.com/rive-app/rive-runtime/commit/d8727299e08ba8517887a8a9f0fae80819208294).

Together these descriptions support a fuller authoring pipeline: XML-like input → schema-generated mirror objects → identity/parent/order resolution and semantic conversions → `.riv` runtime export or `.rev` editor export. Exact phase boundaries remain inferred until producer sources are available.

## Practical implications and remaining questions

The simplest integration boundary to investigate is **compile RML to `.riv`, then load the binary through the existing runtime**. A direct RML importer would need an explicit schema, enum conversion, identity remapping, ordering/import rules, asset handling, and exporter compatibility; merely parsing XML would not provide those behaviors. This is an engineering inference from the contracts above.

For Nuxie, use exported `.riv` now. Its binary reader explicitly requires the `RIVE` fingerprint, and its RML fixture usage likewise exercises a blob asset rather than parsing markup. [Nuxie header reader](/Users/levi/.codex/worktrees/7c27/nuxie-runtime/crates/nuxie-binary/src/lib.rs:6971), [Nuxie blob test](/Users/levi/.codex/worktrees/7c27/nuxie-runtime/crates/nuxie/tests/upstream_wave_a_final.rs:179). Native RML support would be a separate authoring compiler project; syntax conformance cannot be claimed without the producer sources and compatibility tests.

A reproducible RML authoring workflow still needs access to the producer package or an official distributable. Its full grammar, supported types, diagnostics, command invocation, asset resolution, scripting support, round-trip behavior, and version compatibility are not verifiable from this public runtime checkout. The searches covered tracked files in both checkout HEAD and fetched `origin/main`; no RML parser/compiler or direct RML build target was found. No RML compiler execution is claimed by this report.
