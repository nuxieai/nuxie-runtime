//! Compiler-private ordinary paint-group lowering, not CSS admission.
//! The caller must prove line anchors remain distinguishable by the immutable
//! DistanceConstraint, coordinate/mask bounds, independent slot geometry, and
//! the supplied logical packing order and within-item paint order.
use crate::{Diagnostic, wire::{Record, Value}};
use std::collections::{BTreeMap, BTreeSet};
const DISTANCE: f32 = 65_536.;

#[derive(Clone, Copy)]
pub(super) struct Paint { pub geometry: u32, pub fill: u32, pub color: u32 }
pub(super) struct Item { pub slot: u32, pub paints: Vec<Paint> }
fn invalid(message: &str) -> Diagnostic { Diagnostic::new("wrapping-paint-plan", "wrapping", message) }

/// Exact additional record count, including masks, clips and draw ordering.
/// Empty paint plans emit nothing. Item counts must include unpainted items:
/// their geometry still affects line packing and leader selection.
pub(super) fn paint_records(counts: &[usize]) -> Option<usize> {
    let n = counts.len();
    if counts.iter().all(|count| *count == 0) { return Some(0); }
    let (mut replicas, mut clips) = (0usize, 0usize);
    for (j, count) in counts.iter().copied().enumerate() {
        replicas = replicas.checked_add(j.checked_add(1)?.checked_mul(count)?)?;
        clips = clips.checked_add(j.checked_mul(count)?.checked_mul(2)?)?;
    }
    let pairs = n.checked_mul(n.checked_sub(1)?)?.checked_div(2)?;
    n.checked_mul(8)?.checked_sub(5)?.checked_add(pairs.checked_mul(7)?)?
        .checked_add(replicas.checked_mul(3)?)?
        .checked_add(replicas.checked_sub(1)?.checked_mul(2)?)?.checked_add(clips)
}
fn record(records: &[Record], id: u32) -> Option<&Record> {
    (id as usize).checked_add(1).and_then(|index| records.get(index))
}
fn parent(record: &Record) -> Option<u32> {
    match record.get("parentId") { Some(Value::Uint(id)) => Some(*id), _ => None }
}
fn descendant(records: &[Record], mut id: u32, ancestor: u32, slots: &BTreeSet<u32>) -> bool {
    // Bound traversal so malformed parent cycles cannot hang plan validation.
    for _ in 0..records.len() {
        let Some(next) = record(records, id).and_then(parent) else { return false };
        if next == ancestor { return true; }
        if slots.contains(&next) || next == id || next == 0 { return false; }
        id = next;
    }
    false
}

/// Parent layout configuration is owned by the caller. This changes original
/// color values only after all handles and the full expansion have validated.
pub(super) fn paint_items(
    records: &mut Vec<Record>, items: &[Item], row: bool, reverse_main: bool,
    reverse_cross: bool, line_fraction: f32,
) -> Result<(), Diagnostic> {
    if ![0., 0.5, 1.].contains(&line_fraction) {
        return Err(invalid("Line fraction must be start, center or end"));
    }
    let added = paint_records(&items.iter().map(|item| item.paints.len()).collect::<Vec<_>>())
        .ok_or_else(|| invalid("Paint expansion record count overflow"))?;
    let end = records.len().checked_add(added).filter(|n| *n <= u32::MAX as usize)
        .ok_or_else(|| invalid("Paint expansion exceeds ordinary object ID capacity"))?;
    let mut slots = BTreeSet::new();
    let mut fills = BTreeSet::new();
    let mut colors = BTreeSet::new();
    for item in items {
        if !slots.insert(item.slot) || !record(records, item.slot).is_some_and(|r| r.kind == "LayoutComponent") {
            return Err(invalid("Paint slots must be distinct layout components"));
        }
    }
    for item in items {
        for paint in &item.paints {
            if slots.contains(&paint.geometry)
                || !record(records, paint.geometry).is_some_and(|r| r.kind == "LayoutComponent")
                || !descendant(records, paint.geometry, item.slot, &slots) {
                return Err(invalid("Paint geometry must belong beneath its independent slot"));
            }
            if !fills.insert(paint.fill) || !record(records, paint.fill)
                .is_some_and(|r| r.kind == "Fill" && parent(r) == Some(paint.geometry)) {
                return Err(invalid("Every paint needs a unique Fill owned by its geometry"));
            }
            if !colors.insert(paint.color) || !record(records, paint.color)
                .is_some_and(|r| r.kind == "SolidColor" && parent(r) == Some(paint.fill)) {
                return Err(invalid("Every paint needs a unique SolidColor owned by its Fill"));
            }
        }
    }
    if added == 0 { return Ok(()); }
    let originals: Vec<Vec<_>> = items.iter().map(|item| item.paints.iter().map(|paint| {
        (record(records, paint.fill).unwrap().clone(), record(records, paint.color).unwrap().clone())
    }).collect()).collect();
    for item in items { for paint in &item.paints {
        records[paint.color as usize + 1].set("colorValue", Value::Color(0))?;
    }}
    let mut graph = Graph { records, row, sources: BTreeMap::new(), signals: BTreeMap::new() };
    let origin = graph.add("Node", 0)?;
    let mut anchors = Vec::with_capacity(items.len());
    let fraction = if reverse_cross { 1. - line_fraction } else { line_fraction };
    for item in items {
        let node = graph.add("Node", 0)?;
        let constraint = graph.add("TransformConstraint", node)?;
        graph.set(constraint, "targetId", Value::Uint(item.slot))?;
        graph.set(constraint, if row { "originY" } else { "originX" }, Value::Float(fraction))?;
        anchors.push(node);
    }
    let mut groups: Vec<_> = (0..items.len()).collect();
    let mut members = groups.clone();
    if reverse_cross { groups.reverse(); }
    if reverse_main { members.reverse(); }
    let mut paints = Vec::new();
    for i in groups {
        let leader = if i == 0 { None } else {
            let signal = graph.signal(anchors[i-1], anchors[i], origin)?;
            let invert = graph.add("Node", 0)?;
            graph.set(invert, if row { "y" } else { "x" }, Value::Float(DISTANCE))?;
            let constraint = graph.translation(invert, signal, false)?;
            graph.set(constraint, if row { "copyFactorY" } else { "copyFactor" }, Value::Float(if reverse_cross { 1. } else { -1. }))?;
            graph.set(constraint, "offset", Value::Bool(true))?;
            Some(graph.mask(invert)?)
        };
        for &j in &members {
            if j < i { continue; }
            let membership = if i == j { None } else {
                let gate = graph.signal(anchors[i], anchors[j], origin)?;
                Some(graph.mask(gate)?)
            };
            for (paint, (original_fill, original_color)) in items[j].paints.iter().zip(&originals[j]) {
                let foreground = graph.add("ForegroundLayoutDrawable", paint.geometry)?;
                let fill = graph.records.len() as u32 - 1;
                let mut cloned_fill = original_fill.clone();
                cloned_fill.set("parentId", Value::Uint(foreground))?;
                graph.records.push(cloned_fill);
                let mut cloned_color = original_color.clone();
                cloned_color.set("parentId", Value::Uint(fill))?;
                graph.records.push(cloned_color);
                for mask in [leader, membership].into_iter().flatten() {
                    let clip = graph.add("ClippingShape", foreground)?;
                    graph.set(clip, "sourceId", Value::Uint(mask))?;
                }
                paints.push(foreground);
            }
        }
    }
    for pair in paints.windows(2) {
        let rule = graph.add("DrawRules", pair[0])?;
        let target = graph.add("DrawTarget", rule)?;
        graph.set(rule, "drawTargetId", Value::Uint(target))?;
        graph.set(target, "drawableId", Value::Uint(pair[1]))?;
        graph.set(target, "placementValue", Value::Uint(1))?;
    }
    if graph.records.len() != end { return Err(invalid("Paint emission disagrees with preflight count")); }
    Ok(())
}
struct Graph<'a> {
    records: &'a mut Vec<Record>, row: bool,
    sources: BTreeMap<u32, u32>, signals: BTreeMap<(u32,u32), u32>,
}
impl Graph<'_> {
    fn add(&mut self, kind: &'static str, parent: u32) -> Result<u32, Diagnostic> {
        let id = self.records.len() as u32 - 1;
        let mut record = Record::new(kind); record.set("parentId", Value::Uint(parent))?;
        self.records.push(record); Ok(id)
    }
    fn set(&mut self, id: u32, name: &str, value: Value) -> Result<(), Diagnostic> {
        self.records[id as usize + 1].set(name, value)
    }
    fn translation(&mut self, parent: u32, target: u32, local: bool) -> Result<u32, Diagnostic> {
        let id = self.add("TranslationConstraint", parent)?;
        self.set(id, "targetId", Value::Uint(target))?;
        self.set(id, "doesCopy", Value::Bool(!self.row))?;
        self.set(id, "doesCopyY", Value::Bool(self.row))?;
        self.set(id, "sourceSpaceValue", Value::Uint(if local { 1 } else { 0 }))?;
        Ok(id)
    }
    fn signal(&mut self, a: u32, b: u32, origin: u32) -> Result<u32, Diagnostic> {
        if let Some(id) = self.signals.get(&(a,b)) { return Ok(*id); }
        let source = if let Some(id) = self.sources.get(&a) { *id } else {
            let node = self.add("Node", 0)?; self.translation(node, a, false)?;
            self.sources.insert(a, node); node
        };
        let difference = self.add("Node", source)?; self.translation(difference, b, false)?;
        let gate = self.add("Node", 0)?; self.translation(gate, difference, true)?;
        let clamp = self.add("DistanceConstraint", gate)?;
        self.set(clamp, "targetId", Value::Uint(origin))?;
        self.set(clamp, "modeValue", Value::Uint(2))?;
        self.set(clamp, "distance", Value::Float(DISTANCE))?;
        self.signals.insert((a,b), gate); Ok(gate)
    }
    fn mask(&mut self, parent: u32) -> Result<u32, Diagnostic> {
        let shape = self.add("Shape", parent)?;
        let rect = self.add("Rectangle", shape)?;
        for (name,value) in [("x",16384.),("y",16384.),("width",32768.),("height",32768.)] {
            self.set(rect, name, Value::Float(value))?;
        }
        Ok(shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn add(records: &mut Vec<Record>, kind: &'static str, owner: u32) -> u32 {
        let id = records.len() as u32 - 1;
        let mut r = Record::new(kind); r.set("parentId", Value::Uint(owner)).unwrap(); records.push(r); id
    }
    fn fixture(counts: &[usize]) -> (Vec<Record>, Vec<Item>) {
        let mut records = vec![Record::new("Backboard"), Record::new("Artboard")];
        let mut items = Vec::new();
        for &count in counts {
            let slot = add(&mut records, "LayoutComponent", 0);
            let mut paints = Vec::new();
            let mut owner = slot;
            for i in 0..count {
                let geometry = add(&mut records, "LayoutComponent", owner);
                let fill = add(&mut records, "Fill", geometry);
                records[fill as usize + 1].set("fillRule", Value::Uint(1)).unwrap();
                records[fill as usize + 1].set("isVisible", Value::Bool(false)).unwrap();
                let color = add(&mut records, "SolidColor", fill);
                records[color as usize + 1].set("colorValue", Value::Color(0x80ff0000 + i as u32)).unwrap();
                paints.push(Paint { geometry, fill, color }); owner = geometry;
            }
            items.push(Item { slot, paints });
        }
        (records, items)
    }
    #[test]
    fn exact_preflight_handles_nonuniform_and_empty_paint_lists() {
        for counts in [vec![],vec![0],vec![0,0,0],vec![1],vec![2,2,2],vec![0,3,1,0,2],vec![1;8],vec![2;32]] {
            for (row,main,cross,fraction) in [(true,false,false,0.),(false,true,false,0.5),(true,true,true,1.)] {
                let (mut records,items)=fixture(&counts);let start=records.len();
                paint_items(&mut records,&items,row,main,cross,fraction).unwrap();
                assert_eq!(records.len()-start,paint_records(&counts).unwrap());
                crate::wire::encode(&records).unwrap();
            }
        }
        assert_eq!(paint_records(&[1,1,1]),Some(74));
        assert_eq!(paint_records(&[2,2,2]),Some(110));
        assert_eq!(paint_records(&[usize::MAX]),None);
        assert_eq!(paint_records(&[0,usize::MAX]),None);
    }
    #[test]
    fn replicas_preserve_fill_properties_color_and_item_paint_order() {
        let (mut records,items)=fixture(&[2,1]);let start=records.len();
        let originals=records.clone();paint_items(&mut records,&items,true,true,true,0.5).unwrap();
        for item in &items { for paint in &item.paints {
            assert!(matches!(record(&records,paint.color).unwrap().get("colorValue"),Some(Value::Color(0))));
            let mut restored=records[paint.color as usize+1].clone();
            restored.set("colorValue",originals[paint.color as usize+1].get("colorValue").unwrap().clone()).unwrap();
            assert_eq!(crate::wire::encode(&[restored]).unwrap(),crate::wire::encode(&[originals[paint.color as usize+1].clone()]).unwrap());
        }}
        let geometry_order:Vec<_>=records[start..].iter().filter(|r|r.kind=="ForegroundLayoutDrawable").map(|r|parent(r).unwrap()).collect();
        assert_eq!(geometry_order,[items[1].paints[0].geometry,items[1].paints[0].geometry,items[0].paints[0].geometry,items[0].paints[1].geometry]);
        for r in records[start..].iter().filter(|r|r.kind=="Fill") {
            assert!(matches!(r.get("fillRule"),Some(Value::Uint(1))));
            assert!(matches!(r.get("isVisible"),Some(Value::Bool(false))));
        }
    }
    #[test]
    fn invalid_handles_ownership_duplicates_and_cycles_reject_without_mutation() {
        for case in 0..9 {
            let (mut records,mut items)=fixture(&[2,1]);
            match case {
                0=>items[0].slot=u32::MAX,
                1=>items[1].slot=items[0].slot,
                2=>items[0].paints[0].geometry=items[0].slot,
                3=>items[0].paints[0].fill=items[1].paints[0].fill,
                4=>items[0].paints[0].color=items[1].paints[0].color,
                5=>{let paint=items[0].paints[0];items[0].paints.push(paint);},
                6=>{let id=items[0].paints[0].geometry;records[id as usize+1].set("parentId",Value::Uint(id)).unwrap();},
                7=>{let paint=items[1].paints.remove(0);items[0].paints.push(paint);},
                _=>{let slot=items[1].slot;records[slot as usize+1].set("parentId",Value::Uint(items[0].slot)).unwrap();let paint=items[1].paints.remove(0);items[0].paints.push(paint);},
            }
            let before=crate::wire::encode(&records).unwrap();
            assert!(paint_items(&mut records,&items,true,false,false,0.).is_err(),"case {case}");
            assert_eq!(crate::wire::encode(&records).unwrap(),before);
        }
    }
    #[test]
    fn invalid_fractions_and_empty_paint_plans_are_nonmutating() {
        for fraction in [f32::NAN,f32::INFINITY,-1.,0.25,2.] {
            let (mut records,items)=fixture(&[1]);let before=crate::wire::encode(&records).unwrap();
            assert!(paint_items(&mut records,&items,true,false,false,fraction).is_err());
            assert_eq!(crate::wire::encode(&records).unwrap(),before);
        }
        let (mut records,items)=fixture(&[0,0]);let before=crate::wire::encode(&records).unwrap();
        paint_items(&mut records,&items,true,false,false,0.).unwrap();assert_eq!(crate::wire::encode(&records).unwrap(),before);
    }
}
