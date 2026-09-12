# Public content-owner integration: completed output regression

All **794 prior public requests compile successfully** with the frozen public build. **719 Rive files and maps reproduce exactly; 75 files change intentionally for the ordinary content-owner composition.** Of those changes, 67 also move source-map object indices and eight leave the map byte-identical. No new diagnostic, partial output, map-only change, or authored source-ID/path change occurred. Native qualification of the changed files is tracked separately; this record audit does not substitute for it.

The new public compiler SHA-256 is `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5`. Its source freeze contains 125 verified bindings. The full build receipt reports 270 Rust tests and 46 Node tests passing. Results, complete requests, old/new artifact bindings and compile logs are preserved in `output/public-content-owner-regression-r1/`. The raw exact-byte checker exits 1 because 75 files changed; `classification.json` and `semantic-review.json` explicitly classify those differences rather than silently replacing their old references.

## Disposition of all changed records

Every changed file adds one or two pairs of existing ordinary records: one `LayoutComponent` (type 409) and one `LayoutComponentStyle` (type 420) per content owner. Across 75 files there are **92 added layout objects and 92 added styles**: 58 files add one pair and 17 add two. All other object-type counts are unchanged. Every file retains exactly the same property-key/field-type vocabulary and object-type vocabulary, including its unchanged artboard, host style and ordinary white host paint. No asset object, schema vocabulary, file-format version or requirements sidecar is added.

The independent decoded-record checks verify all 75 files:

- Authored names, source IDs and DOM paths remain exact. Ignoring synthetic owners, each authored layout still has the same nearest authored parent.
- Authored layout fields change only `parentId` and `styleId` links. Their name, width and height fields remain unchanged.
- Authored style fields change only `flexDirectionValue` and `layoutAlignmentType`, which describe the outer packing container. Size units, min/max bounds, padding, margins and other authored style fields remain exact.
- Encoded paint colors, paint ownership by authored source, and paint order remain exact. Synthetic helpers stay unpainted.

There are 89 changed parent links and 102 changed style links; 23 authored styles change outer flow direction and 24 change alignment. Source-map numeric indices move in 67 files as ordinary records are inserted; all public identities remain intact. The eight unchanged maps still accompany changed Rive bytes and receive the same record-level audit. Each case has its own decoded before/after file and explicit disposition in `semantic-review.json`.

The affected sources are 46 historical content-box/numeric cases, one value-recovery case whose box sizing computes to content-box, 22 additional core content-owner cases, and six stretch controls. The changes agree with the reviewed emission boundary: eligible point/auto content-box dimensions with nonzero point padding acquire an ordinary unpainted content owner while the existing outer owner retains padding and paint. Structural agreement establishes the intended change category; it does not prove every layout result.

## Native handoff

`changed-native-cases.json` contains all 75 exact original HTML/CSS sources and their original `compileViewport:[390,160]`. Every request has exactly the fields HTML, CSS, width and height; no options were dropped. `changed-native-bindings.json` binds each request, reference index, old/new Rive file and map. Its fixture SHA-256 is `4f37db2d6dcf1f7774d3d665a13869904cc8429884aeb3ebe5ade7ee8c86d9e3`.

The native task covers 34 cases through exact source/file/map bindings to its core corpus and renders the other 41 changed sources independently. Its results and retained failures belong to the separate public native receipt. No private candidate file was used as an old public reference, and this regression review does not infer qualification from an earlier private composition.

## Preserved reference set

The reference set contains all 731 historical public-value regression rows, all 35 new value-native scenes, 22 additional core sources, and six additional stretch controls. Six core sources exactly duplicate historical requests and outputs; their artifacts and provenance remain copied as aliases. Deduplication ignores only JSON formatting/key order, preserves every request field and exact HTML/CSS spelling, and requires identical Rive bytes and maps. Identical output alone never causes source deduplication.

The immutable reference manifest is `output/public-content-owner-regression-references-r1/manifest.json`, SHA-256 `32b141a43436eb877cbca963bcb15556f6677e33ffe346f7ae30880ed2ea0bec`. Its old public compiler is `034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d`; that binary and 118 source-input snapshots are copied under `frozen/`. All 2,400 reference/alias request, Rive and map artifacts were rechecked. Known historical native failures remain attached to their source provenance and are not relabelled as passing by inclusion in this regression.
