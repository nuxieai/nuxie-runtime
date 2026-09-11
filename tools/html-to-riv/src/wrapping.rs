//! Ordinary-file cross-axis alignment for independently sized wrapping slots.
//!
//! This is a lowering primitive, not CSS admission. The caller must prove that
//! slots do not depend on their visible children, line anchors are distinguishable
//! by the immutable DistanceConstraint, and all extents are below RESET_DISTANCE.
use crate::{Diagnostic, wire::{Record, Value}};

const RESET_DISTANCE: f32 = 65_536.;

#[derive(Clone, Copy)]
pub(super) struct Item {
    pub slot: u32,
    pub visible: u32,
    /// Physical start/center/end fraction before wrap reversal.
    pub alignment: f32,
}

/// Append the measurement graph in logical packing order. Only slots are
/// measured; only visible children move, avoiding a layout feedback cycle.
/// Parent wrapping/direction/line alignment is configured by the caller.
pub(super) fn align_items(
    records: &mut Vec<Record>, items: &[Item], row: bool,
    reverse_cross: bool, line_fraction: f32,
) -> Result<(), Diagnostic> {
    let invalid = |message| Diagnostic::new("wrapping-plan", "wrapping", message);
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
    let added = alignment_records(items.len())
        .ok_or_else(|| invalid("Wrapping alignment record count overflow"))?;
    let end = records.len().checked_add(added)
        .filter(|count| *count <= u32::MAX as usize)
        .ok_or_else(|| invalid("Wrapping alignment exceeds ordinary object ID capacity"))?;
    if items.is_empty() { return Ok(()); }
    let mut graph = Graph { records, row };
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
    let mut separations = Vec::with_capacity(items.len() - 1);
    for pair in anchors.windows(2) {
        separations.push(graph.separated(pair[0], pair[1], origin)?);
    }
    // Reset the carried maximum at a line boundary. The two passes provide
    // each participant with the maximum extent of its own line.
    let mut forward = Vec::with_capacity(items.len());
    forward.push(heights[0]);
    for i in 1..items.len() {
        let delta = graph.diff(forward[i - 1], separations[i - 1])?;
        let carry = graph.relu(delta)?;
        forward.push(graph.max(carry, heights[i])?);
    }
    let mut backward = vec![0; items.len()];
    backward[items.len() - 1] = heights[items.len() - 1];
    for i in (0..items.len() - 1).rev() {
        let delta = graph.diff(backward[i + 1], separations[i])?;
        let carry = graph.relu(delta)?;
        backward[i] = graph.max(carry, heights[i])?;
    }
    for (i, item) in items.iter().enumerate() {
        let maximum = graph.max(forward[i], backward[i])?;
        let excess = graph.diff(maximum, heights[i])?;
        let alignment = if reverse_cross { 1. - item.alignment } else { item.alignment };
        let offset = graph.scale(excess, alignment - fraction)?;
        let target = graph.sum(tops[i], offset)?;
        graph.copy(item.visible, target, 1., false, false)?;
    }
    if graph.records.len() != end {
        return Err(invalid("Wrapping alignment emission disagrees with its preflight cost"));
    }
    Ok(())
}

/// One origin; 16 measurement and 11 positioning records per item; 9 boundary
/// records and two 8-record carry steps per adjacent pair: 52N - 24 for N > 0.
pub(super) fn alignment_records(items: usize) -> Option<usize> {
    if items == 0 { Some(0) }
    else { items.checked_mul(52)?.checked_sub(24) }
}

struct Graph<'a> { records: &'a mut Vec<Record>, row: bool }
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
    fn separated(&mut self, a: u32, b: u32, origin: u32) -> Result<u32, Diagnostic> {
        let node = self.diff(a, b)?;
        let constraint = self.add("DistanceConstraint", node)?;
        self.set(constraint, "targetId", Value::Uint(origin))?;
        self.set(constraint, "modeValue", Value::Uint(2))?;
        self.set(constraint, "distance", Value::Float(RESET_DISTANCE))?;
        let negative = self.scale(node, -1.)?;
        self.max(node, negative)
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
                        align_items(&mut records, &items, row, reverse, fraction).unwrap();
                        assert_eq!(records.len() - start, alignment_records(count).unwrap());
                        crate::wire::encode(&records).unwrap();
                    }
                }
            }
        }
        assert_eq!(alignment_records(usize::MAX), None);
    }
    #[test]
    fn rejects_invalid_plan_before_emission() {
        for alignment in [f32::NAN, f32::INFINITY, -1., 0.25, 2.] {
            let (mut records, mut items) = fixture(1);
            items[0].alignment = alignment;
            let before = crate::wire::encode(&records).unwrap();
            assert!(align_items(&mut records, &items, true, false, 0.).is_err());
            assert_eq!(crate::wire::encode(&records).unwrap(), before);
        }
        let (mut records, mut items) = fixture(1);
        items[0].visible = items[0].slot;
        assert!(align_items(&mut records, &items, true, false, 0.).is_err());
        items[0].visible = u32::MAX;
        assert!(align_items(&mut records, &items, true, false, 0.).is_err());
    }
}
