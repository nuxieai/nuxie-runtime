// Validation-only normalized fixed-layout bridge appended to frozen compiler.rs.
// Source decimal provenance remains distinct from converted ordinary wire values.
pub(crate) fn validation_fixed_geometry_recipe(input:&str,out:&std::path::Path)->Result<(),String>{
 use serde::Deserialize;
 use serde_json::json;
 use crate::wire::{Record,Value};
 use computed_provenance::{NumericSize,NumericStyle};
 use scalar_provenance::ScalarProvenance;
 use fixed_layout::{LayoutLength,LayoutStyle};
 #[derive(Clone,Deserialize)] #[serde(deny_unknown_fields)] struct Number {value:f32,unit:String,source:String}
 #[derive(Clone,Deserialize)] #[serde(deny_unknown_fields)] struct Dim {value:f32,unit:String,source:String,min:Option<Number>,max:Option<Number>}
 #[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Parent {width:Dim,height:Dim}
 #[derive(Deserialize)] #[serde(rename_all="camelCase",deny_unknown_fields)] struct Slot {name:String,main:Dim,cross:Dim,visible_main:Dim,visible_cross:Dim,slot_alignment:u32,alignment:f32,color:u32}
 #[derive(Deserialize)] #[serde(rename_all="camelCase",deny_unknown_fields)] struct Recipe {initial_viewport:[f32;2],row:bool,reverse_main:bool,wrap:u32,line_fraction:f32,main_fraction:f32,parent:Parent,slots:Vec<Slot>}
 fn set(r:&mut Record,k:&str,v:Value)->Result<(),String>{r.set(k,v).map_err(|e|format!("{e:?}"))}
 fn units(unit:&str)->Result<u32,String>{match unit{"px"=>Ok(1),"%"=>Ok(2),_=>Err("Unit must be px or %".into())}}
 fn numeric(n:&Number)->Result<NumericSize,String>{
  // Pinned CSS px resolution computes a double before storing Length's float.
  // Direct decimal-to-f32 parsing differs at adversarial double-rounding ties.
  let resolved=n.source.parse::<f64>().map_err(|e|format!("Source decimal: {e}"))?;
  if !resolved.is_finite(){return Err("Nonfinite source decimal".into());}
  let parsed=resolved as f32;
  if !parsed.is_finite() || parsed.to_bits()!=n.value.to_bits(){return Err("Source decimal does not match computed f32 value".into());}
  let scalar=Ok(ScalarProvenance::from_decimal(&n.source,n.value).map_err(|e|format!("Source provenance: {e:?}"))?);
  Ok(if units(&n.unit)?==1{NumericSize::Pixels(scalar)}else{NumericSize::Percent(scalar)})
 }
 fn node(r:&mut Vec<Record>,parent:u32,dims:[&Dim;2])->Result<(u32,LayoutStyle),String>{
  // Convert the resolved fixed px descriptor before emitting any dimension.
  // Percent coefficients retain their authored/native value.
  let mut nums=Vec::new();
  for d in dims{
   let p=Number{value:d.value,unit:d.unit.clone(),source:d.source.clone()};
   let zero=Number{value:0.,unit:"px".into(),source:"0".into()};
   nums.push((numeric(&p)?,numeric(d.min.as_ref().unwrap_or(&zero))?,d.max.as_ref().map(numeric).transpose()?.unwrap_or(NumericSize::Auto)));
  }
  let h=nums.pop().unwrap();let w=nums.pop().unwrap();
  let authored=NumericStyle{width:w.0,min_width:w.1,max_width:w.2,height:h.0,min_height:h.1,max_height:h.2,..NumericStyle::default()};
  let layout=LayoutStyle::from_numeric(&authored).map_err(|e|format!("Fixed layout: {e:?}"))?;
  let id=r.len() as u32-1;let mut n=Record::new("LayoutComponent");let mut s=Record::new("LayoutComponentStyle");
  set(&mut n,"parentId",Value::Uint(parent))?;set(&mut n,"styleId",Value::Uint(id+1))?;
  for (axis,preferred,minimum,maximum) in [("Width",&layout.width,&layout.min_width,&layout.max_width),("Height",&layout.height,&layout.min_height,&layout.max_height)]{
   let write=|record:&mut Record,key:&str,units_key:&str,length:&LayoutLength|->Result<(),String>{
    length.validate().map_err(|e|format!("Fixed layout binding: {e:?}"))?;
    if let (Some(value),Some(unit))=(length.value(),length.units()){set(record,key,Value::Float(value))?;set(record,units_key,Value::Uint(unit))?;}
    Ok(())
   };
   // Preferred scalar lives on the owner; its units live on the style.
   let value=preferred.value().ok_or("Expected definite preferred dimension")?;
   set(&mut n,&axis.to_lowercase(),Value::Float(value))?;
   set(&mut s,&format!("{}UnitsValue",axis.to_lowercase()),Value::Uint(preferred.units().ok_or("Expected preferred units")?))?;
   write(&mut s,&format!("min{axis}"),&format!("min{axis}UnitsValue"),minimum)?;
   write(&mut s,&format!("max{axis}"),&format!("max{axis}UnitsValue"),maximum)?;
  }
  r.extend([n,s]);Ok((id,layout))
 }
 fn normalized_length(v:&LayoutLength)->serde_json::Value{
  match v{
   LayoutLength::Auto=>json!({"kind":"auto"}),
   LayoutLength::Fixed(v)=>json!({"kind":"fixed","sourceProvenance":format!("{:?}",v.authored()),"computedBits":v.computed_bits(),"rawUnits":v.raw_units(),"emittedBits":v.emitted().to_bits(),"emitted":v.emitted()}),
   LayoutLength::Percent(v)=>json!({"kind":"percent","sourceProvenance":format!("{v:?}"),"nativeBits":v.native().to_bits(),"native":v.native()}),
  }
 }
 fn normalized_style(owner:u32,style:&LayoutStyle)->serde_json::Value{
  json!({"owner":owner,"width":normalized_length(&style.width),"height":normalized_length(&style.height),"minWidth":normalized_length(&style.min_width),"minHeight":normalized_length(&style.min_height),"maxWidth":normalized_length(&style.max_width),"maxHeight":normalized_length(&style.max_height)})
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
  let dims=if q.row{[&slot.visible_main,&slot.visible_cross]}else{[&slot.visible_cross,&slot.visible_main]};let (visible,vnum)=node(&mut r,id,dims)?;paint(&mut r,visible,slot.color)?;
  roles.push((id,visible));authored.push((id,num));authored.push((visible,vnum));alignments.push(slot.alignment);map.push(json!({"id":slot.name,"object_id":visible}));
 }
 let references=authored.iter().map(|(id,n)|(*id,n)).collect::<Vec<_>>();
 let full=wrapping_sizes::MachineInterval::new(0.,16384.).unwrap();
 let domains=wrapping_domains::resolve_layout(&r,parent,&roles,&pnum,&references,[full;2]).map_err(|e|format!("Domain: {e:?}"))?;
 let binding=fixed_geometry::bind(&domains).map_err(|e|format!("Geometry: {e:?}"))?;
 if !binding.matches_domains(&domains){return Err("Seed source mismatch".into());}
 let seeds=binding.seeds().iter().map(|(id,s)|json!({"objectId":id,"size":s.size,"sizeBits":s.size.map(f32::to_bits),"local":s.local,"localBits":s.local.map(f32::to_bits),"world":s.world,"worldBits":s.world.map(f32::to_bits)})).collect::<Vec<_>>();
 let retained=binding.source();
 let normalization=json!({"parent":normalized_style(retained.parent.0,&retained.parent.1),"roles":retained.roles.iter().map(|(&owner,style)|normalized_style(owner,style)).collect::<Vec<_>>()});
 std::fs::create_dir_all(out).map_err(|e|e.to_string())?;
 std::fs::write(out.join("scene.riv"),crate::wire::encode(&r).map_err(|e|format!("{e:?}"))?).map_err(|e|e.to_string())?;
 std::fs::write(out.join("seeds.json"),serde_json::to_vec_pretty(&json!({"seeds":seeds,"normalization":normalization})).unwrap()).map_err(|e|e.to_string())?;
 Ok(())
}
