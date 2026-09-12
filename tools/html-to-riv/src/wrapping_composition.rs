//! Closed private composition of a bound base scene and the existing ordinary
//! wrapping emitters. This preserves layout inputs, not a full gate certificate.
//! No public compiler path consumes this experimental candidate yet.
use super::{wrapping, wrapping_domains::{Domains, Slot}, wrapping_paint,
    wrapping_sizes::MachineInterval};
use crate::{Diagnostic, wire::{self, Record, Value}};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(super) enum Unresolved {
    Alignment,
    Epsilon,
    ResourceBudget,
    PaintOwnership,
    BaseChanged,
    GeneratedRole { object: u32 },
    Emission(Diagnostic),
}
impl From<Diagnostic> for Unresolved {
    fn from(value: Diagnostic) -> Self { Self::Emission(value) }
}

/// Owned records and their checked inputs stay together. There is no mutable
/// record accessor and no public export method: future gate/carry/mask proofs
/// must consume this candidate before it can become admitted output.
pub(super) struct Candidate {
    records: Vec<Record>,
    trace: wrapping::Trace,
    viewport: [MachineInterval; 2],
    parent_axes: [MachineInterval; 2],
    slots: Vec<Slot>,
    cross_extents: MachineInterval,
    epsilon: f32,
    alignments: Vec<f32>,
    base_records: usize,
    sizing_records: usize,
    paint_records: usize,
}
impl Candidate {
    pub(super) fn records(&self) -> &[Record] { &self.records }
    pub(super) fn trace(&self) -> &wrapping::Trace { &self.trace }
    pub(super) fn slots(&self) -> &[Slot] { &self.slots }
    pub(super) fn parent_axes(&self) -> [MachineInterval; 2] { self.parent_axes }
    pub(super) fn viewport(&self) -> [MachineInterval; 2] { self.viewport }
    pub(super) fn cross_extents(&self) -> MachineInterval { self.cross_extents }
    pub(super) fn epsilon(&self) -> f32 { self.epsilon }
    pub(super) fn alignments(&self) -> &[f32] { &self.alignments }
    pub(super) fn record_costs(&self) -> (usize, usize, usize) {
        (self.base_records, self.sizing_records, self.paint_records)
    }
}
fn parent(record: &Record) -> Option<u32> {
    match record.get("parentId") { Some(Value::Uint(id)) => Some(*id), _ => None }
}
fn reference(record: &Record, key: &str) -> Option<u32> {
    match record.get(key) { Some(Value::Uint(id)) => Some(*id), _ => None }
}

/// Check independently of the emitter that the base prefix changed only at the
/// original colors deliberately hidden by paint replication. No new layout
/// components/styles may affect the measured tree, and only the expected
/// positioning constraints/paint replicas may attach to an existing visible
/// owner. The helper graph's arithmetic/topology proof is a separate obligation.
fn preserved(
    base: &[Record], final_records: &[Record], colors: &[u32], visible: &BTreeSet<u32>,
) -> Result<(), Unresolved> {
    let Some(prefix) = final_records.get(..base.len()) else { return Err(Unresolved::BaseChanged); };
    let mut expected = base.to_vec();
    for &id in colors {
        let record = expected.get_mut(id as usize + 1).ok_or(Unresolved::PaintOwnership)?;
        if record.kind != "SolidColor" { return Err(Unresolved::PaintOwnership); }
        record.set("colorValue", Value::Color(0))?;
    }
    if wire::encode(prefix)? != wire::encode(&expected)? { return Err(Unresolved::BaseChanged); }
    let slots = visible.iter().filter_map(|id| base.get(*id as usize+1).and_then(parent)).collect::<BTreeSet<_>>();
    let kind = |id:u32| final_records.get(id as usize+1).map(|r|r.kind);
    for (position, record) in final_records.iter().enumerate().skip(base.len()) {
        let object = u32::try_from(position - 1).map_err(|_| Unresolved::ResourceBudget)?;
        if !matches!(record.kind, "Node" | "TranslationConstraint" | "TransformConstraint"
            | "DistanceConstraint" | "Shape" | "Rectangle" | "ForegroundLayoutDrawable"
            | "Fill" | "SolidColor" | "ClippingShape" | "DrawRules" | "DrawTarget") {
            return Err(Unresolved::GeneratedRole { object });
        }
        let owner = parent(record).ok_or(Unresolved::GeneratedRole { object })?;
        let root_node = owner == 0 && record.kind == "Node";
        let visible_attachment = visible.contains(&owner)
            && matches!(record.kind, "ForegroundLayoutDrawable" | "TranslationConstraint");
        let expected_owner = match record.kind {
            "Node" | "TranslationConstraint" | "TransformConstraint" | "DistanceConstraint" | "Shape" => "Node",
            "Rectangle" => "Shape",
            "Fill" | "ClippingShape" | "DrawRules" => "ForegroundLayoutDrawable",
            "SolidColor" => "Fill",
            "DrawTarget" => "DrawRules",
            _ => "", // Foreground replicas must attach directly to bound visible owners.
        };
        let helper_attachment = owner as usize >= base.len() - 1 && owner < object
            && kind(owner)==Some(expected_owner);
        if !(root_node || visible_attachment || helper_attachment) {
            return Err(Unresolved::GeneratedRole { object });
        }
        // Measurements read only independent slots. Scalar helpers read older
        // helper Nodes, whose parent chains end at the artboard. They cannot
        // feed the transformed visible owners back into their own computation.
        let valid_reference = match record.kind {
            "TransformConstraint" => reference(record,"targetId").is_some_and(|id|
                slots.contains(&id) && id < owner),
            "TranslationConstraint" | "DistanceConstraint" => reference(record,"targetId").is_some_and(|id|
                id as usize >= base.len()-1 && kind(id)==Some("Node")
                    && (visible_attachment || id < owner) && id < object),
            "ClippingShape" => reference(record,"sourceId").is_some_and(|id|
                id as usize >= base.len()-1 && kind(id)==Some("Shape") && id < object),
            "DrawRules" => reference(record,"drawTargetId").is_some_and(|id|
                kind(id)==Some("DrawTarget") && final_records.get(id as usize+1).and_then(parent)==Some(object)),
            "DrawTarget" => reference(record,"drawableId").is_some_and(|id|
                id as usize >= base.len()-1 && kind(id)==Some("ForegroundLayoutDrawable") && id < object),
            _ => true,
        };
        if !valid_reference { return Err(Unresolved::GeneratedRole { object }); }
    }
    Ok(())
}

/// `epsilon` is still an explicit experimental input, never a default or proof
/// of admissibility. Alignment fractions correspond to the bound logical slot
/// order. Record expansion is checked against an explicit budget before clone
/// or emission. Failed construction never mutates the caller's base records.
pub(super) fn compose(
    domains: Domains<'_>, alignments: &[f32], epsilon: f32, record_budget: usize,
) -> Result<Candidate, Unresolved> {
    if alignments.len() != domains.base().slots.len()
        || alignments.iter().any(|v| ![0., 0.5, 1.].contains(v)) {
        return Err(Unresolved::Alignment);
    }
    if !epsilon.is_finite() || !(0. ..65536.).contains(&epsilon) { return Err(Unresolved::Epsilon); }
    let base = domains.base().records();
    let count = domains.base().slots.len();
    let sizing_records = wrapping::alignment_records(count).ok_or(Unresolved::ResourceBudget)?;
    let paint_records = wrapping_paint::paint_records(&vec![1;count]).ok_or(Unresolved::ResourceBudget)?;
    let end = base.len().checked_add(sizing_records).and_then(|v| v.checked_add(paint_records))
        .filter(|v| *v <= record_budget && *v <= u32::MAX as usize).ok_or(Unresolved::ResourceBudget)?;
    let mut fills = BTreeMap::new();
    let mut colors = BTreeMap::new();
    for (position, record) in base.iter().enumerate().skip(1) {
        let id = u32::try_from(position - 1).map_err(|_| Unresolved::ResourceBudget)?;
        if record.kind == "Fill" { fills.insert(parent(record).ok_or(Unresolved::PaintOwnership)?, id); }
        if record.kind == "SolidColor" { colors.insert(parent(record).ok_or(Unresolved::PaintOwnership)?, id); }
    }
    let paints = domains.base().slots.iter().map(|slot| {
        let fill = *fills.get(&slot.visible).ok_or(Unresolved::PaintOwnership)?;
        let color = *colors.get(&fill).ok_or(Unresolved::PaintOwnership)?;
        Ok(wrapping_paint::Item { slot:slot.id, paints:vec![wrapping_paint::Paint { geometry:slot.visible, fill, color }] })
    }).collect::<Result<Vec<_>,Unresolved>>()?;
    let items = domains.base().slots.iter().zip(alignments).map(|(slot, &alignment)|
        wrapping::Item { slot:slot.id, visible:slot.visible, alignment }).collect::<Vec<_>>();
    let mut records = base.to_vec();
    let trace = wrapping::align_items(&mut records, &items, domains.base().row,
        domains.base().reverse_cross, domains.base().line_fraction, epsilon)?;
    wrapping_paint::paint_items(&mut records, &paints, domains.base().row, domains.base().reverse_main,
        domains.base().reverse_cross, domains.base().line_fraction, epsilon)?;
    if records.len() != end { return Err(Unresolved::ResourceBudget); }
    preserved(base, &records, &paints.iter().map(|p|p.paints[0].color).collect::<Vec<_>>(),
        &items.iter().map(|i|i.visible).collect())?;
    Ok(Candidate { records, trace, viewport:domains.viewport(), parent_axes:domains.parent_axes(),
        slots:domains.slots().to_vec(), cross_extents:domains.cross_extents(), epsilon, alignments:alignments.to_vec(),
        base_records:base.len(), sizing_records, paint_records })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{computed_provenance::{NumericStyle, NumericSize},
        scalar_provenance::ScalarProvenance, wrapping_domains};
    fn set(r:&mut Record,key:&str,v:Value) { r.set(key,v).unwrap(); }
    fn fixture(count:usize, direction:u32, alignment:u32, wrap:u32) -> (Vec<Record>,Vec<(u32,u32)>) {
        let mut records=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];
        set(&mut records[1],"styleId",Value::Uint(1));
        fn node(records:&mut Vec<Record>,owner:u32)->u32 {
            let id=records.len() as u32-1; let mut node=Record::new("LayoutComponent");
            set(&mut node,"parentId",Value::Uint(owner));set(&mut node,"styleId",Value::Uint(id+1));
            set(&mut node,"width",Value::Float(60.));set(&mut node,"height",Value::Float(30.));
            let mut style=Record::new("LayoutComponentStyle");
            for key in ["widthUnitsValue","heightUnitsValue","minWidthUnitsValue","minHeightUnitsValue"] {set(&mut style,key,Value::Uint(1));}
            for key in ["minWidth","minHeight"] {set(&mut style,key,Value::Float(0.));}
            records.extend([node,style]);id
        }
        let parent=node(&mut records,0);
        set(&mut records[4],"flexDirectionValue",Value::Uint(direction));
        set(&mut records[4],"layoutAlignmentType",Value::Uint(alignment));
        set(&mut records[4],"flexWrapValue",Value::Uint(wrap));
        let mut roles=vec![];
        for _ in 0..count {
            let slot=node(&mut records,parent);let visible=node(&mut records,slot);roles.push((slot,visible));
            let fill_id=records.len() as u32-1;
            let mut fill=Record::new("Fill");set(&mut fill,"parentId",Value::Uint(visible));
            let mut color=Record::new("SolidColor");set(&mut color,"parentId",Value::Uint(fill_id));
            set(&mut color,"colorValue",Value::Color(0xff5678ab));records.extend([fill,color]);
        }
        (records,roles)
    }
    fn numeric()->NumericStyle {
        NumericStyle{width:NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(60.).unwrap())),
            height:NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(30.).unwrap())),..NumericStyle::default()}
    }
    fn domains<'a>(records:&'a[Record],roles:&[(u32,u32)])->Domains<'a> {
        let n=numeric();let values=roles.iter().map(|&(id,_)|(id,&n)).collect::<Vec<_>>();
        wrapping_domains::resolve(records,2,roles,&n,&values,[MachineInterval::new(0.,16384.).unwrap();2]).unwrap()
    }
    #[test]
    fn variable_count_composition_keeps_bound_inputs_and_reproduces_ordinary_bytes() {
        for count in [1,2,3,8] { for (direction,alignment,wrap) in [(0,0,1),(1,4,2),(2,8,1),(3,4,2)] {
            let (records,roles)=fixture(count,direction,alignment,wrap);let original=wire::encode(&records).unwrap();
            let alignments=(0..count).map(|i|(i%3) as f32/2.).collect::<Vec<_>>();
            let candidate=compose(domains(&records,&roles),&alignments,0.25,100_000).unwrap();
            let costs=candidate.record_costs();
            assert_eq!(candidate.records().len(),costs.0+costs.1+costs.2);
            assert_eq!(candidate.slots().len(),count);assert_eq!(candidate.trace().anchors.len(),count);
            assert_eq!(candidate.alignments(),alignments);assert_eq!(candidate.epsilon(),0.25);
            let again=compose(domains(&records,&roles),&alignments,0.25,candidate.records().len()).unwrap();
            assert_eq!(wire::encode(candidate.records()).unwrap(),wire::encode(again.records()).unwrap());
            assert_eq!(wire::encode(&records).unwrap(),original);
        } }
    }
    #[test]
    fn failures_are_atomic_and_budget_is_exact() {
        let (records,roles)=fixture(3,2,0,1);let original=wire::encode(&records).unwrap();
        let candidate=compose(domains(&records,&roles),&[0.,0.5,1.],0.25,1000).unwrap();
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5,1.],0.25,candidate.records().len()-1),Err(Unresolved::ResourceBudget)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5],0.25,1000),Err(Unresolved::Alignment)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.25,1.],0.25,1000),Err(Unresolved::Alignment)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5,1.],f32::NAN,1000),Err(Unresolved::Epsilon)));
        assert_eq!(wire::encode(&records).unwrap(),original);
    }
    #[test]
    fn preservation_rejects_layout_mutation_and_unexpected_base_attachments() {
        let (records,roles)=fixture(2,2,0,1);
        let candidate=compose(domains(&records,&roles),&[0.,1.],0.25,1000).unwrap();
        let colors=records.iter().enumerate().filter(|(_,r)|r.kind=="SolidColor").map(|(i,_)|i as u32-1).collect::<Vec<_>>();
        let visible=roles.iter().map(|&(_,v)|v).collect();
        let mut changed=candidate.records().to_vec();set(&mut changed[5],"width",Value::Float(61.));
        assert!(matches!(preserved(&records,&changed,&colors,&visible),Err(Unresolved::BaseChanged)));
        for (kind,owner) in [("LayoutComponent",0),("LayoutParticipant",roles[0].1),("TranslationConstraint",roles[0].0),("Node",roles[0].1),("ForegroundLayoutDrawable",roles[0].0)] {
            let mut changed=candidate.records().to_vec();let mut extra=Record::new(kind);set(&mut extra,"parentId",Value::Uint(owner));changed.push(extra);
            assert!(matches!(preserved(&records,&changed,&colors,&visible),Err(Unresolved::GeneratedRole{..})),"{kind}");
        }
        for target in [roles[0].1,u32::MAX] {
            let mut changed=candidate.records().to_vec();
            let constraint=changed.iter_mut().skip(records.len()).find(|r|r.kind=="TransformConstraint").unwrap();
            set(constraint,"targetId",Value::Uint(target));
            assert!(matches!(preserved(&records,&changed,&colors,&visible),Err(Unresolved::GeneratedRole{..})));
        }
    }
}
