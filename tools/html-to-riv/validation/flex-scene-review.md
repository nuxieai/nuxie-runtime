# Final-scene prerequisites for flex analysis

The private flex analyzer now binds its restricted direct-leaf model to the completed emitted scene. This is an arithmetic prerequisite, not public flex admission, a generic Rive validator, or a pixel qualification. Runtime/renderer/schema files remain unchanged.

## Closed scene and ancestry

`src/flex_scene.rs` accepts exactly one Backboard and one Artboard followed by LayoutComponent, LayoutComponentStyle, Fill and SolidColor records. Every layout component has an explicit parent that precedes it and is a layout component or artboard. Since additional artboards are rejected, these parent chains terminate at artboard 0. All style links resolve to unique styles, and no styles remain unowned. Fills belong directly to a layout/artboard, have one solid color, and cannot share ownership or contain other record kinds.

Every record must use a closed set of understood properties. Adding an unexpected field to a future emitter therefore invalidates this restricted certificate. In particular, explicit displayValue, opacity, constraint/origin records, measurement-capable hosts and animation records are excluded. The artboard style is further restricted to container flow/alignment and checked context defaults; root size clamps cannot bypass the explicit caller-supplied viewport domain.

All layout ancestors and participants must satisfy the actual default transform, inherited direction, no interpolation, no wrapping, no intrinsic sizing, ordinary flex layout and zero inset/margin/gap checks. This validates the complete final ancestor chain, rather than inferring LTR or identity from a single child's omitted fields. The certificate has a private constructor and is generated only from the final record array.

`intrinsicallySizedValue` is a Bool with default false (schema property 606). Both a leaf's measurement behavior and an ancestor's available-size calculation depend on it. The check therefore applies to every layout, not only the analyzed leaves. See `flex-leaf-premises-review.md` and `flex-cross-target-review.md` for the immutable source paths and default/ContentSize arguments.

The final-source review also corrected an earlier minimum-default audit: `layout_component.rs:2183-2205` converts every flex child's Auto minima to point zero before Taffy, even when the initial Fixed-axis style retained Auto. The restricted zero-min guards remain conservative. This later normalization must be distinguished from the raw encoded/default style and from the browser's automatic-minimum semantics for nonempty content.

`displayValue` defaults to 0/Flex; 1/None would remove an otherwise valid point-sized component from layout. Rejecting every present displayValue closes this independent participation condition. Explicit defaults could be admitted by a later checked extension, but omission is the current restricted profile.

## Connection to the numerical model

The existing final SceneIndex still determines complete layout-child membership and actual native file order. The new scene certificate is required by rigid size propagation, world/corner propagation and the direct-leaf flex bridge. Neither a missing certificate nor an unknown parent size/world bound is replaced with a default proof.

The bridge checks direct participant identity, full childlessness, computed-to-native scalar bindings, actual Fixed point cross dimensions and bounds, exact parent flow/alignment, and actual file order. The cross target/origin proof is documented independently in `flex-cross-target-review.md`. The existing analyzer handles main-axis targets and world positions; the bridge also bounds each far corner and rejects geometry error above its supplied budget. Root size/world domains and their conversion errors remain explicit inputs.

Only paint-only leaf groups are analyzed. Descendants, helper wrappers, intrinsic/auto bases, unproved factor mappings and min/max freeze behavior remain unresolved. A successful local analysis is not a scene-wide/public admission decision. It does not establish browser pixels or reclassify a failed composition as impossible.

## Verification

Mutation tests cover intrinsic true/false, ancestor RTL/interpolation/rotation, artboard origins, display None, unexpected Text/Image/constraint/origin types and properties, wrong/cyclic/nonlayout parents, shared/missing/unowned styles, and malformed paint ownership. Actual compiler tests cover all four directions, CSS order and translucent fills, and reject changed cross scale/bound units or a missing scene certificate. Existing finalization tests still verify that late record/child mutations are observed.

The source-frozen public checkpoint is `output/flex-proof-checkpoint-r2`: 213 Rust tests, 36 Node tests, strict TypeScript, CLI/WASM builds and runtime source isolation pass. All 482 bound public outputs and 107 additional padding/provenance/gap controls retain exact Rive bytes and matching source maps. This is regression evidence, not new pixel qualification. Separate native bridge evidence is recorded in `flex-proof-bridge-review.md`: 4,392 native bound comparisons pass, while 26/96 Chrome pixel frames fail. All36 distinct pairs are visually reviewed. The native evidence does not expand public support.
