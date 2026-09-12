# Private signed rounding and live paint checkpoint

The private scalar module and a validation-only one-owner paint constructor now demonstrate live pixel-aligned painting using ordinary Rive objects on the immutable baseline. No public HTML/CSS feature is admitted by this checkpoint. The actual wrapped Derived composition has not incorporated these masks; its prior 48 pixel failures remain retained and unresolved there.

The constructor uses four 564-record signed-rounding graphs, a thin-extent predicate and four live clipping masks around an ordinary solid rectangle. It preserves unsnapped layout geometry. Its 28 closed recipes cover both axes, quarter/half/three-quarter/integer edges, responsive offsets and extents, negative positions, cumulative offsets, and thin extents at and above 1/16. Each one-owner recipe emits 2321 records. No browser measurement enters construction, no file is recompiled on resize, and ordinary original/clone imports require no observation metadata.

## Evidence

`output/playwright/rounding-paint-r1/receipt.json` records 224 geometry and pixel passes against Chrome 153.0.8010.12 and immutable Metal RasterOrdering, with unchanged gates. All 448 alternate-clear comparisons pass and 140 repeat/clone image pairs are exact. Zero thresholded mismatched pixels does not mean exact RGBA equality: maximum mean channel error is approximately 0.019461. The path receipt independently compares the actual 1120 emitted clips with Chrome-derived expected snapped boundaries; those measurements are validation-only.

Saved root and agent visual notes directly cover 24 full-size sheets / 84 distinct pairs, including empty thin-limit boxes and retained one-pixel lines above the threshold. The consolidated visual receipt binds both sets of notes and all source panels, plus 140 exact same-case transfers. Its source coverage file retains the original pre-review pending label as history; the completed review receipt is authoritative for review completion.

The before/after receipt checks 160 old/new comparisons with identical observed Chrome PNGs and owner boxes. All 24 prior standalone fractional-paint failures are preserved and their corresponding new frames pass. The two before/after triptychs were directly inspected and their notes are separately bound. This is equivalence of observed browser output for those fixtures, not identical authored request bytes or a new qualification of the wrapped candidate.

Scalar r1/r2 evidence covers 106 cases / 1696 frames / 410432 exact numerical f32 checks, all predicate stages included. Signed zeros compare numerically; 32 source comparisons differ only in the raw sign bit of zero after world transforms. The independent rerun reproduces 3922 artifacts byte exactly. Initial observer failures caused by treating shortest-roundtrip native f32 JSON as f64 remain preserved. No tolerance was introduced and no original capture was altered.

The frozen full product build passes 406 Rust / 56 Node tests and native/WASM/TypeScript checks. Public CLI/WASM byte identity permits reusing the recorded 282/794 historical artifact evidence; these are identity transfers, not fresh rendering campaigns. The verifier separately binds the effective immutable baseline build and compiler source snapshots.

## Limits and next integration

The scalar module requires finite selected coordinates in [-16384,16384], identity artboard space and an acyclic source dependency. The closed prototype recipes satisfy the reviewed construction assumptions; arbitrary JSON input is not certified. Native f32 versus Chrome LayoutUnit quantization, large-coordinate subtraction near the thin threshold, multiple overlapping owners, paint order, alpha composition at overlaps and practical aggregate resource cost need further work. The existing composition review records these boundaries in detail.

Integrate through compiler-owned construction, independent emitted-field validation, domain checks and aggregate resource budgets before applying the graph to wrapped Derived paints. Do not append mutations to records after their proof is created. Public wrapping still requires parser/planner integration, diagnostics and transport parity. No runtime, renderer, schema, shared dependency or root build modifications are justified by this result.

Recheck without rebuilding or recapturing:

```sh
python3 tools/html-to-riv/validation/rounding-paint-evidence.py
```
