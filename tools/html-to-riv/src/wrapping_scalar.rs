//! Structural binding of the sizing arithmetic to actual ordinary records.
//! This checks a closed scalar and visible-anchor instruction grammar, including fields omitted
//! to use immutable native defaults. It does not prove paint gates, runtime
//! evaluation scheduling, arithmetic bounds or CSS visible-size equivalence.
use super::wrapping::{Boundary, Trace};
use crate::wire::{Record, Value};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Unresolved { pub object: usize }
#[derive(Debug)]
pub(super) struct Binding { pub nodes: usize, pub constraints: usize, pub origins: usize }
struct Reader<'a> { records:&'a[Record], next:usize, end:usize, row:bool, nodes:usize, constraints:usize, origins:usize }
impl Reader<'_> {
    fn fail(&self)->Unresolved {Unresolved{object:self.next.saturating_sub(1)}}
    fn take(&mut self,kind:&str,fields:&[(&str,Value)])->Result<u32,Unresolved> {
        let r=self.records.get(self.next).filter(|_|self.next<self.end).ok_or_else(||self.fail())?;
        // Reject every unmodelled property, even when it appears harmless.
        // Omitted transform fields therefore retain native identity/defaults;
        // omitted constraint strength is exactly 1 in the pinned consumer.
        if r.kind!=kind || !r.has_only_properties(&fields.iter().map(|(k,_)|*k).collect::<Vec<_>>())
            || fields.iter().any(|(k,v)| !same(r.get(k),v)) {return Err(self.fail());}
        let id=u32::try_from(self.next-1).map_err(|_|self.fail())?;
        self.next+=1;
        if kind=="Node" {self.nodes+=1;}else if kind=="ComponentOrigin" {self.origins+=1;}else{self.constraints+=1;}
        Ok(id)
    }
    fn node(&mut self,parent:u32,value:Option<f32>)->Result<u32,Unresolved> {
        let mut fields=vec![("parentId",Value::Uint(parent))];
        if let Some(v)=value {fields.push((if self.row {"y"}else{"x"},Value::Float(v)));}
        self.take("Node",&fields)
    }
    fn copy(&mut self,owner:u32,target:u32,factor:f32,local:bool,minimum:bool,local_clamp:bool,maximum:bool)->Result<(),Unresolved> {
        let mut fields=vec![("parentId",Value::Uint(owner)),("targetId",Value::Uint(target)),
            ("doesCopy",Value::Bool(!self.row)),("doesCopyY",Value::Bool(self.row)),
            (if self.row {"copyFactorY"}else{"copyFactor"},Value::Float(factor))];
        if local {fields.push(("destSpaceValue",Value::Uint(1)));}
        if minimum {fields.extend([(if self.row {"minY"}else{"min"},Value::Bool(true)),
            (if self.row {"minValueY"}else{"minValue"},Value::Float(0.))]);}
        if local_clamp {fields.push(("minMaxSpaceValue",Value::Uint(1)));}
        if maximum {fields.extend([(if self.row {"maxY"}else{"max"},Value::Bool(true)),
            (if self.row {"maxValueY"}else{"maxValue"},Value::Float(65536.))]);}
        self.take("TranslationConstraint",&fields)?;Ok(())
    }
    fn scale(&mut self,target:u32,factor:f32)->Result<u32,Unresolved> {
        let id=self.node(0,None)?;self.copy(id,target,factor,false,false,false,false)?;Ok(id)
    }
    fn sum(&mut self,a:u32,b:u32)->Result<u32,Unresolved> {
        let id=self.node(a,None)?;self.copy(id,b,1.,true,false,false,false)?;Ok(id)
    }
    fn diff(&mut self,a:u32,b:u32)->Result<u32,Unresolved> {
        let negative=self.scale(b,-1.)?;self.sum(a,negative)
    }
    fn relu(&mut self,a:u32)->Result<u32,Unresolved> {
        let id=self.node(0,None)?;self.copy(id,a,1.,false,true,false,false)?;Ok(id)
    }
    fn max(&mut self,a:u32,b:u32)->Result<u32,Unresolved> {
        let id=self.node(a,None)?;self.copy(id,b,1.,false,true,true,false)?;Ok(id)
    }
    fn landmark(&mut self,slot:u32,fraction:f32)->Result<u32,Unresolved> {
        let id=self.node(0,None)?;
        self.take("TransformConstraint",&[("parentId",Value::Uint(id)),("targetId",Value::Uint(slot)),
            (if self.row {"originY"}else{"originX"},Value::Float(fraction))])?;
        self.scale(id,1.)
    }
    fn boundary(&mut self,a:u32,b:u32,origin:u32,epsilon:f32)->Result<Boundary,Unresolved> {
        let difference=self.diff(a,b)?;
        let negative=self.scale(difference,-1.)?;
        let absolute=self.max(difference,negative)?;
        let constant=self.node(0,Some(-epsilon))?;
        let shifted=self.sum(absolute,constant)?;
        let dead=self.relu(shifted)?;
        let normalized=self.scale(dead,1.)?;
        self.take("DistanceConstraint",&[("parentId",Value::Uint(normalized)),("targetId",Value::Uint(origin)),
            ("modeValue",Value::Uint(2)),("distance",Value::Float(65536.))])?;
        let gate=self.node(0,None)?;
        self.copy(gate,normalized,2.,false,false,false,true)?;
        Ok(Boundary{difference,absolute,dead,normalized,gate})
    }
}
fn same(actual:Option<&Value>,expected:&Value)->bool {
    match (actual,expected) {
        (Some(Value::Uint(a)),Value::Uint(b))=>a==b,
        (Some(Value::Float(a)),Value::Float(b))=>a.to_bits()==b.to_bits(),
        (Some(Value::Bool(a)),Value::Bool(b))=>a==b,
        _=>false,
    }
}

/// Bind every sizing record and trace operand to the native scalar operation
/// ledger. No call to the emitter or wire re-encoding supplies the expectation.
/// The checked base supplies independent, identity-transform measurement slots.
/// Forward references and extra instructions cannot fit this closed grammar.
pub(super) fn bind(records:&[Record],start:usize,end:usize,roles:&[(u32,u32)],alignments:&[f32],
    row:bool,reverse_cross:bool,line_fraction:f32,epsilon:f32,trace:&Trace)->Result<Binding,Unresolved> {
    let mut r=Reader{records,next:start,end,row,nodes:0,constraints:0,origins:0};
    if start==0 || end>records.len() || start>end || roles.is_empty() || roles.len()!=alignments.len()
        || ![0.,0.5,1.].contains(&line_fraction) || alignments.iter().any(|a|![0.,0.5,1.].contains(a))
        || !epsilon.is_finite() || !(0. ..65536.).contains(&epsilon)
        || roles.iter().any(|&(s,v)|s as usize>=start-1 || v as usize>=start-1 || s==v) {return Err(r.fail());}
    let fraction=if reverse_cross {1.-line_fraction}else{line_fraction};
    let origin=r.node(0,None)?;
    let mut expected=Trace{origin:Some(origin),..Trace::default()};
    for &(slot,_) in roles {
        let top=r.landmark(slot,0.)?;let bottom=r.landmark(slot,1.)?;
        expected.tops.push(top);expected.heights.push(r.diff(bottom,top)?);
        expected.anchors.push(r.landmark(slot,fraction)?);
    }
    for pair in expected.anchors.windows(2) {expected.boundaries.push(r.boundary(pair[0],pair[1],origin,epsilon)?);}
    expected.forward.push(expected.heights[0]);
    for i in 1..roles.len() {
        let delta=r.diff(expected.forward[i-1],expected.boundaries[i-1].gate)?;
        let carry=r.relu(delta)?;expected.forward.push(r.max(carry,expected.heights[i])?);
    }
    expected.backward=vec![0;roles.len()];
    expected.backward[roles.len()-1]=expected.heights[roles.len()-1];
    for i in (0..roles.len()-1).rev() {
        let delta=r.diff(expected.backward[i+1],expected.boundaries[i].gate)?;
        let carry=r.relu(delta)?;expected.backward[i]=r.max(carry,expected.heights[i])?;
    }
    for (i,&(_,visible)) in roles.iter().enumerate() {
        let maximum=r.max(expected.forward[i],expected.backward[i])?;
        let alignment=if reverse_cross {1.-alignments[i]}else{alignments[i]};
        let offset=r.scale(maximum,alignment-fraction)?;
        let target=r.sum(expected.anchors[i],offset)?;
        r.take("ComponentOrigin",&[("parentId",Value::Uint(visible)),
            ("originX",Value::Float(if row {0.}else{alignment})),
            ("originY",Value::Float(if row {alignment}else{0.}))])?;
        r.copy(visible,target,1.,false,false,false,false)?;
        expected.line_maxima.push(maximum);expected.offsets.push(offset);expected.targets.push(target);
    }
    if r.next!=end || expected!=*trace {return Err(r.fail());}
    Ok(Binding{nodes:r.nodes,constraints:r.constraints,origins:r.origins})
}
