//! Private finite-fixture ordinary-file border composition. Not public admission.
use nuxie_html_to_riv::{CompileInput, Diagnostic};
#[allow(dead_code)] mod wire;
use wire::{Record, Value};
use nuxie_schema::{definition_by_type_key, FieldKind};
fn vu(b:&[u8],i:&mut usize)->u32 {let mut n=0;let mut s=0;loop{let x=b[*i];*i+=1;n|=((x&127)as u32)<<s;if x<128{return n} s+=7;assert!(s<35)}}
fn records(b:&[u8])->Vec<Record> {assert_eq!(&b[..7],b"RIVE\x07\x03\x00");let mut i=7;let mut count=0;while vu(b,&mut i)!=0{count+=1}i+=((count+3)/4)*4;let mut out=vec![];while i<b.len(){let d=definition_by_type_key(vu(b,&mut i)as u16).unwrap();let mut r=Record::new(d.name);loop{let key=vu(b,&mut i)as u16;if key==0{break}let prop=d.property_by_key_in_hierarchy(key).unwrap();let v=match prop.runtime_type{FieldKind::Uint=>Value::Uint(vu(b,&mut i)),FieldKind::Double=>{let v=f32::from_le_bytes(b[i..i+4].try_into().unwrap());i+=4;Value::Float(v)},FieldKind::Color=>{let v=u32::from_le_bytes(b[i..i+4].try_into().unwrap());i+=4;Value::Color(v)},FieldKind::String|FieldKind::Bytes=>{let n=vu(b,&mut i)as usize;let bytes=b[i..i+n].to_vec();i+=n;if prop.runtime_type==FieldKind::String{Value::String(String::from_utf8(bytes).unwrap())}else{Value::Bytes(bytes)}},FieldKind::Bool=>{let v=b[i]!=0;i+=1;Value::Bool(v)},other=>panic!("unexpected field kind: {:?}",other)};r.set(prop.name,v).unwrap()}out.push(r)}out}
fn set(r:&mut Record,k:&str,v:Value){r.set(k,v).unwrap()}
fn strip(r:&mut Vec<Record>,parent:u32,side:usize,b:f32,color:u32){
 let id=r.len() as u32-1;let mut l=Record::new("LayoutComponent");set(&mut l,"parentId",Value::Uint(parent));set(&mut l,"styleId",Value::Uint(id+1));set(&mut l,"name",Value::String(format!("border-strip-{side}")));
 let mut s=Record::new("LayoutComponentStyle");set(&mut s,"parentId",Value::Uint(id));set(&mut s,"positionTypeValue",Value::Uint(2));
 for(axis,point)in[("width",side>=2),("height",side<2)]{set(&mut l,axis,Value::Float(if point{b}else{0.}));set(&mut s,&format!("{axis}UnitsValue"),Value::Uint(if point{1}else{3}));set(&mut s,if axis=="width"{"layoutWidthScaleType"}else{"layoutHeightScaleType"},Value::Uint(0));}
 let edges=match side{0=>vec![("Left",-b),("Right",-b),("Top",-b)],1=>vec![("Left",-b),("Right",-b),("Bottom",-b)],2=>vec![("Left",-b),("Top",0.),("Bottom",0.)],_=>vec![("Right",-b),("Top",0.),("Bottom",0.)]};
 for(edge,value)in edges{set(&mut s,&format!("position{edge}"),Value::Float(value));set(&mut s,&format!("position{edge}UnitsValue"),Value::Uint(1));}
 r.push(l);r.push(s);let paint=r.len()as u32-1;let mut f=Record::new("Fill");set(&mut f,"parentId",Value::Uint(id));r.push(f);let mut c=Record::new("SolidColor");set(&mut c,"parentId",Value::Uint(paint));set(&mut c,"colorValue",Value::Color(color));r.push(c);
}
fn main(){
 let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),3);let mut input:CompileInput=serde_json::from_slice(&std::fs::read(&a[1]).unwrap()).unwrap();
 let fixtures:serde_json::Value=serde_json::from_str(include_str!("cases.json")).unwrap();let fixture=fixtures.as_array().unwrap().iter().find(|v|v["html"].as_str()==Some(&input.html)&&v["css"].as_str()==Some(&input.css)).expect("exact finite fixture required");input.css=fixture["nativeCss"].as_str().unwrap().into();
 let mut o=nuxie_html_to_riv::compile(&input).unwrap();let mut r=records(&o.riv);assert_eq!(wire::encode(&r).unwrap(),o.riv);let p=std::path::Path::new(&a[2]);std::fs::write(p.with_extension("raw.riv"),&o.riv).unwrap();
 for border in fixture["borders"].as_array().unwrap(){let id=o.source_map.iter().find(|v|Some(v.id.as_str())==border["id"].as_str()).unwrap().object_id;let b=border["width"].as_f64().unwrap()as f32;let color=border["argb"].as_u64().unwrap()as u32;let style_id=match r[id as usize+1].get("styleId").unwrap(){Value::Uint(v)=>*v,_=>panic!()};let s=&mut r[style_id as usize+1];assert_eq!(s.kind,"LayoutComponentStyle");for edge in["Left","Right","Top","Bottom"]{set(s,&format!("border{edge}"),Value::Float(b));set(s,&format!("border{edge}UnitsValue"),Value::Uint(1));}if b>0.{for side in 0..4{strip(&mut r,id,side,b,color)}}}
 // Layout proxy construction depends on contiguous emitted subtrees. Rebuild
 // compiler-owned record order after inserting paint children, and remap IDs.
 let mut owners=std::collections::BTreeMap::new();
 for (index,record) in r.iter().enumerate().skip(2){assert!(matches!(record.kind,"LayoutComponentStyle"|"LayoutComponent"|"Fill"|"SolidColor"));let parent=match record.get("parentId"){Some(Value::Uint(v))=>*v,_=>0};owners.insert(index as u32-1,parent);}
 for(index,record)in r.iter().enumerate().skip(1){if let Some(Value::Uint(style))=record.get("styleId"){owners.insert(*style,index as u32-1);}}
 let mut children:std::collections::BTreeMap<u32,Vec<u32>>=Default::default();for(id,parent)in owners{children.entry(parent).or_default().push(id);}
 fn visit(id:u32,children:&std::collections::BTreeMap<u32,Vec<u32>>,order:&mut Vec<u32>){order.push(id);if let Some(v)=children.get(&id){for child in v{visit(*child,children,order)}}}
 let mut order=vec![];visit(0,&children,&mut order);assert_eq!(order.len()+1,r.len());let remap:std::collections::BTreeMap<_,_>=order.iter().enumerate().map(|(new,old)|(*old,new as u32)).collect();let mut sorted=vec![r[0].clone()];
 for old in order{let mut record=r[old as usize+1].clone();for key in["parentId","styleId"]{if let Some(Value::Uint(value))=record.get(key){let new=remap[value];set(&mut record,key,Value::Uint(new));}}sorted.push(record);}
 for node in &mut o.source_map{node.object_id=remap[&node.object_id];}
 std::fs::write(p,wire::encode(&sorted).unwrap()).unwrap();std::fs::write(p.with_extension("map.json"),serde_json::to_vec_pretty(&o.source_map).unwrap()).unwrap();
}
