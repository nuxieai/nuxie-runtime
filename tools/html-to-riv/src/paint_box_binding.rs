//! Independent closed reader for ordinary rounded-box paint records.
//! Parsing binds actual fields and absent defaults; it does not establish numeric domains.
use crate::wire::{Record,Value};
#[derive(Debug,PartialEq,Eq)]
pub(super) struct Unresolved { pub position:usize }
#[derive(Debug)]
struct Step {power:u32,threshold:u32,difference:u32,stages:[u32;4],bit:u32,accumulator:u32}
#[derive(Debug)]
struct Rounding {rounded:u32,positive:Vec<Step>,negative:Vec<Step>}
impl Rounding {fn matches(&self,t:&super::paint_rounding::Trace)->bool {
 fn steps(a:&[Step],b:&[super::paint_rounding::Step])->bool{a.len()==b.len()&&a.iter().zip(b).all(|(a,b)|a.power==b.power&&a.threshold==b.threshold&&a.difference==b.difference&&a.stages==b.stages&&a.bit==b.bit&&a.accumulator==b.accumulator)}
 self.rounded==t.rounded&&steps(&self.positive,&t.positive)&&steps(&self.negative,&t.negative)
}}
#[derive(Debug)]
pub(super) struct Axis {pub saturated:[u32;2],pub rounded:[u32;2],pub size:u32,pub threshold:u32,pub stages:[u32;4],pub minimum:u32,pub final_end:u32,rounding:[Rounding;2]}
#[derive(Debug)]
pub(super) struct Binding {pub start:usize,pub next:usize,pub geometry:u32,pub corners:[u32;2],pub axes:[Axis;2],pub masks:[u32;4]}
struct Reader<'a>{records:&'a[Record],next:usize,end:usize,y:bool}
fn same(a:Option<&Value>,b:&Value)->bool{match(a,b){(Some(Value::Uint(a)),Value::Uint(b))=>a==b,(Some(Value::Float(a)),Value::Float(b))=>a.to_bits()==b.to_bits(),(Some(Value::Bool(a)),Value::Bool(b))=>a==b,_=>false}}
impl Reader<'_>{
 fn fail(&self)->Unresolved{Unresolved{position:self.next}}
 fn take(&mut self,kind:&str,fields:&[(&str,Value)])->Result<u32,Unresolved>{
  let r=self.records.get(self.next).filter(|_|self.next<self.end).ok_or_else(||self.fail())?;
  if r.kind!=kind || !r.has_only_properties(&fields.iter().map(|(k,_)|*k).collect::<Vec<_>>()) || !fields.iter().all(|(k,v)|same(r.get(k),v)){return Err(self.fail());}
  let id=self.next.checked_sub(1).and_then(|n|u32::try_from(n).ok()).ok_or_else(||self.fail())?;self.next+=1;Ok(id)
 }
 fn node(&mut self,parent:u32,value:Option<f32>)->Result<u32,Unresolved>{let mut f=vec![("parentId",Value::Uint(parent))];if let Some(v)=value{f.push((if self.y{"y"}else{"x"},Value::Float(v)));}self.take("Node",&f)}
 fn copy(&mut self,parent:u32,target:u32,factor:f32,local:bool,source_local:bool,clamp:Option<(f32,f32)>,local_min:bool)->Result<(),Unresolved>{
  let mut f=vec![("parentId",Value::Uint(parent)),("targetId",Value::Uint(target)),("doesCopy",Value::Bool(!self.y)),("doesCopyY",Value::Bool(self.y)),(if self.y{"copyFactorY"}else{"copyFactor"},Value::Float(factor))];
  if local{f.push(("destSpaceValue",Value::Uint(1)));}if source_local{f.push(("sourceSpaceValue",Value::Uint(1)));}
  if let Some((lo,hi))=clamp{f.extend([(if self.y{"minY"}else{"min"},Value::Bool(true)),(if self.y{"minValueY"}else{"minValue"},Value::Float(lo)),(if self.y{"maxY"}else{"max"},Value::Bool(true)),(if self.y{"maxValueY"}else{"maxValue"},Value::Float(hi))]);}
  if local_min{f.extend([("minMaxSpaceValue",Value::Uint(1)),(if self.y{"minY"}else{"min"},Value::Bool(true)),(if self.y{"minValueY"}else{"minValue"},Value::Float(0.))]);}
  self.take("TranslationConstraint",&f)?;Ok(())
 }
 fn scale(&mut self,target:u32,factor:f32,clamp:Option<(f32,f32)>)->Result<u32,Unresolved>{let n=self.node(0,None)?;self.copy(n,target,factor,false,false,clamp,false)?;Ok(n)}
 fn sum(&mut self,a:u32,b:u32)->Result<u32,Unresolved>{let n=self.node(a,None)?;self.copy(n,b,1.,true,false,None,false)?;Ok(n)}
 fn diff(&mut self,a:u32,b:u32)->Result<u32,Unresolved>{let minus=self.scale(b,-1.,None)?;self.sum(a,minus)}
 fn positive(&mut self,source:u32)->Result<[u32;4],Unresolved>{let mut stages=[0;4];let mut previous=source;for(i,n)in stages.iter_mut().enumerate(){*n=self.scale(previous,if i==0{1.}else{18446744073709551616.},Some((0.,1.)))?;previous=*n;}Ok(stages)}
 fn round(&mut self,source:u32)->Result<Rounding,Unresolved>{
  let zero=self.node(0,Some(0.))?;let negative_source=self.scale(source,-1.,None)?;let mut finals=[0;2];let mut passes=Vec::new();
  for negative in [false,true]{let mut acc=zero;let mut steps=Vec::new();
   for k in (0..=14).rev(){let power=(1u32<<k)as f32;let threshold=self.node(acc,Some(power-0.5))?;
    let difference=if negative{self.diff(negative_source,threshold)?}else{self.diff(threshold,source)?};let stages=self.positive(difference)?;
    let bit=if negative{stages[3]}else{let n=self.scale(stages[3],-1.,None)?;self.node(n,Some(1.))?};
    let increment=self.scale(bit,power,None)?;acc=self.sum(acc,increment)?;steps.push(Step{power:power as u32,threshold,difference,stages,bit,accumulator:acc});
   }finals[usize::from(negative)]=acc;passes.push(steps);
  }let diff=self.diff(finals[0],finals[1])?;let rounded=self.scale(diff,1.,None)?;let negative=passes.pop().unwrap();let positive=passes.pop().unwrap();Ok(Rounding{rounded,positive,negative})
 }
 fn axis(&mut self,corners:[u32;2])->Result<Axis,Unresolved>{
  let mut saturated=[0;2];let mut rounded=[0;2];let mut traces=Vec::new();for i in 0..2{saturated[i]=self.scale(corners[i],1.,Some((-16384.,16384.)))?;let trace=self.round(saturated[i])?;rounded[i]=trace.rounded;traces.push(trace);}
  let local=self.node(corners[0],None)?;self.copy(local,corners[1],1.,false,false,None,false)?;
  let size=self.node(0,None)?;self.copy(size,local,1.,false,true,None,false)?;
  let threshold=self.node(size,Some(-1./16.))?;let stages=self.positive(threshold)?;
  let minimum=self.sum(rounded[0],stages[3])?;let final_end=self.node(minimum,None)?;self.copy(final_end,rounded[1],1.,false,false,None,true)?;
  let second=traces.pop().unwrap();let first=traces.pop().unwrap();Ok(Axis{saturated,rounded,size,threshold,stages,minimum,final_end,rounding:[first,second]})
 }
}
/// Consumes one box at `start`, bounded by the containing suffix `end`.
/// The caller binds returned masks into paint replicas and enforces complete suffix consumption.
pub(super) fn consume(records:&[Record],start:usize,end:usize,owner:u32)->Result<Binding,Unresolved>{
 let mut r=Reader{records,next:start,end,y:false};
 if start<2 || start>end || end>records.len() || records.first().map(|r|r.kind)!=Some("Backboard") || records.get(1).map(|r|r.kind)!=Some("Artboard") || !(owner as usize).checked_add(1).filter(|&i|i<start).and_then(|i|records.get(i)).is_some_and(|r|r.kind=="LayoutComponent"){return Err(r.fail());}
 let mut corners=[0;2];for(i,n)in corners.iter_mut().enumerate(){*n=r.node(0,None)?;r.take("TransformConstraint",&[("parentId",Value::Uint(*n)),("targetId",Value::Uint(owner)),("originX",Value::Float(i as f32)),("originY",Value::Float(i as f32))])?;}
 let x=r.axis(corners)?;r.y=true;let y=r.axis(corners)?;let axes=[x,y];let mut masks=[0;4];
 for(axis,a)in axes.iter().enumerate(){for(trailing,target)in [a.rounded[0],a.final_end].into_iter().enumerate(){let shape=r.take("Shape",&[("parentId",Value::Uint(target))])?;r.take("Rectangle",&[("parentId",Value::Uint(shape)),("width",Value::Float(32768.)),("height",Value::Float(32768.)),("x",Value::Float(if axis==0&&trailing==1{-16384.}else{16384.})),("y",Value::Float(if axis==1&&trailing==1{-16384.}else{16384.}))])?;masks[2*axis+trailing]=shape;}}
 Ok(Binding{start,next:r.next,geometry:owner,corners,axes,masks})
}

impl Binding {
 pub(super) fn matches_trace(&self,t:&super::paint_box::Trace)->bool {
  self.start==t.start&&self.next==t.end&&self.geometry==t.geometry&&self.corners==t.corners&&self.masks==t.masks&&self.axes.iter().zip(&t.axes).all(|(a,b)|a.saturated==b.saturated&&a.rounding.iter().zip(&b.rounding).all(|(a,b)|a.matches(b))&&a.size==b.size&&a.threshold==b.threshold&&a.stages==b.stages&&a.minimum==b.minimum&&a.final_end==b.final_end)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn binds_records_and_trace_and_rejects_mutations(){
  let mut records=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent"),Record::new("LayoutComponent")];let start=records.len();
  let mut trace=super::super::paint_box::emit(&mut records,1).unwrap();let bound=consume(&records,start,records.len(),1).unwrap();assert!(bound.matches_trace(&trace));
  assert!(consume(&records,start,records.len(),2).is_err());assert!(consume(&records,start,records.len()-1,1).is_err());
  // Every appended operation must be consumed. Each field mutation is checked
  // against actual records, never a second invocation of the emitter.
  for index in start..records.len(){let mut changed=records.clone();changed[index].set("parentId",Value::Uint(u32::MAX)).unwrap();assert!(consume(&changed,start,changed.len(),1).is_err(),"record {index}");}
  let mut mutations=vec![(trace.corners[0]+1,"targetId",Value::Uint(2)),(trace.corners[1]+1,"originY",Value::Float(0.))];
  for a in &trace.axes{
   for id in a.saturated{mutations.push((id+1,"minMaxSpaceValue",Value::Uint(1)));mutations.push((id+1,"strength",Value::Float(0.5)));}
   for r in &a.rounding{for s in r.positive.iter().chain(&r.negative){mutations.push((s.threshold,"x",Value::Float(99.)));mutations.push((s.difference+1,"targetId",Value::Uint(1)));for id in s.stages{mutations.push((id+1,"doesCopy",Value::Bool(false)));mutations.push((id+1,"copyFactor",Value::Float(3.)));}}}
   mutations.extend([(a.size+1,"sourceSpaceValue",Value::Uint(0)),(a.threshold,"x",Value::Float(0.)),(a.minimum+1,"destSpaceValue",Value::Uint(0)),(a.final_end+1,"minMaxSpaceValue",Value::Uint(0))]);
  }
  // doesCopy=false is already correct on y; force the opposite of each actual value.
  for(id,key,mut value)in mutations{let mut changed=records.clone();let i=id as usize+1;if key=="doesCopy"{if let Some(Value::Bool(v))=changed[i].get(key){value=Value::Bool(!v);}}changed[i].set(key,value).unwrap();assert!(consume(&changed,start,changed.len(),1).is_err(),"{id}.{key}");}
  trace.axes[0].rounding[0].positive[0].difference+=1;assert!(!bound.matches_trace(&trace));
 }
}
