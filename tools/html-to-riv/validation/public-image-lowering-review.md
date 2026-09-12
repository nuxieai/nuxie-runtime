# Public image lowering: source review and qualification corpus

The current ordinary-image lowering has a plausible source-derived sizing construction for the bounded reset contexts below. **This review does not qualify public rendering.** It reads the uncommitted implementation, reuses existing independently captured Chrome reset observations, and supplies 40 source fixtures for the next frozen public run. No compiler/native/browser run, production change or runtime change was performed by this reviewer. The parent subsequently compiled the exact 40 inputs, with the evidence correction below.

[public-image-cases.json](public-image-cases.json) contains **27 compile candidates and 13 diagnostic controls**, using exactly the seven existing authored image files. Static checks passed for JSON structure, unique case names, unique observed IDs per case, existing asset paths and the seven-file set. `expected.outcome:compile` requests a public admission check; it does not assert that geometry or pixels already pass. The corpus SHA256 is `de7ec9961c2c16f5a7c610f4fdc20bba014b1c73b31796d29d2866bbaf9d88a6`.

## Fixture contract and coverage

Each row separates `assetFiles` (exact source-key → module-relative fixture path) from `input` (`html`, `css`, `width`, `height`). A runner loads those files and adds only `input.assets[key] = {kind:"image",bytes:[...]}` to the public request. Metadata such as `notes`, `observeIds` and `expected` is not part of the request. Keys including `__proto__` require own-property handling. Resolve the browser's image sources from the same bytes without rewriting the compile request's authored keys. Compare source-map identities in DOM order, independent of CSS order or generated helper IDs.

The compile viewport is 390×320. To preserve the known sampling controls, the public native/browser run should include the existing image sequence **240×240 → 390×320 → 768×560 → 240×240**, on original and cloned artboards, with one compilation. Verify all observed authored boxes as well as per-image native decoded dimensions, fit, clipping and paint. Preserve complete frame gates and their failures.

| Candidate group | Cases | Purpose |
| --- | ---: | --- |
| Encoded formats and intrinsic start alignment | 7 | PNG opaque/alpha, baseline/progressive JPEG, lossless/lossy/alpha WebP; intrinsic box and following tail |
| Public root reset | 2 | Automatic stretched width and 50% width with automatic ratio height |
| Direction and stretch | 8 | Column/row, automatic axes, fixed main axis with/without cross stretch, reversed row and following tail |
| Fractional source sizes | 2 | Keep both linear and nearest private failure geometries |
| Fit, position and clip | 5 | Contain, cover, none, both scale-down branches, fractional percentage alignment |
| Cascade and overlapping paint | 2 | Noninherited fit/position, inherited sampling, explicit CSS-wide values, alpha and CSS order |
| Asset identity | 1 | Exact-key aliases, own `__proto__`, unused valid bytes, deduplication |

The [Chrome reset characterization](../output/ordinary-image-source-r1/public-reset-characterization/receipt.json) records 13 contexts × four views with Chrome **153.0.8010.12** and the same reset/96×64 image. It has geometry observations only. At width 240, public root auto sizing measures 240×160; start alignment keeps 96×64; 50% width gives 120×80. A 200×120 column gives auto image 200×133.328125, but explicit height 80 with cross stretch gives 200×80. A 200×120 row gives auto image 180×120, but explicit width 80 with cross stretch gives 80×120. These observations justify distinguishing branches; they do not transfer visual evidence to the newly authored sources with tails/backgrounds.

## Source findings

`images::plan` resolves the exact asset key, then distinguishes automatic main and cross dimensions using the parent's physical flex axis and effective self alignment. Both-auto/nonstretch uses encoded intrinsic pixels. Automatic main size, or an automatic nonstretched cross size with a fixed main size, adds the ordinary owner aspect ratio. A sole automatic stretched cross axis retains stretch without adding a ratio. This matches the distinctions seen in the reset observations. The intrinsic substitution currently updates the img's computed sizing/provenance before lowering; img is void, so this does not pass substituted dimensions to authored descendants. Do not reuse this mutation pattern as an inheritance proof for general container content.

`Plan::emit` keeps the authored Layout owner/background/source identity, clips the owner, and adds an ordinary Image plus a Fill/Fill LayoutParticipant. The Image uses origin zero, five existing fit values, percentage-to-alignment conversion, and the narrow ordinary sampler writer. No browser measurement or host setter supplies positions or dimensions. Since image padding is rejected, the authored clip and content box coincide for this profile. CSS order and reverse emission are still handled by the shared authored-child pipeline. Global ImageAsset/FileAssetContents pairs are spliced only after descriptor extraction, preserving existing artboard-local indices and source-map indexing.

`Bounds::image` derives an automatic ratio axis from the bounded opposite axis, clears unsupported lower/witness claims for that axis, rejects unknown containing bounds, and checks finite ratio/fit extents over the existing viewport domain. It does not turn the supplied compile viewport into a baked size. This is a focused exponent guard, not a complete world-position or pixel proof. Native Image fitting divides each layout axis by its decoded intrinsic dimension and then applies the chosen scale/alignment; final qualification must retain actual native finite geometry and pixels. This corpus's seven files all have ratio 96:64; it does not establish every admitted asset aspect ratio or extreme numeric boundary.

Two findings were repaired by the parent during this review and remain explicit diagnostic fixtures: (1) native-f32 rounding could admit an authored position above 100%; `position` now uses authored scalar provenance before accepting the range; (2) an image could feed empty-box zero metrics into ancestor baseline summaries; `finish_child` now propagates unknown image metrics/used-height into those groups. These are source-level confirmations, not claims of a newly run regression suite.

Two boundaries remain to state honestly. First, mapping `image-rendering:pixelated` directly to nearest sampling does not by itself qualify the full CSS property. The corpus preserves the fractional nearest failure and an inheritance/downsampling candidate; admission needs a decision based on fresh evidence or a narrower diagnostic boundary. Second, `value_grammar.rs` still omits the three new image properties. Existing substitution and CSS-wide handling apply, but primitive-invalid present variable values are conservatively Unclassified and then diagnose, rather than receiving the existing 33-property unset-recovery behavior. Do not advertise that broader recovery contract for these properties yet.

## Intentional diagnostics and asset contract

| Controls | Expected public result |
| --- | --- |
| Missing/empty src | `missing-image-source`; img requires a nonempty supplied-asset key |
| Unresolved exact key, including historical `asset:` spelling | `missing-image`; no fetch, path resolution or prefix fallback |
| Image padding, min/max, automatic margins, center/baseline alignment | `unsupported-target-semantics`; separate replaced-element qualification required |
| Ancestor baseline consuming automatic image height | `unsupported-target-semantics`; unknown replaced metric must not become empty-box zero |
| Percentage image height under automatic parent height | Existing automatic-containing-height diagnostic |
| Nonlegacy flex growth | Existing public flex qualification diagnostic |
| Position lengths/offsets or authored percentage outside range | Target diagnostic; valid unsupported CSS is not silently recovered as invalid |

All supplied assets are validated and all distinct supplied byte strings are emitted, including unused assets. Exact encoded bytes determine deduplication, not decoded pixels. Index assignment follows first appearance in sorted source-key order; request insertion order is irrelevant, but adding/renaming keys can change asset ordering. The alias fixture therefore does not assert renaming-invariant bytes. The shared typed asset tests own malformed containers, limits and transport validation; this bounded visual corpus does not replace them.

The earlier [ordinary image visual review](ordinary-image-visual-review.md) retains ten candidate pixel failures: four fractional-linear frames, four fractional-nearest frames, and two fractional-position contain frames. The new sources retain their dimensions/positions as controls; public flex-reset changes mean their old PNGs cannot be automatically substituted for fresh public PNGs. Keep geometry, whole-frame pixels and presence separate, including failures beneath/above individual thresholds. No blanket image, ratio, pixelated or browser-equivalence claim follows from successful import alone.

## Public admission preflight and preserved fixture correction

The parent-owned [r1 compile receipt](../output/public-image-layout-r1/compile-receipt.json) records 27 successful outputs and 13 diagnostics with the intended codes. Its original expectation check is **39/40**, because this corpus incorrectly expected `Nonlegacy flex values` for `image-flex-growth`; the earlier declaration guard actually reports `Nonlegacy flex declarations await aggregate numeric and visual qualification, including unmatched or overridden declarations`. This is a fixture wording error, not a compiler admission failure. The current fixture changes only that expected substring to `Nonlegacy flex declarations await`. The original input, expected substring and mismatch remain in [the r1 result](../output/public-image-layout-r1/image-flex-growth/compile-result.json) and frozen fixture (original SHA256 `057715b9f3edc7430d767d4686ef29ecd6526be50984373644791734129c1321`).

A read-only comparison by this reviewer confirmed **all 40 current hydrated requests exactly equal their r1 request.json**, including HTML, CSS, viewport and every supplied byte. All 40 recorded results meet the corrected current expectations; no rerun or historical receipt was rewritten. The [r1 build receipt](../output/public-image-layout-r1/build-receipt.json) binds its candidate compiler/source snapshots and explicitly retains an open malformed-JPEG recovery admission issue. This preflight establishes admission for these exact inputs, not final release or public paint qualification. Fresh native/Chrome capture is parent-owned and remains pending in this review.

Compile-receipt SHA256: `c6ea76de33baa78145d24705a0c7c9ba09d251f6fb1985186bcea13e04156f7d`; build-receipt SHA256: `e501931c6f32d9d99f4d58537e3a054953ecee1e1bb1c9067cced4e513602739`.

## Review snapshot

These SHA256 values identify the source read for this review, **not a final frozen compiler build**. Parent-owned edits and final qualification must bind their final source/binaries independently. Immutable consumer baseline remains `6c7ac16617835b5f581784ff08a9e779bb52faf3`.

| Module-relative source | SHA256 |
| --- | --- |
| `src/images.rs` | `027dd2b80f03a25ef9489ee66d267d1345b2fc530c18923e3aeefc32dd317df9` |
| `src/compiler.rs` | `912b8a70352bd93d29f71d569b9262319da682a4c3480a449233c50d4cbca3a4` |
| `src/numeric.rs` | `d093c82e0f59e9978b0d073ef335ef8346893b9ddf7d4a2a257a3052cb510a8e` |
| `src/reset.css` | `a61eb8e0d9c0cfa6e544fdd6d1a931e92a0dca18c6ca02806b1aae2dc7f03c5a` |
| `src/assets.rs` | `a1a2db57accd9fc72d9e3fff15d9d84fead7e2501073809430bfff3cb3f2b069` |
| `src/value_grammar.rs` | `b89c163683ffacba309375728c65476fc72853003605ed124fd22e991ed80f1d` |
| `src/flex.rs` | `0f25fd88f7dddcee08b0c53db4447db18ff72a0158aabdec0bfd1937c604663c` |
| `src/baseline.rs` | `bf4f42624b6eb1b60f9ef0b3333d0b3cb4842d63de88e52de106a7e7507ac9a5` |
| `src/wire.rs` | `3298102ed25ec660b3a9ac01414b6c6cb309e67961a6181c9475be0cbcf19725` |

Existing evidence receipts: Chrome reset `965521cb6f5e0438b4dfd3b9071d4dfe5d01cfa25482ed7270cf83c55e9c1cf3`; private ordinary-image visual review `c28c362d68c8539cc50089ca76f878110e33863e5a9a6259071a7db8690ef06a`. Their source/asset bindings remain authoritative for their own observations only.
