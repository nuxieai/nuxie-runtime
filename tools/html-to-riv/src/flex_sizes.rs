//! Conditional size propagation for direct, nonflexing definite boxes. World
//! positions and flexible targets remain separate; this is not CSS admission.
use super::{computed_provenance::NumericSize,flex_descriptor::{Group,Item,RecordLength},flex_numeric::ErrorEnvelope};
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Unresolved { MissingParent, ParentContext, Topology, LocalOverrides, FlexibleTarget, Bounds, NativeBinding, IntrinsicSize, Metadata, Arithmetic }
pub(crate) type Domain=Result<ErrorEnvelope,Unresolved>;
#[derive(Clone,Debug)]
pub(crate) struct SizeDomains {pub axes:[Domain;2]}
fn all_error(reason:Unresolved)->SizeDomains {SizeDomains{axes:[Err(reason.clone()),Err(reason)]}}
fn zero(value:&NumericSize)->bool {matches!(value,NumericSize::Pixels(Ok(v)) if v.is_exact_zero() && v.native()==0.)}
fn min_binding(record:&RecordLength)->bool {
    // Undefined minimum is automatic for Fixed. In the admitted no-inset,
    // definite, nonflexing profile its min-content result is capped by the
    // preferred size (pinned flexbox.rs:817-840), so cannot enlarge that target.
    (record.units.is_none() && record.value.is_none()) || (record.units==Some(1) && record.value==Some(0.))
}
fn absent(record:&RecordLength)->bool {record.units.is_none() && record.value.is_none()}
fn resolve(value:&NumericSize,raw:&RecordLength,owner:&Domain)->Domain {
    let (scalar,units)=match value {NumericSize::Pixels(Ok(v))=>(v,1),NumericSize::Percent(Ok(v))=>(v,2),NumericSize::Auto=>return Err(Unresolved::IntrinsicSize),_=>return Err(Unresolved::Metadata)};
    if !scalar.is_nonnegative() {return Err(Unresolved::Metadata);}
    if raw.units!=Some(units) || !raw.value.is_some_and(|v|v.to_bits()==scalar.native().to_bits()) {return Err(Unresolved::NativeBinding);}
    if units==1 {ErrorEnvelope::new(scalar.ideal_bounds().lower(),scalar.ideal_bounds().upper(),scalar.absolute_error_upper()).map_err(|_|Unresolved::Arithmetic)}
    else {owner.as_ref().map_err(Clone::clone)?.preferred_percent(scalar).map_err(|_|Unresolved::Arithmetic)}
}
fn child(item:&Item,parent:&SizeDomains,row:bool,parent_id:u32)->SizeDomains {
    if item.authored_id!=item.native.participant_id || item.native.parent_id!=Some(parent_id) {return all_error(Unresolved::Topology);}
    if !item.local_facts.as_ref().is_some_and(|f|f.direct_box_defaults()) {return all_error(Unresolved::LocalOverrides);}
    let n=&item.computed.numeric;let raw=&item.native;
    if !n.grow.as_ref().is_ok_and(|v|v.is_exact_zero()) || !n.shrink.as_ref().is_ok_and(|v|v.is_exact_zero()) || raw.linked_grow!=Some(0.) || raw.linked_shrink!=Some(0.) || raw.main_scale!=Some(0) || raw.cross_scale!=Some(0) {return all_error(Unresolved::FlexibleTarget);}
    if !zero(&n.minimum) || !zero(&n.cross_minimum) || !matches!(n.maximum,NumericSize::Auto) || !matches!(n.cross_maximum,NumericSize::Auto) || !min_binding(&raw.minimum) || !min_binding(&raw.cross_minimum) || !absent(&raw.maximum) || !absent(&raw.cross_maximum) {return all_error(Unresolved::Bounds);}
    let main=usize::from(!row);let cross=1-main;
    let mut axes=[Err(Unresolved::Metadata),Err(Unresolved::Metadata)];
    axes[main]=resolve(&n.main,&raw.main,&parent.axes[main]);
    axes[cross]=resolve(&n.cross,&raw.cross,&parent.axes[cross]);
    SizeDomains{axes}
}
/// Caller supplies the whole viewport domain and its conversion error explicitly.
/// Groups may arrive child-first. Compiler object IDs place each parent before
/// descendants; inconsistent/missing ownership remains unresolved.
pub(crate) fn propagate(groups:&[Group],viewport:[ErrorEnvelope;2])->BTreeMap<u32,SizeDomains> {
    let mut result=BTreeMap::from([(0,SizeDomains{axes:viewport.map(Ok)})]);
    let ordered:BTreeMap<_,_>=groups.iter().map(|g|(g.parent_id,g)).collect();
    for (parent_id,group) in ordered {
        let parent=result.get(&parent_id).cloned().unwrap_or_else(||all_error(Unresolved::MissingParent));
        let valid=group.no_local_constraint_or_origin && group.parent_local_facts.direct_box_defaults() && group.helpers.is_empty() && group.wrappers.is_empty();
        for item in &group.items {
            let sizes=if item.authored_id<=parent_id {all_error(Unresolved::Topology)}
                else if !valid {all_error(Unresolved::ParentContext)}
                else {child(item,&parent,group.row,parent_id)};
            result.insert(item.authored_id,sizes);
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{compile_profile_with_descriptors,FlexPolicy};
    use crate::CompileInput;
    #[test]
    fn final_cross_bounds_and_constraints_cannot_be_hidden_by_computed_style() {
        let input=CompileInput{html:"<div id=p><div id=a></div></div>".into(),css:"#p{width:100px;height:100px}#a{width:20px;height:30px}".into(),width:400.,height:200.};
        let (output,mut groups)=compile_profile_with_descriptors(&input,FlexPolicy::Candidate).unwrap();
        let id=output.source_map.iter().find(|n|n.id=="a").unwrap().object_id;
        let viewport=[ErrorEnvelope::new(0.,16384.,0.001).unwrap();2];
        assert!(propagate(&groups,viewport)[&id].axes.iter().all(Result::is_ok));
        let group=groups.iter_mut().find(|g|g.items.iter().any(|i|i.authored_id==id)).unwrap();
        group.items[0].native.cross_minimum.units=Some(3);
        assert!(matches!(propagate(&groups,viewport)[&id].axes[0],Err(Unresolved::Bounds)));
        let group=groups.iter_mut().find(|g|g.items.iter().any(|i|i.authored_id==id)).unwrap();
        group.items[0].native.cross_minimum.units=None;group.no_local_constraint_or_origin=false;
        assert!(matches!(propagate(&groups,viewport)[&id].axes[0],Err(Unresolved::ParentContext)));
    }
    #[test]
    fn real_child_first_groups_propagate_percentages_and_keep_flexible_targets_unknown() {
        let input=CompileInput{html:"<div id=p><div id=a><div id=b></div></div></div>".into(),css:"#p{width:50%;height:100px}#a{width:33.333333333%;height:20px}#b{width:2px;height:3px}".into(),width:400.,height:200.};
        let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Candidate).unwrap();
        let viewport=[ErrorEnvelope::new(0.,16384.,0.001).unwrap();2];
        let domains=propagate(&groups,viewport);
        let id=|name|output.source_map.iter().find(|n|n.id==name).unwrap().object_id;
        let a=domains[&id("a")].axes[0].as_ref().unwrap();assert!(a.upper()>=16384.*0.5*0.33333333333);assert!(a.error_upper()>0.001);
        assert_eq!(domains[&id("b")].axes[0].as_ref().unwrap().lower(),2.);
        let changed=CompileInput{css:format!("{}#p{{flex-direction:row}}#a{{width:auto;flex:1 1 0px}}",input.css),..input};
        let (_,groups)=compile_profile_with_descriptors(&changed,FlexPolicy::Candidate).unwrap();
        assert!(propagate(&groups,viewport).values().any(|v|v.axes.iter().any(|d|matches!(d,Err(Unresolved::FlexibleTarget)))));
    }
}
