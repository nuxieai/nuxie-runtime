# Public content-box owner checkpoint

The public compiler now emits a separate ordinary content owner for content-box elements with nonzero fixed padding and point/auto preferred and min/max dimensions. This preserves a small authored content dimension when adding and then subtracting large outer padding would lose it. The corrected corpus passes all geometry gates. Six existing fractional-edge paint failures remain; this is partial L12 support, not exact browser rendering or general content-box qualification.

The runtime, renderer, schema, dependency resolution and pixel gates remain unchanged. The output uses existing `LayoutComponent` and `LayoutComponentStyle` objects. No metadata, host callback or runtime policy participates in import or layout.

## Implementation and admission

The authored outer object retains its source identity, translated border-box dimensions, padding and paint. An unpainted inner object stores the authored content dimensions and bounds, and owns the authored children and their distribution. The outer packs that object along its actual parent's axis, preserving automatic cross-axis stretch. This parent-axis choice fixes the 16 preserved stretch regressions of the initial private composition.

Computed styles and inheritance stay attached to authored elements. A small emission plan is shared by layout emission and numeric checking; it does not copy inherited custom-property environments. Numeric bounds evaluate the outer and then the actual inner, preventing padding cancellation from hiding an overflowing descendant percentage chain. The 20 controls retain 12 successful inputs and reject eight unsafe inputs. Six of those diagnostics become necessary because the new graph preserves a small inner value that the old graph cancelled to zero.

Public restrictions on padding, alignment wrappers, baselines, gaps and nonlegacy flex remain. Percentage dimensions and bounds do not use the new owner. Existing admitted percentage contexts retain their prior lowering; five proposed percentage/point mixtures retain diagnostics. No new claim is made about percentage-plus-point owners, unknown intrinsic sizing or aggregate world-coordinate precision.

Diagnostic descriptors bind the actual inner parent and authored content provenance. They represent authored child lists; they do not enumerate every generated edge as a separately certified arithmetic group. All size, world and flex proof consumers explicitly reject the marked composition until it has its own arithmetic model. A regression first proves an ordinary eligible group, then changes only the composition marker and requires rejection even with previously valid domains. See the [independent source review](public-content-owner-source-review.md).

## Public and native validation

The frozen public build passes **270 Rust tests, 46 Node tests, strict TypeScript and native/WASM builds**. Inputs remain unchanged across the build. Seven public integration tests cover 34 exact scene/map references, inheritance, 20 numeric controls, five retained diagnostics, depth 128/129, element count 8192/8193 and a large inherited custom-property environment. CLI/WASM tests cover the corresponding output, diagnostic and resource boundaries.

All **794 prior requests** compile. **719 files/maps remain exact**; the other **75 files** have reviewed ordinary composition changes, with source-map index changes in 67. Authored IDs, paths and nearest authored parents remain exact. The changed files add 92 ordinary layout/style pairs without new object or property vocabulary. Paint ownership, colors and order remain intact. Complete old references are preserved rather than replaced. See the [regression review](public-content-owner-regression-review.md).

| Corpus | Frames | Geometry passes | Pixel passes | Alternate-clear passes |
| --- | ---: | ---: | ---: | ---: |
| 34 changed references with exact source/file/map evidence transfer | 272 | 272 | 266 | 544 |
| 41 freshly captured changed references | 328 | 328 | 328 | 656 |
| Separate 128-level resource control | 8 | 8 | 8 | 16 |

Each changed file is compiled once at its original request viewport. Original and clone resize through 240×160, 390×200, 768×120 and back to 240×160. The 75-reference native aggregate therefore contains 600 geometry passes, 594 pixel passes and 1,200 clear passes. The depth control brings the totals to 608, 602 and 1,216 respectively. It is unpainted, so its white images do not validate painted descendant complexity.

A separate unpainted 8,192-element file imports, clones and resizes with all 16,385 native layout objects present and finite. That test establishes native resource handling and geometry only; it has no Chrome or pixel qualification. See the [native and visual review](public-content-owner-native-review.md).

Full fresh visual coverage consists of 94 directly inspected complete Chrome/native pairs on 16 sheets and 242 exact complete-RGBA transfers. The 34 core cases retain their bound prior visual review, including all failures. The parent additionally inspected the complete pairs in sheets 5 and 15, fresh references 651/frame 1 and 713/frame 0, and the retained fractional control/frame 0. Padding, child sizing and ordering agree in these inspected frames. Fractional boundaries visibly differ; the white depth image offers no additional painted-content evidence.

## Retained limits

The fractional control fails the existing mismatch-ratio gate in frames 0, 2, 3, 4, 6 and 7. Some passing frames also contain visible boundary coverage differences: reference 651/frame 1 has a maximum channel difference of 103 at one recorded edge pixel. Gates have not changed, and passing them is not pixel identity.

The large rounded-outer child is offscreen. Native width 999999.8125px differs from Chrome's projected rectangle width 999999.75px by 0.0625px, inside the existing 0.1px geometry gate. Its screenshot does not prove the offscreen geometry. This composition repairs the measured failure without establishing all-coordinate exactness.

L12 stays partial. The all-item counts remain 13 qualified, 14 partial, three investigating and 69 pending. Broader typography, paint, assets, layout combinations and compiler quality remain within the goal.

## Evidence and reproduction

- Public build: `output/public-content-owner-build-r1/summary.json` and `frozen/source-bindings.json`.
- CLI SHA-256: `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5`.
- WASM SHA-256: `c475d543a5f9a4ffcf5f629ef05db5b970b102e2b7682be44419f4b200ed977e`.
- Output disposition: `output/public-content-owner-regression-r1/receipt.json` and `semantic-review.json`.
- Native/visual coverage: `output/public-content-owner-native-r1/receipt.json`.
- Immutable baseline: `6c7ac16617835b5f581784ff08a9e779bb52faf3`; pinned Chrome `153.0.8010.12`; Rust Metal RasterOrdering.

Run `python3 tools/html-to-riv/validation/check-target-runtime.py` for source identity. `public-content-owner-checkpoint.py` verifies the bound checkpoint evidence without regenerating renders. To reproduce the public build independently, run `python3 tools/html-to-riv/validation/public-value-build.py NEW_OUTPUT_DIRECTORY`; despite its historical name, the script runs the current complete public suite and freezes its inputs. The native driver accepts an optional explicit `compileViewport` for exact historical requests, defaulting to 390×160; its original/clone resize sequence and gates are unchanged.
