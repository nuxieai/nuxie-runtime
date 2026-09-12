//! Read-only immutable native lifecycle timings. Draw means command recording, not GPU rendering.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::{error::Error, fmt::Write as _, fs, path::PathBuf, time::Instant};
fn main()->Result<(),Box<dyn Error>> {
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.len()<5 {return Err("usage: wrapped-lifecycle-probe SCENE.riv NEW_OUTPUT WARMUPS REPEATS WIDTHxHEIGHT...".into());}
 let out=PathBuf::from(&args[1]);if out.exists(){return Err("fresh output required".into());}
 let warmups:usize=args[2].parse()?;let repeats:usize=args[3].parse()?;
 if warmups>20 || !(1..=100).contains(&repeats) || args.len()>20 {return Err("bounded warmups/repeats/viewports required".into());}
 let sizes=args[4..].iter().map(|s|->Result<[f32;2],Box<dyn Error>>{let(w,h)=s.split_once('x').ok_or("WIDTHxHEIGHT required")?;let d=[w.parse()?,h.parse()?];if !d.iter().all(|v:&f32|v.is_finite()&&*v>0.&&*v<=16384.){return Err("bounded finite viewport required".into());}Ok(d)}).collect::<Result<Vec<_>,_>>()?;
 let bytes=fs::read(&args[0])?;fs::create_dir(&out)?;fs::write(out.join("scene.riv"),&bytes)?;
 let mut json=String::from("{\"schema\":1,\"drawScope\":\"CPU RecordingFactory command recording; no GPU render or presentation\",\"trials\":[");
 for trial in 0..warmups+repeats {
  let t=Instant::now();let mut factory=PersistentFactory::new(RecordingFactory::new());let handle=RuntimeFactoryHandle::from_factory(&mut factory).ok_or("retained context")?;let setup=t.elapsed().as_nanos();
  let t=Instant::now();let file=File::import(&bytes,handle,None,None,None).ok_or("ordinary import failed")?;let import=t.elapsed().as_nanos();
  let t=Instant::now();let original=file.with_file(File::artboard_default).ok_or("missing artboard")?;original.update_pass(true);let settle=t.elapsed().as_nanos();
  let objects=original.with_artboard(|a|a.objects().len());
  let t=Instant::now();let clone=original.instance().ok_or("clone failed")?;let cloning=t.elapsed().as_nanos();
  let mut frames=String::new();let mut n=0;
  for (instance_index,instance) in [&original,&clone].into_iter().enumerate(){for (step,&[w,h]) in sizes.iter().enumerate(){
   let t=Instant::now();instance.set_size(w,h);instance.update_pass(true);let update=t.elapsed().as_nanos();
   let t=Instant::now();factory.borrow_mut().frame_size(w.ceil()as u32,h.ceil()as u32);factory.borrow_mut().clear_color(0xffffffff);factory.borrow_mut().add_sample(n as f32);let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);drop(renderer);factory.borrow_mut().add_frame();let draw=t.elapsed().as_nanos();
   let finite=instance.with_artboard(|a|a.objects().iter().flatten().all(|o|o.with(|o|o.as_world_transform_component().is_none_or(|w|w.world_transform().values().iter().all(|v|v.is_finite()))).unwrap_or(true)));
   if !finite{return Err("nonfinite world geometry".into());}
   if n>0 {frames.push(',');}write!(&mut frames,"{{\"instance\":{instance_index},\"step\":{step},\"width\":{w},\"height\":{h},\"resizeUpdateNs\":{update},\"drawRecordingNs\":{draw}}}")?;n+=1;
  }}
  let stream_bytes=factory.borrow().stream().len();
  let t=Instant::now();drop(clone);drop(original);drop(file);drop(factory);let release=t.elapsed().as_nanos();
  if trial>0 {json.push(',');}write!(&mut json,"{{\"trial\":{trial},\"warmup\":{},\"objects\":{objects},\"fileBytes\":{},\"recordedStreamBytes\":{stream_bytes},\"factorySetupNs\":{setup},\"importNs\":{import},\"initialSettleNs\":{settle},\"cloneNs\":{cloning},\"releaseNs\":{release},\"frames\":[{frames}]}}",trial<warmups,bytes.len())?;
 }
 json.push_str("]}\n");fs::write(out.join("timings.json"),json)?;Ok(())
}
