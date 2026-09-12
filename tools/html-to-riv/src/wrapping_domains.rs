//! Join authored numeric dimensions to a checked independent-slot base scene.
//! This preserves each original containing-block basis across clamping. It is
//! not a certificate for generated constraints, line gates, or raster coverage.
use super::{computed_provenance::NumericStyle, wrapping_sizes::{self, MachineInterval},
    wrapping_slots::{self, Binding, Sizes}};
use crate::wire::Record;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    Structure(wrapping_slots::Unresolved),
    AuthoredRole { object: u32 },
    Dimension { object: u32, axis: usize, reason: wrapping_sizes::Unresolved },
    CrossExtents(wrapping_sizes::Unresolved),
}

#[derive(Clone, Debug)]
pub(super) struct Slot {
    pub object: u32,
    pub axes: [MachineInterval; 2],
}

pub(super) struct Domains<'a> {
    // The borrow prevents mutating the base while treating these dimensions as
    // bound to it. Augmentation needs its own preservation check and token.
    base: Binding<'a>,
    viewport: [MachineInterval; 2],
    parent_axes: [MachineInterval; 2],
    slots: Vec<Slot>,
    cross_extents: MachineInterval,
}
impl<'a> Domains<'a> {
    pub(super) fn base(&self) -> &Binding<'a> { &self.base }
    pub(super) fn viewport(&self) -> [MachineInterval; 2] { self.viewport }
    pub(super) fn parent_axes(&self) -> [MachineInterval; 2] { self.parent_axes }
    pub(super) fn slots(&self) -> &[Slot] { &self.slots }
    pub(super) fn cross_extents(&self) -> MachineInterval { self.cross_extents }
}

fn dimensions(
    object: u32, authored: &NumericStyle, native: &Sizes,
    parent: [MachineInterval; 2],
) -> Result<[MachineInterval; 2], Unresolved> {
    let preferred = [&authored.width, &authored.height];
    let minimum = [&authored.min_width, &authored.min_height];
    let maximum = [&authored.max_width, &authored.max_height];
    let resolve = |axis: usize| {
        wrapping_sizes::used_size((preferred[axis], &native.preferred[axis]),
            (minimum[axis], &native.minimum[axis]),
            (maximum[axis], &native.maximum[axis]), parent[axis])
            .map_err(|reason| Unresolved::Dimension { object, axis, reason })
    };
    Ok([resolve(0)?, resolve(1)?])
}

/// Viewport endpoints are an explicit observation domain, not the compile-time
/// viewport sampled by a browser. A caller may enclose the full positive runtime
/// domain with zero as its lower endpoint; this never invents a minimum viewport.
/// Authored metadata must name exactly the bound slot IDs, once each. Parent
/// axes resolve against the viewport and slot axes against those parent content
/// axes. The structural binder supplies the required zero-inset fixed-size shape.
/// Explicit minima are required for parent and slots; absent/automatic minima
/// cannot inherit a proof merely because a stored preferred size is definite.
pub(super) fn resolve<'a>(
    records: &'a [Record], parent: u32, roles: &[(u32, u32)],
    authored_parent: &NumericStyle, authored_slots: &[(u32, &NumericStyle)],
    viewport: [MachineInterval; 2],
) -> Result<Domains<'a>, Unresolved> {
    let base = wrapping_slots::inspect(records, parent, roles).map_err(Unresolved::Structure)?;
    let mut authored = BTreeMap::new();
    for &(object, values) in authored_slots {
        if !base.slots.iter().any(|slot| slot.id == object) || authored.insert(object, values).is_some() {
            return Err(Unresolved::AuthoredRole { object });
        }
    }
    let parent_axes = dimensions(parent, authored_parent, &base.parent_sizes, viewport)?;
    let slots = base.slots.iter().map(|slot| {
        let numeric = authored.get(&slot.id).ok_or(Unresolved::AuthoredRole { object: slot.id })?;
        Ok(Slot { object: slot.id, axes: dimensions(slot.id, numeric, &slot.sizes, parent_axes)? })
    }).collect::<Result<Vec<_>, Unresolved>>()?;
    let cross = usize::from(base.row);
    let cross_extents = wrapping_sizes::cross_extents(
        &slots.iter().map(|slot| slot.axes[cross]).collect::<Vec<_>>())
        .map_err(Unresolved::CrossExtents)?;
    Ok(Domains { base, viewport, parent_axes, slots, cross_extents })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{computed_provenance::NumericSize, scalar_provenance::ScalarProvenance};
    use crate::wire::Value;

    fn number(value: f32, percent: bool) -> NumericSize {
        let n = Ok(ScalarProvenance::exact_constant(value).unwrap());
        if percent { NumericSize::Percent(n) } else { NumericSize::Pixels(n) }
    }
    fn authored(width: f32, height: f32, percent: bool) -> NumericStyle {
        NumericStyle { width: number(width, percent), height: number(height, percent),
            ..NumericStyle::default() }
    }
    fn set(r: &mut Record, name: &str, value: Value) { r.set(name, value).unwrap(); }
    fn scene() -> Vec<Record> {
        let mut r = vec![Record::new("Backboard"), Record::new("Artboard"), Record::new("LayoutComponentStyle")];
        set(&mut r[1], "styleId", Value::Uint(1));
        for (id, parent, width, height, units) in [(2, 0, 50., 25., 2), (4, 2, 60., 30., 1), (6, 4, 100., 100., 2)] {
            let mut node = Record::new("LayoutComponent");
            for (key, value) in [("parentId", parent), ("styleId", id+1)] { set(&mut node, key, Value::Uint(value)); }
            set(&mut node, "width", Value::Float(width)); set(&mut node, "height", Value::Float(height));
            let mut style = Record::new("LayoutComponentStyle");
            for key in ["widthUnitsValue", "heightUnitsValue"] { set(&mut style, key, Value::Uint(units)); }
            for key in ["minWidth", "minHeight"] {
                set(&mut style, key, Value::Float(0.));
                set(&mut style, &format!("{key}UnitsValue"), Value::Uint(1));
            }
            if id == 2 { set(&mut style, "flexWrapValue", Value::Uint(1)); }
            r.extend([node, style]);
        }
        let mut fill = Record::new("Fill"); set(&mut fill, "parentId", Value::Uint(6));
        let mut color = Record::new("SolidColor"); set(&mut color, "parentId", Value::Uint(8));
        set(&mut color, "colorValue", Value::Color(0xff12ab34)); r.extend([fill, color]); r
    }
    fn viewport() -> [MachineInterval; 2] {
        [MachineInterval::new(0., 16384.).unwrap(), MachineInterval::new(0., 8192.).unwrap()]
    }

    #[test]
    fn actual_parent_axes_and_original_slot_bases_survive_conflicting_bounds() {
        let mut records = scene();
        let mut parent = authored(50., 25., true);
        // Width minimum wins over maximum; height resolves from its own axis.
        parent.min_width = number(200., false); parent.max_width = number(100., false);
        for (key, value) in [("minWidth", 200.), ("maxWidth", 100.)] {
            set(&mut records[4], key, Value::Float(value));
            set(&mut records[4], &format!("{key}UnitsValue"), Value::Uint(1));
        }
        let mut slot = authored(60., 30., false);
        slot.width = number(80., true);
        set(&mut records[5], "width", Value::Float(80.));
        set(&mut records[6], "widthUnitsValue", Value::Uint(2));
        let d = resolve(&records, 2, &[(4,6)], &parent, &[(4,&slot)], viewport()).unwrap();
        assert_eq!(d.parent_axes, [MachineInterval::new(200.,200.).unwrap(), MachineInterval::new(0.,2048.).unwrap()]);
        assert_eq!(d.slots[0].axes, [MachineInterval::new(160.,160.).unwrap(), MachineInterval::new(30.,30.).unwrap()]);
        assert_eq!(d.cross_extents, MachineInterval::new(30.,30.).unwrap());
        assert_eq!(d.viewport, viewport());
    }

    #[test]
    fn exact_slot_metadata_ownership_and_structure_are_required() {
        let mut records = scene(); let parent = authored(50.,25.,true); let slot = authored(60.,30.,false);
        for entries in [vec![], vec![(6,&slot)], vec![(4,&slot),(4,&slot)]] {
            assert!(matches!(resolve(&records,2,&[(4,6)],&parent,&entries,viewport()), Err(Unresolved::AuthoredRole{..})));
        }
        set(&mut records[5], "height", Value::Float(31.));
        assert!(matches!(resolve(&records,2,&[(4,6)],&parent,&[(4,&slot)],viewport()),
            Err(Unresolved::Dimension{object:4,axis:1,reason:wrapping_sizes::Unresolved::NativeBinding})));
        set(&mut records[6], "aspectRatio", Value::Float(2.));
        assert!(matches!(resolve(&records,2,&[(4,6)],&parent,&[(4,&slot)],viewport()), Err(Unresolved::Structure(_))));
    }

    #[test]
    fn no_implicit_viewport_floor_or_axis_swap_can_supply_missing_separation() {
        let mut records = scene(); let parent = authored(50.,25.,true); let mut slot = authored(60.,30.,false);
        slot.height = number(50.,true);
        set(&mut records[5], "height", Value::Float(50.));
        set(&mut records[6], "heightUnitsValue", Value::Uint(2));
        assert!(matches!(resolve(&records,2,&[(4,6)],&parent,&[(4,&slot)],viewport()),
            Err(Unresolved::CrossExtents(wrapping_sizes::Unresolved::NoPositiveCrossLowerBound))));
        // Switching the actual native parent to column changes the cross axis,
        // where the fixed60px width now supplies a valid positive bound.
        set(&mut records[4], "flexDirectionValue", Value::Uint(0));
        let d = resolve(&records,2,&[(4,6)],&parent,&[(4,&slot)],viewport()).unwrap();
        assert_eq!(d.cross_extents, MachineInterval::new(60.,60.).unwrap());
        assert_eq!(d.slots[0].axes[1].lower(),0.);
        assert_eq!(d.slots[0].axes[1].upper(),1024.);
    }
}
