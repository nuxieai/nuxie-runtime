# Support against the unchanged runtime

The replacement compiler has no qualified release yet. All 99 previous backlog items remain in scope, with qualification reopened against the immutable target in TARGET.md. Historical parser tests and browser references remain useful; previous native images exercised modified code.

| Feature family | Existing-Rive approach to test | Current decision |
| --- | --- | --- |
| Colors, selectors, cascade, custom properties, relative units, text casing | Compute in compiler; emit ordinary values | Preserve algorithms selectively; end-to-end revalidation pending |
| Flex, alignment, sizing, aspect ratio, positioning | Ordinary LayoutComponent properties; possible layout/spacer/containment compositions | Exact CSS combinations unresolved; independent grow/shrink and shared solver corrections cannot be assumed |
| Text, spacing, wrapping, tabs, ellipsis | Ordinary Text/FontAsset/Style records | CSS metrics and reflow unresolved; removed shaping policies cannot be installed |
| Borders, corners, decorations | Ordinary shape/path/fill compositions tied to existing layout | Responsive dimensions, overlap, clipping and glyph-relative placement require proof |
| Stacking and overflow | Ordinary hierarchy/draw ordering/clipping shapes | Candidate file arrangements need ancestry, resize and pixel tests |
| Linear/radial gradients | Ordinary gradient records; radial ellipse via transformed wrapper is a candidate | Premultiplied alpha, hard stops, repeat and responsive endpoints/radii unresolved |
| Group opacity | Search for serializable baseline group isolation | Multiplying child opacity fails overlap semantics; host canvas availability does not prove file support |
| Images and assets | Ordinary embedded asset/image objects | Revalidate baseline decoder, layout, filtering and backend pixels |

The 36 former custom capabilities are unavailable on this target. That establishes that the old mechanism is unsupported, not that every associated CSS feature is impossible to represent differently. See validation/immutable-compiler-audit.md for the complete per-capability mapping and validation/immutable-runtime-audit.md and validation/immutable-renderer-audit.md for source inventories.

CSS Grid, editor integration, scripts, interactions, bindings and animation stay excluded. No silent raster, fixed-layout or recompile-on-resize fallback is enabled.

Initial experiments now provide56geometry/50pixelpasses and6preserved direct-corner failures, with a bounded ordinary-file corner composition passing8/8. These are not public compiler qualifications; see validation/ordinary-layout-review.md.
