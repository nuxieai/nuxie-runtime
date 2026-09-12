//! Private live rounded rectangle masks. This emits ordinary file objects only.
//! Caller proves raw corner finiteness, identity coordinate space, positive
//! extents, thin-predicate accuracy and artboard coverage; handles are not proof.
use crate::{Diagnostic, wire::{Record, Value}};
use super::paint_rounding;

pub(super) const RECORDS: usize = 2310;
#[derive(Debug)]
pub(super) struct Axis {
    pub saturated: [u32; 2], pub rounding: [paint_rounding::Trace; 2],
    pub size: u32, pub threshold: u32, pub stages: [u32; 4],
    pub minimum: u32, pub final_end: u32,
}
#[derive(Debug)]
pub(super) struct Trace {
    pub start: usize, pub end: usize, pub geometry: u32,
    pub corners: [u32; 2], pub axes: [Axis; 2], pub masks: [u32; 4],
}
fn invalid() -> Diagnostic { Diagnostic::new("paint-box-plan", "paint-box", "Expected ordinary layout geometry and available object-ID capacity") }
/// Selected inputs saturate to [-16384,16384]. Thin flags use raw corners.
/// Appends exactly RECORDS, or restores the complete original record vector.
pub(super) fn emit(records: &mut Vec<Record>, geometry: u32) -> Result<Trace, Diagnostic> {
    if records.first().map(|r|r.kind)!=Some("Backboard") || records.get(1).map(|r|r.kind)!=Some("Artboard")
        || !(geometry as usize).checked_add(1).and_then(|i| records.get(i)).is_some_and(|r|r.kind=="LayoutComponent")
        || records.len().checked_add(RECORDS).filter(|n|*n<=u32::MAX as usize).is_none() { return Err(invalid()); }
    let start=records.len();
    let result=(|| {
        let mut g=Graph { records, y: false };
        let mut corners=[0;2];
        for (i, fraction) in [0.,1.].into_iter().enumerate() {
            let n=g.add("Node",0)?; let c=g.add("TransformConstraint",n)?;
            g.set(c,"targetId",Value::Uint(geometry))?;
            g.set(c,"originX",Value::Float(fraction))?; g.set(c,"originY",Value::Float(fraction))?; corners[i]=n;
        }
        let mut axes=Vec::new();
        for y in [false,true] {
            g.y=y; let mut saturated=[0;2]; let mut rounding=Vec::new();
            for i in 0..2 {
                let n=g.add("Node",0)?; let c=g.copy(n,corners[i],1.,false)?;
                g.clamp(c,-16384.,16384.)?; saturated[i]=n;
                rounding.push(paint_rounding::round(g.records,n,y)?);
            }
            let local=g.add("Node",corners[0])?; g.copy(local,corners[1],1.,false)?;
            let size=g.add("Node",0)?; let c=g.copy(size,local,1.,false)?;
            g.set(c,"sourceSpaceValue",Value::Uint(1))?;
            let threshold=g.add("Node",size)?; g.set(threshold,if y {"y"} else {"x"},Value::Float(-1./16.))?;
            let mut stages=[0;4]; let mut previous=threshold;
            for (i,stage) in stages.iter_mut().enumerate() {
                let n=g.add("Node",0)?; let c=g.copy(n,previous,if i==0 {1.} else {2f32.powi(64)},false)?;
                g.clamp(c,0.,1.)?; *stage=n; previous=n;
            }
            let minimum=g.add("Node",rounding[0].rounded)?; g.copy(minimum,previous,1.,true)?;
            let final_end=g.add("Node",minimum)?; let c=g.copy(final_end,rounding[1].rounded,1.,false)?;
            g.set(c,"minMaxSpaceValue",Value::Uint(1))?;
            g.set(c,if y {"minY"} else {"min"},Value::Bool(true))?;
            g.set(c,if y {"minValueY"} else {"minValue"},Value::Float(0.))?;
            let b=rounding.pop().unwrap(); let a=rounding.pop().unwrap();
            axes.push(Axis{saturated,rounding:[a,b],size,threshold,stages,minimum,final_end});
        }
        let mut masks=[0;4];
        for (axis,a) in axes.iter().enumerate() {
            for (edge,target) in [a.rounding[0].rounded,a.final_end].into_iter().enumerate() {
                let mask=g.add("Shape",target)?; let rect=g.add("Rectangle",mask)?;
                for (key,value) in [("width",32768.),("height",32768.),("x",if axis==0&&edge==1 {-16384.} else {16384.}),("y",if axis==1&&edge==1 {-16384.} else {16384.})] {g.set(rect,key,Value::Float(value))?;}
                masks[axis*2+edge]=mask;
            }
        }
        if g.records.len()!=start+RECORDS { return Err(invalid()); }
        let b=axes.pop().unwrap(); let a=axes.pop().unwrap();
        Ok(Trace{start,end:g.records.len(),geometry,corners,axes:[a,b],masks})
    })();
    if result.is_err() {records.truncate(start);} result
}
struct Graph<'a> {records:&'a mut Vec<Record>, y:bool}
impl Graph<'_> {
    fn add(&mut self,kind:&'static str,parent:u32)->Result<u32,Diagnostic>{let id=self.records.len() as u32-1; let mut r=Record::new(kind); r.set("parentId",Value::Uint(parent))?; self.records.push(r); Ok(id)}
    fn set(&mut self,id:u32,k:&str,v:Value)->Result<(),Diagnostic>{self.records[id as usize+1].set(k,v)}
    fn copy(&mut self,owner:u32,target:u32,factor:f32,local:bool)->Result<u32,Diagnostic>{let c=self.add("TranslationConstraint",owner)?; self.set(c,"targetId",Value::Uint(target))?; self.set(c,"doesCopy",Value::Bool(!self.y))?; self.set(c,"doesCopyY",Value::Bool(self.y))?; self.set(c,if self.y {"copyFactorY"} else {"copyFactor"},Value::Float(factor))?; if local {self.set(c,"destSpaceValue",Value::Uint(1))?;} Ok(c)}
    fn clamp(&mut self,c:u32,lo:f32,hi:f32)->Result<(),Diagnostic>{for (k,v) in [(if self.y {"minY"} else {"min"},Value::Bool(true)),(if self.y {"minValueY"} else {"minValue"},Value::Float(lo)),(if self.y {"maxY"} else {"max"},Value::Bool(true)),(if self.y {"maxValueY"} else {"maxValue"},Value::Float(hi))] {self.set(c,k,v)?;} Ok(())}
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn exact_cost_schema_and_selected_root_saturation() {
  let mut records=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent")];
  let trace=emit(&mut records,1).unwrap(); assert_eq!((trace.start,trace.end),(3,3+RECORDS));
  for (y,a) in [false,true].into_iter().zip(&trace.axes) {for id in a.saturated {assert!(matches!(records[id as usize+1].get("parentId"),Some(Value::Uint(0)))); assert!(matches!(records[id as usize+2].get(if y {"doesCopy"} else {"doesCopyY"}),Some(Value::Bool(false))));}}
  assert!(records[3..].iter().all(|r|matches!(r.kind,"Node"|"TransformConstraint"|"TranslationConstraint"|"Shape"|"Rectangle"))); crate::wire::encode(&records).unwrap();
 }
 #[test] fn invalid_handles_and_roots_do_not_mutate() {
  for case in 0..4 {let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent")]; if case==2 {r[0]=Record::new("Node");} if case==3 {r[1]=Record::new("Node");} let before=crate::wire::encode(&r).unwrap(); assert!(emit(&mut r,if case==0 {u32::MAX} else if case==1 {0} else {1}).is_err()); assert_eq!(crate::wire::encode(&r).unwrap(),before);}
 }
}
