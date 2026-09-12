//! Ordinary-file cross-axis alignment for independently sized wrapping slots.
//!
//! This is a lowering primitive, not CSS admission. The caller must prove that
//! slots do not depend on their visible children, line anchors are distinguishable
//! by the immutable DistanceConstraint, and all carried extents are below RESET_DISTANCE. Visible children use ordinary
//! ComponentOrigin anchors so their own resolved sizes participate without
//! measuring their constrained world transforms.
use crate::{Diagnostic, wire::{Record, Value}};

const RESET_DISTANCE: f32 = 65_536.;

#[derive(Clone, Copy)]
pub(super) struct Item {
    pub slot: u32,
    pub visible: u32,
    /// Physical start/center/end fraction before wrap reversal.
    pub alignment: f32,
}

/// Read-only observation handles for a private native experiment. These IDs
/// add no file records or names and are not a runtime-policy sidecar.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Trace {
    pub origin: Option<u32>,
    pub tops: Vec<u32>,
    pub heights: Vec<u32>,
    pub anchors: Vec<u32>,
    pub boundaries: Vec<Boundary>,
    pub forward: Vec<u32>,
    pub backward: Vec<u32>,
    pub line_maxima: Vec<u32>,
    pub offsets: Vec<u32>,
    pub targets: Vec<u32>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Boundary {
    pub difference: u32,
    pub absolute: u32,
    pub dead: u32,
    pub normalized: u32,
    pub gate: u32,
}

/// Append the measurement graph in logical packing order. Only slots are
/// measured; only visible children move, avoiding a layout feedback cycle.
/// Parent wrapping/direction/line alignment is configured by the caller.
/// `epsilon` is an explicit caller input, not a semantic certificate or default:
/// same-line error must fit inside it, and distinct-line residuals must exceed
/// the immutable normalizer's 0.001 early-return region. The experimental 1/64
/// value does not prove this condition for arbitrary layouts.
pub(super) fn align_items(
    records: &mut Vec<Record>, items: &[Item], row: bool,
    reverse_cross: bool, line_fraction: f32, epsilon: f32,
) -> Result<Trace, Diagnostic> {
    let invalid = |message| Diagnostic::new("wrapping-plan", "wrapping", message);
    if !epsilon.is_finite() || !(0. ..RESET_DISTANCE).contains(&epsilon) {
        return Err(invalid("Line gate epsilon must be finite, nonnegative and below the reset distance"));
    }
    if ![0., 0.5, 1.].contains(&line_fraction)
        || items.iter().any(|item| ![0., 0.5, 1.].contains(&item.alignment)) {
        return Err(invalid("Positional wrapping requires start, center or end fractions"));
    }
    for item in items {
        for id in [item.slot, item.visible] {
            if !(id as usize).checked_add(1).and_then(|index| records.get(index))
                .is_some_and(|r| r.kind == "LayoutComponent") {
                return Err(invalid("Wrapping measurement and visible handles must identify layout components"));
            }
        }
        if item.slot == item.visible {
            return Err(invalid("A wrapping slot must be independent of its visible child"));
        }
    }
    let visible = items.iter().map(|item| item.visible).collect::<std::collections::BTreeSet<_>>();
    if visible.len() != items.len() || records.iter().any(|record| record.kind == "ComponentOrigin"
        && matches!(record.get("parentId"),Some(Value::Uint(id)) if visible.contains(id))) {
        return Err(invalid("Wrapping visible owners require one newly owned component origin"));
    }
    let added = alignment_records(items.len())
        .ok_or_else(|| invalid("Wrapping alignment record count overflow"))?;
    let end = records.len().checked_add(added)
        .filter(|count| *count <= u32::MAX as usize)
        .ok_or_else(|| invalid("Wrapping alignment exceeds ordinary object ID capacity"))?;
    if items.is_empty() { return Ok(Trace::default()); }
    let mut graph = Graph { records, row, epsilon };
    let fraction = if reverse_cross { 1. - line_fraction } else { line_fraction };
    let origin = graph.add("Node", 0)?;
    let mut tops = Vec::with_capacity(items.len());
    let mut heights = Vec::with_capacity(items.len());
    let mut anchors = Vec::with_capacity(items.len());
    for item in items {
        let top = graph.landmark(item.slot, 0.)?;
        let bottom = graph.landmark(item.slot, 1.)?;
        tops.push(top);
        heights.push(graph.diff(bottom, top)?);
        anchors.push(graph.landmark(item.slot, fraction)?);
    }
    let mut boundaries = Vec::with_capacity(items.len() - 1);
    for pair in anchors.windows(2) {
        boundaries.push(graph.separated(pair[0], pair[1], origin)?);
    }
    // Reset the carried maximum at a line boundary. The two passes provide
    // each participant with the maximum extent of its own line.
    let mut forward = Vec::with_capacity(items.len());
    forward.push(heights[0]);
    for i in 1..items.len() {
        let delta = graph.diff(forward[i - 1], boundaries[i - 1].gate)?;
        let carry = graph.relu(delta)?;
        forward.push(graph.max(carry, heights[i])?);
    }
    let mut backward = vec![0; items.len()];
    backward[items.len() - 1] = heights[items.len() - 1];
    for i in (0..items.len() - 1).rev() {
        let delta = graph.diff(backward[i + 1], boundaries[i].gate)?;
        let carry = graph.relu(delta)?;
        backward[i] = graph.max(carry, heights[i])?;
    }
    let mut line_maxima=Vec::with_capacity(items.len());
    let mut offsets=Vec::with_capacity(items.len());
    let mut targets=Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let maximum = graph.max(forward[i], backward[i])?;
        let alignment = if reverse_cross { 1. - item.alignment } else { item.alignment };
        // The slot anchor is line_origin + line_max * fraction. Land the
        // visible child's own alignment anchor at line_origin + line_max *
        // alignment; native land_anchor subtracts alignment * visible_size.
        let offset = graph.scale(maximum, alignment - fraction)?;
        let target = graph.sum(anchors[i], offset)?;
        let origin = graph.add("ComponentOrigin", item.visible)?;
        graph.set(origin, "originX", Value::Float(if row { 0. } else { alignment }))?;
        graph.set(origin, "originY", Value::Float(if row { alignment } else { 0. }))?;
        graph.copy(item.visible, target, 1., false, false)?;
        line_maxima.push(maximum);offsets.push(offset);targets.push(target);
    }
    if graph.records.len() != end {
        return Err(invalid("Wrapping alignment emission disagrees with its preflight cost"));
    }
    Ok(Trace{origin:Some(origin),tops,heights,anchors,boundaries,forward,backward,line_maxima,offsets,targets})
}

/// One origin; 16 measurement and 8 positioning records per item; 18 boundary
/// records and two 8-record carry steps per adjacent pair: 58N - 33 for N > 0.
pub(super) fn alignment_records(items: usize) -> Option<usize> {
    if items == 0 { Some(0) }
    else { items.checked_mul(58)?.checked_sub(33) }
}

struct Graph<'a> { records: &'a mut Vec<Record>, row: bool, epsilon: f32 }
impl Graph<'_> {
    fn add(&mut self, kind: &'static str, parent: u32) -> Result<u32, Diagnostic> {
        let id = self.records.len() as u32 - 1;
        let mut record = Record::new(kind);
        record.set("parentId", Value::Uint(parent))?;
        self.records.push(record);
        Ok(id)
    }
    fn set(&mut self, id: u32, key: &str, value: Value) -> Result<(), Diagnostic> {
        self.records[id as usize + 1].set(key, value)
    }
    fn copy(&mut self, parent: u32, target: u32, factor: f32, local: bool, minimum: bool) -> Result<u32, Diagnostic> {
        let id = self.add("TranslationConstraint", parent)?;
        self.set(id, "targetId", Value::Uint(target))?;
        self.set(id, "doesCopy", Value::Bool(!self.row))?;
        self.set(id, "doesCopyY", Value::Bool(self.row))?;
        self.set(id, if self.row { "copyFactorY" } else { "copyFactor" }, Value::Float(factor))?;
        if local { self.set(id, "destSpaceValue", Value::Uint(1))?; }
        if minimum {
            self.set(id, if self.row { "minY" } else { "min" }, Value::Bool(true))?;
            self.set(id, if self.row { "minValueY" } else { "minValue" }, Value::Float(0.))?;
        }
        Ok(id)
    }
    fn scale(&mut self, a: u32, factor: f32) -> Result<u32, Diagnostic> {
        let node = self.add("Node", 0)?;
        self.copy(node, a, factor, false, false)?;
        Ok(node)
    }
    fn sum(&mut self, a: u32, b: u32) -> Result<u32, Diagnostic> {
        let node = self.add("Node", a)?;
        self.copy(node, b, 1., true, false)?;
        Ok(node)
    }
    fn diff(&mut self, a: u32, b: u32) -> Result<u32, Diagnostic> {
        let negative = self.scale(b, -1.)?;
        self.sum(a, negative)
    }
    fn relu(&mut self, a: u32) -> Result<u32, Diagnostic> {
        let node = self.add("Node", 0)?;
        self.copy(node, a, 1., false, true)?;
        Ok(node)
    }
    fn max(&mut self, a: u32, b: u32) -> Result<u32, Diagnostic> {
        let node = self.add("Node", a)?;
        let constraint = self.copy(node, b, 1., false, true)?;
        self.set(constraint, "minMaxSpaceValue", Value::Uint(1))?;
        Ok(node)
    }
    fn landmark(&mut self, target: u32, fraction: f32) -> Result<u32, Diagnostic> {
        let node = self.add("Node", 0)?;
        let constraint = self.add("TransformConstraint", node)?;
        self.set(constraint, "targetId", Value::Uint(target))?;
        self.set(constraint, if self.row { "originY" } else { "originX" }, Value::Float(fraction))?;
        self.scale(node, 1.)
    }
    fn separated(&mut self, a: u32, b: u32, origin: u32) -> Result<Boundary, Diagnostic> {
        let difference = self.diff(a, b)?;
        // Match wrapping_paint::Graph::snapped: absolute value first, then a
        // dead zone, normalization, doubling and an exact upper clamp.
        let negative = self.scale(difference, -1.)?;
        let absolute = self.max(difference, negative)?;
        let epsilon = self.add("Node", 0)?;
        self.set(epsilon, if self.row { "y" } else { "x" }, Value::Float(-self.epsilon))?;
        let shifted = self.sum(absolute, epsilon)?;
        let dead = self.relu(shifted)?;
        let normalized = self.scale(dead, 1.)?;
        let constraint = self.add("DistanceConstraint", normalized)?;
        self.set(constraint, "targetId", Value::Uint(origin))?;
        self.set(constraint, "modeValue", Value::Uint(2))?;
        self.set(constraint, "distance", Value::Float(RESET_DISTANCE))?;
        let gate = self.add("Node", 0)?;
        let constraint = self.copy(gate, normalized, 2., false, false)?;
        self.set(constraint, if self.row { "maxY" } else { "max" }, Value::Bool(true))?;
        self.set(constraint, if self.row { "maxValueY" } else { "maxValue" }, Value::Float(RESET_DISTANCE))?;
        Ok(Boundary{difference,absolute,dead,normalized,gate})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(count: usize) -> (Vec<Record>, Vec<Item>) {
        let mut records = vec![Record::new("Backboard"), Record::new("Artboard")];
        let items = (0..count).map(|i| {
            let slot = records.len() as u32 - 1;
            records.push(Record::new("LayoutComponent"));
            records.push(Record::new("LayoutComponent"));
            Item { slot, visible: slot + 1, alignment: (i % 3) as f32 / 2. }
        }).collect();
        (records, items)
    }
    #[test]
    fn preflight_matches_variable_participant_graphs() {
        for count in [0, 1, 2, 3, 8, 16, 32] {
            for row in [false, true] {
                for reverse in [false, true] {
                    for fraction in [0., 0.5, 1.] {
                        let (mut records, items) = fixture(count);
                        let start = records.len();
                        align_items(&mut records, &items, row, reverse, fraction, 1./64.).unwrap();
                        assert_eq!(records.len() - start, alignment_records(count).unwrap());
                        crate::wire::encode(&records).unwrap();
                    }
                }
            }
        }
        assert_eq!(alignment_records(usize::MAX), None);
    }
    #[test]
    fn invalid_epsilon_diagnoses_before_any_emission_even_for_empty_items() {
        for epsilon in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY,-0.001,RESET_DISTANCE,RESET_DISTANCE*2.] {
            for count in [0,3] {
                let (mut records,items)=fixture(count);
                let before=crate::wire::encode(&records).unwrap();
                assert!(align_items(&mut records,&items,true,false,0.,epsilon).is_err());
                assert_eq!(crate::wire::encode(&records).unwrap(),before);
            }
        }
        for epsilon in [0.,1./64.,RESET_DISTANCE.next_down()] {
            let (mut records,items)=fixture(0);
            assert_eq!(align_items(&mut records,&items,false,false,0.,epsilon).unwrap(),Trace::default());
        }
    }
    #[test]
    fn trace_identifies_clamped_unsigned_gates_consumed_by_both_carry_passes() {
        for row in [false,true] {
            let (mut records,items)=fixture(3);
            let start=records.len();
            let trace=align_items(&mut records,&items,row,true,0.5,1./64.).unwrap();
            assert_eq!(records.len()-start,141);
            for ids in [&trace.tops,&trace.heights,&trace.anchors,&trace.forward,&trace.backward,&trace.line_maxima,&trace.offsets,&trace.targets] {
                assert_eq!(ids.len(),3);
                assert!(ids.iter().all(|&id|records[id as usize+1].kind=="Node"));
            }
            assert_eq!(trace.boundaries.len(),2);
            let factor=if row {"copyFactorY"} else {"copyFactor"};
            for boundary in &trace.boundaries {
                let constraints:Vec<_>=records.iter().filter(|r|matches!(r.get("parentId"),Some(Value::Uint(v)) if *v==boundary.gate)).collect();
                assert_eq!(constraints.len(),1);
                let clamp=constraints[0];assert_eq!(clamp.kind,"TranslationConstraint");
                assert!(matches!(clamp.get("targetId"),Some(Value::Uint(v)) if *v==boundary.normalized));
                assert!(matches!(clamp.get(factor),Some(Value::Float(2.))));
                assert!(matches!(clamp.get(if row {"maxY"} else {"max"}),Some(Value::Bool(true))));
                assert!(matches!(clamp.get(if row {"maxValueY"} else {"maxValue"}),Some(Value::Float(v)) if *v==RESET_DISTANCE));
                let normalized=&records[boundary.normalized as usize+1];assert_eq!(normalized.kind,"Node");
                let distance=records.iter().find(|r|r.kind=="DistanceConstraint" && matches!(r.get("parentId"),Some(Value::Uint(v)) if *v==boundary.normalized)).unwrap();
                assert!(matches!(distance.get("targetId"),Some(Value::Uint(v)) if Some(*v)==trace.origin));
                assert!(matches!(distance.get("distance"),Some(Value::Float(v)) if *v==RESET_DISTANCE));
                // Both line-maximum passes subtract the finalized gate, never
                // its approximate normalizer output or signed difference.
                let reset_consumers=records.iter().filter(|r|r.kind=="TranslationConstraint"
                    && matches!(r.get("targetId"),Some(Value::Uint(v)) if *v==boundary.gate)
                    && matches!(r.get(factor),Some(Value::Float(-1.)))).count();
                assert_eq!(reset_consumers,2);
            }
            let constants=records[start..].iter().filter(|r|r.kind=="Node" && matches!(r.get(if row {"y"} else {"x"}),Some(Value::Float(v)) if *v == -1./64.)).count();
            assert_eq!(constants,2);
            let (mut repeated,repeated_items)=fixture(3);
            let repeated_trace=align_items(&mut repeated,&repeated_items,row,true,0.5,1./64.).unwrap();
            assert_eq!(trace,repeated_trace);
            assert_eq!(crate::wire::encode(&records).unwrap(),crate::wire::encode(&repeated).unwrap());
        }
    }
    #[test]
    fn existing_or_duplicate_visible_origins_reject_atomically() {
        for duplicate in [false,true] {
            let (mut records,mut items)=fixture(2);
            if duplicate {items[1].visible=items[0].visible;} else {
                let mut origin=Record::new("ComponentOrigin");
                origin.set("parentId",Value::Uint(items[0].visible)).unwrap();records.push(origin);
            }
            let before=crate::wire::encode(&records).unwrap();
            assert!(align_items(&mut records,&items,true,false,0.,0.25).is_err());
            assert_eq!(crate::wire::encode(&records).unwrap(),before);
        }
    }
    #[test]
    fn rejects_invalid_plan_before_emission() {
        for alignment in [f32::NAN, f32::INFINITY, -1., 0.25, 2.] {
            let (mut records, mut items) = fixture(1);
            items[0].alignment = alignment;
            let before = crate::wire::encode(&records).unwrap();
            assert!(align_items(&mut records, &items, true, false, 0., 1./64.).is_err());
            assert_eq!(crate::wire::encode(&records).unwrap(), before);
        }
        let (mut records, mut items) = fixture(1);
        items[0].visible = items[0].slot;
        assert!(align_items(&mut records, &items, true, false, 0., 1./64.).is_err());
        items[0].visible = u32::MAX;
        assert!(align_items(&mut records, &items, true, false, 0., 1./64.).is_err());
    }
}
