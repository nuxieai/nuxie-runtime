// Validation-only bridge appended to a frozen compiler.rs, never product source.
pub(crate) fn validation_derived_recipe(input:&str,out:&std::path::Path)->Result<(),String>{
 use serde::Deserialize;
 use serde_json::json;
 use crate::wire::{Record,Value};
 use computed_provenance::{NumericSize,NumericStyle};
 use scalar_provenance::ScalarProvenance;
 #[derive(Clone,Deserialize)] #[serde(deny_unknown_fields)] struct Number {value:f32,unit:String}
 #[derive(Clone,Deserialize)] #[serde(deny_unknown_fields)] struct Dim {value:f32,unit:String,min:Option<Number>,max:Option<Number>}
 #[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Parent {width:Dim,height:Dim}
 #[derive(Deserialize)] #[serde(rename_all="camelCase",deny_unknown_fields)] struct Slot {name:String,main:Dim,cross:Dim,visible_main:Dim,visible_cross:Dim,slot_alignment:u32,alignment:f32,color:u32}
 #[derive(Deserialize)] #[serde(rename_all="camelCase",deny_unknown_fields)] struct Recipe {initial_viewport:[f32;2],row:bool,reverse_main:bool,wrap:u32,line_fraction:f32,main_fraction:f32,parent:Parent,slots:Vec<Slot>}
 fn set(r:&mut Record,k:&str,v:Value)->Result<(),String>{r.set(k,v).map_err(|e|format!("{e:?}"))}
 fn units(unit:&str)->Result<u32,String>{match unit{"px"=>Ok(1),"%"=>Ok(2),_=>Err("Unit must be px or %".into())}}
 fn numeric(n:&Number)->Result<NumericSize,String>{let scalar=Ok(ScalarProvenance::exact_constant(n.value).map_err(|e|format!("{e:?}"))?);Ok(if units(&n.unit)?==1{NumericSize::Pixels(scalar)}else{NumericSize::Percent(scalar)})}
 fn node(r:&mut Vec<Record>,parent:u32,dims:[&Dim;2])->Result<(u32,NumericStyle),String>{
  let id=r.len() as u32-1;let mut n=Record::new("LayoutComponent");let mut s=Record::new("LayoutComponentStyle");
  set(&mut n,"parentId",Value::Uint(parent))?;set(&mut n,"styleId",Value::Uint(id+1))?;
  let mut nums=Vec::new();
  for (axis,d) in ["Width","Height"].into_iter().zip(dims){
   let p=Number{value:d.value,unit:d.unit.clone()};let zero=Number{value:0.,unit:"px".into()};let min=d.min.as_ref().unwrap_or(&zero);
   set(&mut n,&axis.to_lowercase(),Value::Float(d.value))?;set(&mut s,&format!("{}UnitsValue",axis.to_lowercase()),Value::Uint(units(&d.unit)?))?;
   set(&mut s,&format!("min{axis}"),Value::Float(min.value))?;set(&mut s,&format!("min{axis}UnitsValue"),Value::Uint(units(&min.unit)?))?;
   if let Some(max)=&d.max {set(&mut s,&format!("max{axis}"),Value::Float(max.value))?;set(&mut s,&format!("max{axis}UnitsValue"),Value::Uint(units(&max.unit)?))?;}
   nums.push((numeric(&p)?,numeric(min)?,d.max.as_ref().map(numeric).transpose()?.unwrap_or(NumericSize::Auto)));
  }
  let h=nums.pop().unwrap();let w=nums.pop().unwrap();r.extend([n,s]);Ok((id,NumericStyle{width:w.0,min_width:w.1,max_width:w.2,height:h.0,min_height:h.1,max_height:h.2,..NumericStyle::default()}))
 }
 fn paint(r:&mut Vec<Record>,owner:u32,color:u32)->Result<(),String>{let fill=r.len() as u32-1;let mut f=Record::new("Fill");set(&mut f,"parentId",Value::Uint(owner))?;let mut c=Record::new("SolidColor");set(&mut c,"parentId",Value::Uint(fill))?;set(&mut c,"colorValue",Value::Color(color))?;r.extend([f,c]);Ok(())}
 let q:Recipe=serde_json::from_str(input).map_err(|e|e.to_string())?;
 if ![1,2].contains(&q.wrap)||![0.,0.5,1.].contains(&q.line_fraction)||![0.,0.5,1.].contains(&q.main_fraction)||q.slots.is_empty(){return Err("Invalid positional recipe".into());}
 let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];set(&mut r[1],"styleId",Value::Uint(1))?;
 for (k,v) in ["width","height"].into_iter().zip(q.initial_viewport){set(&mut r[1],k,Value::Float(v))?;}
 paint(&mut r,0,0xffffffff)?;
 let (parent,pnum)=node(&mut r,0,[&q.parent.width,&q.parent.height])?;
 let style=&mut r[parent as usize+2];set(style,"flexWrapValue",Value::Uint(q.wrap))?;set(style,"flexDirectionValue",Value::Uint(if q.row{2}else{0}+u32::from(q.reverse_main)))?;
 let (h,v)=if q.row{(q.main_fraction,q.line_fraction)}else{(q.line_fraction,q.main_fraction)};
 set(style,"layoutAlignmentType",Value::Uint((h*2.)as u32+3*(v*2.)as u32))?;
 let mut roles=Vec::new();let mut authored=Vec::new();let mut alignments=Vec::new();let mut map=vec![json!({"id":"p","object_id":parent})];
 for slot in &q.slots {
  if slot.slot_alignment>8{return Err("Invalid slotAlignment".into());}
  let dims=if q.row{[&slot.main,&slot.cross]}else{[&slot.cross,&slot.main]};let (id,num)=node(&mut r,parent,dims)?;
  set(&mut r[id as usize+2],"layoutAlignmentType",Value::Uint(slot.slot_alignment))?;
  let dims=if q.row{[&slot.visible_main,&slot.visible_cross]}else{[&slot.visible_cross,&slot.visible_main]};let (visible,_)=node(&mut r,id,dims)?;paint(&mut r,visible,slot.color)?;
  roles.push((id,visible));authored.push((id,num));alignments.push(slot.alignment);map.push(json!({"id":slot.name,"object_id":visible}));
 }
 let references=authored.iter().map(|(id,n)|(*id,n)).collect::<Vec<_>>();
 let full=wrapping_sizes::MachineInterval::new(0.,16384.).unwrap();
 let domains=wrapping_domains::resolve(&r,parent,&roles,&pnum,&references,[full;2]).map_err(|e|format!("Domain: {e:?}"))?;
 let derived=wrapping_composition::compose_with_bounds(domains,&alignments,100000).map_err(|e|format!("Derived: {e:?}"))?;
 let candidate=derived.candidate();let t=candidate.trace();let a=derived.arithmetic();
 let trace=json!({"parent":parent,"slots":roles.iter().map(|x|x.0).collect::<Vec<_>>(),"visible":roles.iter().map(|x|x.1).collect::<Vec<_>>(),"origin":t.origin,"tops":t.tops,"heights":t.heights,"anchors":t.anchors,"forward":t.forward,"backward":t.backward,"maxima":t.line_maxima,"offsets":t.offsets,"targets":t.targets,"boundaries":t.boundaries.iter().map(|b|json!({"difference":b.difference,"absolute":b.absolute,"dead":b.dead,"normalized":b.normalized,"gate":b.gate})).collect::<Vec<_>>()});
 let proof=json!({"viewport":[[0.,16384.],[0.,16384.]],"epsilon":candidate.epsilon(),"epsilonBits":candidate.epsilon().to_bits(),"radius":a.radius(),"sameLineError":a.same_line_error(),"differentLineError":a.different_line_error(),"recordCosts":candidate.record_costs(),"axes":format!("{:?}",a.axes()),"visibleWorld":format!("{:?}",a.visible_world()),"normalizer":format!("{:?}",a.normalizer()),"measured":format!("{:?}",a.measured()),"carry":format!("{:?}",a.carry()),"position":format!("{:?}",derived.position()),"scalar":format!("{:?}",derived.scalar()),"masks":format!("{:?}",derived.masks()),"paint":format!("{:?}",derived.paint())});
 std::fs::create_dir_all(out).map_err(|e|e.to_string())?;
 for (name,records) in [("base.riv",r.as_slice()),("scene.riv",candidate.records())]{std::fs::write(out.join(name),crate::wire::encode(records).map_err(|e|format!("{e:?}"))?).map_err(|e|e.to_string())?;}
 for (name,value) in [("trace.json",trace),("scene.map.json",json!(map)),("proof.json",proof)]{std::fs::write(out.join(name),serde_json::to_vec_pretty(&value).unwrap()).map_err(|e|e.to_string())?;}
 Ok(())
}
