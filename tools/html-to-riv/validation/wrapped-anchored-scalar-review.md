# Anchored wrapping instruction and layout association

The private sizing graph now uses its already measured slot anchor and carried line maximum:

```
offset = line_maximum * (desired_fraction - line_fraction)
target = slot_anchor + offset
visible_origin = ComponentOrigin(active=desired_fraction, other=0)
visible_world = target - visible_native_extent * desired_fraction
```

Fractions are physical after wrap reversal. This supplies the missing visible-size term without any new native layout participant, measurement dependency on a moved visible object, host setter, or runtime change. The target is prelanding; the visible world coordinate is the landed value. `wrapping_position.rs` separately bounds these quantities and origin multiplication.

`wrapping_scalar.rs` now checks the new target's operands, the exact ComponentOrigin fields/placement, and the subsequent strength-one world copy. All scalar helper origins remain native zero defaults. The new Binding counts ordinary origins separately from constraints. The reader consumes every sizing record and still verifies all trace handles. Field, operand, constraint-order and trace mutations reject, now including a changed visible origin.

The original `wrapping_composition::preserved` base-prefix check remains exact except for original color hiding. Its generated-role check permits ComponentOrigin only on a bound visible owner, rejects duplicate origins and requires exactly one origin per visible role. The sizing reader further binds its active/orthogonal fractions and rejects all unmodelled properties. The low-level sizing emitter rejects an existing origin or repeated visible handle before changing records. No origin attaches to a slot or scalar helper.

## Native layout and scheduling extension

The previous layout-isolation source argument excluded generated LayoutComponent, LayoutComponentStyle and LayoutParticipant. They remain excluded. ComponentOrigin is not a layout provider; its on-added callback marks HasComponentOrigin on the owning visible component, and its property-change callback dirties world transform. It does not change style, min/max, layout dimensions or flex participation. `layout_component.rs::build_own_transform_with` skips pivot math under the checked identity rotation/scales, so initial own transforms stay unchanged. Actual layout extents remain the inputs consumed by `local_anchor`.

The existing explicit slot-to-helper and helper-to-visible target dependencies remain. ComponentOrigin supplies a local native extent at each final constraint invocation, not a new target edge back from visible to helper. Successful initialization installs the unique origin before the first update. LayoutComponent's two constraint passes each overwrite the active world coordinate with the current target and then land the origin; the second pass therefore does not accumulate the first landing. The orthogonal origin is zero and the final copy retains that coordinate. Finite dimensions and initial world translations remain required on both axes.

The reduced positioning stage uses eight records per item instead of eleven. Sizing total is58N-33 forN>0 (141 records atN3), with the existing paint cost unchanged. There is no expansion of public support or relaxation of resource limits. Historical snapped-sizing files and their61N-33 cost belong to the previous frozen emitter; they must not be reproduced with the changed emitter and called unchanged evidence.

## Validation scope

Arithmetic and structural tests establish the updated private assumptions. A new native-only experiment separately checks ordinary import and original/clone resize against independently grouped slot geometry. It must preserve mismatches and pre-fix controls. It does not substitute for a Chrome/native pixel campaign, certify the derived-epsilon candidate by association, or establish paint-gate/mask semantics. Public wrapping remains unadmitted until these remaining gates pass.
