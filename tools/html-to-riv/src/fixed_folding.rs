//! Private lowering of an already-derived rounded paint suffix. Constants come
//! from actual graph evaluation and mask intersection, never ideal CSS edges.
//! No public admission or geometry/native equivalence claim is made here.
use super::{fixed_geometry, fixed_paint, fixed_scalar, wrapping_composition::{Derived, PaintProof}};
use crate::wire::{self, Record, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(super) enum Unresolved { Source, Route, Span, Grammar(u32), Order, Coordinate,
    Scalar(fixed_scalar::Unresolved), Emission }
pub(super) struct Folded {
    source: Vec<u8>,
    geometry: fixed_geometry::Binding,
    paint: fixed_paint::Folded,
}
impl Folded {
    pub(super) fn records(&self) -> &[Record] { self.paint.records() }
    pub(super) fn rects(&self) -> &[fixed_paint::Rect] { self.paint.rects() }
    pub(super) fn start(&self) -> usize { self.paint.start() }
    pub(super) fn validate(&self, derived:&Derived, geometry:&fixed_geometry::Binding) -> Result<(),Unresolved> {
        if wire::encode(derived.candidate().records()).map_err(|_|Unresolved::Source)? != self.source
            || !self.geometry.same_binding(geometry) { return Err(Unresolved::Source); }
        let (start,rects)=descriptors(derived,geometry)?;
        self.paint.validate(&derived.candidate().records()[..start],&rects).map_err(|_|Unresolved::Emission)
    }
}
fn uint(r:&Record,key:&str,id:u32)->Result<u32,Unresolved>{
    if let Some(Value::Uint(v))=r.get(key){Ok(*v)}else{Err(Unresolved::Grammar(id))}
}
fn float(r:&Record,key:&str,id:u32)->Result<f32,Unresolved>{
    if let Some(Value::Float(v))=r.get(key){if v.is_finite(){return Ok(*v);}}
    Err(Unresolved::Grammar(id))
}
fn color(r:&Record,id:u32)->Result<u32,Unresolved>{
    if let Some(Value::Color(v))=r.get("colorValue"){Ok(*v)}else{Err(Unresolved::Grammar(id))}
}
fn record(records:&[Record],id:u32)->Result<&Record,Unresolved>{
    (id as usize).checked_add(1).and_then(|i|records.get(i)).ok_or(Unresolved::Grammar(id))
}
fn only(r:&Record,kind:&str,fields:&[&str],id:u32)->Result<(),Unresolved>{
    if r.kind==kind && r.has_only_properties(fields){Ok(())}else{Err(Unresolved::Grammar(id))}
}
fn children(records:&[Record],start:usize)->Result<BTreeMap<u32,Vec<u32>>,Unresolved>{
    let mut result=BTreeMap::<u32,Vec<u32>>::new();
    for (index,r) in records.iter().enumerate().skip(start){
        let id=u32::try_from(index.checked_sub(1).ok_or(Unresolved::Span)?).map_err(|_|Unresolved::Span)?;
        result.entry(uint(r,"parentId",id)?).or_default().push(id);
    }
    Ok(result)
}
fn quad(records:&[Record],shape:u32,children:&BTreeMap<u32,Vec<u32>>,worlds:&BTreeMap<u32,[f32;2]>) -> Result<[f32;4],Unresolved>{
    let s=record(records,shape)?;only(s,"Shape",&["parentId"],shape)?;
    let rects:Vec<_>=children.get(&shape).into_iter().flatten().copied()
        .filter(|id|record(records,*id).is_ok_and(|r|r.kind=="Rectangle")).collect();
    if rects.len()!=1{return Err(Unresolved::Grammar(shape));}
    let id=rects[0];let r=record(records,id)?;only(r,"Rectangle",&["parentId","x","y","width","height"],id)?;
    let center=[float(r,"x",id)?,float(r,"y",id)?];let size=[float(r,"width",id)?,float(r,"height",id)?];
    if size.iter().any(|v|*v<=0.){return Err(Unresolved::Coordinate);}
    let world=worlds.get(&shape).ok_or(Unresolved::Grammar(shape))?;
    // The bound rounded graph uses 32768px rectangles and +/-16384px centers.
    // descriptors() additionally proves bounded integer Shape translations, so
    // native path/shape transforms and these reordered integer sums are exact.
    // This helper alone does not certify arbitrary fractional path arithmetic.
    let left=center[0]-size[0]*0.5;let top=center[1]-size[1]*0.5;
    let right=left+size[0];let bottom=top+size[1];
    let q=[left+world[0],top+world[1],right+world[0],bottom+world[1]];
    if q.iter().all(|v|v.is_finite()){Ok(q)}else{Err(Unresolved::Coordinate)}
}
fn intersect(a:[f32;4],b:[f32;4])->[f32;4]{[a[0].max(b[0]),a[1].max(b[1]),a[2].min(b[2]),a[3].min(b[3])]}
fn integer(value:f32)->Result<i32,Unresolved>{
    if value.is_finite() && value.fract()==0. && (-16384. ..=16384.).contains(&value){Ok(value as i32)}else{Err(Unresolved::Coordinate)}
}
/// Follow actual After targets in the immutable reverse-iteration drawable list.
/// If A has After(B), reverse drawing visits A before B. Only a single complete
/// closed chain over the replica set is accepted; defaults/cycles reject.
fn order(records:&[Record],start:usize,replicas:&BTreeSet<u32>)->Result<Vec<u32>,Unresolved>{
    if replicas.is_empty(){return Err(Unresolved::Order);}
    let mut next=BTreeMap::new();let mut incoming=BTreeSet::new();let mut targets=BTreeSet::new();
    for (i,r) in records.iter().enumerate().skip(start).filter(|(_,r)|r.kind=="DrawRules"){
        let id=(i-1)as u32;only(r,"DrawRules",&["parentId","drawTargetId"],id)?;
        let from=uint(r,"parentId",id)?;let target=uint(r,"drawTargetId",id)?;let t=record(records,target)?;
        only(t,"DrawTarget",&["parentId","drawableId","placementValue"],target)?;
        let to=uint(t,"drawableId",target)?;
        if !replicas.contains(&from)||!replicas.contains(&to)||uint(t,"parentId",target)?!=id||uint(t,"placementValue",target)?!=1
            ||!targets.insert(target)||next.insert(from,to).is_some()||!incoming.insert(to){return Err(Unresolved::Order);}
    }
    if records.iter().skip(start).filter(|r|r.kind=="DrawTarget").count()!=targets.len(){return Err(Unresolved::Order);}
    let heads:Vec<_>=replicas.difference(&incoming).copied().collect();if heads.len()!=1{return Err(Unresolved::Order);}
    let mut sequence=Vec::new();let mut seen=BTreeSet::new();let mut current=heads[0];
    loop{if !seen.insert(current){return Err(Unresolved::Order);}sequence.push(current);match next.get(&current){Some(id)=>current=*id,None=>break}}
    if seen!=*replicas||next.len()+1!=replicas.len(){return Err(Unresolved::Order);}Ok(sequence)
}
fn lower(records:&[Record],start:usize,boxes:&BTreeMap<[u32;4],u32>,worlds:&BTreeMap<u32,[f32;2]>) -> Result<Vec<fixed_paint::Rect>,Unresolved>{
    let child=children(records,start)?;let mut painted=BTreeMap::new();
    for (index,f) in records.iter().enumerate().skip(start).filter(|(_,r)|r.kind=="Fill"){
        let fill=(index-1)as u32;only(f,"Fill",&["parentId"],fill)?;let shape=uint(f,"parentId",fill)?;
        if uint(record(records,shape)?,"parentId",shape)?!=0{return Err(Unresolved::Grammar(shape));}
        let colors=child.get(&fill).ok_or(Unresolved::Grammar(fill))?;
        if colors.len()!=1{return Err(Unresolved::Grammar(fill));}let color_id=colors[0];let c=record(records,color_id)?;
        only(c,"SolidColor",&["parentId","colorValue"],color_id)?;let value=color(c,color_id)?;
        let mut clips=Vec::new();
        for id in child.get(&shape).ok_or(Unresolved::Grammar(shape))? {
            let r=record(records,*id)?;
            match r.kind {"Rectangle"|"Fill"|"DrawRules"=>{},"ClippingShape"=>{
                only(r,"ClippingShape",&["parentId","sourceId"],*id)?;clips.push(uint(r,"sourceId",*id)?);
            },_=>return Err(Unresolved::Grammar(*id))}
        }
        let first:[u32;4]=clips.get(..4).ok_or(Unresolved::Grammar(shape))?.try_into().map_err(|_|Unresolved::Grammar(shape))?;
        let owner=*boxes.get(&first).ok_or(Unresolved::Grammar(shape))?;
        let mut edges=quad(records,shape,&child,worlds)?;
        for mask in clips{edges=intersect(edges,quad(records,mask,&child,worlds)?);}
        // Inactive line membership is discovered from actual disjoint masks.
        // Do not cap a mask at the artboard: the preserved artboard clips at runtime.
        let rect=if edges[0]>=edges[2]||edges[1]>=edges[3]||value>>24==0{None}else{Some(fixed_paint::Rect{
            geometry:owner,left:integer(edges[0])?,top:integer(edges[1])?,right:integer(edges[2])?,bottom:integer(edges[3])?,color:value})};
        if painted.insert(shape,rect).is_some(){return Err(Unresolved::Grammar(shape));}
    }
    let sequence=order(records,start,&painted.keys().copied().collect())?;
    Ok(sequence.into_iter().filter_map(|id|painted[&id]).collect())
}
fn descriptors(derived:&Derived,geometry:&fixed_geometry::Binding)->Result<(usize,Vec<fixed_paint::Rect>),Unresolved>{
    let candidate=derived.candidate();let records=candidate.records();
    if !matches!(derived.paint(),PaintProof::Ordered(_))||candidate.original_paint().is_some()
        ||!candidate.paint_trace().integral.is_empty()||records.iter().any(|r|r.kind=="ForegroundLayoutDrawable"){return Err(Unresolved::Route);}
    let (base,sizing,paint)=candidate.record_costs();let start=base.checked_add(sizing).ok_or(Unresolved::Span)?;
    if base!=geometry.base_record_count()||start.checked_add(paint)!=Some(records.len()){return Err(Unresolved::Span);}
    let mut boxes=BTreeMap::new();for b in &candidate.paint_trace().boxes{
        if b.start<start||b.end>records.len()||boxes.insert(b.masks,b.geometry).is_some(){return Err(Unresolved::Span);}
    }
    if boxes.len()!=candidate.slots().len(){return Err(Unresolved::Route);}
    // All authored visible paint must already be hidden. Keep ordinary artboard
    // background paint and every prefix field intact; reject other visible prefix paint.
    for (i,r) in records.iter().enumerate().take(base).filter(|(_,r)|r.kind=="SolidColor"){
        let id=(i-1)as u32;let fill=uint(r,"parentId",id)?;let f=record(records,fill)?;
        if f.kind!="Fill" {return Err(Unresolved::Grammar(fill));}
        if uint(f,"parentId",fill)?!=0&&color(r,id)?!=0{return Err(Unresolved::Source);}
    }
    let outputs:Vec<_>=records.iter().enumerate().skip(start).filter(|(_,r)|r.kind=="Shape").map(|(i,_)|(i-1)as u32).collect();
    let evaluation=fixed_scalar::evaluate_derived(derived,geometry,&outputs).map_err(Unresolved::Scalar)?;
    if !evaluation.matches_records(records){return Err(Unresolved::Source);}
    for id in &outputs {
        let matrix=evaluation.world_matrix(*id).ok_or(Unresolved::Grammar(*id))?;
        if matrix[..4]!=[1.,0.,0.,1.]
            || matrix[4..].iter().any(|v|v.fract()!=0.||v.abs()>65536.) {
            return Err(Unresolved::Coordinate);
        }
    }
    Ok((start,lower(records,start,&boxes,evaluation.values())?))
}
pub(super) fn fold(derived:&Derived,geometry:&fixed_geometry::Binding)->Result<Folded,Unresolved>{
    let (start,rects)=descriptors(derived,geometry)?;let records=derived.candidate().records();
    let paint=fixed_paint::Folded::new(&records[..start],&rects).map_err(|_|Unresolved::Emission)?;
    Ok(Folded{source:wire::encode(records).map_err(|_|Unresolved::Source)?,geometry:geometry.clone(),paint})
}

#[cfg(test)]mod tests{
    use super::*;
    fn add(r:&mut Vec<Record>,kind:&'static str,parent:u32)->u32{let id=r.len()as u32-1;let mut n=Record::new(kind);n.set("parentId",Value::Uint(parent)).unwrap();r.push(n);id}
    #[test]fn actual_target_chain_and_mutations(){
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard")];let a=add(&mut r,"Shape",0);let b=add(&mut r,"Shape",0);let c=add(&mut r,"Shape",0);
        for (from,to)in[(c,a),(a,b)]{let rule=add(&mut r,"DrawRules",from);let target=add(&mut r,"DrawTarget",rule);r[rule as usize+1].set("drawTargetId",Value::Uint(target)).unwrap();r[target as usize+1].set("drawableId",Value::Uint(to)).unwrap();r[target as usize+1].set("placementValue",Value::Uint(1)).unwrap();}
        let ids=[a,b,c].into_iter().collect();assert_eq!(order(&r,2,&ids).unwrap(),vec![c,a,b]);
        let mut m=r.clone();m.last_mut().unwrap().set("drawableId",Value::Uint(c)).unwrap();assert!(order(&m,2,&ids).is_err());
        let mut m=r.clone();m.last_mut().unwrap().set("placementValue",Value::Uint(0)).unwrap();assert!(order(&m,2,&ids).is_err());
        let mut m=r.clone();m.last_mut().unwrap().set("parentId",Value::Uint(0)).unwrap();assert!(order(&m,2,&ids).is_err());
        let mut m=r.clone();add(&mut m,"DrawTarget",0);assert!(order(&m,2,&ids).is_err());
    }
    #[test]fn actual_mask_quad_signed_centers_empty_intersection_and_integer_guard(){
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard")];let s=add(&mut r,"Shape",0);let q=add(&mut r,"Rectangle",s);
        for(k,v)in[("x",-1.5),("y",1.5),("width",19.),("height",15.)]{r[q as usize+1].set(k,Value::Float(v)).unwrap();}
        let children=children(&r,2).unwrap();let worlds=[(s,[4.,-3.])].into_iter().collect();assert_eq!(quad(&r,s,&children,&worlds).unwrap(),[-7.,-9.,12.,6.]);
        assert_eq!(intersect([-7.,-9.,12.,6.],[0.,0.,32768.,32768.]),[0.,0.,12.,6.]);
        let empty=intersect([0.,0.,12.,6.],[65536.,0.,98304.,32768.]);assert!(empty[0]>=empty[2]);
        assert_eq!(integer(-16384.).unwrap(),-16384);assert_eq!(integer(16384.).unwrap(),16384);
        for v in [0.5,16385.,f32::NAN,f32::INFINITY]{assert!(integer(v).is_err());}
        r[q as usize+1].set("originX",Value::Float(0.)).unwrap();assert!(quad(&r,s,&children,&worlds).is_err());
    }
    #[test]fn inactive_replica_is_removed_by_actual_clip_and_active_owner_is_retained(){
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent")];
        let start=r.len();let mut worlds=BTreeMap::new();
        fn shape(r:&mut Vec<Record>,worlds:&mut BTreeMap<u32,[f32;2]>,world:[f32;2])->u32{
            let s=add(r,"Shape",0);let q=add(r,"Rectangle",s);worlds.insert(s,world);
            for(k,v)in[("x",5.),("y",10.),("width",10.),("height",20.)]{r[q as usize+1].set(k,Value::Float(v)).unwrap();}s
        }
        let masks=std::array::from_fn(|_|shape(&mut r,&mut worlds,[0.,0.]));
        let gate=shape(&mut r,&mut worlds,[65536.,0.]);
        let mut replicas=Vec::new();let mut clip_ids=Vec::new();
        for inactive in [false,true]{
            let s=shape(&mut r,&mut worlds,[0.,0.]);replicas.push(s);
            let fill=add(&mut r,"Fill",s);let color=add(&mut r,"SolidColor",fill);r[color as usize+1].set("colorValue",Value::Color(0x80ee5533)).unwrap();
            for mask in masks.into_iter().chain(inactive.then_some(gate)){
                let clip=add(&mut r,"ClippingShape",s);r[clip as usize+1].set("sourceId",Value::Uint(mask)).unwrap();clip_ids.push(clip);
            }
        }
        let rule=add(&mut r,"DrawRules",replicas[0]);let target=add(&mut r,"DrawTarget",rule);
        r[rule as usize+1].set("drawTargetId",Value::Uint(target)).unwrap();
        r[target as usize+1].set("drawableId",Value::Uint(replicas[1])).unwrap();r[target as usize+1].set("placementValue",Value::Uint(1)).unwrap();
        let boxes=[(masks,1)].into_iter().collect();
        let rects=lower(&r,start,&boxes,&worlds).unwrap();assert_eq!(rects,vec![fixed_paint::Rect{geometry:1,left:0,top:0,right:10,bottom:20,color:0x80ee5533}]);
        // Changing the actual membership mask changes active count. No per-owner
        // deduplication or CSS-order shortcut is permitted to conceal this.
        let mut changed_worlds=worlds.clone();changed_worlds.insert(gate,[0.,0.]);assert_eq!(lower(&r,start,&boxes,&changed_worlds).unwrap().len(),2);
        let mut changed=r.clone();changed[clip_ids[0]as usize+1].set("sourceId",Value::Uint(gate)).unwrap();assert!(lower(&changed,start,&boxes,&worlds).is_err());
        let wrong_owner=[(masks,2)].into_iter().collect();assert_ne!(lower(&r,start,&wrong_owner,&worlds).unwrap(),rects);
    }

}
