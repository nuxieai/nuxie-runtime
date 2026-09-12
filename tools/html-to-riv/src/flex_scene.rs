//! Closed scene prerequisites for the private direct-box arithmetic model.
//! This certifies structure/defaults, not dimensions, CSS fidelity or pixels.
use super::flex_structure::LocalFacts;
use crate::wire::{Record, Value};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) enum Unresolved {
    Root,
    RecordKind { position: usize },
    Property { position: usize },
    Parent { id: u32 },
    Style { id: u32 },
    LocalDefaults { id: u32 },
    StyleOwnership,
    PaintOwnership,
}

// Only inspect() can construct this token. All ancestry and ownership facts
// come from the completed records; no caller supplies a "known LTR" boolean.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Certificate {
    _checked: (),
}

fn uint(record: &Record, name: &str) -> Option<u32> {
    match record.get(name) {
        Some(Value::Uint(v)) => Some(*v),
        _ => None,
    }
}
fn fields(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "Backboard" => &[],
        "Artboard" => &["name", "width", "height", "styleId", "originX", "originY"],
        "LayoutComponent" => &[
            "name",
            "parentId",
            "styleId",
            "width",
            "height",
            "fractionalWidth",
            "fractionalHeight",
            "x",
            "y",
            "rotation",
            "scaleX",
            "scaleY",
        ],
        "LayoutComponentStyle" => &[
            "flexDirectionValue",
            "layoutAlignmentType",
            "widthUnitsValue",
            "heightUnitsValue",
            "layoutWidthScaleType",
            "layoutHeightScaleType",
            "flexBasis",
            "flexBasisUnitsValue",
            "minWidth",
            "minHeight",
            "maxWidth",
            "maxHeight",
            "minWidthUnitsValue",
            "minHeightUnitsValue",
            "maxWidthUnitsValue",
            "maxHeightUnitsValue",
            "paddingLeft",
            "paddingTop",
            "paddingRight",
            "paddingBottom",
            "paddingLeftUnitsValue",
            "paddingTopUnitsValue",
            "paddingRightUnitsValue",
            "paddingBottomUnitsValue",
            "borderLeft",
            "borderTop",
            "borderRight",
            "borderBottom",
            "borderLeftUnitsValue",
            "borderTopUnitsValue",
            "borderRightUnitsValue",
            "borderBottomUnitsValue",
            "gapHorizontal",
            "gapVertical",
            "gapHorizontalUnitsValue",
            "gapVerticalUnitsValue",
            "marginLeft",
            "marginTop",
            "marginRight",
            "marginBottom",
            "marginLeftUnitsValue",
            "marginTopUnitsValue",
            "marginRightUnitsValue",
            "marginBottomUnitsValue",
            "flexWrapValue",
            "aspectRatio",
            "positionTypeValue",
            "positionLeft",
            "positionTop",
            "positionRight",
            "positionBottom",
            "positionLeftUnitsValue",
            "positionTopUnitsValue",
            "positionRightUnitsValue",
            "positionBottomUnitsValue",
            "directionValue",
            "animationStyleType",
            "interpolationType",
            "interpolatorId",
            "interpolationTime",
            "layoutTypeValue",
            "intrinsicallySizedValue",
        ],
        "Fill" => &["parentId"],
        "SolidColor" => &["parentId", "colorValue"],
        _ => return None,
    })
}

/// The initial family has one artboard, uniquely owned styles, and paint-only
/// Fill/SolidColor children. Layout parent IDs strictly decrease to artboard 0,
/// so checking every layout's local defaults also checks every ancestor's LTR,
/// intrinsic-sizing, interpolation and transform state. The caller must still
/// supply viewport/world domains and prove each actual group's numeric mapping.
pub(super) fn inspect(records: &[Record]) -> Result<Certificate, Unresolved> {
    if records.first().map(|r| r.kind) != Some("Backboard")
        || records.get(1).map(|r| r.kind) != Some("Artboard")
    {
        return Err(Unresolved::Root);
    }
    let mut style_owners = BTreeMap::new();
    let mut fill_owners = BTreeMap::new();
    let mut color_owners = BTreeMap::new();
    for (position, record) in records.iter().enumerate() {
        let allowed = fields(record.kind).ok_or(Unresolved::RecordKind { position })?;
        if !record.has_only_properties(allowed) {
            return Err(Unresolved::Property { position });
        }
        if (record.kind == "Backboard" && position != 0)
            || (record.kind == "Artboard" && position != 1)
        {
            return Err(Unresolved::Root);
        }
        if position == 0 {
            continue;
        }
        let id = (position - 1) as u32;
        let parent = uint(record, "parentId");
        let parent_kind = parent
            .and_then(|p| records.get(p as usize + 1))
            .map(|r| r.kind);
        match record.kind {
            "Artboard" | "LayoutComponent" => {
                if record.kind == "LayoutComponent"
                    && !(parent.is_some_and(|p| p < id)
                        && matches!(parent_kind, Some("Artboard" | "LayoutComponent")))
                {
                    return Err(Unresolved::Parent { id });
                }
                let style_id = uint(record, "styleId").ok_or(Unresolved::Style { id })?;
                let style = records
                    .get(style_id as usize + 1)
                    .filter(|r| r.kind == "LayoutComponentStyle")
                    .ok_or(Unresolved::Style { id })?;
                if style_owners.insert(style_id, id).is_some() {
                    return Err(Unresolved::StyleOwnership);
                }
                if id == 0
                    && !style.has_only_properties(&[
                        "flexDirectionValue",
                        "layoutAlignmentType",
                        "directionValue",
                        "animationStyleType",
                        "interpolationType",
                        "interpolatorId",
                        "interpolationTime",
                        "layoutTypeValue",
                        "intrinsicallySizedValue",
                    ])
                {
                    return Err(Unresolved::Style { id });
                }
                // The last argument is irrelevant to direct_box_defaults. Full
                // direct-child membership is indexed separately by SceneIndex.
                if !LocalFacts::inspect(record, Some(style), false).direct_box_defaults() {
                    return Err(Unresolved::LocalDefaults { id });
                }
            }
            "Fill" => {
                if !(parent.is_some_and(|p| p < id)
                    && matches!(parent_kind, Some("Artboard" | "LayoutComponent")))
                {
                    return Err(Unresolved::PaintOwnership);
                }
                if fill_owners.insert(parent.unwrap(), id).is_some() {
                    return Err(Unresolved::PaintOwnership);
                }
            }
            "SolidColor" => {
                if !(parent.is_some_and(|p| p < id) && parent_kind == Some("Fill"))
                    || !matches!(record.get("colorValue"), Some(Value::Color(_)))
                {
                    return Err(Unresolved::PaintOwnership);
                }
                if color_owners.insert(parent.unwrap(), id).is_some() {
                    return Err(Unresolved::PaintOwnership);
                }
            }
            "LayoutComponentStyle" => (),
            _ => unreachable!("kind already checked"),
        }
    }
    let style_count = records
        .iter()
        .filter(|r| r.kind == "LayoutComponentStyle")
        .count();
    if style_count != style_owners.len() {
        return Err(Unresolved::StyleOwnership);
    }
    if fill_owners
        .values()
        .any(|fill| !color_owners.contains_key(fill))
    {
        return Err(Unresolved::PaintOwnership);
    }
    Ok(Certificate { _checked: () })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set(record: &mut Record, name: &str, value: Value) {
        record.set(name, value).unwrap();
    }
    fn scene() -> Vec<Record> {
        let mut records = vec![
            Record::new("Backboard"),
            Record::new("Artboard"),
            Record::new("LayoutComponentStyle"),
            Record::new("LayoutComponent"),
            Record::new("LayoutComponentStyle"),
            Record::new("LayoutComponent"),
            Record::new("LayoutComponentStyle"),
            Record::new("Fill"),
            Record::new("SolidColor"),
        ];
        for (position, style) in [(1, 1), (3, 3), (5, 5)] {
            set(&mut records[position], "styleId", Value::Uint(style));
        }
        for (position, parent) in [(3, 0), (5, 2), (7, 4), (8, 6)] {
            set(&mut records[position], "parentId", Value::Uint(parent));
        }
        set(&mut records[8], "colorValue", Value::Color(0xff112233));
        records
    }
    #[test]
    fn checks_full_ancestry_and_measurement_vocabulary() {
        let records = scene();
        assert!(inspect(&records).is_ok());
        for (position, name, value) in [
            (2, "directionValue", Value::Uint(2)),
            (4, "intrinsicallySizedValue", Value::Bool(true)),
            (2, "interpolationTime", Value::Float(1.)),
            (4, "animationStyleType", Value::Uint(1)),
            (3, "rotation", Value::Float(0.2)),
            (1, "originX", Value::Float(0.5)),
        ] {
            let mut changed = records.clone();
            set(&mut changed[position], name, value);
            assert!(
                matches!(inspect(&changed), Err(Unresolved::LocalDefaults { .. })),
                "{name}"
            );
        }
        for kind in ["Text", "Image", "ComponentOrigin", "TransformConstraint"] {
            let mut changed = records.clone();
            changed.push(Record::new(kind));
            assert!(
                matches!(inspect(&changed), Err(Unresolved::RecordKind { .. })),
                "{kind}"
            );
        }
        let mut changed = records.clone();
        set(&mut changed[3], "opacity", Value::Float(1.));
        assert!(matches!(
            inspect(&changed),
            Err(Unresolved::Property { .. })
        ));
        let mut changed = records.clone();
        set(&mut changed[6], "displayValue", Value::Uint(1));
        assert!(matches!(
            inspect(&changed),
            Err(Unresolved::Property { .. })
        ));
        let mut changed = records.clone();
        set(&mut changed[2], "minWidth", Value::Float(200.));
        assert!(matches!(
            inspect(&changed),
            Err(Unresolved::Style { id: 0 })
        ));
        let mut changed = records.clone();
        set(&mut changed[3], "parentId", Value::Uint(4));
        assert!(matches!(inspect(&changed), Err(Unresolved::Parent { .. })));
        let mut changed = records;
        set(&mut changed[5], "parentId", Value::Uint(6));
        assert!(matches!(inspect(&changed), Err(Unresolved::Parent { .. })));
    }
    #[test]
    fn rejects_shared_or_missing_style_and_misowned_paint() {
        let records = scene();
        let mut changed = records.clone();
        set(&mut changed[5], "styleId", Value::Uint(3));
        assert!(matches!(inspect(&changed), Err(Unresolved::StyleOwnership)));
        let mut changed = records.clone();
        set(&mut changed[5], "styleId", Value::Uint(6));
        assert!(matches!(inspect(&changed), Err(Unresolved::Style { .. })));
        let mut changed = records.clone();
        changed.push(Record::new("LayoutComponentStyle"));
        assert!(matches!(inspect(&changed), Err(Unresolved::StyleOwnership)));
        for (position, parent) in [(7, 3), (8, 4)] {
            let mut changed = records.clone();
            set(&mut changed[position], "parentId", Value::Uint(parent));
            assert!(matches!(inspect(&changed), Err(Unresolved::PaintOwnership)));
        }
        let mut changed = records.clone();
        changed.pop();
        assert!(matches!(inspect(&changed), Err(Unresolved::PaintOwnership)));
        let mut changed = records;
        changed.push(changed[8].clone());
        assert!(matches!(inspect(&changed), Err(Unresolved::PaintOwnership)));
    }
}
