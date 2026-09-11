# Smallest ordinary embedded-font text experiment

Read-only groundwork on the immutable runtime, 2026-09-11. No text support, import success, Chrome parity or responsive behavior is claimed. This document proposes one bytes-only experiment; public assets/text admission remains a separate compiler decision.

## Existing records and relationships

All paths below start at `crates/nuxie-runtime/src/mechanical_port/source/` unless stated otherwise. Property names/types should be obtained through `nuxie-schema` and the existing wire writer, not encoded by guessing byte widths.

| Record | Type | Properties / relationship |
|---|---:|---|
| FontAsset | 141 | Inherits Asset name203 and FileAsset assetId204; CDN UUID359/baseURL362 are unnecessary for embedded data. Global asset record, not a Shape child. |
| FileAssetContents | 106 | bytes212 is CoreBytesType (length-prefixed blob); optional signature911. **Not a Component: no parent5.** Must occur while the intended FontAsset importer is active, normally immediately after that asset. |
| Text | 134 | Component parent5 and node x13/y14. align281; sizing284 (AutoWidth0, AutoHeight1, Fixed2); overflow287; width285/height286; originX366/originY367; paragraphSpacing371; origin377; wrap683; verticalAlign685; fitFromBaseline703; newer trim1026–1028. Start by omitting nonessential options rather than assuming CSS-equivalent defaults. |
| TextStylePaint | 137 | Inherits TextStyle properties: fontSize274, lineHeight370, letterSpacing390, fontAssetId279. parent5=Text. Owns ordinary Fill/SolidColor paints for glyph paths. Do not instantiate abstract TextStyle573 merely to get these properties. |
| TextValueRun | 135 | parent5=Text; styleId272 refers to the **artboard object ID** of TextStylePaint; text268 is UTF-8 string. |
| Fill / SolidColor | 20 / 18 | Fill parent5=TextStylePaint; SolidColor parent5=Fill, color37=ARGB. Same ordinary paint hierarchy as shapes. |

Schema: `generated/assets/{asset,file_asset,font_asset,file_asset_contents}_base.rs`; `generated/text/{text,text_style,text_style_paint,text_value_run}_base.rs`; `generated/shapes/paint/{fill,solid_color}_base.rs`.

**Two namespaces must not be conflated:** TextValueRun.styleId272 resolves an artboard object through CoreContext (`text/text_value_run.rs:140–153`). TextStyle.fontAssetId279 is an index into the backboard file-assets vector (`importers/backboard_importer.rs:222–231`), not the arbitrary FileAsset.assetId204 and not an artboard object index. For the first embedded font with no preceding assets, use fontAssetId=0. The file-level asset record can carry assetId=1 for identification without changing that vector index.

## Candidate file record sequence

Use the existing wire encoder's ordinary header/property table. Track artboard-local IDs separately from global file record positions. Each Artboard starts its own object list with itself at local ID0. FontAsset and FileAssetContents records preceding the Artboard do not consume artboard-local IDs. The placeholders below are local identities, not global record offsets.

```text
Backboard
FontAsset141 {name203:"Roboto subset", assetId204:1}
FileAssetContents106 {bytes212:<exact roboto-a.ttf bytes>}
Artboard {name:"embedded-text", width:240, height:100}
Text134 T {parent5:0, x13:20, y14:20,
           sizing284:0, originX366:0, originY367:0}
TextStylePaint137 S {parent5:T, fontAssetId279:0,
                     fontSize274:32, letterSpacing390:0}
Fill20 F {parent5:S}
SolidColor18 {parent5:F, color37:0xff000000}
TextValueRun135 {parent5:T, styleId272:S, text268:"aaaa"}
```

**Importer-confirmed numbering:** `artboard.rs:2676–2691` asserts an empty per-artboard object list and adds the Artboard itself first. `component.rs:742–759` appends subsequent Components through the active ArtboardImporter; `importers/artboard_importer.rs:18–19` delegates to that artboard's `add_object`. Global assets instead register with BackboardImporter (`assets/file_asset.rs:91–101`), while FileAssetContents hands bytes to FileAssetImporter and is not a Component. Thus this exact sequence has T=1, S=2, F=3, SolidColor=4, TextValueRun=5, and fontAssetId=0 independently. Adding a second Artboard restarts its local numbering at0. The current compiler's `Emitter` expression `records.len() - 1` works only for its existing no-global-assets record arrangement; extending font emission must introduce explicit local-ID accounting rather than reuse that expression. Unknown/null object slots inside an artboard must also follow importer accounting, rather than counting only successful typed records.

This uses the already-present vector glyph path. Baseline `text/text_style_paint.rs` owns a ShapePaintContainer and glyph paths; `text/text_value_run.rs:123–137` registers each run with its parent Text. Keep one style/run initially so style indexing, painting and font decode failures remain distinguishable. Node coordinates set a starting placement, but origin/baseline semantics must be measured; y=20 is not a claim that the first glyph's CSS baseline is20.

No `FontAsset::set_font_occurrence`, host font restoration, text metric policy, runtime source-map mutation or custom renderer adapter belongs in the experiment. Some existing tests use these setters and therefore are not sufficient evidence for bytes-only font provisioning.

## Why embedded provisioning should use the normal import path

1. `assets/file_asset_contents.rs:21–31` obtains the latest file-asset importer and hands it the contents occurrence. It retains bytes directly from the CoreBytesType reader (`generated/assets/file_asset_contents_base.rs:54–60`).
2. `importers/file_asset_importer.rs:58–103` takes the embedded bytes; when an optional loader does not consume them, it invokes the asset's ordinary decode path. The experiment should use no custom asset loader and no external fetch.
3. `assets/font_asset.rs::FontAsset::decode` calls the ordinary factory's `decode_font(data)` and then `HbFont::decode(decoded.bytes())`. This normal import-time decode populates the font; a post-import setter would bypass the mechanism we need to prove.
4. `text/font_hb.rs:8–16,82` uses **harfrust shaping plus skrifa outlines**, despite the historical HbFont filename. It is not Chrome's platform glyph rasterizer. Font feature selection, legacy kern handling, standard 2048 scaling, fallback and variable outlines are baseline behavior, not available CSS policies.
5. Backboard resolution assigns decoded FontAsset handles to referencers by asset-vector index. Observe that the style has a usable font and that the recording contains actual glyph paths; import returning Ok alone does not prove decoding succeeded, because the generic importer can finish without a populated font.

## Fixture available locally

`fixtures/fonts/roboto-a.ttf` exists in the immutable worktree; it is omitted by an ordinary ignored-file search, so discover it with `rg --files --hidden --no-ignore`. `fixtures/fonts/README.md` documents a Roboto Regular 2.137 subset containing only **U+0061 (`a`)**, SHA-256 `b481b059ee94961c7b18585a596935aaa7cc44b68879c096d2cd06922e0431b1`. Its Apache-2.0 license is `fixtures/fonts/LICENSE-ROBOTO.txt`.

Use only repeated `a` in the first probe. Do not use “Hello”, punctuation, emoji or spaces and then mistake fallback for embedded-font success. Copy this fixture and its license into compiler-owned test assets if needed; do not make runtime fixtures depend on the compiler. Existing `tests/host_semantic_projections.rs` references this font, while `tests/cpp_probe_text_support/mod.rs:120–122` installs a host FontAsset directly and is not a bytes-only template. Existing `.riv` text-related samples include `fixtures/fl-e8/text_style_feature.riv` and `text_variation_modifier.riv`; these can help inspect record structure but are larger than the proposed control. No populated upstream font checkout was found by the targeted search; Playwright's bundled codicon font is unrelated and should not become the text fixture.

## Next concrete experiment and preserved risks

First emit this file in a compiler-local fixture generator, import its bytes with the immutable factory and record/render “aaaa”. Bind font/file/binary hashes. Assert an actual nonempty glyph path and expected FontAsset/style/run identity, then load the same exact font in pinned Chrome through local `@font-face` with font 32px and the existing explicit reset. Await `document.fonts.ready` and assert the intended font loaded. Compare glyph advance/bounds/baseline separately from full native PNG pixels. Preserve any mismatch rather than adding a glyph adapter.

Then add a LayoutComponent parent and an ordinary Fixed-width or AutoHeight Text candidate, resize the same original and clone through narrow/wide/return widths, and verify line placement without recompilation. Add a second differently colored run only after the one-run case proves style/font indexing. A short uppercase/lowercase/general-text corpus needs a full, licensed font fixture first.

A07 `font` shorthand can be parsed in the compiler, but must resolve to an admitted family/weight/style font asset and ordinary fontSize/lineHeight properties. Family names are not themselves a Rive wire font resource. Do not assume that a numeric CSS line-height multiplier equals property370's units, that CSS normal line-height equals the baseline metrics, or that an italic/bold keyword synthesizes the right font. The first experiment should omit lineHeight370, then measure an explicit ordinary lineHeight control before proposing shorthand admission.

Known architectural risks from `immutable-runtime-audit.md` and `immutable-renderer-audit.md` remain: baseline shaping/fallback/kerning can differ; wrapping and trailing whitespace/tab behavior are not the old CSS policies; glyph hinting/AA and subpixel placement differ from Chrome even with equal advances; legacy layout rounding and origin/trim fields can alter bounds. Ordinary inherited opacity is not isolated CSS group opacity. No old modified-renderer text receipt transfers. This experiment determines the first viable ordinary text entry point; it does not reintroduce any rejected runtime extension.
