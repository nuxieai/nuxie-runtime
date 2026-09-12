//! Independent constant evaluation of closed ordinary translation-only graphs.
//!
//! No runtime dependency, viewport sample, layout measurement or caller-provided
//! seed is used. Results are bound to the complete encoded input. The pure
//! entrypoint admits bare identity artboards, literal Nodes and strength-one
//! TranslationConstraints. Geometry-backed entrypoints additionally require
//! independently certified local layout/size seeds and model the identity-axis
//! Transform/Distance/ComponentOrigin operations of closed wrapping graphs.
//! Paint-only records require the existing immutable Derived grammar token.
//! These APIs are a folding foundation, not public admission or native evidence.
use crate::wire::{self, Record, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Encoding, Empty, InvalidHandle(u32), Unsupported(u32),
    Fields(u32), NonFinite(u32), Cycle(u32), Depth }
#[derive(Clone, Debug)]
pub(super) struct Evaluation { encoded: Vec<u8>, values: BTreeMap<u32,[f32;2]>, matrices:BTreeMap<u32,[f32;6]> }
impl Evaluation {
    pub(super) fn matrices(&self)->&BTreeMap<u32,[f32;6]>{&self.matrices}
    pub(super) fn world_matrix(&self,id:u32)->Option<[f32;6]>{self.matrices.get(&id).copied()}
    pub(super) fn world(&self,id:u32)->Option<[f32;2]>{self.values.get(&id).copied()}
    pub(super) fn values(&self)->&BTreeMap<u32,[f32;2]>{&self.values}
    pub(super) fn matches_records(&self,records:&[Record])->bool{
        wire::encode(records).is_ok_and(|bytes|bytes==self.encoded)
    }
}
fn uint(r:&Record,key:&str,default:u32,id:u32)->Result<u32,Unresolved>{
    match r.get(key){None=>Ok(default),Some(Value::Uint(v))=>Ok(*v),_=>Err(Unresolved::Fields(id))}
}
fn float(r:&Record,key:&str,default:f32,id:u32)->Result<f32,Unresolved>{
    let v=match r.get(key){None=>default,Some(Value::Float(v))=>*v,_=>return Err(Unresolved::Fields(id))};
    if v.is_finite(){Ok(v)}else{Err(Unresolved::NonFinite(id))}
}
fn boolean(r:&Record,key:&str,default:bool,id:u32)->Result<bool,Unresolved>{
    match r.get(key){None=>Ok(default),Some(Value::Bool(v))=>Ok(*v),_=>Err(Unresolved::Fields(id))}
}
fn finite<const N:usize>(v:[f32;N],id:u32)->Result<[f32;N],Unresolved>{
    if v.iter().all(|x|x.is_finite()){Ok(v)}else{Err(Unresolved::NonFinite(id))}
}
// Explicit f32 operation order from the immutable Mat2D implementation. Keeping
// the zero matrix entries (including inversion's negative zeros) avoids replacing
// source-local conversion with algebraic subtraction that rounds differently.
type Matrix=[f32;6];
fn point(a:Matrix,b:[f32;2])->[f32;2]{[
    a[0].mul_add(b[0],a[2]*b[1])+a[4],
    a[1].mul_add(b[0],a[3]*b[1])+a[5],
]}
fn multiply(a:Matrix,b:Matrix)->Matrix{[
    a[0].mul_add(b[0],a[2]*b[1]),a[1].mul_add(b[0],a[3]*b[1]),
    a[0].mul_add(b[2],a[2]*b[3]),a[1].mul_add(b[2],a[3]*b[3]),
    a[0].mul_add(b[4],a[2]*b[5])+a[4],a[1].mul_add(b[4],a[3]*b[5])+a[5],
]}
fn inverse(a:Matrix)->Matrix{
    let det=1./a[0].mul_add(a[3],-(a[1]*a[2]));
    [a[3]*det,-a[1]*det,-a[2]*det,a[0]*det,
        a[2].mul_add(a[5],-(a[3]*a[4]))*det,
        a[1].mul_add(a[4],-(a[0]*a[5]))*det]
}
struct Reader<'a>{records:&'a[Record],children:BTreeMap<u32,Vec<u32>>,states:Vec<u8>,world:BTreeMap<u32,Matrix>,geometry:Option<&'a super::fixed_geometry::Binding>,base_len:usize,origins:BTreeMap<u32,[f32;2]>,certified_paint:bool}
impl Reader<'_>{
    fn record(&self,id:u32)->Result<&Record,Unresolved>{(id as usize).checked_add(1).and_then(|i|self.records.get(i)).ok_or(Unresolved::InvalidHandle(id))}
    fn visit(&mut self,id:u32,depth:usize)->Result<Matrix,Unresolved>{
        if depth>1024{return Err(Unresolved::Depth);}
        let state=*self.states.get(id as usize).ok_or(Unresolved::InvalidHandle(id))?;
        if state==1{return Err(Unresolved::Cycle(id));}
        if state==2{return Ok(self.world[&id]);}
        self.states[id as usize]=1;
        let r=self.record(id)?.clone();
        let mut world=if id==0 && r.kind=="Artboard"{
            if self.geometry.is_none() && !r.has_only_properties(&["name","width","height"]){return Err(Unresolved::Fields(id));}
            float(&r,"width",0.,id)?;float(&r,"height",0.,id)?;
            [1.,0.,0.,1.,0.,0.]
        }else if let Some(seed)=self.geometry.and_then(|g|g.seeds().get(&id)).copied(){
            let parent=self.visit(uint(&r,"parentId",0,id)?,depth+1)?;
            finite(multiply(multiply(parent,[1.,0.,0.,1.,seed.local[0],seed.local[1]]),[1.,0.,0.,1.,0.,0.]),id)?
        }else{
            if r.kind!="Node" && !(self.certified_paint && matches!(r.kind,"Shape"|"Rectangle")){return Err(Unresolved::Unsupported(id));}
            if !(self.certified_paint && matches!(r.kind,"Shape"|"Rectangle")) && !r.has_only_properties(&["name","parentId","x","y"]){return Err(Unresolved::Fields(id));}
            let parent=uint(&r,"parentId",0,id)?;
            let parent_world=self.visit(parent,depth+1)?;
            // Node/Shape/Rectangle share Node's transform path. Rectangle origins
            // shift generated vertices, not this world matrix (PathBase ->
            // Node; ParametricPathBase origin defaults affect Rectangle::update).
            // Default rotation=+0, scale=(1,1); from_rotation preserves -sin(0).
            let local=[1.,0.,-0.,1.,float(&r,"x",0.,id)?,float(&r,"y",0.,id)?];
            finite(multiply(parent_world,local),id)?
        };
        for child in self.children.get(&id).cloned().unwrap_or_default(){
            let c=self.record(child)?.clone();
            match c.kind{
                "Node"=>{}, // Child positions cannot affect their Node parent.
                "LayoutComponent" if self.geometry.is_some_and(|g|g.seeds().contains_key(&child))=>{},
                _ if (child as usize+1)<self.base_len && !matches!(c.kind,"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint")=>{},
                "ComponentOrigin" if self.geometry.is_some()=>{},
                "Shape"|"Rectangle"|"Fill"|"SolidColor"|"ClippingShape"|"DrawRules"|"DrawTarget" if self.certified_paint=>{},
                "TransformConstraint" if id!=0 && self.geometry.is_some()=>{world=self.transform(world,id,child,&c,depth+1)?;},
                "DistanceConstraint" if id!=0 && self.geometry.is_some()=>{world=self.distance(world,id,child,&c,depth+1)?;},
                "TranslationConstraint" if id!=0=>{
                    world=self.translation(id,&r,world,child,&c,depth+1)?;
                },
                _=>return Err(Unresolved::Unsupported(child)),
            }
        }
        self.states[id as usize]=2;self.world.insert(id,world);Ok(world)
    }
    fn translation(&mut self,owner:u32,node:&Record,mut world:Matrix,id:u32,c:&Record,depth:usize)->Result<Matrix,Unresolved>{
        if !c.has_only_properties(&["name","parentId","targetId","strength","sourceSpaceValue","destSpaceValue","minMaxSpaceValue",
            "doesCopy","doesCopyY","copyFactor","copyFactorY","offset","min","max","minY","maxY","minValue","maxValue","minValueY","maxValueY"]){return Err(Unresolved::Fields(id));}
        if float(c,"strength",1.,id)?!=1.{return Err(Unresolved::Unsupported(id));}
        let source=uint(c,"sourceSpaceValue",0,id)?;
        let dest=uint(c,"destSpaceValue",0,id)?;
        let clamp=uint(c,"minMaxSpaceValue",0,id)?;
        if source>1||dest>1||clamp>1{return Err(Unresolved::Unsupported(id));}
        let target=uint(c,"targetId",u32::MAX,id)?;
        let parent=self.visit(uint(node,"parentId",0,owner)?,depth)?;
        let original=[world[4],world[5]];
        let factors=[float(c,"copyFactor",1.,id)?,float(c,"copyFactorY",1.,id)?];
        let copy=[boolean(c,"doesCopy",true,id)?,boolean(c,"doesCopyY",true,id)?];
        let offset=boolean(c,"offset",false,id)?;
        let literal=self.geometry.and_then(|g|g.seeds().get(&owner)).map(|s|s.local).unwrap_or([float(node,"x",0.,owner)?,float(node,"y",0.,owner)?]);
        let mut value=original;
        if target!=u32::MAX{
            let mut target_world=self.visit(target,depth)?;
            if source==1{
                // Artboard targets are excluded: only ordinary Node parents
                // have the independently modeled source-local relationship.
                let t=self.record(target)?.clone();
                if t.kind!="Node" && !self.geometry.is_some_and(|g|g.seeds().contains_key(&target)){return Err(Unresolved::Unsupported(target));}
                let tp=self.visit(uint(&t,"parentId",0,target)?,depth)?;
                target_world=finite(multiply(inverse(tp),target_world),id)?;
            }
            value=[target_world[4],target_world[5]];
            for axis in 0..2{
                if !copy[axis]{value[axis]=if dest==1{0.}else{original[axis]};}
                else{value[axis]*=factors[axis];finite(value,id)?;
                    if offset{value[axis]+=literal[axis];finite(value,id)?;}}
            }
            if dest==1{value=finite(point(parent,value),id)?;}
        }
        if clamp==1{value=finite(point(inverse(parent),value),id)?;}
        for (axis,min,max,lo,hi) in [(0,"min","max","minValue","maxValue"),(1,"minY","maxY","minValueY","maxValueY")]{
            let lower=float(c,lo,0.,id)?;let upper=float(c,hi,0.,id)?;
            if boolean(c,max,false,id)?&&value[axis]>upper{value[axis]=upper;}
            if boolean(c,min,false,id)?&&value[axis]<lower{value[axis]=lower;}
        }
        if clamp==1{value=finite(point(parent,value),id)?;}
        // Preserve the runtime's strength-one blend rather than assigning b.
        // In particular, arithmetic zero signs are observable in the receipt.
        world[4]=original[0]*(1.-1.)+value[0]*1.;
        world[5]=original[1]*(1.-1.)+value[1]*1.;
        self.land(world,owner,id)
    }
}
impl Reader<'_>{
    fn anchor(&self,owner:u32)->Result<[f32;2],Unresolved>{
        let Some(origin)=self.origins.get(&owner)else{return Ok([0.;2]);};
        let seed=self.geometry.and_then(|g|g.seeds().get(&owner)).ok_or(Unresolved::Unsupported(owner))?;
        finite([origin[0]*seed.size[0],origin[1]*seed.size[1]],owner)
    }
    fn land(&self,mut world:Matrix,owner:u32,id:u32)->Result<Matrix,Unresolved>{
        let a=self.anchor(owner)?;
        if a[0]!=0.||a[1]!=0.{
            world[4]-=(world[0]*a[0]+world[2]*a[1])*1.;
            world[5]-=(world[1]*a[0]+world[3]*a[1])*1.;
        }
        finite(world,id)
    }
    fn parent_world(&mut self,owner:u32,depth:usize)->Result<Matrix,Unresolved>{
        let r=self.record(owner)?;let p=uint(r,"parentId",0,owner)?;self.visit(p,depth)
    }
    fn transform(&mut self,a:Matrix,owner:u32,id:u32,c:&Record,depth:usize)->Result<Matrix,Unresolved>{
        if !c.has_only_properties(&["name","parentId","targetId","strength","sourceSpaceValue","destSpaceValue","originX","originY"]){return Err(Unresolved::Fields(id));}
        if float(c,"strength",1.,id)?!=1.{return Err(Unresolved::Unsupported(id));}
        let target=uint(c,"targetId",u32::MAX,id)?;
        if target==0{return Err(Unresolved::Unsupported(target));} // Dynamic artboard bounds.
        let target_record=self.record(target)?;
        let size=if let Some(seed)=self.geometry.and_then(|g|g.seeds().get(&target)){seed.size}
            else if target_record.kind=="Node"{[0.;2]}else{return Err(Unresolved::Unsupported(target));};
        let local=[1.,0.,0.,1.,0.+size[0]*float(c,"originX",0.,id)?,0.+size[1]*float(c,"originY",0.,id)?];
        let mut b=finite(multiply(self.visit(target,depth)?,finite(local,id)?),id)?;
        let source=uint(c,"sourceSpaceValue",0,id)?;let dest=uint(c,"destSpaceValue",0,id)?;
        if source>1||dest>1{return Err(Unresolved::Unsupported(id));}
        if source==1{let p=self.parent_world(target,depth)?;b=finite(multiply(inverse(p),b),id)?;}
        if dest==1{let p=self.parent_world(owner,depth)?;b=finite(multiply(p,b),id)?;}
        if a[..4]!=[1.,0.,0.,1.]||b[..4]!=[1.,0.,0.,1.]{return Err(Unresolved::Unsupported(id));}
        // Exact identity-axis specialization of Mat2D::decompose and the
        // TransformConstraint strength-one interpolation/recomposition. Keep
        // native operations, including signed zero and multiply/add ordering.
        let decompose=|m:Matrix|{
            let denom=m[0]*m[0]+m[1]*m[1];let sx=denom.sqrt();
            [m[1].atan2(m[0]),sx,(m[0]*m[3]-m[2]*m[1])/sx,(m[0]*m[2]+m[1]*m[3]).atan2(denom)]
        };
        let ca=decompose(a);let cb=decompose(b);
        let aa=ca[0]%(std::f32::consts::PI*2.);let ab=cb[0]%(std::f32::consts::PI*2.);
        let rotation=aa+(ab-aa)*1.;
        let sx=ca[1]*0.+cb[1]*1.;let sy=ca[2]*0.+cb[2]*1.;let skew=ca[3]*0.+cb[3]*1.;
        if rotation!=0.||skew!=0.||sx!=1.||sy!=1.{return Err(Unresolved::Unsupported(id));}
        let result=[1.*sx,0.*sx,-0.*sy,1.*sy,a[4]*0.+b[4]*1.,a[5]*0.+b[5]*1.];
        self.land(result,owner,id)
    }
    fn distance(&mut self,mut world:Matrix,owner:u32,id:u32,c:&Record,depth:usize)->Result<Matrix,Unresolved>{
        if !c.has_only_properties(&["name","parentId","targetId","strength","distance","modeValue"]){return Err(Unresolved::Fields(id));}
        if float(c,"strength",1.,id)?!=1.{return Err(Unresolved::Unsupported(id));}
        let mode=uint(c,"modeValue",0,id)?;if mode>2{return Err(Unresolved::Unsupported(id));}
        let distance=float(c,"distance",100.,id)?;
        let target=self.visit(uint(c,"targetId",u32::MAX,id)?,depth)?;
        let anchor=self.anchor(owner)?;
        let anchor_world=finite([world[0]*anchor[0]+world[2]*anchor[1],world[1]*anchor[0]+world[3]*anchor[1]],id)?;
        let ours=finite([world[4]+anchor_world[0],world[5]+anchor_world[1]],id)?;
        let mut delta=finite([ours[0]-target[4],ours[1]-target[5]],id)?;
        let square=delta[0].mul_add(delta[0],delta[1]*delta[1]);finite([square],id)?;
        let length=square.sqrt();
        if (mode==0&&length<distance)||(mode==1&&length>distance)||length<0.001_f32{return Ok(world);}
        let scale=distance/length;finite([scale],id)?;delta=finite([delta[0]*scale,delta[1]*scale],id)?;
        let position=finite([target[4]+delta[0],target[5]+delta[1]],id)?;
        world[4]=(position[0]-ours[0]).mul_add(1.,ours[0])-anchor_world[0];
        world[5]=(position[1]-ours[1]).mul_add(1.,ours[1])-anchor_world[1];
        finite(world,id)
    }
}

pub(super) fn evaluate(records:&[Record],outputs:&[u32])->Result<Evaluation,Unresolved>{
    if outputs.is_empty(){return Err(Unresolved::Empty);}
    if records.first().map(|r|r.kind)!=Some("Backboard")||!records[0].has_only_properties(&[])
        || records.get(1).map(|r|r.kind)!=Some("Artboard"){return Err(Unresolved::Encoding);}
    let encoded=wire::encode(records).map_err(|_|Unresolved::Encoding)?;
    let mut children:BTreeMap<u32,Vec<u32>>=BTreeMap::new();
    for (index,r) in records.iter().enumerate().skip(2){
        let id=u32::try_from(index-1).map_err(|_|Unresolved::Encoding)?;
        if !matches!(r.kind,"Node"|"TranslationConstraint"){return Err(Unresolved::Unsupported(id));}
        let parent=uint(r,"parentId",0,id)?;
        let owner=(parent as usize).checked_add(1).and_then(|i|records.get(i)).ok_or(Unresolved::InvalidHandle(parent))?;
        if !(owner.kind=="Node" || (r.kind=="Node" && parent==0 && owner.kind=="Artboard")){
            return Err(Unresolved::Unsupported(parent));
        }
        children.entry(parent).or_default().push(id);
    }
    let mut reader=Reader{records,children,states:vec![0;records.len()-1],world:BTreeMap::new(),geometry:None,base_len:0,origins:BTreeMap::new(),certified_paint:false};
    for id in outputs{reader.visit(*id,0)?;}
    // The certificate binds a complete closed file, not only the requested
    // dependency closure. Unvisited dangling objects/cycles must not turn an
    // import-invalid scene into an apparently resolved constant evaluation.
    for (index,r) in records.iter().enumerate().skip(2){
        if r.kind=="Node"{reader.visit((index-1)as u32,0)?;}
    }
    Ok(Evaluation{encoded,values:reader.world.iter().map(|(id,m)|(*id,[m[4],m[5]])).collect(),matrices:reader.world})
}

/// Evaluate an independently closed constraint-only extension of the exact
/// certified base. Paint grammar is not inferred from ignored fields here.
pub(super) fn evaluate_with_geometry(records:&[Record],geometry:&super::fixed_geometry::Binding,outputs:&[u32])->Result<Evaluation,Unresolved>{
    geometry_evaluation(records,geometry,outputs,false)
}
/// A Derived token independently owns/binds every paint record and its ordering
/// grammar. That proof permits paint-only children to be ignored for scalar
/// dependencies, while Shape transforms are still evaluated, never seeded.
pub(super) fn evaluate_derived(derived:&super::wrapping_composition::Derived,geometry:&super::fixed_geometry::Binding,outputs:&[u32])->Result<Evaluation,Unresolved>{
    let c=derived.candidate();
    if !matches!(derived.paint(),super::wrapping_composition::PaintProof::Ordered(_))
        || c.original_paint().is_some() || !c.integral().geometries().is_empty(){return Err(Unresolved::Unsupported(0));}
    if !c.layout_authored().is_some_and(|s|geometry.matches_source(s)){return Err(Unresolved::Encoding);}
    if c.record_costs().0!=geometry.base_record_count(){return Err(Unresolved::Encoding);}
    geometry_evaluation(c.records(),geometry,outputs,true)
}
fn geometry_evaluation(records:&[Record],geometry:&super::fixed_geometry::Binding,outputs:&[u32],certified_paint:bool)->Result<Evaluation,Unresolved>{
    if outputs.is_empty(){return Err(Unresolved::Empty);}
    let base_len=geometry.base_record_count();
    if records.get(..base_len).is_none_or(|r|!geometry.matches_base(r)){return Err(Unresolved::Encoding);}
    let encoded=wire::encode(records).map_err(|_|Unresolved::Encoding)?;
    let mut children:BTreeMap<u32,Vec<u32>>=BTreeMap::new();let mut origins=BTreeMap::new();
    for(index,r)in records.iter().enumerate().skip(2){
        let id=u32::try_from(index-1).map_err(|_|Unresolved::Encoding)?;
        let parent=uint(r,"parentId",0,id)?;
        if index>=base_len{
            let owner=(parent as usize).checked_add(1).and_then(|i|records.get(i)).ok_or(Unresolved::InvalidHandle(parent))?;
            match r.kind{
                "Node"=>if !(parent==0||owner.kind=="Node"||geometry.seeds().contains_key(&parent)){return Err(Unresolved::Unsupported(parent));},
                "TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"=>{
                    if !(owner.kind=="Node"||geometry.seeds().contains_key(&parent)||(certified_paint&&owner.kind=="Shape")){return Err(Unresolved::Unsupported(parent));}
                },
                "ComponentOrigin"=>{
                    if !geometry.seeds().contains_key(&parent)||!r.has_only_properties(&["parentId","originX","originY"]){return Err(Unresolved::Fields(id));}
                    let slot=uint(owner,"parentId",0,parent)?;
                    let slot_record=(slot as usize).checked_add(1).and_then(|i|records.get(i)).ok_or(Unresolved::InvalidHandle(slot))?;
                    if !geometry.seeds().contains_key(&slot)||uint(slot_record,"parentId",0,slot)?!=geometry.source().parent.0{return Err(Unresolved::Unsupported(parent));}
                    let value=[float(r,"originX",0.,id)?,float(r,"originY",0.,id)?];
                    if !value.iter().all(|v|[0.,0.5,1.].contains(v)){return Err(Unresolved::Unsupported(id));}
                    if origins.insert(parent,value).is_some(){return Err(Unresolved::Unsupported(parent));}
                },
                "Shape"|"Rectangle"|"Fill"|"SolidColor"|"ClippingShape"|"DrawRules"|"DrawTarget" if certified_paint=>{},
                _=>return Err(Unresolved::Unsupported(id)),
            }
        }
        children.entry(parent).or_default().push(id);
    }
    let mut reader=Reader{records,children,states:vec![0;records.len()-1],world:BTreeMap::new(),geometry:Some(geometry),base_len,origins,certified_paint};
    for id in outputs{reader.visit(*id,0)?;}
    for (index,r)in records.iter().enumerate().skip(1){
        let id=(index-1)as u32;
        if id==0||r.kind=="Node"||geometry.seeds().contains_key(&id)||(certified_paint&&matches!(r.kind,"Shape"|"Rectangle")){reader.visit(id,0)?;}
    }
    Ok(Evaluation{encoded,values:reader.world.iter().map(|(id,m)|(*id,[m[4],m[5]])).collect(),matrices:reader.world})
}

#[cfg(test)]mod tests{
    use super::*;
    fn literal(value:f32,y:bool)->Vec<Record>{
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("Node")];
        r[2].set(if y{"y"}else{"x"},Value::Float(value)).unwrap();r
    }
    #[test]fn actual_signed_rounding_graph_ties_neighbors_and_subnormal(){
        for y in [false,true]{for value in [-16384.,-2.5,-1.5,-0.5,-0.,0.,0.5,1.5,2.5,16384.,
            f32::from_bits(0.5_f32.to_bits()-1),f32::from_bits(0.5_f32.to_bits()+1),f32::from_bits(1)]{
            let mut r=literal(value,y);let t=super::super::paint_rounding::round(&mut r,1,y).unwrap();
            let e=evaluate(&r,&[t.rounded]).unwrap();
            let expected=(f64::from(value)+0.5).floor()as f32;
            assert_eq!(e.world(t.rounded).unwrap()[usize::from(y)],expected,"{value} {y}");
            assert!(e.matches_records(&r));assert!(e.values().len()>100);
            let mut changed=r.clone();changed[2].set(if y{"y"}else{"x"},Value::Float(value+1.)).unwrap();
            assert!(!e.matches_records(&changed));
        }}
    }
    #[test]fn cycles_dynamic_targets_and_unknown_fields_are_unresolved(){
        let mut r=literal(1.,false);r[2].set("parentId",Value::Uint(1)).unwrap();
        assert!(matches!(evaluate(&r,&[1]),Err(Unresolved::Cycle(1))));
        let mut r=literal(1.,false);r[2].set("rotation",Value::Float(0.)).unwrap();
        assert!(matches!(evaluate(&r,&[1]),Err(Unresolved::Fields(1))));
        for kind in ["TransformConstraint","DistanceConstraint"]{
            let mut r=literal(1.,false);let mut c=Record::new(kind);
            c.set("parentId",Value::Uint(1)).unwrap();c.set("targetId",Value::Uint(3)).unwrap();
            r.push(c);r.push(Record::new("LayoutComponent"));assert!(evaluate(&r,&[1]).is_err());
        }
        assert!(evaluate(&literal(f32::INFINITY,false),&[1]).is_err());
    }
    #[test]fn targetless_local_clamp_and_minimum_wins(){
        let mut r=literal(10.,false);let mut node=Record::new("Node");node.set("parentId",Value::Uint(1)).unwrap();node.set("x",Value::Float(5.)).unwrap();r.push(node);
        let mut c=Record::new("TranslationConstraint");c.set("parentId",Value::Uint(2)).unwrap();
        c.set("minMaxSpaceValue",Value::Uint(1)).unwrap();c.set("min",Value::Bool(true)).unwrap();c.set("max",Value::Bool(true)).unwrap();
        c.set("minValue",Value::Float(8.)).unwrap();c.set("maxValue",Value::Float(3.)).unwrap();r.push(c);
        assert_eq!(evaluate(&r,&[2]).unwrap().world(2).unwrap(),[18.,0.]);
    }
    #[test]fn local_source_destination_offset_factor_and_uncopied_axis(){
        let mut r=literal(30.,false);
        for (parent,x,y) in [(1,5.,0.),(0,100.,0.),(3,2.,7.)]{
            let mut n=Record::new("Node");n.set("parentId",Value::Uint(parent)).unwrap();
            n.set("x",Value::Float(x)).unwrap();n.set("y",Value::Float(y)).unwrap();r.push(n);
        }
        let mut c=Record::new("TranslationConstraint");
        for (k,v) in [("parentId",4),("targetId",2),("sourceSpaceValue",1),("destSpaceValue",1)]{
            c.set(k,Value::Uint(v)).unwrap();
        }
        c.set("copyFactor",Value::Float(2.)).unwrap();c.set("doesCopyY",Value::Bool(false)).unwrap();
        c.set("offset",Value::Bool(true)).unwrap();r.push(c);
        assert_eq!(evaluate(&r,&[4]).unwrap().world(4).unwrap(),[112.,0.]);
        r[6].set("strength",Value::Float(0.5)).unwrap();
        assert!(evaluate(&r,&[4]).is_err());
    }

    #[test]fn actual_paint_box_layout_corners_are_not_invented(){
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent")];
        let t=super::super::paint_box::emit(&mut r,1).unwrap();
        assert!(matches!(evaluate(&r,&[t.axes[0].rounding[0].rounded]),Err(Unresolved::Unsupported(_))));
    }

    #[test]fn unrelated_invalid_objects_cannot_hide_outside_requested_closure(){
        let mut r=literal(1.,false);
        let mut dangling=Record::new("Node");dangling.set("parentId",Value::Uint(999)).unwrap();r.push(dangling);
        assert!(matches!(evaluate(&r,&[1]),Err(Unresolved::InvalidHandle(999))));
        r[3].set("parentId",Value::Uint(2)).unwrap();
        assert!(matches!(evaluate(&r,&[1]),Err(Unresolved::Cycle(2))));
        r[3]=Record::new("TranslationConstraint");r[3].set("parentId",Value::Uint(999)).unwrap();
        assert!(evaluate(&r,&[1]).is_err());
        r[3]=Record::new("Node");r[3].set("rotation",Value::Float(0.)).unwrap();
        assert!(matches!(evaluate(&r,&[1]),Err(Unresolved::Fields(2))));
    }

    fn geometry_fixture()->(Vec<Record>,super::super::fixed_geometry::Binding,Vec<super::super::wrapping::Item>){
        use super::super::{fixed_layout::{LayoutStyle,LayoutLength,FixedLayoutLength},scalar_provenance::ScalarProvenance,wrapping_domains,wrapping_sizes::MachineInterval,fixed_geometry,wrapping};
        let length=|v|LayoutLength::Fixed(FixedLayoutLength::new(ScalarProvenance::exact_constant(v).unwrap()).unwrap());
        let style=|w,h|LayoutStyle{width:length(w),height:length(h),min_width:length(0.),min_height:length(0.),max_width:LayoutLength::Auto,max_height:LayoutLength::Auto};
        let p=style(20.,100.);let slot=style(20.,20.);let visible=style(10.,10.);
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];
        r[1].set("styleId",Value::Uint(1)).unwrap();r[1].set("width",Value::Float(320.)).unwrap();r[1].set("height",Value::Float(240.)).unwrap();
        fn node(r:&mut Vec<Record>,parent:u32,s:&LayoutStyle)->u32{
            let id=r.len()as u32-1;let mut n=Record::new("LayoutComponent");n.set("parentId",Value::Uint(parent)).unwrap();n.set("styleId",Value::Uint(id+1)).unwrap();
            n.set("width",Value::Float(s.width.value().unwrap())).unwrap();n.set("height",Value::Float(s.height.value().unwrap())).unwrap();
            let mut t=Record::new("LayoutComponentStyle");
            for k in ["widthUnitsValue","heightUnitsValue","minWidthUnitsValue","minHeightUnitsValue"]{t.set(k,Value::Uint(1)).unwrap();}
            for k in ["minWidth","minHeight"]{t.set(k,Value::Float(0.)).unwrap();}
            r.extend([n,t]);id
        }
        let parent=node(&mut r,0,&p);r[parent as usize+2].set("flexWrapValue",Value::Uint(1)).unwrap();r[parent as usize+2].set("flexDirectionValue",Value::Uint(2)).unwrap();
        let mut roles=Vec::new();let mut sources=Vec::new();let mut items=Vec::new();
        for _ in 0..2{
            let a=node(&mut r,parent,&slot);let b=node(&mut r,a,&visible);
            let fill=r.len()as u32-1;let mut f=Record::new("Fill");f.set("parentId",Value::Uint(b)).unwrap();let mut color=Record::new("SolidColor");color.set("parentId",Value::Uint(fill)).unwrap();color.set("colorValue",Value::Color(0xffff0000)).unwrap();r.extend([f,color]);
            roles.push((a,b));sources.push((a,&slot));sources.push((b,&visible));items.push(wrapping::Item{slot:a,visible:b,alignment:0.5});
        }
        let d=wrapping_domains::resolve_layout(&r,parent,&roles,&p,&sources,[MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
        let g=fixed_geometry::bind(&d).unwrap();(r,g,items)
    }
    #[test]fn actual_sizing_graph_lands_origins_and_recomposes_children(){
        let(mut r,g,items)=geometry_fixture();
        let trace=super::super::wrapping::align_items(&mut r,&items,true,false,0.,0.25).unwrap();
        let child=r.len()as u32-1;let mut n=Record::new("Node");n.set("parentId",Value::Uint(items[1].visible)).unwrap();n.set("y",Value::Float(2.)).unwrap();r.push(n);
        let e=evaluate_with_geometry(&r,&g,&[child]).unwrap();
        assert_eq!(e.world(items[0].visible).unwrap(),[0.,5.]);
        assert_eq!(e.world(items[1].visible).unwrap(),[0.,25.]);
        assert_eq!(e.world(child).unwrap(),[0.,27.]);
        assert_eq!(e.world(trace.boundaries[0].gate).unwrap()[1],65536.);
        assert!(e.world_matrix(child).unwrap()[..4]==[1.,0.,0.,1.]);
        assert!(e.matches_records(&r));
        let mut changed=r.clone();changed[1].set("width",Value::Float(321.)).unwrap();
        assert!(matches!(evaluate_with_geometry(&changed,&g,&[child]),Err(Unresolved::Encoding)));
        assert!(evaluate(&r,&[child]).is_err());
    }
    #[test]fn uncertified_paint_and_origin_on_slot_are_unresolved(){
        let(mut r,g,items)=geometry_fixture();
        let mut o=Record::new("ComponentOrigin");o.set("parentId",Value::Uint(items[0].slot)).unwrap();r.push(o);
        assert!(evaluate_with_geometry(&r,&g,&[items[0].slot]).is_err());r.pop();
        let mut shape=Record::new("Shape");shape.set("parentId",Value::Uint(0)).unwrap();r.push(shape);
        assert!(evaluate_with_geometry(&r,&g,&[items[0].slot]).is_err());
    }

    #[test]fn derived_rectangle_paths_are_in_complete_world_matrix_results(){
        use super::super::{wrapping_domains,wrapping_composition,fixed_geometry,wrapping_sizes::MachineInterval};
        let(r,g,items)=geometry_fixture();let roles=items.iter().map(|i|(i.slot,i.visible)).collect::<Vec<_>>();
        let source=g.source();let references=source.roles.iter().map(|(id,s)|(*id,s)).collect::<Vec<_>>();
        let d=wrapping_domains::resolve_layout(&r,source.parent.0,&roles,&source.parent.1,&references,[MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
        let derived=wrapping_composition::compose_with_bounds(d,&[0.5,0.5],100000).unwrap();
        let c=derived.candidate();let source=c.layout_authored().unwrap();
        let references=source.roles.iter().map(|(id,s)|(*id,s)).collect::<Vec<_>>();
        let d=wrapping_domains::resolve_layout(&c.records()[..c.record_costs().0],source.parent.0,&roles,&source.parent.1,&references,[MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
        let actual=fixed_geometry::bind(&d).unwrap();let e=evaluate_derived(&derived,&actual,&[items[0].visible]).unwrap();
        let mut count=0;
        for(index,r)in c.records().iter().enumerate().filter(|(_,r)|r.kind=="Rectangle"){
            count+=1;let id=(index-1)as u32;let parent=uint(r,"parentId",0,id).unwrap();
            let path=e.world_matrix(id).expect("Rectangle is a world-transform owner");
            let shape=e.world_matrix(parent).unwrap();
            assert_eq!(path[4],shape[4]+float(r,"x",0.,id).unwrap());
            assert_eq!(path[5],shape[5]+float(r,"y",0.,id).unwrap());
            assert_eq!(path[..4],[1.,0.,0.,1.]);
        }
        assert!(count>0);
    }

}
