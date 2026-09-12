// Validation-only constructor appended inside the copied compiler module.
pub(crate) fn validation_fixed_scalar_recipe(input:&str,out:&std::path::Path)->Result<(),String>{
 use serde::Deserialize;use serde_json::json;use crate::wire::{Record,Value};
 #[derive(Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
 struct Recipe{kind:String,axis:usize,input_bits:u32,source_space:u32,dest_space:u32,clamp_space:u32}
 fn set(r:&mut Vec<Record>,id:u32,key:&str,value:Value)->Result<(),String>{r[id as usize+1].set(key,value).map_err(|e|format!("{e:?}"))}
 fn add(r:&mut Vec<Record>,kind:&'static str,parent:u32)->Result<u32,String>{let id=r.len()as u32-1;let mut n=Record::new(kind);n.set("parentId",Value::Uint(parent)).map_err(|e|format!("{e:?}"))?;r.push(n);Ok(id)}
 fn literal(r:&mut Vec<Record>,parent:u32,axis:&str,value:f32)->Result<u32,String>{let id=add(r,"Node",parent)?;set(r,id,axis,Value::Float(value))?;Ok(id)}
 let q:Recipe=serde_json::from_str(input).map_err(|e|e.to_string())?;
 if q.axis>1||q.source_space>1||q.dest_space>1||q.clamp_space>1{return Err("invalid recipe".into());}
 let axis=if q.axis==0{"x"}else{"y"};let v=f32::from_bits(q.input_bits);if !v.is_finite(){return Err("nonfinite".into());}
 let mut r=vec![Record::new("Backboard"),Record::new("Artboard")];
 for key in ["width","height"]{set(&mut r,0,key,Value::Float(100.))?;}
 let output=if q.kind=="round"{
  let source=literal(&mut r,0,axis,v)?;
  paint_rounding::round(&mut r,source,q.axis==1).map_err(|e|format!("{e:?}"))?.rounded
 }else if q.kind=="spaces"||q.kind=="multiple"{
  let tp=literal(&mut r,0,axis,30.25)?;
  let target=literal(&mut r,tp,axis,v)?;
  let p=literal(&mut r,0,axis,-100.5)?;
  let owner=literal(&mut r,p,axis,2.25)?;
  let c=add(&mut r,"TranslationConstraint",owner)?;
  for(key,value)in[("targetId",target),("sourceSpaceValue",q.source_space),("destSpaceValue",q.dest_space),("minMaxSpaceValue",q.clamp_space)]{set(&mut r,c,key,Value::Uint(value))?;}
  set(&mut r,c,"doesCopy",Value::Bool(q.axis==0))?;set(&mut r,c,"doesCopyY",Value::Bool(q.axis==1))?;
  set(&mut r,c,if q.axis==0{"copyFactor"}else{"copyFactorY"},Value::Float(-2.))?;
  set(&mut r,c,"offset",Value::Bool(true))?;
  for(key,value)in[(if q.axis==0{"min"}else{"minY"},Value::Bool(true)),(if q.axis==0{"minValue"}else{"minValueY"},Value::Float(-120.)),(if q.axis==0{"max"}else{"maxY"},Value::Bool(true)),(if q.axis==0{"maxValue"}else{"maxValueY"},Value::Float(80.))]{set(&mut r,c,key,value)?;}
  if q.kind=="multiple"{
   let c=add(&mut r,"TranslationConstraint",owner)?;
   set(&mut r,c,"minMaxSpaceValue",Value::Uint(1))?;
   set(&mut r,c,if q.axis==0{"min"}else{"minY"},Value::Bool(true))?;
   set(&mut r,c,if q.axis==0{"max"}else{"maxY"},Value::Bool(true))?;
   set(&mut r,c,if q.axis==0{"minValue"}else{"minValueY"},Value::Float(9.))?;
   set(&mut r,c,if q.axis==0{"maxValue"}else{"maxValueY"},Value::Float(4.))?;
  }
  owner
 }else{return Err("unknown kind".into());};
 let e=fixed_scalar::evaluate(&r,&[output]).map_err(|e|format!("Evaluate: {e:?}"))?;
 if !e.matches_records(&r){return Err("evaluation binding".into());}
 let observed=e.values().iter().map(|(id,v)|json!({"objectId":id,"world":v,"bits":[v[0].to_bits(),v[1].to_bits()]})).collect::<Vec<_>>();
 std::fs::create_dir_all(out).map_err(|e|e.to_string())?;
 std::fs::write(out.join("scene.riv"),crate::wire::encode(&r).map_err(|e|format!("{e:?}"))?).map_err(|e|e.to_string())?;
 std::fs::write(out.join("evaluation.json"),serde_json::to_vec_pretty(&json!({"output":output,"recordCount":r.len(),"values":observed})).unwrap()).map_err(|e|e.to_string())?;Ok(())
}
