# L02 order: implementation investigation

Status: investigation only; syntax is not admitted yet.

CSS order takes an integer, defaults to zero, and is not inherited. Flex items
sort by ascending value with source order breaking ties; their painting order
changes too. Non-flex boxes are unaffected in this module's profile. Logical
source order remains distinct. Grid and interaction/navigation implementation
remain excluded. [CSS Display 3](https://drafts.csswg.org/css-display-3/#order-property)
and [CSS Flexbox 1](https://drafts.csswg.org/css-flexbox-1/#order-property), checked
2026-09-09.

Observed local seams:
- Compiler lib.rs assigns each object's artboard-local ID from records.len() and
  appends SourceNode during recursive emission. Child paths derive from authored
  child_elements enumeration. Reordering that enumeration would couple visual
  order to object IDs and map ordering; do not silently alter source identity.
- LayoutComponent::sync_layout_children follows layout_providers_children over
  container children. A separate stable ordering there could retain object IDs,
  but it must be opt-in and survive resize/rebuild of layout nodes.
- Artboard::sort_draw_order iterates imported drawable occurrences and applies
  DrawRules/DrawTarget links. Changing layout-provider order alone would not prove
  correct paint overlap. Inspect the existing draw-rule format before selecting
  an extension or a compiler lowering strategy.

Next bounded experiment: compile/import three differently ordered siblings with
fixed boxes and overlapping paint, preserving authored source-map paths and
object identity. Test equal and negative orders, nested parents, row/column and
reverse directions, wrap, clipping and hidden siblings. Compare against Chromium
and resize one compilation. Include :nth-child and sibling-selector controls to
prove selectors still evaluate the original DOM. Require a paint-overlap test;
positions alone cannot qualify order.

Before implementation, choose between explicit child-order/runtime policy and a
format-supported separation of stable object identities from visual traversal.
The searches above do not establish that the format lacks a suitable primitive.
Integer bounds, invalid-token behavior, var()/CSS-wide keywords, and the full
accepted-value contract remain to be defined and tested. No feature or capability
has been added by this investigation.

Implementation decision: source identities are authored IDs/paths; object indices
are per-artifact references, already variable when CSS changes emitted paint
records. Stable sibling-subtree emission plus a source-ordered map preserves the
public identity contract without a runtime extension. Initial imported layout
test passes. See order-review.md for contract and remaining validation.
