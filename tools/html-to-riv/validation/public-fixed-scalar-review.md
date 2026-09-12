# Private fixed scalar evaluator: source audit

Read-only audit of the compiler evaluator against the immutable Rust runtime. No native execution, browser measurement or builds were performed for this review. Tests exercise expected examples; they do not independently establish runtime equivalence.

## Boundary issue found and repaired

`fixed_scalar.rs:139–151` builds the complete parent index, but calls `visit` only for requested output dependencies. `visit:78–86` skips child Nodes without inspecting their own parent-linked descendants. Consequently the evaluator can certify requested Node 1 in `[Backboard, Artboard, Node(x=1), Node(parentId=999)]`: the final Node is indexed under 999 and never validated. A requested constant is mathematically independent of it, but the full file does not meet the claimed closed modeled-graph premise. Native `component.rs:609–634` resolves every component's parent and returns MissingObject if absent; `artboard.rs:778–785` runs dirty import callbacks over all objects. Its `can_continue` helper at 283–284 tolerates MissingObject and stops only on InvalidObject; therefore this finding does **not** establish that a dangling parent necessarily prevents import.

A full encoded-byte binding proves input identity, not full-file import validity. Before treating this token as a whole-file folding certificate, globally validate the admitted kinds, property sets, parent handles, target handles and acyclic dependencies, including unrequested Nodes; alternatively require an independently established whole-file validity token as an explicit precondition. Add mutation controls for a dangling unrequested parent, an unrequested Node with an unsupported child constraint, and a disconnected cycle. This issue did not change the modeled arithmetic on the closed rounding fixtures, but the original API/documentation did not enforce its claimed closed-graph premise globally. The finding was sent to the parent agent before this note was written.

The parent agent repaired the boundary during this review. The revised evaluator globally restricts record kinds to Node/TranslationConstraint, checks every parent role/handle, and visits every Node after the requested outputs. Thus all constraints and Node property sets are validated, including disconnected cycles and unsupported fields outside the requested closure. I inspected these changes and the added dangling-parent, self-parent-cycle, dangling-constraint and unrelated-rotation regression controls. No unresolved concrete arithmetic mismatch was found in the supported subset during this source audit. The earlier defect is retained above as review history, not the current source status.

## Arithmetic and ordering checked

- Node defaults are rotation +0 and unit scales (`generated/transform_component_base.rs:20–25`). `transform_component.rs:165–185` builds the local rotation/translation matrix, scales it and composes with parent world. `math/mat2d.rs:61–67` uses exact zero sine/unit cosine for zero rotation; its negative sine entry is preserved by the evaluator. Node-only, identity-axis fields are intentionally whitelisted rather than admitting arbitrary explicit rotation/scale values.
- Matrix multiplication uses the pinned fused multiply-add operation ordering (`math/mat2d.rs:255–262`); point mapping likewise uses fused product sums then adds translation (`331–339`). Inversion uses the fused determinant and translated numerator (`181–195`). The evaluator retains full six-entry matrices and those operations, rather than treating local-space conversion as simplified scalar subtraction. Its admitted axes are invertible identity axes; nonfinite intermediate results conservatively reject.
- Translation target acquisition and source-local conversion follow `constraints/translation_constraint.rs:23–74`. Local conversion is inverse target-parent matrix multiplied by target world. Destination-local mapping uses the owner's parent world (`99–107`). A missing target uses original world translation and bypasses target-copy/offset handling. The evaluator explicitly admits the absent sentinel and conservatively rejects other unresolved handles.
- Copy-disabled axes are original world coordinates for world destination and zero for local destination (`76–98`). Copy factors precede composed-local offsets. The Node offset is literal x/y because `transform_component.rs:221–223` returns those coordinates; dispatch is in `generated/core_registry.rs:3377–3390`. Target constraints affect world transforms without rewriting those source literals.
- Min/max-space local mapping occurs before clamps; maximum applies before minimum, so minimum wins conflicts; mapping back occurs afterward (`translation_constraint.rs:109–146`). The strength-one evaluator retains the multiply/add blend (`148–159`) instead of direct assignment, preserving finite arithmetic and zero-sign behavior. Node anchors are zero (`transform_component.rs:228–229`), so `constraints/constraint.rs:201–204` returns without landing adjustment.
- Import walks objects in stored sequence (`artboard.rs:778–785`). Constraint dirty registration appends to its owner (`constraints/constraint.rs:69–84`, `transform_component.rs:242–243`), and application iterates that vector (`transform_component.rs:188–196`). Thus evaluating owner constraints in record order is justified for this closed subset. Parent dependency edges are registered by `transform_component.rs:102–110`; target-to-owner edges by `constraints/targeted_constraint.rs:128–135`. DFS over parent and constraint targets models their acyclic dependency closure. Self-targets or cycles are conservatively rejected rather than emulating runtime cycle behavior.

## Intentionally unresolved scope

Styled artboards, layout participants, origins, collapsed state, nonidentity transforms, animation, non-unit constraint strengths, unnamed transform spaces, unknown fields and nonfinite arithmetic do not have this certificate. A bare artboard's width/height are accepted as finite metadata; no layout result is inferred from them. External runtime mutations invalidate the constant premise even if the original file bytes are unchanged: the token is for the authored closed graph, not arbitrary later property changes.

`TransformConstraint` is unresolved even for Node targets: its target-bound origin selection, decomposition and recomposition (`constraints/transform_constraint.rs:24–33, 108–142`) are not implemented. `DistanceConstraint` is unresolved because its anchor, square-root distance, 0.001 threshold and normalization arithmetic (`constraints/distance_constraint.rs:45–92`) are not implemented. In particular actual `paint_box` corners use TransformConstraint targets that are LayoutComponents (`tools/html-to-riv/src/paint_box.rs:28–32`); this evaluator must not seed those from a viewport or browser measurement. Actual `paint_rounding` graphs fed literal Nodes are in the intended first arithmetic scope.

All runtime paths above are relative to `crates/nuxie-runtime/src/mechanical_port/source/`. This is a source-semantic audit with the identified closure gap repaired, not native qualification.

## Reviewed source bindings

- `tools/html-to-riv/src/fixed_scalar.rs`: `521d19bb570387a9e1b2be6d119fa6c82c79eee02fc5ada1b860dbe3ee4cffd5` (after closure fix)
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs`: `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/constraint.rs`: `d89f7c20f18c2f3ca437129d3098a6f98b11326bd0ad121aa932570bd64f364e`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/targeted_constraint.rs`: `3e2b462b6ecaf322ab7c333542c649a7c4ff3904f103b5c67001a885fd068b84`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/transform_constraint.rs`: `450f0443e9499db7c5b7a4261d77fe5ede678e40f975ec0b77278c6b5b240f8d`
- `crates/nuxie-runtime/src/mechanical_port/source/constraints/distance_constraint.rs`: `ee2d2df3ec8fe3bf2d23ff974823c7f71260206ff7f50c15c44dfc6e9b9844cf`
- `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs`: `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2`
- `crates/nuxie-runtime/src/mechanical_port/source/transform_component.rs`: `861f8b7d460f7cbb07456901acf19e70720bfd7559d8a50964bbbc00ddff6cac`
- `crates/nuxie-runtime/src/mechanical_port/source/component.rs`: `3cc15a54e88209e1ecf233c397f5a8edbd7e61405c04d540b3b5346c31a7b5a1`
- `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs`: `6f218337cbf8daa1567f20244cadd9f0676cb1e6546ae0bc23751d02b404da01`

## Subsequent native confirmation (separate from the source audit)

The frozen validation-only copied constructor was then compared with the unchanged native Node probe. All 60 ordinary literal-input scenes completed: 28 actual signed-rounding graphs across both axes and 32 source/destination/clamp-space combinations, offsets, copy-disabled axes and multiple-constraint/targetless min-over-max cases. Each original and cloned artboard followed 100×100, 60×160, 160×60, 100×100. All 480 frames and 140,096 coordinate comparisons matched **f32 bits**, with zero signed-zero differences. Every constructor recipe produced identical scene and evaluator files twice; original/clone and repeated-resize observed bits matched.

The reference results came from fixed_scalar before native probing, with no measured inputs or patched records. Native Rust Display decimals were converted back to f32; JSON integer spelling `-0` was explicitly retained as negative zero rather than parsed to Python integer zero. The runner retains mismatch lists and counts even though this run found none. This confirms the tested machine-semantic subset; it does not admit layout seeds, browser equivalence, TransformConstraint, or arbitrary graph optimizations. No product source was edited for this experiment.

- `output/public-fixed-scalar-r1/receipt.json`: `b728aa10ed4190d1c6d2b13f973c676c875f22304d97d41a869236e3962d8ec5`
- `output/public-fixed-scalar-r1/native-receipt.json`: `cd7987e00b6f0986efe1514ff623990759224496e56211bcbeda3015b05828e3`
- `output/public-fixed-scalar-r1/observations.json`: `6545142a7c24742004b7195d6d42b815f79515bacc6ece31a834f2d00c0517c7`
- `output/public-fixed-scalar-r1/candidate`: `6ee5353157b5e9b25656b48097b55ee22c1fb733ad746851f2b6f8d45101b9dd`
- `output/public-fixed-scalar-r1/node-probe`: `2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53`
