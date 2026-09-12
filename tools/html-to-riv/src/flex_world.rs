//! Conditional direct-layout world origins and far corners, with each native
//! addition accounted. This consumes independently established size domains.
use super::{flex_descriptor::Group,flex_numeric::ErrorEnvelope,flex_sizes::{Domain,SizeDomains,Unresolved}};
use std::collections::BTreeMap;
#[derive(Clone,Debug)]
pub(crate) struct WorldDomains {pub origins:[Domain;2],pub far_edges:[Domain;2]}
fn add(a:&Domain,b:&Domain)->Domain {a.as_ref().map_err(Clone::clone)?.add(*b.as_ref().map_err(Clone::clone)?).map_err(|_|Unresolved::Arithmetic)}
fn zero()->Domain {Ok(ErrorEnvelope::new(0.,0.,0.).expect("exact zero envelope"))}
fn unknown()->WorldDomains {WorldDomains{origins:[Err(Unresolved::ParentContext),Err(Unresolved::ParentContext)],far_edges:[Err(Unresolved::ParentContext),Err(Unresolved::ParentContext)]}}
/// Root origins are explicit; no missing ancestor is silently replaced by zero.
/// An origin proof is separate from far-edge arithmetic and raster fidelity.
pub(crate) fn propagate(groups:&[Group],sizes:&BTreeMap<u32,SizeDomains>,root:[ErrorEnvelope;2])->BTreeMap<u32,WorldDomains> {
    let root_origins=root.map(Ok);
    let root_edges=std::array::from_fn(|axis|sizes.get(&0).map_or(Err(Unresolved::MissingParent),|d|add(&root_origins[axis],&d.axes[axis])));
    let mut worlds=BTreeMap::from([(0,WorldDomains{origins:root_origins,far_edges:root_edges})]);
    let ordered:BTreeMap<_,_>=groups.iter().map(|g|(g.parent_id,g)).collect();
    for (parent_id,group) in ordered {
        let parent=worlds.get(&parent_id).cloned().unwrap_or_else(unknown);
        let main=usize::from(!group.row);let cross=1-main;
        let expected_flow=if group.row{3}else{1};
        let expected_alignment=if group.logical_reverse{0}else if group.row{2}else{6};
        let mut file:Vec<_>=(0..group.items.len()).collect();if !group.logical_reverse{file.reverse();}
        let ids:Vec<_>=file.iter().map(|&i|group.items[i].native.participant_id).collect();
        let valid=!group.content_owner && group.scene_certificate.is_ok() && group.no_local_constraint_or_origin && group.parent_local_facts.direct_box_defaults() && group.helpers.is_empty() && group.wrappers.is_empty() && group.native_flow==Some(expected_flow) && group.native_alignment==Some(expected_alignment) && ids==group.native_participant_file_order && group.items.iter().all(|i|i.native.parent_id==Some(parent_id) && i.authored_id==i.native.participant_id && i.computed.start_aligned_cross && i.local_facts.as_ref().is_some_and(|f|f.direct_box_defaults()));
        if !valid {for item in &group.items {worlds.insert(item.authored_id,unknown());}continue;}
        let get_size=|i:usize,axis:usize|sizes.get(&group.items[i].authored_id).map_or(Err(Unresolved::MissingParent),|d|d.axes[axis].clone());
        let occupied=file.iter().fold(zero(),|sum,&i|add(&sum,&get_size(i,main)));
        let first=if group.logical_reverse {
            sizes.get(&parent_id).ok_or(Unresolved::MissingParent).and_then(|d|d.axes[main].clone()).and_then(|owner|occupied.as_ref().map_err(Clone::clone).and_then(|used|owner.sub(*used).map_err(|_|Unresolved::Arithmetic)))
        }else{zero()};
        let mut total=zero();
        for (position,&i) in file.iter().rev().enumerate() {
            let item=&group.items[i];let offset=if position==0{first.clone()}else{zero()};
            let local=add(&total,&offset);
            let mut origins=[Err(Unresolved::ParentContext),Err(Unresolved::ParentContext)];
            origins[main]=add(&parent.origins[main],&local);
            origins[cross]=parent.origins[cross].clone(); // zero local cross offset
            let edges=std::array::from_fn(|axis|add(&origins[axis],&get_size(i,axis)));
            worlds.insert(item.authored_id,WorldDomains{origins,far_edges:edges});
            total=add(&total,&add(&offset,&get_size(i,main)));
        }
    }
    worlds
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{compile_profile_with_descriptors,FlexPolicy,flex_sizes};
    use crate::CompileInput;
    #[test]
    fn nested_actual_groups_account_for_parent_translation_and_far_corners() {
        for direction in ["row","row-reverse","column","column-reverse"] {
            let input=CompileInput{html:"<div id=p><div id=a></div><div id=b><div id=c></div></div></div>".into(),css:format!("#p{{width:100px;height:100px;flex-direction:{direction}}}#a{{width:20px;height:20px}}#b{{width:30px;height:30px}}#c{{width:5px;height:5px}}"),width:200.,height:200.};
            let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Candidate).unwrap();
            let size=flex_sizes::propagate(&groups,[ErrorEnvelope::new(200.,200.,0.).unwrap();2]);
            let roots=[ErrorEnvelope::new(1_000_000.,1_000_000.,0.1).unwrap();2];
            let worlds=propagate(&groups,&size,roots);
            let id=|name|output.source_map.iter().find(|n|n.id==name).unwrap().object_id;
            let axis=usize::from(direction.starts_with("column"));
            let offset=if direction.ends_with("reverse"){50.}else{20.};
            let c=&worlds[&id("c")];let origin=c.origins[axis].as_ref().unwrap();
            assert!(origin.lower()<=1_000_000.+offset && origin.upper()>=1_000_000.+offset);
            assert!(origin.error_upper()>0.1);
            assert!(c.far_edges[axis].as_ref().unwrap().error_upper()>origin.error_upper());
            let mut changed=groups;let parent=changed.iter_mut().find(|g|g.parent_id==id("p")).unwrap();parent.native_participant_file_order.reverse();
            assert!(propagate(&changed,&size,roots)[&id("c")].origins[axis].is_err());
        }
    }
}
