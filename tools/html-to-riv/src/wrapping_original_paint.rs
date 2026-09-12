//! Independent ownership binding for retaining the original integral paint.
//! Numeric nonoverlap and sizing-constraint field proofs are separate premises.
use crate::wire::{self,Record,Value};
use super::{wrapping_integral,wrapping_slots};
use std::collections::BTreeSet;

#[derive(Debug,PartialEq,Eq)]
pub(super) enum Unresolved {
    Span, Roles, Proof, Structure(wrapping_slots::Unresolved), Prefix, Encoding,
    Paint { owner:u32 }, SizingRecord { position:usize },
}
#[derive(Debug)]
pub(super) struct Binding {
    owners:BTreeSet<u32>,
    paints:Vec<(u32,u32,u32)>,
    paint_start:usize,
}
fn parent(record:&Record)->Option<u32>{match record.get("parentId"){Some(Value::Uint(v))=>Some(*v),_=>None}}
fn at(records:&[Record],id:u32)->Option<&Record>{(id as usize).checked_add(1).and_then(|i|records.get(i))}
fn id(index:usize)->Result<u32,Unresolved>{index.checked_sub(1).and_then(|i|u32::try_from(i).ok()).ok_or(Unresolved::Encoding)}

/// Reinspect the unchanged base, inferring its common wrapping parent from the
/// first supplied slot. No paint suffix or paint drawable is permitted in the
/// intervening sizing span. The caller separately binds every sizing operand.
pub(super) fn bind(base:&[Record],records:&[Record],paint_start:usize,
    roles:&[(u32,u32)],proof:&wrapping_integral::Nonoverlap)->Result<Binding,Unresolved>{
    if paint_start<base.len()||records.len()!=paint_start{return Err(Unresolved::Span);}
    if !proof.matches_base(base){return Err(Unresolved::Proof);}
    let first=roles.first().ok_or(Unresolved::Roles)?;
    let root=at(base,first.0).and_then(parent).ok_or(Unresolved::Roles)?;
    let inspected=wrapping_slots::inspect(base,root,roles).map_err(Unresolved::Structure)?;
    let owners: BTreeSet<_>=roles.iter().map(|r|r.1).collect();
    if owners.len()!=roles.len()||proof.geometries()!=&owners{return Err(Unresolved::Proof);}
    if wire::encode(base).map_err(|_|Unresolved::Encoding)?!=wire::encode(&records[..base.len()]).map_err(|_|Unresolved::Encoding)?{return Err(Unresolved::Prefix);}
    let mut paints=Vec::new();
    for slot in &inspected.slots{
        let fills=base.iter().enumerate().filter(|(_,r)|r.kind=="Fill"&&parent(r)==Some(slot.visible)).collect::<Vec<_>>();
        if fills.len()!=1{return Err(Unresolved::Paint{owner:slot.visible});}
        let (fi,fill)=fills[0];let fill_id=id(fi)?;
        if !fill.has_only_properties(&["parentId"]){return Err(Unresolved::Paint{owner:slot.visible});}
        let colors=base.iter().enumerate().filter(|(_,r)|r.kind=="SolidColor"&&parent(r)==Some(fill_id)).collect::<Vec<_>>();
        if colors.len()!=1{return Err(Unresolved::Paint{owner:slot.visible});}
        let (ci,color)=colors[0];let color_id=id(ci)?;
        if !color.has_only_properties(&["parentId","colorValue"])||!matches!(color.get("colorValue"),Some(Value::Color(_))){return Err(Unresolved::Paint{owner:slot.visible});}
        paints.push((slot.visible,fill_id,color_id));
    }
    for(position,record)in records.iter().enumerate().skip(base.len()){
        if !matches!(record.kind,"Node"|"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"|"ComponentOrigin"){
            return Err(Unresolved::SizingRecord{position});
        }
    }
    Ok(Binding{owners,paints,paint_start})
}

#[cfg(test)]mod tests{
 use super::*;
 use super::super::{computed_provenance::{NumericSize,NumericStyle},fixed_layout::LayoutStyle,
    scalar_provenance::ScalarProvenance,wrapping_domains,wrapping_sizes::MachineInterval};
 fn set(r:&mut Record,k:&str,v:Value){r.set(k,v).unwrap();}
 struct Fixture{records:Vec<Record>,parent:u32,roles:Vec<(u32,u32)>,parent_style:LayoutStyle,styles:Vec<(u32,LayoutStyle)>}
 fn fixture(count:usize)->Fixture{
  let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];set(&mut r[1],"styleId",Value::Uint(1));
  fn node(r:&mut Vec<Record>,parent:u32,size:f32)->(u32,LayoutStyle){
   let id=r.len()as u32-1;let mut n=Record::new("LayoutComponent");let mut s=Record::new("LayoutComponentStyle");
   set(&mut n,"parentId",Value::Uint(parent));set(&mut n,"styleId",Value::Uint(id+1));
   for axis in ["width","height"]{set(&mut n,axis,Value::Float(size));set(&mut s,&format!("{axis}UnitsValue"),Value::Uint(1));}
   for axis in ["Width","Height"]{set(&mut s,&format!("min{axis}"),Value::Float(0.));set(&mut s,&format!("min{axis}UnitsValue"),Value::Uint(1));}
   r.extend([n,s]);let numeric=NumericStyle{width:NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(size).unwrap())),height:NumericSize::Pixels(Ok(ScalarProvenance::exact_constant(size).unwrap())),..NumericStyle::default()};
   (id,LayoutStyle::from_numeric(&numeric).unwrap())
  }
  let(parent,parent_style)=node(&mut r,0,200.);set(&mut r[parent as usize+2],"flexDirectionValue",Value::Uint(2));set(&mut r[parent as usize+2],"flexWrapValue",Value::Uint(1));
  let mut roles=Vec::new();let mut styles=Vec::new();
  for _ in 0..count{
   let(slot,s)=node(&mut r,parent,20.);let(visible,v)=node(&mut r,slot,20.);roles.push((slot,visible));styles.extend([(slot,s),(visible,v)]);
   let fill=r.len()as u32-1;let mut f=Record::new("Fill");set(&mut f,"parentId",Value::Uint(visible));let mut c=Record::new("SolidColor");set(&mut c,"parentId",Value::Uint(fill));set(&mut c,"colorValue",Value::Color(0x80ff0033));r.extend([f,c]);
  }
  Fixture{records:r,parent,roles,parent_style,styles}
 }
 fn proof(f:&Fixture)->wrapping_integral::Nonoverlap{
  let refs=f.styles.iter().map(|(id,s)|(*id,s)).collect::<Vec<_>>();
  let domains=wrapping_domains::resolve_layout(&f.records,f.parent,&f.roles,&f.parent_style,&refs,[MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
  wrapping_integral::nonoverlap(&domains,&vec![0.;f.roles.len()]).unwrap()
 }
 #[test]fn binds_actual_original_ownership_without_paint_suffix(){
  let f=fixture(2);let proof=proof(&f);let mut records=f.records.clone();let mut node=Record::new("Node");set(&mut node,"parentId",Value::Uint(0));records.push(node);
  let b=bind(&f.records,&records,records.len(),&f.roles,&proof).unwrap();assert_eq!(b.owners,proof.geometries().clone());assert_eq!(b.paints.len(),2);assert_eq!(b.paint_start,records.len());
 }
 #[test]fn rejects_changed_original_color_parent_and_paint_topology(){
  let f=fixture(2);let proof=proof(&f);
  let color=f.records.iter().position(|r|r.kind=="SolidColor").unwrap();let fill=color-1;
  for(index,key,value)in [(color,"colorValue",Value::Color(0)),(fill,"parentId",Value::Uint(f.parent)),(color,"parentId",Value::Uint(f.roles[0].1)),(f.roles[0].1 as usize+1,"parentId",Value::Uint(f.parent))]{
   let mut records=f.records.clone();set(&mut records[index],key,value);assert!(bind(&f.records,&records,records.len(),&f.roles,&proof).is_err());
  }
  // Reinspection also rejects an invalid supplied base instead of assuming
  // prefix byte equality alone establishes original Fill/Color ownership.
  for(index,parent)in [(fill,f.parent),(color,f.roles[0].1)]{
   let mut base=f.records.clone();set(&mut base[index],"parentId",Value::Uint(parent));assert!(bind(&base,&base,base.len(),&f.roles,&proof).is_err());
  }
 }
 #[test]fn rejects_same_ids_when_certificate_belongs_to_different_base(){
  let f=fixture(2);let certificate=proof(&f);let mut changed=f.records.clone();
  // The original token proves20px visible content inside20px slots. Keeping
  // every object ID but widening the first visible to30px introduces overlap.
  set(&mut changed[f.roles[0].1 as usize+1],"width",Value::Float(30.));
  assert!(wrapping_slots::inspect(&changed,f.parent,&f.roles).is_ok());
  assert_eq!(bind(&changed,&changed,changed.len(),&f.roles,&certificate).unwrap_err(),Unresolved::Proof);
 }
 #[test]fn rejects_unproven_roles_bad_span_and_injected_paint_objects(){
  let f=fixture(2);let proof=proof(&f);
  assert!(bind(&f.records,&f.records,f.records.len()-1,&f.roles,&proof).is_err());
  assert!(bind(&f.records,&f.records,f.records.len()+1,&f.roles,&proof).is_err());
  assert!(bind(&f.records,&f.records,f.records.len(),&[],&proof).is_err());
  assert!(bind(&f.records,&f.records,f.records.len(),&[(u32::MAX,0)],&proof).is_err());
  let other=fixture(1);let foreign=super::tests::proof(&other);assert_eq!(bind(&f.records,&f.records,f.records.len(),&f.roles,&foreign).unwrap_err(),Unresolved::Proof);
  for kind in ["Shape","ForegroundLayoutDrawable","Fill","SolidColor","Rectangle","ClippingShape","DrawRules","DrawTarget","LayoutComponent","LayoutComponentStyle"]{
   let mut records=f.records.clone();let mut record=Record::new(kind);set(&mut record,"parentId",Value::Uint(0));records.push(record);
   assert!(matches!(bind(&f.records,&records,records.len(),&f.roles,&proof),Err(Unresolved::SizingRecord{..})),"{kind}");
   assert!(bind(&f.records,&records,f.records.len(),&f.roles,&proof).is_err());
  }
 }
}
