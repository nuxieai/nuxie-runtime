//! Closed private composition of a bound base scene and the existing ordinary
//! wrapping emitters. Layout inputs remain intact while rounded paint observes
//! the finalized visible geometry through a separately bound suffix.
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
    Arithmetic(super::wrapping_coordinates::Unresolved),
    Scalar(super::wrapping_scalar::Unresolved),
    Position(super::wrapping_position::Unresolved),
    Masks(super::wrapping_masks::Unresolved),
    PaintBinding(super::wrapping_paint_binding::Unresolved),
    PaintBoxDomains(super::paint_box_domains::Unresolved),
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
    paint_trace: wrapping_paint::Trace,
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
    pub(super) fn paint_trace(&self) -> &wrapping_paint::Trace { &self.paint_trace }
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
/// A source-derived arithmetic analysis attached to the exact candidate it
/// configured. Sizing, paint records and their numeric domains are checked
/// together; actual native/Chrome visual qualification is not implied.
pub(super) struct Derived {
    candidate:Candidate,
    arithmetic:super::wrapping_coordinates::Proof,
    scalar:super::wrapping_scalar::Binding,
    position:super::wrapping_position::Proof,
    masks:super::wrapping_masks::Proof,
    paint:super::wrapping_paint_binding::Binding,
    paint_box_domains:super::paint_box_domains::Proof,
}
impl Derived {
    pub(super) fn candidate(&self)->&Candidate {&self.candidate}
    pub(super) fn arithmetic(&self)->&super::wrapping_coordinates::Proof {&self.arithmetic}
    pub(super) fn scalar(&self)->&super::wrapping_scalar::Binding {&self.scalar}
    pub(super) fn position(&self)->&super::wrapping_position::Proof {&self.position}
    pub(super) fn masks(&self)->&super::wrapping_masks::Proof {&self.masks}
    pub(super) fn paint(&self)->&super::wrapping_paint_binding::Binding {&self.paint}
    pub(super) fn paint_box_domains(&self)->&super::paint_box_domains::Proof {&self.paint_box_domains}
}
pub(super) fn compose_with_bounds(domains:Domains<'_>,alignments:&[f32],record_budget:usize)->Result<Derived,Unresolved> {
    let masks=super::wrapping_masks::prove(&domains).map_err(Unresolved::Masks)?;
    let arithmetic=super::wrapping_coordinates::prove(&domains).map_err(Unresolved::Arithmetic)?;
    let row=domains.base().row;
    let reverse_cross=domains.base().reverse_cross;
    let reverse_main=domains.base().reverse_main;
    let line_fraction=domains.base().line_fraction;
    let roles=domains.base().slots.iter().map(|s|(s.id,s.visible)).collect::<Vec<_>>();
    let position=super::wrapping_position::prove(&arithmetic,domains.slots(),alignments,row,line_fraction,reverse_cross)
        .map_err(Unresolved::Position)?;
    let paint_box_domains=super::paint_box_domains::prove(&arithmetic,&position,domains.slots(),domains.viewport(),row)
        .map_err(Unresolved::PaintBoxDomains)?;
    let base=domains.base().records();
    let candidate=compose(domains,alignments,arithmetic.epsilon(),record_budget)?;
    let (start,sizing,_)=candidate.record_costs();
    let scalar=super::wrapping_scalar::bind(candidate.records(),start,start+sizing,&roles,alignments,
        row,reverse_cross,line_fraction,arithmetic.epsilon(),candidate.trace()).map_err(Unresolved::Scalar)?;
    let paint=super::wrapping_paint_binding::bind(base,candidate.records(),start+sizing,candidate.records().len(),
        &roles,row,reverse_main,reverse_cross,line_fraction,arithmetic.epsilon()).map_err(Unresolved::PaintBinding)?;
    if !paint.matches_trace(candidate.paint_trace()) {
        return Err(Unresolved::PaintBinding(super::wrapping_paint_binding::Unresolved{position:start+sizing}));
    }
    Ok(Derived{candidate,arithmetic,scalar,position,masks,paint,paint_box_domains})
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
    sizing_end: usize,
) -> Result<(), Unresolved> {
    if sizing_end < base.len() || sizing_end > final_records.len() { return Err(Unresolved::BaseChanged); }
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
    let mut origins=BTreeSet::new();
    for (position, record) in final_records.iter().enumerate().skip(base.len()) {
        let object = u32::try_from(position - 1).map_err(|_| Unresolved::ResourceBudget)?;
        let paint_span = position >= sizing_end;
        let span_start = if paint_span { sizing_end } else { base.len() };
        let span_end = if paint_span { final_records.len() } else { sizing_end };
        let in_span = |id:u32| (id as usize).checked_add(1)
            .is_some_and(|index| index >= span_start && index < span_end);
        let allowed = if paint_span {
            matches!(record.kind,"Node"|"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"
                |"Shape"|"Rectangle"|"Fill"|"SolidColor"|"ClippingShape"|"DrawRules"|"DrawTarget")
        } else {
            matches!(record.kind,"Node"|"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"|"ComponentOrigin")
        };
        if !allowed { return Err(Unresolved::GeneratedRole { object }); }
        let owner = parent(record).ok_or(Unresolved::GeneratedRole { object })?;
        let root_attachment = owner == 0 && (record.kind == "Node" || paint_span && record.kind == "Shape");
        let visible_attachment = !paint_span && visible.contains(&owner)
            && matches!(record.kind,"TranslationConstraint"|"ComponentOrigin");
        let expected_owner = match record.kind {
            "Node"|"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"|"Shape" => "Node",
            "Rectangle"|"Fill"|"ClippingShape"|"DrawRules" => "Shape",
            "SolidColor" => "Fill", "DrawTarget" => "DrawRules", _ => "",
        };
        let helper_attachment = in_span(owner) && owner < object && kind(owner)==Some(expected_owner);
        if !(root_attachment || visible_attachment || helper_attachment) {
            return Err(Unresolved::GeneratedRole { object });
        }
        if record.kind=="ComponentOrigin" && (!origins.insert(owner)
            || !record.has_only_properties(&["parentId","originX","originY"])) {
            return Err(Unresolved::GeneratedRole { object });
        }
        // Slots feed sizing, sizing positions visible owners, then paint reads
        // those finalized owners. Neither span may borrow helpers from the other.
        // Exact paint targets/fields are independently checked by its closed reader.
        let valid_reference = match record.kind {
            "TransformConstraint" => reference(record,"targetId").is_some_and(|id|
                (slots.contains(&id) || paint_span && visible.contains(&id)) && id < owner),
            "TranslationConstraint"|"DistanceConstraint" => reference(record,"targetId").is_some_and(|id|
                in_span(id) && kind(id)==Some("Node") && (visible_attachment || id < owner) && id < object),
            "ClippingShape" => reference(record,"sourceId").is_some_and(|id|
                in_span(id) && kind(id)==Some("Shape") && id < object),
            "DrawRules" => reference(record,"drawTargetId").is_some_and(|id|
                in_span(id) && kind(id)==Some("DrawTarget") && final_records.get(id as usize+1).and_then(parent)==Some(object)),
            "DrawTarget" => reference(record,"drawableId").is_some_and(|id|
                in_span(id) && kind(id)==Some("Shape") && id < object),
            _ => true,
        };
        if !valid_reference { return Err(Unresolved::GeneratedRole { object }); }
    }
    if origins!=*visible {return Err(Unresolved::PaintOwnership);}
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
    let paint_trace=wrapping_paint::paint_items(&mut records, &paints, domains.base().row, domains.base().reverse_main,
        domains.base().reverse_cross, domains.base().line_fraction, epsilon)?;
    if records.len() != end { return Err(Unresolved::ResourceBudget); }
    preserved(base, &records, &paints.iter().map(|p|p.paints[0].color).collect::<Vec<_>>(),
        &items.iter().map(|i|i.visible).collect(),base.len()+sizing_records)?;
    Ok(Candidate { records, trace, paint_trace, viewport:domains.viewport(), parent_axes:domains.parent_axes(),
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
    fn source_derived_epsilon_configures_the_actual_owned_graph() {
        for count in [1,2,3,8] {for (direction,alignment,wrap) in [(0,0,1),(1,4,2),(2,8,1),(3,4,2)] {
            let (records,roles)=fixture(count,direction,alignment,wrap);
            let fractions=(0..count).map(|i|(i%3) as f32/2.).collect::<Vec<_>>();
            let derived=compose_with_bounds(domains(&records,&roles),&fractions,100_000).unwrap();
            let epsilon=derived.arithmetic().epsilon();
            assert_eq!(derived.candidate().epsilon(),epsilon);
            assert!((epsilon as f64)>=derived.arithmetic().same_line_error());
            assert!(derived.arithmetic().normalizer().interpolated.lower()>32768.);
            assert_eq!(derived.arithmetic().carry().line_maxima.len(),count);
            let axis=if direction>=2 {"y"}else{"x"};
            let constants=derived.candidate().records().iter().skip(records.len()).filter(|r|
                r.kind=="Node" && matches!(r.get(axis),Some(Value::Float(v)) if *v == -epsilon)).count();
            assert_eq!(constants,(count-1)+count*(count-1)/2);
        }}
        // This bound fixture derives its error budget rather than inheriting
        // the earlier private experiment's fixed1/64 epsilon.
        let (records,roles)=fixture(3,2,0,1);
        let derived=compose_with_bounds(domains(&records,&roles),&[0.;3],100_000).unwrap();
        assert_ne!(derived.arithmetic().epsilon(),1./64.);
    }
    #[test]
    fn derived_analysis_rejects_visible_world_overflow_and_tiny_line_gaps() {
        for overflow in [false,true] {
            let (mut records,roles)=fixture(2,2,if overflow {2}else{0},1);
            let parent=numeric();let mut slot=numeric();
            let value=if overflow {2_f32.powi(126)}else{0.0005};
            let scalar=NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(value).unwrap()));
            if overflow {slot.width=scalar;}else{slot.height=scalar;}
            for &(id,visible) in &roles {
                set(&mut records[id as usize+1],if overflow {"width"}else{"height"},Value::Float(value));
                if overflow {
                    set(&mut records[id as usize+2],"layoutAlignmentType",Value::Uint(2));
                    set(&mut records[visible as usize+1],"width",Value::Float(f32::MAX));
                }
            }
            let values=roles.iter().map(|&(id,_)|(id,&slot)).collect::<Vec<_>>();
            let d=wrapping_domains::resolve(&records,2,&roles,&parent,&values,
                [MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
            let result=compose_with_bounds(d,&[0.,1.],100_000);
            if overflow {
                assert!(matches!(result,Err(Unresolved::Arithmetic(super::super::wrapping_coordinates::Unresolved::Arithmetic))));
            }else{
                assert!(matches!(result,Err(Unresolved::Arithmetic(super::super::wrapping_coordinates::Unresolved::Separation))
                    |Err(Unresolved::Arithmetic(super::super::wrapping_coordinates::Unresolved::Normalizer(_)))));
            }
        }
    }
    #[test]
    fn ordinary_artboard_background_is_preserved_without_replication() {
        let (mut records,roles)=fixture(3,2,4,1);
        let id=records.len() as u32-1;
        let mut fill=Record::new("Fill");set(&mut fill,"parentId",Value::Uint(0));
        let mut color=Record::new("SolidColor");set(&mut color,"parentId",Value::Uint(id));
        set(&mut color,"colorValue",Value::Color(0xffffffff));records.extend([fill,color]);
        let derived=compose_with_bounds(domains(&records,&roles),&[0.,0.5,1.],100_000).unwrap();
        assert_eq!(wire::encode(&derived.candidate().records()[id as usize+1..id as usize+3]).unwrap(),
            wire::encode(&records[id as usize+1..id as usize+3]).unwrap());
        assert_eq!(derived.paint().replicas,6);
        assert_eq!(derived.candidate().records().iter().filter(|r|r.kind=="Fill"
            && matches!(r.get("parentId"),Some(Value::Uint(0)))).count(),1);
        let duplicate_id=records.len() as u32-1;
        let extra=records[id as usize+1].clone();
        let mut extra_color=records[id as usize+2].clone();
        set(&mut extra_color,"parentId",Value::Uint(duplicate_id));
        records.extend([extra,extra_color]);
        let numeric=numeric();
        assert!(wrapping_domains::resolve(&records,2,&roles,&numeric,
            &roles.iter().map(|&(id,_)|(id,&numeric)).collect::<Vec<_>>(),
            [MachineInterval::new(0.,16384.).unwrap();2]).is_err());
    }
    #[test]
    fn mask_domain_covers_initial_file_and_every_declared_resize() {
        for axis in 0..2 {
            let (mut records,roles)=fixture(2,2,0,1);
            let n=numeric();let values=roles.iter().map(|&(id,_)|(id,&n)).collect::<Vec<_>>();
            let mut viewport=[MachineInterval::new(0.,16384.).unwrap();2];
            viewport[axis]=MachineInterval::new(0.,32768_f32.next_up()).unwrap();
            let d=wrapping_domains::resolve(&records,2,&roles,&n,&values,viewport).unwrap();
            assert!(matches!(compose_with_bounds(d,&[0.,1.],100_000),
                Err(Unresolved::Masks(super::super::wrapping_masks::Unresolved::Viewport{axis:a})) if a==axis));
            for value in [16385.,-1.,f32::INFINITY,f32::NAN] {
                set(&mut records[1],if axis==0 {"width"}else{"height"},Value::Float(value));
                let d=domains(&records,&roles);
                assert!(matches!(compose_with_bounds(d,&[0.,1.],100_000),
                    Err(Unresolved::Masks(super::super::wrapping_masks::Unresolved::InitialViewport{axis:a})) if a==axis));
            }
            set(&mut records[1],if axis==0 {"width"}else{"height"},Value::Float(16384.));
            let d=compose_with_bounds(domains(&records,&roles),&[0.,1.],100_000).unwrap();
            assert_eq!(d.masks().active().min,[0.,0.]);
            assert_eq!(d.masks().viewport()[axis].upper(),16384.);
        }
    }
    #[test]
    fn sizing_binding_rejects_fields_operands_and_trace_mutations() {
        use super::super::wrapping_scalar;
        for row in [false,true] {
            let (records,roles)=fixture(3,if row {2}else{0},4,1);
            let candidate=compose(domains(&records,&roles),&[0.,0.5,1.],0.25,100_000).unwrap();
            let (start,size,_)=candidate.record_costs();
            let bind=|changed:&[Record],trace:&wrapping::Trace| wrapping_scalar::bind(changed,start,start+size,
                &roles,&[0.,0.5,1.],row,false,0.5,0.25,trace);
            let binding=bind(candidate.records(),candidate.trace()).unwrap();
            assert_eq!(binding.nodes+binding.constraints+binding.origins,size);
            // Each mutation breaks a premise actually used by the scalar
            // ledger: identity, strength, exact mode, origin or operand.
            for (kind,key,value) in [
                ("Node","scaleX",Value::Float(2.)),
                ("Node",if row {"x"}else{"y"},Value::Float(1.)),
                ("TranslationConstraint","strength",Value::Float(0.5)),
                ("TranslationConstraint","sourceSpaceValue",Value::Uint(1)),
                ("TranslationConstraint","offset",Value::Bool(true)),
                ("TranslationConstraint","targetId",Value::Uint(candidate.trace().heights[2])),
                ("TransformConstraint",if row {"originX"}else{"originY"},Value::Float(0.5)),
                ("ComponentOrigin",if row {"originY"}else{"originX"},Value::Float(0.5)),
                ("DistanceConstraint","modeValue",Value::Uint(0)),
                ("DistanceConstraint","distance",Value::Float(65535.)),
                ("DistanceConstraint","targetId",Value::Uint(candidate.trace().tops[0])),
            ] {
                let mut changed=candidate.records().to_vec();
                let r=changed[start..start+size].iter_mut().find(|r|r.kind==kind).unwrap();
                set(r,key,value);assert!(bind(&changed,candidate.trace()).is_err(),"{kind}.{key}");
            }
            let mut reordered=candidate.records().to_vec();
            let normalized=candidate.trace().boundaries[0].normalized;
            let constraints=reordered.iter().enumerate().filter(|(_,r)|
                matches!(r.get("parentId"),Some(Value::Uint(p)) if *p==normalized)).map(|(i,_)|i).collect::<Vec<_>>();
            assert_eq!(constraints.len(),2);reordered.swap(constraints[0],constraints[1]);
            assert!(bind(&reordered,candidate.trace()).is_err());
            let mut changed=compose(domains(&records,&roles),&[0.,0.5,1.],0.25,100_000).unwrap();
            changed.trace.targets.swap(0,1);
            assert!(bind(candidate.records(),changed.trace()).is_err());
        }
    }
    #[test]
    fn failures_are_atomic_and_budget_is_exact() {
        let (records,roles)=fixture(3,2,0,1);let original=wire::encode(&records).unwrap();
        let candidate=compose(domains(&records,&roles),&[0.,0.5,1.],0.25,100_000).unwrap();
        assert_eq!(candidate.record_costs(),(23,141,7073));
        let exact=compose(domains(&records,&roles),&[0.,0.5,1.],0.25,7237).unwrap();
        assert_eq!(exact.records().len(),7237);
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5,1.],0.25,candidate.records().len()-1),Err(Unresolved::ResourceBudget)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5],0.25,100_000),Err(Unresolved::Alignment)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.25,1.],0.25,100_000),Err(Unresolved::Alignment)));
        assert!(matches!(compose(domains(&records,&roles),&[0.,0.5,1.],f32::NAN,100_000),Err(Unresolved::Epsilon)));
        assert_eq!(wire::encode(&records).unwrap(),original);
    }
    #[test]
    fn preservation_rejects_layout_mutation_and_unexpected_base_attachments() {
        let (records,roles)=fixture(2,2,0,1);
        let candidate=compose(domains(&records,&roles),&[0.,1.],0.25,100_000).unwrap();
        let colors=records.iter().enumerate().filter(|(_,r)|r.kind=="SolidColor").map(|(i,_)|i as u32-1).collect::<Vec<_>>();
        let visible=roles.iter().map(|&(_,v)|v).collect();
        let mut changed=candidate.records().to_vec();set(&mut changed[5],"width",Value::Float(61.));
        assert!(matches!(preserved(&records,&changed,&colors,&visible,records.len()+candidate.record_costs().1),Err(Unresolved::BaseChanged)));
        for (kind,owner) in [("LayoutComponent",0),("LayoutParticipant",roles[0].1),("TranslationConstraint",roles[0].0),("Node",roles[0].1),("ForegroundLayoutDrawable",roles[0].0)] {
            let mut changed=candidate.records().to_vec();let mut extra=Record::new(kind);set(&mut extra,"parentId",Value::Uint(owner));changed.push(extra);
            assert!(matches!(preserved(&records,&changed,&colors,&visible,records.len()+candidate.record_costs().1),Err(Unresolved::GeneratedRole{..})),"{kind}");
        }
        for target in [roles[0].1,u32::MAX] {
            let mut changed=candidate.records().to_vec();
            let constraint=changed.iter_mut().skip(records.len()).find(|r|r.kind=="TransformConstraint").unwrap();
            set(constraint,"targetId",Value::Uint(target));
            assert!(matches!(preserved(&records,&changed,&colors,&visible,records.len()+candidate.record_costs().1),Err(Unresolved::GeneratedRole{..})));
        }
    }
    #[test]
    fn paint_reads_final_visible_owners_without_feedback_into_sizing() {
        let (records,roles)=fixture(2,2,4,1);
        let candidate=compose(domains(&records,&roles),&[0.,1.],0.25,100_000).unwrap();
        let colors=records.iter().enumerate().filter(|(_,r)|r.kind=="SolidColor")
            .map(|(i,_)|i as u32-1).collect::<Vec<_>>();
        let visible=roles.iter().map(|&(_,v)|v).collect();
        let boundary=records.len()+candidate.record_costs().1;
        let validate=|changed:&[Record]|preserved(&records,changed,&colors,&visible,boundary);
        validate(candidate.records()).unwrap();
        let paint_node=boundary as u32-1;
        assert_eq!(candidate.records()[boundary].kind,"Node");
        let paint_corner=candidate.records().iter().enumerate().skip(boundary).find(|(_,r)|
            r.kind=="TransformConstraint" && reference(r,"targetId")==Some(roles[0].1)).unwrap().0;
        assert!(paint_corner>=boundary);
        // A final visible-position constraint may never target a paint helper,
        // even if the helper is an otherwise legal Node.
        let mut changed=candidate.records().to_vec();
        let position=changed[records.len()..boundary].iter_mut().find(|r|
            r.kind=="TranslationConstraint" && parent(r)==Some(roles[0].1)).unwrap();
        set(position,"targetId",Value::Uint(paint_node));assert!(validate(&changed).is_err());
        // Paint must read the visible owner through its bound corner target,
        // not attach below a sizing helper or borrow its scalar values.
        for key in ["parentId","targetId"] {
            let mut changed=candidate.records().to_vec();
            let constraint=changed[boundary..].iter_mut().find(|r|r.kind=="TranslationConstraint").unwrap();
            set(constraint,key,Value::Uint(candidate.trace().anchors[0]));
            assert!(validate(&changed).is_err(),"paint {key} crossed sizing span");
        }
        let mut changed=candidate.records().to_vec();
        set(&mut changed[paint_corner],"targetId",Value::Uint(2)); // enclosing layout parent
        assert!(validate(&changed).is_err());
        let mut changed=candidate.records().to_vec();
        let mut extra=Record::new("ComponentOrigin");set(&mut extra,"parentId",Value::Uint(roles[0].1));changed.push(extra);
        assert!(validate(&changed).is_err());
    }
    #[test]
    fn rounded_paint_domain_is_a_required_derived_premise() {
        let (records,roles)=fixture(2,2,0,1);
        let n=numeric();let values=roles.iter().map(|&(id,_)|(id,&n)).collect::<Vec<_>>();
        for axis in 0..2 {
            let mut viewport=[MachineInterval::new(0.,16384.).unwrap();2];
            viewport[axis]=MachineInterval::new(0.,16384_f32.next_up()).unwrap();
            let d=wrapping_domains::resolve(&records,2,&roles,&n,&values,viewport).unwrap();
            // Existing line masks cover this domain, but rounded edge masks
            // have the stricter independently declared viewport certificate.
            assert!(matches!(compose_with_bounds(d,&[0.,1.],100_000),Err(Unresolved::PaintBoxDomains(_))));
        }
    }

    #[test]
    fn owned_paint_trace_matches_independently_consumed_records() {
        let (records,roles)=fixture(2,2,4,1);
        let mut candidate=compose(domains(&records,&roles),&[0.,1.],0.25,100_000).unwrap();
        let (start,sizing,_)=candidate.record_costs();
        let binding=super::super::wrapping_paint_binding::bind(&records,candidate.records(),start+sizing,
            candidate.records().len(),&roles,true,false,false,0.5,0.25).unwrap();
        assert!(binding.matches_trace(candidate.paint_trace()));
        candidate.paint_trace.boxes.swap(0,1);
        assert!(!binding.matches_trace(candidate.paint_trace()));
        candidate.paint_trace.boxes.swap(0,1);
        candidate.paint_trace.boxes[0].axes[0].rounding[0].positive[0].stages[0]+=1;
        assert!(!binding.matches_trace(candidate.paint_trace()));
    }

}
