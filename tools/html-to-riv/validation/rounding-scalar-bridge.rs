// Private ordinary TranslationConstraint positive-predicate experiment.
pub(crate) fn validation_rounding_recipe(input:&str,out:&std::path::Path)->Result<(),String>{
 use serde::Deserialize;use serde_json::json;use crate::wire::{Record,Value};
 #[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]struct Recipe{axis:String,input_bits:u32,dynamic:bool}
 fn set(r:&mut Record,k:&str,v:Value)->Result<(),String>{r.set(k,v).map_err(|e|format!("{e:?}"))}
 fn add(r:&mut Vec<Record>,kind:&'static str,parent:u32)->Result<u32,String>{let id=r.len()as u32-1;let mut n=Record::new(kind);set(&mut n,"parentId",Value::Uint(parent))?;r.push(n);Ok(id)}
 fn property(r:&mut[Record],id:u32,k:&str,v:Value)->Result<(),String>{set(&mut r[id as usize+1],k,v)}
 let q:Recipe=serde_json::from_str(input).map_err(|e|e.to_string())?;let y=match q.axis.as_str(){"x"=>false,"y"=>true,_=>return Err("axis".into())};let axis=if y{"y"}else{"x"};let input=f32::from_bits(q.input_bits);if !input.is_finite() || input.abs()>16384.{return Err("finite input required".into());}
 let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];set(&mut r[1],"width",Value::Float(100.))?;set(&mut r[1],"height",Value::Float(100.))?;set(&mut r[1],"styleId",Value::Uint(1))?;set(&mut r[2],"flexDirectionValue",Value::Uint(if y{0}else{2}))?;
 let parent=if q.dynamic{
  let spacer=add(&mut r,"LayoutComponent",0)?;let style=add(&mut r,"LayoutComponentStyle",0)?;property(&mut r,spacer,"styleId",Value::Uint(style))?;
  for(k,v)in[("width",if y{1.}else{50.}),("height",if y{50.}else{1.})]{property(&mut r,spacer,k,Value::Float(v))?;}
  property(&mut r,style,if y{"heightUnitsValue"}else{"widthUnitsValue"},Value::Uint(2))?;property(&mut r,style,if y{"widthUnitsValue"}else{"heightUnitsValue"},Value::Uint(1))?;
  let owner=add(&mut r,"LayoutComponent",0)?;let s=add(&mut r,"LayoutComponentStyle",0)?;property(&mut r,owner,"styleId",Value::Uint(s))?;for k in["width","height"]{property(&mut r,owner,k,Value::Float(1.))?;property(&mut r,s,&format!("{k}UnitsValue"),Value::Uint(1))?;}owner
 }else{0};
 let source=add(&mut r,"Node",parent)?;property(&mut r,source,axis,Value::Float(if q.dynamic{-50.}else{input}))?;
 let trace=paint_rounding::round(&mut r,source,y).map_err(|e|format!("{e:?}"))?;
 let step=|s:&paint_rounding::Step|json!({"power":s.power,"threshold":s.threshold,"difference":s.difference,"stages":s.stages,"bit":s.bit,"accumulator":s.accumulator});
 let data=json!({"source":source,"axis":q.axis,"inputBits":q.input_bits,"dynamic":q.dynamic,"rounded":trace.rounded,"positive":trace.positive.iter().map(step).collect::<Vec<_>>(),"negative":trace.negative.iter().map(step).collect::<Vec<_>>(),"recordCount":r.len(),"addedRecords":paint_rounding::RECORDS});
 std::fs::create_dir_all(out).map_err(|e|e.to_string())?;std::fs::write(out.join("scene.riv"),crate::wire::encode(&r).map_err(|e|format!("{e:?}"))?).map_err(|e|e.to_string())?;std::fs::write(out.join("trace.json"),serde_json::to_vec_pretty(&data).unwrap()).map_err(|e|e.to_string())?;Ok(())
}
