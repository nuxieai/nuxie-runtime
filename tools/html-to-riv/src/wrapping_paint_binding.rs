//! Closed structural reader for the private one-solid-paint wrapping composition.
//! This binds records and omitted defaults, not numeric bounds, scheduling or pixels.
use crate::wire::{Record, Value};
use std::collections::{BTreeMap, BTreeSet};
const D:f32=65536.;
#[derive(Debug,PartialEq,Eq)]
pub(super) struct Unresolved { pub position:usize }
#[derive(Debug)]
pub(super) struct Binding { pub signals:usize, pub masks:usize, pub replicas:usize, pub boxes:Vec<super::paint_box_binding::Binding> }
struct Reader<'a>{ records:&'a[Record], next:usize, end:usize, row:bool, epsilon:f32,
    sources:BTreeMap<u32,u32>, signals:BTreeMap<(u32,u32),u32>, masks:usize }
fn same(actual:Option<&Value>,v:&Value)->bool {match(actual,v){
    (Some(Value::Uint(a)),Value::Uint(b))=>a==b,
    (Some(Value::Float(a)),Value::Float(b))=>a.to_bits()==b.to_bits(),
    (Some(Value::Bool(a)),Value::Bool(b))=>a==b,
    (Some(Value::Color(a)),Value::Color(b))=>a==b,_=>false}}
fn exact(r:&Record,kind:&str,fields:&[(&str,Value)])->bool {
    r.kind==kind && r.has_only_properties(&fields.iter().map(|(k,_)|*k).collect::<Vec<_>>())
        && fields.iter().all(|(k,v)|same(r.get(k),v))
}
impl Reader<'_>{
    fn fail(&self)->Unresolved{Unresolved{position:self.next}}
    fn take(&mut self,kind:&str,fields:&[(&str,Value)])->Result<u32,Unresolved>{
        let r=self.records.get(self.next).filter(|_|self.next<self.end).ok_or_else(||self.fail())?;
        if !exact(r,kind,fields){return Err(self.fail());}
        let id=u32::try_from(self.next-1).map_err(|_|self.fail())?;self.next+=1;Ok(id)
    }
    fn node(&mut self,parent:u32,constant:Option<f32>)->Result<u32,Unresolved>{
        let mut f=vec![("parentId",Value::Uint(parent))];
        if let Some(v)=constant {f.push((if self.row{"y"}else{"x"},Value::Float(v)));}
        self.take("Node",&f)
    }
    fn translation(&mut self,parent:u32,target:u32,local:bool,inverse:bool)->Result<(),Unresolved>{
        let mut f=vec![("parentId",Value::Uint(parent)),("targetId",Value::Uint(target)),
            ("doesCopy",Value::Bool(!self.row)),("doesCopyY",Value::Bool(self.row)),
            ("sourceSpaceValue",Value::Uint(u32::from(local)))];
        if inverse {f.extend([(if self.row{"copyFactorY"}else{"copyFactor"},Value::Float(-1.)),("offset",Value::Bool(true))]);}
        self.take("TranslationConstraint",&f)?;Ok(())
    }
    fn scalar(&mut self,parent:u32,target:u32,factor:f32,local:bool,min:bool,local_min:bool,max:bool)->Result<(),Unresolved>{
        let mut f=vec![("parentId",Value::Uint(parent)),("targetId",Value::Uint(target)),
            ("doesCopy",Value::Bool(!self.row)),("doesCopyY",Value::Bool(self.row)),
            (if self.row{"copyFactorY"}else{"copyFactor"},Value::Float(factor))];
        if local {f.push(("destSpaceValue",Value::Uint(1)));}
        if min {f.extend([(if self.row{"minY"}else{"min"},Value::Bool(true)),(if self.row{"minValueY"}else{"minValue"},Value::Float(0.))]);}
        if local_min {f.push(("minMaxSpaceValue",Value::Uint(1)));}
        if max {f.extend([(if self.row{"maxY"}else{"max"},Value::Bool(true)),(if self.row{"maxValueY"}else{"maxValue"},Value::Float(D))]);}
        self.take("TranslationConstraint",&f)?;Ok(())
    }
    fn scale(&mut self,target:u32,factor:f32)->Result<u32,Unresolved>{
        let n=self.node(0,None)?;self.scalar(n,target,factor,false,false,false,false)?;Ok(n)
    }
    fn signal(&mut self,a:u32,b:u32,origin:u32)->Result<u32,Unresolved>{
        if let Some(&n)=self.signals.get(&(a,b)){return Ok(n);}
        let source=if let Some(&n)=self.sources.get(&a){n}else{
            let n=self.node(0,None)?;self.translation(n,a,false,false)?;self.sources.insert(a,n);n};
        let world_b=self.node(source,None)?;self.translation(world_b,b,false,false)?;
        let difference=self.node(0,None)?;self.translation(difference,world_b,true,false)?;
        let negative=self.scale(difference,-1.)?;
        let absolute=self.node(difference,None)?;self.scalar(absolute,negative,1.,false,true,true,false)?;
        let constant=self.node(0,Some(-self.epsilon))?;
        let shifted=self.node(absolute,None)?;self.scalar(shifted,constant,1.,true,false,false,false)?;
        let dead=self.node(0,None)?;self.scalar(dead,shifted,1.,false,true,false,false)?;
        let normalized=self.scale(dead,1.)?;
        self.take("DistanceConstraint",&[("parentId",Value::Uint(normalized)),("targetId",Value::Uint(origin)),("modeValue",Value::Uint(2)),("distance",Value::Float(D))])?;
        let gate=self.node(0,None)?;self.scalar(gate,normalized,2.,false,false,false,true)?;
        self.signals.insert((a,b),gate);Ok(gate)
    }
    fn mask(&mut self,owner:u32)->Result<u32,Unresolved>{
        let shape=self.take("Shape",&[("parentId",Value::Uint(owner))])?;
        self.take("Rectangle",&[("parentId",Value::Uint(shape)),("x",Value::Float(16384.)),("y",Value::Float(16384.)),("width",Value::Float(32768.)),("height",Value::Float(32768.))])?;
        self.masks+=1;Ok(shape)
    }
}
/// `base` is the original checked slot graph, before original colors are hidden.
/// `start..end` is the complete generated paint suffix (indices, not object IDs).
/// Sizing helpers between base and start are checked separately. The caller must
/// establish the base layout independence; this reader checks direct paint ownership.
/// All absent generated fields stay absent, enforcing native defaults fail-closed.
pub(super) fn bind(base:&[Record],records:&[Record],start:usize,end:usize,roles:&[(u32,u32)],
    row:bool,reverse_main:bool,reverse_cross:bool,line_fraction:f32,epsilon:f32)->Result<Binding,Unresolved>{
    let mut r=Reader{records,next:start,end,row,epsilon,sources:BTreeMap::new(),signals:BTreeMap::new(),masks:0};
    if base.len()<2 || base[0].kind!="Backboard" || base[1].kind!="Artboard" || start<base.len() || start>end || end!=records.len() || roles.is_empty()
        || ![0.,0.5,1.].contains(&line_fraction) || !epsilon.is_finite() || !(0. ..D).contains(&epsilon){return Err(r.fail());}
    let mut owned=BTreeSet::new();let mut originals=Vec::new();
    for &(slot,visible) in roles {
        let si=usize::try_from(slot).ok().and_then(|i|i.checked_add(1)).ok_or_else(||r.fail())?;
        let vi=usize::try_from(visible).ok().and_then(|i|i.checked_add(1)).ok_or_else(||r.fail())?;
        if !owned.insert(slot) || !owned.insert(visible) || slot==0 || visible==0
            || !base.get(si).is_some_and(|x|x.kind=="LayoutComponent")
            || !base.get(vi).is_some_and(|x|x.kind=="LayoutComponent" && same(x.get("parentId"),&Value::Uint(slot))) {return Err(r.fail());}
        let fills:Vec<_>=base.iter().enumerate().filter(|(_,x)|x.kind=="Fill" && same(x.get("parentId"),&Value::Uint(visible))).collect();
        if fills.len()!=1{return Err(r.fail());}let (fi,fill)=fills[0];
        if fi<=vi{return Err(r.fail());}
        if !exact(fill,"Fill",&[("parentId",Value::Uint(visible))]){return Err(r.fail());}
        let colors:Vec<_>=base.iter().enumerate().filter(|(_,x)|x.kind=="SolidColor" && same(x.get("parentId"),&Value::Uint((fi-1) as u32))).collect();
        if colors.len()!=1{return Err(r.fail());}let (ci,color)=colors[0];
        if ci<=fi{return Err(r.fail());}
        let Some(Value::Color(value))=color.get("colorValue") else{return Err(r.fail());};
        if !exact(color,"SolidColor",&[("parentId",Value::Uint((fi-1) as u32)),("colorValue",Value::Color(*value))])
            || !records.get(fi).is_some_and(|x|exact(x,"Fill",&[("parentId",Value::Uint(visible))]))
            || !records.get(ci).is_some_and(|x|exact(x,"SolidColor",&[("parentId",Value::Uint((fi-1) as u32)),("colorValue",Value::Color(0))])){return Err(r.fail());}
        originals.push(*value);
    }
    let mut boxes=Vec::new();let mut box_masks=BTreeMap::new();
    for &(_,visible) in roles {if !box_masks.contains_key(&visible){
        let b=super::paint_box_binding::consume(records,r.next,end,visible).map_err(|e|Unresolved{position:e.position})?;
        r.next=b.next;box_masks.insert(visible,b.masks);boxes.push(b);
    }}
    let origin=r.node(0,None)?;let mut anchors=Vec::new();
    let fraction=if reverse_cross{1.-line_fraction}else{line_fraction};
    for &(slot,_) in roles {let n=r.node(0,None)?;
        r.take("TransformConstraint",&[("parentId",Value::Uint(n)),("targetId",Value::Uint(slot)),(if row{"originY"}else{"originX"},Value::Float(fraction))])?;anchors.push(n);}
    let mut groups:Vec<_>=(0..roles.len()).collect();let mut members=groups.clone();
    if reverse_cross{groups.reverse();}if reverse_main{members.reverse();}
    let mut replicas=Vec::new();
    for i in groups {
        let leader=if i==0{None}else{let signal=r.signal(anchors[i-1],anchors[i],origin)?;
            let inverse=r.node(0,Some(D))?;r.translation(inverse,signal,false,true)?;Some(r.mask(inverse)?)};
        for &j in &members {if j<i{continue;}
            let member=if i==j{None}else{let signal=r.signal(anchors[i],anchors[j],origin)?;Some(r.mask(signal)?)};
            let foreground=r.take("Shape",&[("parentId",Value::Uint(0))])?;
            r.take("Rectangle",&[("parentId",Value::Uint(foreground)),("width",Value::Float(32768.)),("height",Value::Float(32768.)),("x",Value::Float(16384.)),("y",Value::Float(16384.))])?;
            let fill=r.take("Fill",&[("parentId",Value::Uint(foreground))])?;
            r.take("SolidColor",&[("parentId",Value::Uint(fill)),("colorValue",Value::Color(originals[j]))])?;
            for mask in box_masks[&roles[j].1] {r.take("ClippingShape",&[("parentId",Value::Uint(foreground)),("sourceId",Value::Uint(mask))])?;}
            for mask in [leader,member].into_iter().flatten(){r.take("ClippingShape",&[("parentId",Value::Uint(foreground)),("sourceId",Value::Uint(mask))])?;}
            replicas.push(foreground);
        }
    }
    for pair in replicas.windows(2){let target=u32::try_from(r.next).map_err(|_|r.fail())?;
        let rule=r.take("DrawRules",&[("parentId",Value::Uint(pair[0])),("drawTargetId",Value::Uint(target))])?;
        r.take("DrawTarget",&[("parentId",Value::Uint(rule)),("drawableId",Value::Uint(pair[1])),("placementValue",Value::Uint(1))])?;
    }
    if r.next!=end{return Err(r.fail());}
    Ok(Binding{signals:r.signals.len(),masks:r.masks,replicas:replicas.len(),boxes})
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::wrapping_paint::{self, Item, Paint};
    fn add(records:&mut Vec<Record>,kind:&'static str,parent:u32)->u32 {
        let id=records.len() as u32-1;let mut r=Record::new(kind);
        r.set("parentId",Value::Uint(parent)).unwrap();records.push(r);id
    }
    fn scene(n:usize)->(Vec<Record>,Vec<Item>,Vec<(u32,u32)>) {
        let mut records=vec![Record::new("Backboard"),Record::new("Artboard")];
        let mut items=Vec::new();let mut roles=Vec::new();
        for i in 0..n {let slot=add(&mut records,"LayoutComponent",0);
            let visible=add(&mut records,"LayoutComponent",slot);
            let fill=add(&mut records,"Fill",visible);let color=add(&mut records,"SolidColor",fill);
            records[color as usize+1].set("colorValue",Value::Color(0x80ab0000+i as u32)).unwrap();
            items.push(Item{slot,paints:vec![Paint{geometry:visible,fill,color}]});roles.push((slot,visible));
        }(records,items,roles)
    }
    #[test]
    fn binds_actual_emission_all_axes_orders_fractions_and_counts(){
        for n in [1,2,3,8] {for row in [false,true]{for main in [false,true]{for cross in [false,true]{for fraction in [0.,0.5,1.]{
            let (base,items,roles)=scene(n);let mut records=base.clone();let start=records.len();
            wrapping_paint::paint_items(&mut records,&items,row,main,cross,fraction,0.03125).unwrap();
            let b=bind(&base,&records,start,records.len(),&roles,row,main,cross,fraction,0.03125).unwrap();
            assert_eq!(b.signals,n*(n-1)/2);assert_eq!(b.masks,n-1+n*(n-1)/2);assert_eq!(b.replicas,n*(n+1)/2);
        }}}}}
    }
    #[test]
    fn rejects_operand_space_clamp_mask_clip_color_and_order_mutations(){
        let (base,items,roles)=scene(3);let mut records=base.clone();let start=records.len();
        wrapping_paint::paint_items(&mut records,&items,true,true,true,0.5,0.03125).unwrap();
        // Each mutation targets a real emitted operation, including fields that
        // normally remain absent to select immutable runtime defaults.
        let mutations=[
            ("Node","rotation",Value::Float(1.)),
            ("TransformConstraint","originX",Value::Float(0.5)),
            ("TransformConstraint","targetId",Value::Uint(roles[1].1)),
            ("TranslationConstraint","sourceSpaceValue",Value::Uint(1)),
            ("TranslationConstraint","destSpaceValue",Value::Uint(1)),
            ("TranslationConstraint","strength",Value::Float(0.5)),
            ("DistanceConstraint","modeValue",Value::Uint(1)),
            ("DistanceConstraint","distance",Value::Float(D-1.)),
            ("DistanceConstraint","targetId",Value::Uint(roles[0].0)),
            ("Rectangle","width",Value::Float(32767.)),
            ("Rectangle","x",Value::Float(16383.)),
            ("Shape","scaleX",Value::Float(2.)),
            ("ClippingShape","sourceId",Value::Uint(roles[0].1)),
            ("ClippingShape","isVisible",Value::Bool(false)),
            ("Fill","isVisible",Value::Bool(false)),
            ("SolidColor","colorValue",Value::Color(0)),
            ("DrawRules","drawTargetId",Value::Uint(0)),
            ("DrawTarget","drawableId",Value::Uint(roles[0].1)),
            ("DrawTarget","placementValue",Value::Uint(0)),
        ];
        for (kind,key,value) in mutations {
            let mut changed=records.clone();let p=(start..changed.len()).find(|&p|changed[p].kind==kind).unwrap();
            changed[p].set(key,value).unwrap();
            assert!(bind(&base,&changed,start,changed.len(),&roles,true,true,true,0.5,0.03125).is_err(),"{kind}.{key}");
        }
        for (key,value) in [("maxValueY",Value::Float(D-1.)),("minMaxSpaceValue",Value::Uint(0)),("offset",Value::Bool(false))]{
            let mut changed=records.clone();let p=(start..changed.len()).find(|&p|changed[p].kind=="TranslationConstraint" && changed[p].get(key).is_some()).unwrap();
            changed[p].set(key,value).unwrap();assert!(bind(&base,&changed,start,changed.len(),&roles,true,true,true,0.5,0.03125).is_err());
        }
        let mut changed=records.clone();changed.swap(start+1,start+2);
        assert!(bind(&base,&changed,start,changed.len(),&roles,true,true,true,0.5,0.03125).is_err());
        let mut changed=records.clone();changed.push(Record::new("Node"));
        assert!(bind(&base,&changed,start,changed.len(),&roles,true,true,true,0.5,0.03125).is_err());
        assert!(bind(&base,&records,start,records.len()-1,&roles,true,true,true,0.5,0.03125).is_err());
        assert!(bind(&base,&records,start,records.len(),&roles,true,false,true,0.5,0.03125).is_err());
        assert!(bind(&base,&records,start,records.len(),&roles,true,true,true,0.5,0.0625).is_err());
    }
    #[test]
    fn rejects_cross_owner_cache_and_trace_confusion(){
        let (base,items,roles)=scene(2);let mut records=base.clone();let start=records.len();
        let mut trace=wrapping_paint::paint_items(&mut records,&items,false,false,false,0.,0.03125).unwrap();
        let b=bind(&base,&records,start,records.len(),&roles,false,false,false,0.,0.03125).unwrap();assert!(b.matches_trace(&trace));assert_eq!(b.boxes.len(),2);
        let first_mask=b.boxes[0].masks[0];let second_mask=b.boxes[1].masks[0];
        let clip=(start..records.len()).find(|&p|records[p].kind=="ClippingShape"&&same(records[p].get("sourceId"),&Value::Uint(first_mask))).unwrap();
        let mut changed=records.clone();changed[clip].set("sourceId",Value::Uint(second_mask)).unwrap();
        assert!(bind(&base,&changed,start,changed.len(),&roles,false,false,false,0.,0.03125).is_err());
        trace.boxes.swap(0,1);assert!(!b.matches_trace(&trace));
    }
    #[test]
    fn rejects_original_paint_and_role_mismatches(){
        let (base,items,roles)=scene(2);let mut records=base.clone();let start=records.len();
        wrapping_paint::paint_items(&mut records,&items,false,false,false,0.,0.03125).unwrap();
        let mut wrong=base.clone();wrong[items[0].paints[0].color as usize+1].set("colorValue",Value::Color(123)).unwrap();
        assert!(bind(&wrong,&records,start,records.len(),&roles,false,false,false,0.,0.03125).is_err());
        let mut wrong=records.clone();wrong[items[0].paints[0].color as usize+1].set("colorValue",Value::Color(123)).unwrap();
        assert!(bind(&base,&wrong,start,wrong.len(),&roles,false,false,false,0.,0.03125).is_err());
        assert!(bind(&base,&records,start,records.len(),&[roles[0],roles[0]],false,false,false,0.,0.03125).is_err());
        assert!(bind(&base,&records,start,records.len(),&[(u32::MAX,u32::MAX-1)],false,false,false,0.,0.03125).is_err());
        let mut wrong=base.clone();wrong[0]=Record::new("Fill");
        assert!(bind(&wrong,&records,start,records.len(),&roles,false,false,false,0.,0.03125).is_err());
    }
}

impl Binding {pub(super) fn matches_trace(&self,t:&super::wrapping_paint::Trace)->bool{self.boxes.len()==t.boxes.len()&&self.boxes.iter().zip(&t.boxes).all(|(a,b)|a.matches_trace(b))}}
