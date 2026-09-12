//! Independent constant evaluation of closed ordinary translation-only graphs.
//!
//! No runtime dependency, viewport sample, layout measurement or caller-provided
//! seed is used. Results are bound to the complete encoded input. This first
//! evaluator models only bare identity artboards, literal Nodes and strength-one
//! TranslationConstraints. TransformConstraint needs bounds/decomposition proofs;
//! DistanceConstraint needs distance/threshold arithmetic. Both remain unresolved,
//! even for Node targets. This is a folding foundation, not public admission.
use crate::wire::{self, Record, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Encoding, Empty, InvalidHandle(u32), Unsupported(u32),
    Fields(u32), NonFinite(u32), Cycle(u32), Depth }
#[derive(Clone, Debug)]
pub(super) struct Evaluation { encoded: Vec<u8>, values: BTreeMap<u32,[f32;2]> }
impl Evaluation {
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
struct Reader<'a>{records:&'a[Record],children:BTreeMap<u32,Vec<u32>>,states:Vec<u8>,world:BTreeMap<u32,Matrix>}
impl Reader<'_>{
    fn record(&self,id:u32)->Result<&Record,Unresolved>{self.records.get(id as usize+1).ok_or(Unresolved::InvalidHandle(id))}
    fn visit(&mut self,id:u32,depth:usize)->Result<Matrix,Unresolved>{
        if depth>1024{return Err(Unresolved::Depth);}
        let state=*self.states.get(id as usize).ok_or(Unresolved::InvalidHandle(id))?;
        if state==1{return Err(Unresolved::Cycle(id));}
        if state==2{return Ok(self.world[&id]);}
        self.states[id as usize]=1;
        let r=self.record(id)?.clone();
        let mut world=if id==0 && r.kind=="Artboard"{
            if !r.has_only_properties(&["name","width","height"]){return Err(Unresolved::Fields(id));}
            float(&r,"width",0.,id)?;float(&r,"height",0.,id)?;
            [1.,0.,0.,1.,0.,0.]
        }else{
            if r.kind!="Node"{return Err(Unresolved::Unsupported(id));}
            if !r.has_only_properties(&["name","parentId","x","y"]){return Err(Unresolved::Fields(id));}
            let parent=uint(&r,"parentId",0,id)?;
            let parent_world=self.visit(parent,depth+1)?;
            // Default rotation=+0, scale=(1,1); from_rotation preserves -sin(0).
            let local=[1.,0.,-0.,1.,float(&r,"x",0.,id)?,float(&r,"y",0.,id)?];
            finite(multiply(parent_world,local),id)?
        };
        for child in self.children.get(&id).cloned().unwrap_or_default(){
            let c=self.record(child)?.clone();
            match c.kind{
                "Node"=>{}, // Child positions cannot affect their Node parent.
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
        let literal=[float(node,"x",0.,owner)?,float(node,"y",0.,owner)?];
        let mut value=original;
        if target!=u32::MAX{
            let mut target_world=self.visit(target,depth)?;
            if source==1{
                // Artboard targets are excluded: only ordinary Node parents
                // have the independently modeled source-local relationship.
                let t=self.record(target)?.clone();
                if t.kind!="Node"{return Err(Unresolved::Unsupported(target));}
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
        let owner=records.get(parent as usize+1).ok_or(Unresolved::InvalidHandle(parent))?;
        if !(owner.kind=="Node" || (r.kind=="Node" && parent==0 && owner.kind=="Artboard")){
            return Err(Unresolved::Unsupported(parent));
        }
        children.entry(parent).or_default().push(id);
    }
    let mut reader=Reader{records,children,states:vec![0;records.len()-1],world:BTreeMap::new()};
    for id in outputs{reader.visit(*id,0)?;}
    // The certificate binds a complete closed file, not only the requested
    // dependency closure. Unvisited dangling objects/cycles must not turn an
    // import-invalid scene into an apparently resolved constant evaluation.
    for (index,r) in records.iter().enumerate().skip(2){
        if r.kind=="Node"{reader.visit((index-1)as u32,0)?;}
    }
    Ok(Evaluation{encoded,values:reader.world.into_iter().map(|(id,m)|(id,[m[4],m[5]])).collect()})
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

}
