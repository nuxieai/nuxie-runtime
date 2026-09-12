// Frozen private emitter experiment; no public compiler admission.
use nuxie_html_to_riv::Diagnostic;
#[allow(dead_code)] #[path="wire.rs"] mod wire;
#[allow(dead_code)] #[path="wrapping.rs"] mod wrapping;
use wire::{Record,Value};
fn set(r:&mut Record,k:&str,v:Value){r.set(k,v).unwrap()}
fn node(r:&mut Vec<Record>,parent:u32,w:f32,h:f32,wu:u32,hu:u32)->u32{
 let id=r.len() as u32-1;let mut n=Record::new("LayoutComponent");
 set(&mut n,"parentId",Value::Uint(parent));set(&mut n,"styleId",Value::Uint(id+1));
 set(&mut n,"width",Value::Float(w));set(&mut n,"height",Value::Float(h));
 let mut s=Record::new("LayoutComponentStyle");
 for(k,v)in[("widthUnitsValue",wu),("heightUnitsValue",hu),("minWidthUnitsValue",1),("minHeightUnitsValue",1)]{set(&mut s,k,Value::Uint(v));}
 for k in ["minWidth","minHeight"]{set(&mut s,k,Value::Float(0.));}
 r.extend([n,s]);id
}
fn main(){
 let a:Vec<_>=std::env::args().collect();let out=std::path::Path::new(&a[1]);let row=a[2]=="row";let reverse:bool=a[3]=="1";let wrap:u32=a[4].parse().unwrap();let level:u32=a[5].parse().unwrap();let percent=a[6]=="percent";
 let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];
 set(&mut r[1],"styleId",Value::Uint(1));set(&mut r[1],"width",Value::Float(240.));set(&mut r[1],"height",Value::Float(240.));
 let parent=node(&mut r,0,100.,100.,2,2);
 set(&mut r[parent as usize+2],"flexDirectionValue",Value::Uint(if row{if reverse{3}else{2}}else{if reverse{1}else{0}}));
 set(&mut r[parent as usize+2],"flexWrapValue",Value::Uint(wrap));
 set(&mut r[parent as usize+2],"layoutAlignmentType",Value::Uint(if row{level*3}else{level}));
 let mut items=vec![];let mut slots=vec![];let mut visible=vec![];
 for i in 0..3{
  let main=[60.,60.,40.][i];let cross=[50.,30.,70.][i];
  let slot=node(&mut r,parent,if row{main}else{cross},if row{cross}else{main},1,1);
  set(&mut r[slot as usize+2],"layoutAlignmentType",Value::Uint((i*4)as u32));
  let vc=if percent{[50.,150.,75.][i]}else{[20.,40.,90.][i]};let units=if percent{2}else{1};
  let v=node(&mut r,slot,if row{20.}else{vc},if row{vc}else{20.},if row{1}else{units},if row{units}else{1});
  let fill=r.len()as u32-1;let mut f=Record::new("Fill");set(&mut f,"parentId",Value::Uint(v));let mut c=Record::new("SolidColor");set(&mut c,"parentId",Value::Uint(fill));set(&mut c,"colorValue",Value::Color(0xff3578ac));r.extend([f,c]);
  slots.push(slot);visible.push(v);items.push(wrapping::Item{slot,visible:v,alignment:[0.,0.5,1.][i]});
 }
 std::fs::write(out.join("base.riv"),wire::encode(&r).unwrap()).unwrap();let start=r.len();
 let trace=wrapping::align_items(&mut r,&items,row,wrap==2,level as f32/2.,1./64.).unwrap();assert_eq!(r.len()-start,wrapping::alignment_records(3).unwrap());
 std::fs::write(out.join("scene.riv"),wire::encode(&r).unwrap()).unwrap();
 std::fs::write(out.join("trace.json"),format!("{{\"slots\":{:?},\"visible\":{:?},\"anchors\":{:?},\"maxima\":{:?},\"offsets\":{:?},\"targets\":{:?},\"gates\":{:?}}}\n",slots,visible,trace.anchors,trace.line_maxima,trace.offsets,trace.targets,trace.boundaries.iter().map(|b|b.gate).collect::<Vec<_>>())).unwrap();
}
