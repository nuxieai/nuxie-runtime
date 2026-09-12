# Pinned native defaults for direct flex proof

Read-only audit. Baseline: `6c7ac16617835b5f581784ff08a9e779bb52faf3`. Every file listed below is byte-identical to that baseline. These are generated importer defaults and actual runtime translations, not assumptions inferred from compiler fields. No runtime or compiler source changed.

## Executable interpretation

References below are repository-relative `file:line`; paths under `source/` expand to `crates/nuxie-runtime/src/mechanical_port/source/`.

| Concern | Raw default and actual interpretation | Source references |
| --- | --- | --- |
| Minimum | Numeric field 0, units Undefined(0). This is **Auto in Taffy**, not universally a zero minimum. Fill replaces an Undefined minimum with Point(0) on that axis. Fixed retains automatic minimum and needs a separate content/target proof. Explicit Auto units3 is not replaced. | `source/generated/layout/layout_sizing_style_base.rs:86`, `:194`; `source/layout/layout_sizing_style.rs:92`; `source/layout/layout_participant.rs:274`; `source/layout/layout_style_applier.rs:607`, `:717` |
| Maximum | Numeric 0 with Undefined units becomes Auto/no maximum, not a maximum of zero. Explicit point0 is materially different. | `source/generated/layout/layout_sizing_style_base.rs:113`; `source/layout/layout_sizing_style.rs:96`; `source/layout/layout_style_applier.rs:611`, `:717` |
| Automatic main minimum | If unresolved, pinned flexbox measures min-content, caps by preferred and maximum, then floors by padding+border. Fixed point sizing alone does not make the raw minimum Point(0); leaf/no-inset proof can justify its effect. | `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:817` through `:840` |
| Margins | Raw zero with Undefined units. Crucial later normalization converts **Undefined margins to Point(0)** before Taffy; only explicit Auto distributes free space. The generic `auto()` helper alone would give the wrong conclusion. | `source/generated/layout/layout_component_style_base.rs:161`, `:183`; `source/layout/layout_style_applier.rs:615` through `:624` |
| Padding, borders, gaps | Raw zero, Undefined units. `length()` maps non-percent units to stored finite numeric value, hence zero. A present nonzero field with omitted units is not safely ignorable. Percentage paths divide coefficient by100 before owner multiplication. | `source/generated/layout/layout_component_style_base.rs:6`, `:159`, `:165`, `:187`; `source/layout/layout_component_style.rs:245` through `:262`; `source/layout/layout_style_applier.rs:728` |
| Insets | Position sidecar defaults zero/Undefined; Undefined maps to Auto/unset inset, not an authored point-zero inset. Position type defaults1 (Relative); checker should require no effective inset. | `source/generated/layout/layout_component_style_base.rs:18`, `:177`; `source/layout/layout_component_style.rs:221`; `source/layout/layout_style_applier.rs:498`, `:734` |
| Basis and factors | Stored default basis0 with units Auto(3). Main Fill takes stored basis/units and linked grow=shrink=fraction; non-Fill forces both factors0 and Auto basis regardless of stored basis units. Fractional width/height default1, so missing fraction on Fill does not mean0. Base participant initially sets Fill basis0 Point, but LayoutComponent subsequently applies stored units. | `source/generated/layout/layout_component_style_base.rs:169`, `:173`; `source/generated/layout_component_base.rs:27`; `source/layout/layout_participant.rs:256`; `source/layout_component.rs:3340` through `:3363` |
| Scale and dimensions | Scale default0 Fixed; width/height units default1 Point; component dimensions default0. Fill1 and Hug2 use Auto dimensions. | `source/generated/layout/layout_sizing_style_base.rs:48`; `source/generated/layout_component_base.rs:27`; `source/layout/layout_enums.rs:119`; `source/layout/layout_participant.rs:215` |
| Wrap, direction, flex direction | Wrap0 NoWrap. Direction0 Inherit is mapped to Taffy LTR by `set_direction` (only Rtl selects RTL); logical edge mapping also uses context.is_ltr, so inspect ancestor/context construction before universal LTR proof. Flex direction default2 is Row, not compiler reset Column; emitted compiler direction overrides it. | `source/generated/layout/layout_component_style_base.rs:178`; `source/layout/layout_style_applier.rs:194`, `:216`, `:492`; `source/layout/layout_component_style.rs:231` |
| Aspect ratio | Raw0 becomes NaN optional/absent unless strictly positive. Do not treat ratio0 as a zero-size constraint. | `source/generated/layout/layout_component_style_base.rs:170`; `source/layout/layout_component_style.rs:185` |
| Own transform | Rotation0, scaleX/Y1, Node x/y0. These establish default own identity only with no overriding properties/constraints/animation; they do not eliminate layout translation. | `source/generated/transform_component_base.rs:19`; `source/generated/node_base.rs:43`; `source/layout_component.rs:937` through `:978` |
| Origin | LayoutComponent pivot defaults0 only when no ComponentOrigin helper; helper lookup determines effective origin. Artboard originX/Y default0 separately. Layout translation explicitly accounts for parent artboard origin. | `source/layout_component.rs:858` through `:919`; `source/generated/artboard_base.rs:27` |

For compiler zero-minimum omission, `numeric_input` can validate the expected encoding shape but must not assert importer-equivalent Point(0) for every participant. The direct structural checker must retain the distinction between Fill forced zero and Fixed automatic minimum. Bounds encoded as a partial value/units pair should be decoded exactly or rejected by the restricted checker, never silently assigned the compiler's expected default.

## Required final-record checks

Resolve property defaults with type-aware records, including inherited generated bases and sidecars. Check actual style binding, scale, dimension units, basis units, fraction bits, min/max pairs, all edge/gap properties, wrap/direction/aspect fields, and own transform fields. Check ancestors and helper/constraint/animation targets after complete emission. Compiler absence of authored transform CSS is insufficient when synthetic helpers may modify the component later.

No part of this audit sets world arithmetic error to zero. `layout_translation`, layout offsets, own-transform composition and ancestor multiplication remain arithmetic to bound even when all stored rotations/scales/origins are default. The imported file version also affects layout composition (`source/layout_component.rs:294`); bind the proof to actual compiler file version and immutable importer behavior.

## Source receipt

SHA-256 values below are current file bytes, each compared directly against `git show BASE:path`.

| Source | SHA-256 | Baseline equality |
| --- | --- | --- |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/layout/layout_component_style_base.rs` | `8f362d8252c18917b68ebd940fa597d5470948ebdf5c9228244702af79031f8c` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/layout/layout_sizing_style_base.rs` | `fbc311cf9228245c3fe28f4dd2b3abe87970d98ac2e73048a40b66e0679db1bf` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/layout_component_base.rs` | `10911fbe1d164784a5660c6bede729b038506fbff19ad40bbd35cdffdc3011b4` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/transform_component_base.rs` | `8508ae7006d9d14555ab7d54a9f7a7538c915c6c098f6902a4826ac4bb879924` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/node_base.rs` | `6373332cc40517a7734baa2a6caec64e33bca6b7fa5962792a0ebe5c001b88a8` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/artboard_base.rs` | `7747238c2ed331800999a69730860311b6b8a1eefce47263a3cfb398f310e9e8` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs` | `46b8b93aa0ee7cd97063716a64730e8174410de8d1fb44f1f8ad96310f5f1335` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_sizing_style.rs` | `117e939de98ee9516f21c9450fdc729164eeda82c2a3edd088c80a354f02ed1f` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_participant.rs` | `fceb9920fa6442d4ff9fc9c31086f9fcdd2def7ae0d445db69d5c899b2925c46` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs` | `233d9061c9fdeca344f1ab4e35c25c1413a4a2e06fbc2880beaad26aa47cb9f0` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_enums.rs` | `00e6c94c7d19965048b08c754b2656464a19102a96e4e6658e8166841d5b892a` | identical |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` | identical |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` | identical |
