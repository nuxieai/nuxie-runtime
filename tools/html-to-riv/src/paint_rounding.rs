//! Private signed scalar paint rounding, using ordinary constraints only.
//! This is not public CSS admission. Native f32 rounding is distinct from
//! Chrome LayoutUnit quantization and from qualification of rounded paint.
use crate::{Diagnostic, wire::{Record, Value}};
#[derive(Debug)]
pub(super) struct Step { pub power: u32, pub threshold:u32, pub difference:u32,
    pub stages:[u32;4], pub bit:u32, pub accumulator:u32 }
#[derive(Debug)]
pub(super) struct Trace { pub rounded:u32, pub positive:Vec<Step>, pub negative:Vec<Step> }
pub(super) const RECORDS:usize = 564;
/// Caller must prove the source's selected world coordinate stays finite in
/// [-16384,16384], with identity artboard space and no dependency back from
/// source to this graph. Source must be an ordinary Node. Unselected components
/// are not an output. Half ties round toward positive infinity. Read-only trace
/// IDs are observation handles, never a required runtime sidecar.
/// Invalid handles and object-ID capacity fail before mutation. Record fields
/// are schema checked; emission is transactional even on unexpected errors.
pub(super) fn round(records:&mut Vec<Record>,source:u32,y:bool)->Result<Trace,Diagnostic>{
 let invalid=||Diagnostic::new("paint-rounding-plan","paint-rounding","Expected ordinary Node source and available object-ID capacity");
 if records.first().map(|r|r.kind)!=Some("Backboard") || records.get(1).map(|r|r.kind)!=Some("Artboard") || !(source as usize).checked_add(1).and_then(|i|records.get(i)).is_some_and(|r|r.kind=="Node") || records.len().checked_add(RECORDS).filter(|n|*n<=u32::MAX as usize).is_none(){return Err(invalid());}
 let start=records.len();let result=(||{
 let mut g=Graph{records,y};let zero=g.constant(0,0.)?;let negative_source=g.scale(source,-1.)?;
 let mut passes=Vec::new();
 for negative in [false,true]{let mut a=zero;let mut steps=Vec::new();
  for k in (0..=14).rev(){let power=1u32<<k;let threshold=g.constant(a,power as f32-0.5)?;
   let difference=if negative{g.diff(negative_source,threshold)?}else{g.diff(threshold,source)?};
   let mut stages=[0;4];let mut prev=difference;
   for i in 0..4{let n=g.scale(prev,if i==0{1.}else{2f32.powi(64)})?;let c=n+1;
    for(k,v)in[(if y{"minY"}else{"min"},Value::Bool(true)),(if y{"minValueY"}else{"minValue"},Value::Float(0.)),(if y{"maxY"}else{"max"},Value::Bool(true)),(if y{"maxValueY"}else{"maxValue"},Value::Float(1.))]{g.set(c,k,v)?;}stages[i]=n;prev=n;
   }
   let bit=if negative{prev}else{let n=g.scale(prev,-1.)?;g.constant(n,1.)?};
   let increment=g.scale(bit,power as f32)?;a=g.sum(a,increment)?;
   steps.push(Step{power,threshold,difference,stages,bit,accumulator:a});
  }passes.push(steps);
 }
 let difference=g.diff(passes[0].last().unwrap().accumulator,passes[1].last().unwrap().accumulator)?;let rounded=g.scale(difference,1.)?;
 if g.records.len()!=start+RECORDS{return Err(invalid());}
 let negative=passes.pop().unwrap();let positive=passes.pop().unwrap();Ok(Trace{rounded,positive,negative})
 })();if result.is_err(){records.truncate(start);}result
}
struct Graph<'a>{records:&'a mut Vec<Record>,y:bool}
impl Graph<'_>{
 fn add(&mut self,kind:&'static str,parent:u32)->Result<u32,Diagnostic>{let id=self.records.len()as u32-1;let mut r=Record::new(kind);r.set("parentId",Value::Uint(parent))?;self.records.push(r);Ok(id)}
 fn set(&mut self,id:u32,k:&str,v:Value)->Result<(),Diagnostic>{self.records[id as usize+1].set(k,v)}
 fn constant(&mut self,parent:u32,v:f32)->Result<u32,Diagnostic>{let n=self.add("Node",parent)?;self.set(n,if self.y{"y"}else{"x"},Value::Float(v))?;Ok(n)}
 fn copy(&mut self,n:u32,target:u32,factor:f32,local:bool)->Result<(),Diagnostic>{let c=self.add("TranslationConstraint",n)?;self.set(c,"targetId",Value::Uint(target))?;self.set(c,"doesCopy",Value::Bool(!self.y))?;self.set(c,"doesCopyY",Value::Bool(self.y))?;self.set(c,if self.y{"copyFactorY"}else{"copyFactor"},Value::Float(factor))?;if local{self.set(c,"destSpaceValue",Value::Uint(1))?;}Ok(())}
 fn scale(&mut self,target:u32,factor:f32)->Result<u32,Diagnostic>{let n=self.add("Node",0)?;self.copy(n,target,factor,false)?;Ok(n)}
 fn sum(&mut self,a:u32,b:u32)->Result<u32,Diagnostic>{let n=self.add("Node",a)?;self.copy(n,b,1.,true)?;Ok(n)}
 fn diff(&mut self,a:u32,b:u32)->Result<u32,Diagnostic>{let minus=self.scale(b,-1.)?;self.sum(a,minus)}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn invalid_handle_is_transactional(){let mut r=vec![Record::new("Backboard"),Record::new("Artboard")];let before=crate::wire::encode(&r).unwrap();assert!(round(&mut r,u32::MAX,false).is_err());assert_eq!(crate::wire::encode(&r).unwrap(),before);assert!(round(&mut r,0,false).is_err());assert_eq!(crate::wire::encode(&r).unwrap(),before);}
 #[test]fn exact_cost_and_ordinary_schema(){for y in[false,true]{let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("Node")];let trace=round(&mut r,1,y).unwrap();assert_eq!(r.len(),3+RECORDS);assert_eq!(trace.positive.len(),15);assert_eq!(trace.negative.len(),15);assert!(matches!(r[trace.rounded as usize+1].get("parentId"),Some(Value::Uint(0))));assert!(r[3..].iter().all(|r|matches!(r.kind,"Node"|"TranslationConstraint")));crate::wire::encode(&r).unwrap();}}
}
