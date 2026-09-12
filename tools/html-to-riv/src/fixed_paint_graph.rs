//! Private constant-paint graph lowering. Keeps actual paths, clipping objects,
//! clip order and draw rules; removes only the evaluated paint-helper graph.
//! Full base+sizing IDs/bytes survive. This is not public CSS admission.
use super::{fixed_geometry, fixed_scalar, wrapping_composition::{Derived, PaintProof}};
use crate::wire::{self, Record, Value};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) enum Unresolved { Source, Route, Span, Fields(u32), Reference(u32),
    Coordinate(u32), Matrix(u32), Scalar(fixed_scalar::Unresolved), Emission }
pub(super) struct Folded {
    source: Vec<u8>,
    geometry: fixed_geometry::Binding,
    records: Vec<Record>,
    mapping: BTreeMap<u32,u32>,
    start: usize,
}
impl Folded {
    pub(super) fn records(&self)->&[Record]{&self.records}
    /// Includes identity entries for every prefix object (Backboard has no ID).
    pub(super) fn mapping(&self)->&BTreeMap<u32,u32>{&self.mapping}
    pub(super) fn start(&self)->usize{self.start}
    pub(super) fn validate(&self,derived:&Derived,geometry:&fixed_geometry::Binding)->Result<(),Unresolved>{
        let source=derived.candidate().records();
        if wire::encode(source).map_err(|_|Unresolved::Source)?!=self.source || !self.geometry.same_binding(geometry){return Err(Unresolved::Source);}
        let plan=plan(derived,geometry)?;
        if self.start!=plan.start || self.mapping!=plan.mapping{return Err(Unresolved::Source);}
        validate_output(source,&self.records,&plan)
    }
}
struct Plan {start:usize,mapping:BTreeMap<u32,u32>,shapes:BTreeMap<u32,[f32;2]>}
fn kept(kind:&str)->bool{matches!(kind,"Shape"|"Rectangle"|"Fill"|"SolidColor"|"ClippingShape"|"DrawRules"|"DrawTarget")}
fn fields(kind:&str)->Option<&'static[&'static str]>{Some(match kind{
    "Shape"=>&["parentId"],"Rectangle"=>&["parentId","x","y","width","height"],
    "Fill"=>&["parentId"],"SolidColor"=>&["parentId","colorValue"],
    "ClippingShape"=>&["parentId","sourceId"],"DrawRules"=>&["parentId","drawTargetId"],
    "DrawTarget"=>&["parentId","drawableId","placementValue"],_=>return None,
})}
fn reference(key:&str)->bool{matches!(key,"parentId"|"sourceId"|"drawTargetId"|"drawableId")}
fn uint(r:&Record,key:&str,id:u32)->Result<u32,Unresolved>{match r.get(key){Some(Value::Uint(v))=>Ok(*v),_=>Err(Unresolved::Fields(id))}}
fn float(r:&Record,key:&str,id:u32)->Result<f32,Unresolved>{match r.get(key){Some(Value::Float(v))if v.is_finite()=>Ok(*v),_=>Err(Unresolved::Fields(id))}}
fn at(records:&[Record],id:u32)->Result<&Record,Unresolved>{(id as usize).checked_add(1).and_then(|i|records.get(i)).ok_or(Unresolved::Reference(id))}
fn remap(plan:&Plan,id:u32)->Result<u32,Unresolved>{plan.mapping.get(&id).copied().ok_or(Unresolved::Reference(id))}
fn same(a:&Value,b:&Value)->bool{match(a,b){
    (Value::Uint(a),Value::Uint(b))|(Value::Color(a),Value::Color(b))=>a==b,
    (Value::Float(a),Value::Float(b))=>a.to_bits()==b.to_bits(),_=>false,
}}
// Immutable Mat2D multiply: preserve FMA order and signed-zero behavior. This
// gate independently models the proposed root reparenting, not an ideal sum.
fn multiply(a:[f32;6],b:[f32;6])->[f32;6]{[
    a[0].mul_add(b[0],a[2]*b[1]),a[1].mul_add(b[0],a[3]*b[1]),
    a[0].mul_add(b[2],a[2]*b[3]),a[1].mul_add(b[2],a[3]*b[3]),
    a[0].mul_add(b[4],a[2]*b[5])+a[4],a[1].mul_add(b[4],a[3]*b[5])+a[5],
]}
fn local(x:f32,y:f32)->[f32;6]{[1.,0.,-0.,1.,x,y]}
fn rooted(x:f32,y:f32)->[f32;6]{multiply([1.,0.,0.,1.,0.,0.],local(x,y))}
fn matrix_match(id:u32,actual:[f32;6],expected:[f32;6])->Result<(),Unresolved>{
    if actual.iter().all(|v|v.is_finite()) && actual.map(f32::to_bits)==expected.map(f32::to_bits){Ok(())}else{Err(Unresolved::Matrix(id))}
}
fn structure(records:&[Record],start:usize)->Result<Plan,Unresolved>{
    if start<2||start>records.len()||records.len()>u32::MAX as usize{return Err(Unresolved::Span);}
    let mut mapping=(0..start-1).map(|id|(id as u32,id as u32)).collect::<BTreeMap<_,_>>();
    let mut next=start-1;
    for (i,r) in records.iter().enumerate().skip(start){
        let id=(i-1)as u32;
        if kept(r.kind){
            let names=fields(r.kind).ok_or(Unresolved::Fields(id))?;
            if !r.has_only_properties(names)||names.iter().any(|k|r.get(k).is_none()){return Err(Unresolved::Fields(id));}
            mapping.insert(id,u32::try_from(next).map_err(|_|Unresolved::Span)?);next+=1;
        }else if !matches!(r.kind,"Node"|"TranslationConstraint"|"TransformConstraint"|"DistanceConstraint"){
            return Err(Unresolved::Fields(id));
        }
    }
    let plan=Plan{start,mapping,shapes:BTreeMap::new()};
    for (i,r) in records.iter().enumerate().skip(start).filter(|(_,r)|kept(r.kind)){
        let id=(i-1)as u32;
        for key in fields(r.kind).unwrap().iter().copied().filter(|k|reference(k)){
            // Only Shape parent links deliberately drop their original helper.
            if r.kind=="Shape"&&key=="parentId"{uint(r,key,id)?;continue;}
            remap(&plan,uint(r,key,id)?)?;
        }
    }
    Ok(plan)
}
fn bind_matrices(records:&[Record],plan:&mut Plan,matrices:&BTreeMap<u32,[f32;6]>)->Result<(),Unresolved>{
    for (i,r) in records.iter().enumerate().skip(plan.start).filter(|(_,r)|r.kind=="Shape"){
        let id=(i-1)as u32;let m=*matrices.get(&id).ok_or(Unresolved::Matrix(id))?;
        if m[..4]!=[1.,0.,0.,1.]||m[4..].iter().any(|v|!v.is_finite()||v.fract()!=0.||v.abs()>65536.){return Err(Unresolved::Coordinate(id));}
        matrix_match(id,rooted(m[4],m[5]),m)?;plan.shapes.insert(id,[m[4],m[5]]);
    }
    for (i,r) in records.iter().enumerate().skip(plan.start).filter(|(_,r)|r.kind=="Rectangle"){
        let id=(i-1)as u32;let parent=uint(r,"parentId",id)?;let translation=plan.shapes.get(&parent).ok_or(Unresolved::Reference(parent))?;
        let x=float(r,"x",id)?;let y=float(r,"y",id)?;
        // The independently bound rounded-paint route uses these exact paths.
        if float(r,"width",id)?!=32768.||float(r,"height",id)?!=32768.||x.abs()!=16384.||y.abs()!=16384.{return Err(Unresolved::Coordinate(id));}
        let proposed=multiply(rooted(translation[0],translation[1]),local(x,y));
        matrix_match(id,proposed,*matrices.get(&id).ok_or(Unresolved::Matrix(id))?)?;
    }
    Ok(())
}
fn plan(derived:&Derived,geometry:&fixed_geometry::Binding)->Result<Plan,Unresolved>{
    let c=derived.candidate();let records=c.records();
    if !matches!(derived.paint(),PaintProof::Ordered(_))||c.original_paint().is_some()||!c.paint_trace().integral.is_empty()
        ||records.iter().any(|r|r.kind=="ForegroundLayoutDrawable"){return Err(Unresolved::Route);}
    let(base,sizing,paint)=c.record_costs();let start=base.checked_add(sizing).ok_or(Unresolved::Span)?;
    if base!=geometry.base_record_count()||start.checked_add(paint)!=Some(records.len()){return Err(Unresolved::Span);}
    let mut plan=structure(records,start)?;
    let outputs:Vec<_>=records.iter().enumerate().skip(start).filter(|(_,r)|matches!(r.kind,"Shape"|"Rectangle")).map(|(i,_)|(i-1)as u32).collect();
    let evaluation=fixed_scalar::evaluate_derived(derived,geometry,&outputs).map_err(Unresolved::Scalar)?;
    if !evaluation.matches_records(records){return Err(Unresolved::Source);}
    bind_matrices(records,&mut plan,evaluation.matrices())?;
    Ok(plan)
}
fn emit(source:&[Record],plan:&Plan)->Result<Vec<Record>,Unresolved>{
    let mut result=source[..plan.start].to_vec();
    for (i,r)in source.iter().enumerate().skip(plan.start).filter(|(_,r)|kept(r.kind)){
        let id=(i-1)as u32;let mut output=r.clone();
        for key in fields(r.kind).unwrap().iter().copied().filter(|k|reference(k)){
            let target=if r.kind=="Shape"&&key=="parentId"{0}else{remap(plan,uint(r,key,id)?)?};
            output.set(key,Value::Uint(target)).map_err(|_|Unresolved::Emission)?;
        }
        if r.kind=="Shape"{let [x,y]=*plan.shapes.get(&id).ok_or(Unresolved::Matrix(id))?;
            output.set("x",Value::Float(x)).map_err(|_|Unresolved::Emission)?;output.set("y",Value::Float(y)).map_err(|_|Unresolved::Emission)?;}
        result.push(output);
    }
    validate_output(source,&result,plan)?;Ok(result)
}
/// Independent actual-output reader: checks complete field presence/absence,
/// ordinal mapping and actual references; never invokes the emitter.
fn validate_output(source:&[Record],output:&[Record],plan:&Plan)->Result<(),Unresolved>{
    if output.len()!=plan.mapping.len()+1||output.len()<plan.start
        ||wire::encode(&source[..plan.start]).map_err(|_|Unresolved::Source)?!=wire::encode(&output[..plan.start]).map_err(|_|Unresolved::Source)?{return Err(Unresolved::Source);}
    let mut next=plan.start;
    for (i,original)in source.iter().enumerate().skip(plan.start).filter(|(_,r)|kept(r.kind)){
        let id=(i-1)as u32;let mapped=remap(plan,id)?;
        if mapped as usize+1!=next{return Err(Unresolved::Reference(id));}let actual=at(output,mapped)?;next+=1;
        let names=if original.kind=="Shape"{&["parentId","x","y"][..]}else{fields(original.kind).unwrap()};
        if actual.kind!=original.kind||!actual.has_only_properties(names){return Err(Unresolved::Fields(id));}
        for key in names{
            let expected=if original.kind=="Shape"&&*key=="parentId"{Value::Uint(0)}
                else if original.kind=="Shape"&&matches!(*key,"x"|"y"){
                    let p=plan.shapes.get(&id).ok_or(Unresolved::Matrix(id))?;Value::Float(p[usize::from(*key=="y")])
                }else if reference(key){Value::Uint(remap(plan,uint(original,key,id)?)?)}
                else{original.get(key).ok_or(Unresolved::Fields(id))?.clone()};
            if !actual.get(key).is_some_and(|v|same(v,&expected)){return Err(Unresolved::Fields(id));}
        }
    }
    if next!=output.len(){return Err(Unresolved::Span);}Ok(())
}
pub(super) fn fold(derived:&Derived,geometry:&fixed_geometry::Binding)->Result<Folded,Unresolved>{
    let p=plan(derived,geometry)?;let source=derived.candidate().records();let records=emit(source,&p)?;
    Ok(Folded{source:wire::encode(source).map_err(|_|Unresolved::Source)?,geometry:geometry.clone(),records,mapping:p.mapping,start:p.start})
}

#[cfg(test)]mod tests{
    use super::*;
    fn add(r:&mut Vec<Record>,kind:&'static str,parent:u32)->u32{let id=r.len()as u32-1;let mut n=Record::new(kind);n.set("parentId",Value::Uint(parent)).unwrap();r.push(n);id}
    fn fixture()->(Vec<Record>,Plan,u32,u32){
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent")];
        let helper=add(&mut r,"Node",0);let s=add(&mut r,"Shape",helper);let q=add(&mut r,"Rectangle",s);
        for(k,v)in[("x",16384.),("y",-16384.),("width",32768.),("height",32768.)]{r[q as usize+1].set(k,Value::Float(v)).unwrap();}
        let fill=add(&mut r,"Fill",s);let color=add(&mut r,"SolidColor",fill);r[color as usize+1].set("colorValue",Value::Color(0x80ee5533)).unwrap();
        let clip=add(&mut r,"ClippingShape",s);r[clip as usize+1].set("sourceId",Value::Uint(s)).unwrap();
        let rule=add(&mut r,"DrawRules",s);let target=add(&mut r,"DrawTarget",rule);r[rule as usize+1].set("drawTargetId",Value::Uint(target)).unwrap();r[target as usize+1].set("drawableId",Value::Uint(s)).unwrap();r[target as usize+1].set("placementValue",Value::Uint(1)).unwrap();
        let mut p=structure(&r,3).unwrap();let matrix=rooted(-7.,23.);let matrices=[(s,matrix),(q,multiply(matrix,local(16384.,-16384.)))].into_iter().collect();bind_matrices(&r,&mut p,&matrices).unwrap();(r,p,s,q)
    }
    #[test]fn literal_path_and_reference_order_survive_only_helper_removed(){
        let(r,p,s,q)=fixture();let output=emit(&r,&p).unwrap();assert_eq!(output.len(),r.len()-1);assert!(!output[3..].iter().any(|r|r.kind=="Node"));
        assert_eq!(p.mapping[&0],0);assert_eq!(p.mapping[&1],1);assert_eq!(p.mapping[&s],s-1);
        let rect=at(&output,p.mapping[&q]).unwrap();assert!(matches!(rect.get("y"),Some(Value::Float(v))if *v==-16384.));validate_output(&r,&output,&p).unwrap();
    }
    #[test]fn every_reference_and_shape_literal_mutation_rejects(){
        let(r,p,_,_)=fixture();let output=emit(&r,&p).unwrap();
        for i in p.start..output.len(){
            for key in ["parentId","sourceId","drawTargetId","drawableId","placementValue"]{
                if output[i].get(key).is_some(){let mut changed=output.clone();changed[i].set(key,Value::Uint(u32::MAX)).unwrap();assert!(validate_output(&r,&changed,&p).is_err(),"{i}.{key}");}
            }
        }
        for key in ["x","y"]{let mut changed=output.clone();changed[3].set(key,Value::Float(0.)).unwrap();assert!(validate_output(&r,&changed,&p).is_err());}
        let mut changed=output.clone();changed[3].set("scaleX",Value::Float(1.)).unwrap();assert!(validate_output(&r,&changed,&p).is_err());
        let mut changed=output.clone();changed.swap(4,5);assert!(validate_output(&r,&changed,&p).is_err());
        let mut changed=output.clone();changed.pop();assert!(validate_output(&r,&changed,&p).is_err());
        let mut changed=output;changed[2].set("x",Value::Float(1.)).unwrap();assert!(validate_output(&r,&changed,&p).is_err());
    }
    #[test]fn signed_zero_and_rectangle_matrix_hazards_fail_closed(){
        let(r,_,s,q)=fixture();let shape=rooted(-7.,23.);let rectangle=multiply(shape,local(16384.,-16384.));
        for index in [0,1,2,3,4,5]{let mut altered=rectangle;altered[index]=if altered[index]==0.{-0.}else{altered[index]+1.};
            let mut p=structure(&r,3).unwrap();let matrices=[(s,shape),(q,altered)].into_iter().collect();assert!(bind_matrices(&r,&mut p,&matrices).is_err());}
        let mut minus_zero=shape;minus_zero[2]=-0.;assert_eq!(minus_zero,shape);assert!(matrix_match(s,rooted(shape[4],shape[5]),minus_zero).is_err());
        for x in [0.5,65537.,f32::INFINITY]{let mut p=structure(&r,3).unwrap();let matrices=[(s,rooted(x,23.)),(q,rectangle)].into_iter().collect();assert!(bind_matrices(&r,&mut p,&matrices).is_err());}
    }
    #[test]fn references_to_dropped_helpers_and_unrecognized_fields_reject(){
        let(r,_,s,q)=fixture();let mut changed=r.clone();changed[q as usize+1].set("parentId",Value::Uint(2)).unwrap();assert!(structure(&changed,3).is_err());
        let mut changed=r.clone();changed[s as usize+1].set("x",Value::Float(0.)).unwrap();assert!(structure(&changed,3).is_err());
        let mut changed=r.clone();changed.push(Record::new("Image"));assert!(structure(&changed,3).is_err());
        let mut p=structure(&r,3).unwrap();p.mapping.remove(&s);assert!(emit(&r,&p).is_err());
    }
}
