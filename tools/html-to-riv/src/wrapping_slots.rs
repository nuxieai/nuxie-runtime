//! Private structural prerequisite for independent wrapping slots.
//!
//! This inspects a completed *base* scene, before adding measurement/paint
//! constraints. It is not a certificate for the subsequently augmented scene,
//! arithmetic separation, Chrome fidelity, or public CSS admission.
use super::flex_descriptor::RecordLength;
use super::flex_structure::LocalFacts;
use crate::wire::{Record, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    Root,
    Role,
    Record { position: usize },
    Properties { id: u32 },
    Parent { id: u32 },
    Style { id: u32 },
    Defaults { id: u32 },
    Size { id: u32 },
    Ownership,
}
#[derive(Debug)]
pub(super) struct Sizes {
    pub preferred: [RecordLength; 2],
    pub minimum: [RecordLength; 2],
    pub maximum: [RecordLength; 2],
}
#[derive(Debug)]
pub(super) struct Slot {
    pub id: u32,
    pub visible: u32,
    pub style: u32,
    pub sizes: Sizes,
    pub visible_sizes: Sizes,
    /// Physical x/y fractions used by this nonwrapping slot to place its child.
    pub visible_fraction: [f32; 2],
}
pub(super) struct Binding<'a> {
    // Keep the examined array borrowed: this token does not survive mutation.
    _records: &'a [Record],
    pub parent: u32,
    pub parent_style: u32,
    pub parent_sizes: Sizes,
    pub row: bool,
    pub reverse_main: bool,
    pub reverse_cross: bool,
    pub line_fraction: f32,
    pub main_fraction: f32,
    pub slots: Vec<Slot>,
}
impl<'a> Binding<'a> {
    pub(super) fn records(&self) -> &'a [Record] { self._records }
}
fn uint(r: &Record, name: &str) -> Option<u32> {
    match r.get(name) { Some(Value::Uint(v)) => Some(*v), _ => None }
}
fn float(r: &Record, name: &str) -> Option<f32> {
    match r.get(name) { Some(Value::Float(v)) => Some(*v), _ => None }
}
fn at(records: &[Record], id: u32) -> Option<&Record> {
    (id as usize).checked_add(1).and_then(|p| records.get(p))
}
fn style_fields() -> Vec<String> {
    let mut fields = ["flexDirectionValue", "layoutAlignmentType", "widthUnitsValue",
        "heightUnitsValue", "layoutWidthScaleType", "layoutHeightScaleType",
        "flexWrapValue", "aspectRatio", "positionTypeValue", "directionValue",
        "animationStyleType", "interpolationType", "interpolatorId", "interpolationTime",
        "layoutTypeValue", "intrinsicallySizedValue"].map(str::to_owned).to_vec();
    for name in ["minWidth", "minHeight", "maxWidth", "maxHeight", "paddingLeft",
        "paddingTop", "paddingRight", "paddingBottom", "borderLeft", "borderTop",
        "borderRight", "borderBottom", "marginLeft", "marginTop", "marginRight",
        "marginBottom", "gapHorizontal", "gapVertical", "positionLeft", "positionTop",
        "positionRight", "positionBottom"] {
        fields.push(name.into()); fields.push(format!("{name}UnitsValue"));
    }
    fields
}
fn sizes(r: &Record, s: &Record, id: u32) -> Result<Sizes, Unresolved> {
    let read = |name: &str, owner: &Record| RecordLength {
        value: float(owner, name), units: uint(s, &format!("{name}UnitsValue")),
    };
    let preferred = [read("width", r), read("height", r)];
    let minimum = [read("minWidth", s), read("minHeight", s)];
    let maximum = [read("maxWidth", s), read("maxHeight", s)];
    let definite = |v: &RecordLength| matches!(v.units, Some(1 | 2))
        && v.value.is_some_and(|v| v.is_finite() && v >= 0.);
    if !preferred.iter().all(definite)
        || minimum.iter().chain(&maximum).any(|v|
            !(definite(v) || (v.units.unwrap_or(0) == 0
                && v.value.is_none_or(|x| x == 0.)))) {
        return Err(Unresolved::Size { id });
    }
    Ok(Sizes { preferred, minimum, maximum })
}

/// `roles` is logical packing order, and must exactly match completed direct
/// child order. Unknown objects, extra descendants, style sharing, native
/// constraints, and intrinsic/scale overrides fail closed. Every slot has one
/// definite visible child, whose own children may only be solid fills.
///
/// The single wrapping parent is the artboard's only layout child. Its fixed
/// scales and the artboard's top-left alignment establish a zero-origin base
/// scope. Finite used sizes and unchanged percentage bases require the separate
/// machine interval resolver; this function establishes only stored dimensions.
pub(super) fn inspect<'a>(records: &'a [Record], parent: u32, roles: &[(u32, u32)])
    -> Result<Binding<'a>, Unresolved> {
    if records.first().map(|r| r.kind) != Some("Backboard")
        || records.get(1).map(|r| r.kind) != Some("Artboard") || parent == 0 {
        return Err(Unresolved::Root);
    }
    if roles.is_empty() { return Err(Unresolved::Role); }
    let mut layouts = BTreeMap::from([(0, None), (parent, Some(0))]);
    for &(slot, visible) in roles {
        if layouts.insert(slot, Some(parent)).is_some()
            || layouts.insert(visible, Some(slot)).is_some() { return Err(Unresolved::Role); }
    }
    let mut styles = BTreeSet::new();
    let mut fills = BTreeMap::new();
    let mut colors = BTreeSet::new();
    let mut observed = Vec::new();
    let sf = style_fields();
    let sf: Vec<_> = sf.iter().map(String::as_str).collect();
    for (position, r) in records.iter().enumerate() {
        if position == 0 {
            if !r.has_only_properties(&[]) { return Err(Unresolved::Root); }
            continue;
        }
        let id = u32::try_from(position - 1).map_err(|_| Unresolved::Root)?;
        match r.kind {
            "Artboard" | "LayoutComponent" => {
                if (r.kind == "Artboard") != (id == 0) { return Err(Unresolved::Root); }
                let expected = layouts.get(&id).ok_or(Unresolved::Role)?;
                if uint(r, "parentId") != *expected || expected.is_some_and(|p| p >= id) {
                    return Err(Unresolved::Parent { id });
                }
                let allowed: &[&str] = if id == 0 {
                    &["name", "width", "height", "styleId", "originX", "originY"]
                } else {
                    &["name", "parentId", "styleId", "width", "height", "x", "y", "rotation", "scaleX", "scaleY"]
                };
                if !r.has_only_properties(allowed) { return Err(Unresolved::Properties { id }); }
                let sid = uint(r, "styleId").ok_or(Unresolved::Style { id })?;
                let s = at(records, sid).filter(|r| r.kind == "LayoutComponentStyle")
                    .ok_or(Unresolved::Style { id })?;
                if !styles.insert(sid) { return Err(Unresolved::Ownership); }
                let facts = LocalFacts::inspect(r, Some(s), false);
                if !(facts.zero_box_insets_and_gaps && facts.zero_margins
                    && facts.no_aspect_ratio && facts.no_intrinsic_sizing
                    && facts.unset_position_insets && facts.own_transform_defaults
                    && facts.no_direction_override && facts.no_style_interpolation
                    && facts.ordinary_flex_layout
                    && (id == parent || facts.no_wrap))
                    || uint(s, "layoutWidthScaleType").unwrap_or(0) != 0
                    || uint(s, "layoutHeightScaleType").unwrap_or(0) != 0
                    || uint(s, "layoutAlignmentType").unwrap_or(0) > 8
                    || uint(s, "flexDirectionValue").unwrap_or(2) > 3 {
                    return Err(Unresolved::Defaults { id });
                }
                if id == 0 {
                    if !s.has_only_properties(&["flexDirectionValue", "layoutAlignmentType"])
                        || uint(s, "layoutAlignmentType").unwrap_or(0) != 0
                        || !matches!(uint(s, "flexDirectionValue").unwrap_or(2), 0 | 2) {
                        return Err(Unresolved::Style { id });
                    }
                } else { sizes(r, s, id)?; }
                if *expected == Some(parent) { observed.push(id); }
            }
            "LayoutComponentStyle" => {
                if !r.has_only_properties(&sf) { return Err(Unresolved::Properties { id }); }
            }
            "Fill" => {
                let owner = uint(r, "parentId").ok_or(Unresolved::Ownership)?;
                if !r.has_only_properties(&["parentId"])
                    || !roles.iter().any(|&(_, visible)| visible == owner)
                    || owner >= id || fills.insert(id, owner).is_some() {
                    return Err(Unresolved::Ownership);
                }
            }
            "SolidColor" => {
                let owner = uint(r, "parentId").ok_or(Unresolved::Ownership)?;
                if !r.has_only_properties(&["parentId", "colorValue"])
                    || !fills.contains_key(&owner) || owner >= id
                    || !colors.insert(owner) || !matches!(r.get("colorValue"), Some(Value::Color(_))) {
                    return Err(Unresolved::Ownership);
                }
            }
            _ => return Err(Unresolved::Record { position }),
        }
    }
    if observed != roles.iter().map(|&(s, _)| s).collect::<Vec<_>>()
        || layouts.keys().any(|&id| !at(records, id).is_some_and(|r|
            r.kind == if id == 0 { "Artboard" } else { "LayoutComponent" }))
        || styles.len() != records.iter().filter(|r| r.kind == "LayoutComponentStyle").count()
        || fills.keys().any(|id| !colors.contains(id))
        || roles.iter().any(|&(_, visible)| fills.values().filter(|&&v| v == visible).count() != 1) {
        return Err(Unresolved::Ownership);
    }
    let pr = at(records, parent).ok_or(Unresolved::Role)?;
    let psid = uint(pr, "styleId").ok_or(Unresolved::Style { id: parent })?;
    let ps = at(records, psid).ok_or(Unresolved::Style { id: parent })?;
    let direction = uint(ps, "flexDirectionValue").unwrap_or(2);
    let row = direction >= 2;
    let wrap = uint(ps, "flexWrapValue").unwrap_or(0);
    if !matches!(wrap, 1 | 2) { return Err(Unresolved::Defaults { id: parent }); }
    let alignment = uint(ps, "layoutAlignmentType").unwrap_or(0);
    let line_fraction = (if row { alignment / 3 } else { alignment % 3 }) as f32 / 2.;
    let main_fraction = (if row { alignment % 3 } else { alignment / 3 }) as f32 / 2.;
    let slots = roles.iter().map(|&(id, visible)| {
        let r = at(records, id).ok_or(Unresolved::Role)?;
        let style = uint(r, "styleId").ok_or(Unresolved::Style { id })?;
        let s = at(records, style).unwrap();
        let vr = at(records, visible).ok_or(Unresolved::Role)?;
        let vsid = uint(vr, "styleId").ok_or(Unresolved::Style { id: visible })?;
        let vs = at(records, vsid).ok_or(Unresolved::Style { id: visible })?;
        let direction = uint(s, "flexDirectionValue").unwrap_or(2);
        let alignment = uint(s, "layoutAlignmentType").unwrap_or(0);
        let mut visible_fraction = [(alignment % 3) as f32 / 2., (alignment / 3) as f32 / 2.];
        if direction % 2 == 1 {
            let main = usize::from(direction < 2);
            visible_fraction[main] = 1. - visible_fraction[main];
        }
        Ok(Slot { id, visible, style, sizes: sizes(r, s, id)?,
            visible_sizes: sizes(vr, vs, visible)?, visible_fraction })
    }).collect::<Result<_, Unresolved>>()?;
    Ok(Binding { _records: records, parent, parent_style: psid,
        parent_sizes: sizes(pr, ps, parent)?, row, reverse_main: direction % 2 == 1,
        reverse_cross: wrap == 2, line_fraction, main_fraction, slots })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set(r: &mut Record, name: &str, value: Value) { r.set(name, value).unwrap(); }
    fn scene() -> Vec<Record> {
        let mut r = vec![Record::new("Backboard"), Record::new("Artboard"), Record::new("LayoutComponentStyle")];
        set(&mut r[1], "styleId", Value::Uint(1));
        for (id, parent) in [(2, 0), (4, 2), (6, 4)] {
            let mut node = Record::new("LayoutComponent");
            set(&mut node, "parentId", Value::Uint(parent));
            set(&mut node, "styleId", Value::Uint(id + 1));
            set(&mut node, "width", Value::Float(100.));
            set(&mut node, "height", Value::Float(50.));
            let mut style = Record::new("LayoutComponentStyle");
            set(&mut style, "widthUnitsValue", Value::Uint(1));
            set(&mut style, "heightUnitsValue", Value::Uint(1));
            if id == 2 { set(&mut style, "flexWrapValue", Value::Uint(1)); }
            r.extend([node, style]);
        }
        let mut fill = Record::new("Fill"); set(&mut fill, "parentId", Value::Uint(6));
        let mut color = Record::new("SolidColor"); set(&mut color, "parentId", Value::Uint(8));
        set(&mut color, "colorValue", Value::Color(0xffffffff)); r.extend([fill, color]); r
    }
    #[test]
    fn binds_final_sizes_and_all_native_direction_fractions() {
        let mut r = scene();
        let default = inspect(&r, 2, &[(4, 6)]).unwrap();
        assert!(default.row);
        assert!(!default.reverse_main);
        assert_eq!(default.line_fraction, 0.);
        set(&mut r[6], "minHeight", Value::Float(12.));
        set(&mut r[6], "minHeightUnitsValue", Value::Uint(2));
        for direction in 0..4 { for alignment in 0..9 { for wrap in 1..3 {
            set(&mut r[4], "flexDirectionValue", Value::Uint(direction));
            set(&mut r[4], "layoutAlignmentType", Value::Uint(alignment));
            set(&mut r[4], "flexWrapValue", Value::Uint(wrap));
            let b = inspect(&r, 2, &[(4, 6)]).unwrap();
            assert_eq!((b.row, b.reverse_main, b.reverse_cross), (direction >= 2, direction % 2 == 1, wrap == 2));
            assert_eq!(b.line_fraction, (if direction >= 2 { alignment / 3 } else { alignment % 3 }) as f32 / 2.);
            assert_eq!(b.main_fraction, (if direction >= 2 { alignment % 3 } else { alignment / 3 }) as f32 / 2.);
            assert_eq!(b.slots[0].sizes.minimum[1].value, Some(12.));
            assert_eq!(b.slots[0].sizes.minimum[1].units, Some(2));
        } } }
    }
    #[test]
    fn binds_every_slot_in_packing_order_and_rejects_partial_roles() {
        let mut r = scene();
        // Append a second independent sibling plus its uniquely owned paint.
        for (source, replacements) in [
            (5, vec![("parentId", 2), ("styleId", 11)]),
            (6, vec![]),
            (7, vec![("parentId", 10), ("styleId", 13)]),
            (8, vec![]),
            (9, vec![("parentId", 12)]),
            (10, vec![("parentId", 14)]),
        ] {
            let mut record = r[source].clone();
            for (name, value) in replacements { set(&mut record, name, Value::Uint(value)); }
            r.push(record);
        }
        let roles = [(4, 6), (10, 12)];
        let bound = inspect(&r, 2, &roles).unwrap();
        assert_eq!(bound.slots.iter().map(|s| (s.id, s.visible, s.style)).collect::<Vec<_>>(),
            vec![(4, 6, 5), (10, 12, 11)]);
        assert!(inspect(&r, 2, &roles[..1]).is_err());
        assert!(inspect(&r, 2, &[(10, 12), (4, 6)]).is_err());
        assert!(inspect(&r, 2, &[(4, 12), (10, 6)]).is_err());
        for direction in [0, 2] {
            let mut changed = r.clone();
            set(&mut changed[2], "flexDirectionValue", Value::Uint(direction));
            assert!(inspect(&changed, 2, &roles).is_ok());
        }
        let mut changed = r.clone();
        set(&mut changed[11], "styleId", Value::Uint(5));
        assert!(inspect(&changed, 2, &roles).is_err());
        let mut changed = r;
        set(&mut changed[13], "parentId", Value::Uint(4));
        assert!(inspect(&changed, 2, &roles).is_err());
    }

    #[test]
    fn visible_role_binds_its_native_sizes_and_slot_physical_alignment() {
        let mut r = scene();
        set(&mut r[7], "width", Value::Float(75.));
        set(&mut r[8], "widthUnitsValue", Value::Uint(2));
        for direction in 0..4 { for alignment in 0..9 {
            set(&mut r[6], "flexDirectionValue", Value::Uint(direction));
            set(&mut r[6], "layoutAlignmentType", Value::Uint(alignment));
            let b = inspect(&r, 2, &[(4,6)]).unwrap();
            let slot = &b.slots[0];
            assert_eq!(slot.visible_sizes.preferred[0].value, Some(75.));
            assert_eq!(slot.visible_sizes.preferred[0].units, Some(2));
            let x = (alignment % 3) as f32 / 2.;
            let y = (alignment / 3) as f32 / 2.;
            assert_eq!(slot.visible_fraction, [if direction == 3 {1.-x} else {x},
                if direction == 1 {1.-y} else {y}]);
        } }
    }
    #[test]
    fn bound_fields_feed_machine_sizes_and_detect_stale_authored_metadata() {
        use super::super::{computed_provenance::NumericSize,
            scalar_provenance::ScalarProvenance, wrapping_sizes::{self, MachineInterval}};
        let mut r = scene();
        set(&mut r[6], "minHeight", Value::Float(80.));
        set(&mut r[6], "minHeightUnitsValue", Value::Uint(2));
        set(&mut r[6], "maxHeight", Value::Float(20.));
        set(&mut r[6], "maxHeightUnitsValue", Value::Uint(1));
        let preferred = NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(50.).unwrap()));
        let minimum = NumericSize::Percent(Ok(ScalarProvenance::exact_constant(80.).unwrap()));
        let maximum = NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(20.).unwrap()));
        let resolve = |records: &[Record]| {
            let binding = inspect(records, 2, &[(4, 6)]).unwrap();
            let sizes = &binding.slots[0].sizes;
            wrapping_sizes::used_size((&preferred, &sizes.preferred[1]),
                (&minimum, &sizes.minimum[1]), (&maximum, &sizes.maximum[1]),
                MachineInterval::new(100., 200.).unwrap())
        };
        assert_eq!(resolve(&r).unwrap(), MachineInterval::new(80., 160.).unwrap());
        // Structurally valid edits still require new matching source metadata.
        set(&mut r[6], "minHeight", Value::Float(81.));
        assert_eq!(resolve(&r), Err(wrapping_sizes::Unresolved::NativeBinding));
    }
    #[test]
    fn rejects_size_feedback_transforms_unknown_records_and_hidden_styles() {
        let r = scene(); assert!(inspect(&r, 2, &[(4, 6)]).is_ok());
        for (position, name, value) in [
            (6, "layoutHeightScaleType", Value::Uint(1)),
            (6, "aspectRatio", Value::Float(2.)),
            (6, "paddingLeft", Value::Float(1.)),
            (6, "positionLeftUnitsValue", Value::Uint(1)),
            (6, "intrinsicallySizedValue", Value::Bool(true)),
            (4, "gapVertical", Value::Float(1.)),
            (4, "layoutAlignmentType", Value::Uint(9)),
            (2, "layoutAlignmentType", Value::Uint(4)),
            (2, "flexDirectionValue", Value::Uint(1)),
            (2, "flexDirectionValue", Value::Uint(3)),
            (1, "originX", Value::Float(0.5)),
            (5, "x", Value::Float(1.)),
            (5, "height", Value::Float(f32::NAN)),
            (6, "heightUnitsValue", Value::Uint(3)),
            (5, "styleId", Value::Uint(3)),
            (7, "parentId", Value::Uint(2)),
        ] {
            let mut changed = r.clone(); set(&mut changed[position], name, value);
            assert!(inspect(&changed, 2, &[(4, 6)]).is_err(), "{position} {name}");
        }
        for kind in ["TransformConstraint", "LayoutComponent", "Image", "Text", "ComponentOrigin", "LayoutComponentStyle"] {
            let mut changed = r.clone(); changed.push(Record::new(kind));
            assert!(inspect(&changed, 2, &[(4, 6)]).is_err(), "{kind}");
        }
        assert!(inspect(&r, 2, &[(4, 4)]).is_err());
        assert!(inspect(&r, 2, &[(4, 6), (4, 6)]).is_err());
        assert!(inspect(&r[..r.len()-1], 2, &[(4, 6)]).is_err());
    }
}
