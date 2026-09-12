// Private harness; candidate builder freezes this beside the actual compiler sources.
use nuxie_html_to_riv::Diagnostic;
#[allow(dead_code)] #[path = "wire.rs"] mod wire;
#[path = "wrapping.rs"] mod wrapping;
use wire::{Record,Value};
use nuxie_schema::{definition_by_type_key,FieldKind};
fn vu(b:&[u8],i:&mut usize)->u32{let mut n=0;let mut s=0;loop{let x=b[*i];*i+=1;n|=((x&127)as u32)<<s;if x<128{return n} s+=7;assert!(s<35)}}
fn records(b:&[u8])->Vec<Record>{assert_eq!(&b[..7],b"RIVE\x07\x03\x00");let mut i=7;let mut count=0;while vu(b,&mut i)!=0{count+=1}i+=((count+3)/4)*4;let mut out=vec![];while i<b.len(){let d=definition_by_type_key(vu(b,&mut i)as u16).unwrap();let mut r=Record::new(d.name);loop{let key=vu(b,&mut i)as u16;if key==0{break}let prop=d.property_by_key_in_hierarchy(key).unwrap();let v=match prop.runtime_type{FieldKind::Uint=>Value::Uint(vu(b,&mut i)),FieldKind::Double=>{let v=f32::from_le_bytes(b[i..i+4].try_into().unwrap());i+=4;Value::Float(v)},FieldKind::Color=>{let v=u32::from_le_bytes(b[i..i+4].try_into().unwrap());i+=4;Value::Color(v)},FieldKind::String|FieldKind::Bytes=>{let n=vu(b,&mut i)as usize;let bytes=b[i..i+n].to_vec();i+=n;if prop.runtime_type==FieldKind::String{Value::String(String::from_utf8(bytes).unwrap())}else{Value::Bytes(bytes)}},FieldKind::Bool=>{let v=b[i]!=0;i+=1;Value::Bool(v)},other=>panic!("unexpected field kind: {:?}",other)};r.set(prop.name,v).unwrap()}out.push(r)}out}


#[allow(dead_code)] #[path="wrapping_paint.rs"] mod wrapping_paint;

fn owned(records:&[Record],kind:&str,owner:u32)->u32 {
 let matches:Vec<_>=records.iter().enumerate().filter(|(_,r)|r.kind==kind && matches!(r.get("parentId"),Some(Value::Uint(id))if *id==owner)).collect();
 assert_eq!(matches.len(),1);matches[0].0 as u32-1
}
fn main() {
 let a:Vec<_>=std::env::args().collect();let raw=std::fs::read(&a[1]).unwrap();let mut r=records(&raw);
 assert_eq!(wire::encode(&r).unwrap(),raw);
 let ids:Vec<u32>=a[3].split(',').map(|v|v.parse().unwrap()).collect();assert_eq!(ids.len(),10);
 let row=a[4]=="row";let wrap:u32=a[5].parse().unwrap();let level:u32=a[6].parse().unwrap();let direction:u32=a[7].parse().unwrap();
 r[ids[0] as usize+2].set("flexDirectionValue",Value::Uint(direction)).unwrap();
 r[ids[0] as usize+2].set("flexWrapValue",Value::Uint(wrap)).unwrap();
 r[ids[0] as usize+2].set("layoutAlignmentType",Value::Uint(if row{level*3}else{level})).unwrap();
 let items:Vec<_>=(0..3).map(|i|wrapping::Item{slot:ids[1+i],visible:ids[4+i],alignment:[1.,0.5,0.][i]}).collect();
 let initial=r.len();let trace=wrapping::align_items(&mut r,&items,row,wrap==2,level as f32/2.,0.015625).unwrap();
 assert_eq!(r.len()-initial,wrapping::alignment_records(3).unwrap());let sized=r.len();
 std::fs::write(&a[9],wire::encode(&r).unwrap()).unwrap();
 let boundaries=trace.boundaries.iter().map(|b|format!("{{\"difference\":{},\"absolute\":{},\"dead\":{},\"normalized\":{},\"gate\":{}}}",b.difference,b.absolute,b.dead,b.normalized,b.gate)).collect::<Vec<_>>().join(",");
 std::fs::write(&a[8],format!("{{\"origin\":{},\"tops\":{:?},\"heights\":{:?},\"anchors\":{:?},\"boundaries\":[{}],\"forward\":{:?},\"backward\":{:?},\"line_maxima\":{:?},\"offsets\":{:?},\"targets\":{:?}}}",trace.origin.unwrap(),trace.tops,trace.heights,trace.anchors,boundaries,trace.forward,trace.backward,trace.line_maxima,trace.offsets,trace.targets)).unwrap();
 let paints:Vec<_>=(0..3).map(|i|wrapping_paint::Item{slot:ids[1+i],paints:[ids[4+i],ids[7+i]].iter().map(|id|{let fill=owned(&r,"Fill",*id);let color=owned(&r,"SolidColor",fill);wrapping_paint::Paint{geometry:*id,fill,color}}).collect()}).collect();
 wrapping_paint::paint_items(&mut r,&paints,row,direction==1||direction==3,wrap==2,level as f32/2.,0.015625).unwrap();
 assert_eq!(r.len()-sized,wrapping_paint::paint_records(&[2,2,2]).unwrap());
 std::fs::write(&a[2],wire::encode(&r).unwrap()).unwrap();
 eprintln!("records raw={} sized={} final={} sizingAdded={} paintAdded={}",initial,sized,r.len(),sized-initial,r.len()-sized);
}
