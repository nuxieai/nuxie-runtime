//! Read-only observation through the immutable baseline's ordinary public APIs.
//! Usage: baseline-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]
//! Imports once, settles once, clones once, then resizes/draws each occurrence.
//! No compiler crate, source map, sidecar requirements, or CSS policy installation.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::{error::Error, fmt::Write as _, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 3 {
        return Err("usage: baseline-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]".into());
    }
    let sizes = args[2..].iter().map(|arg| -> Result<[f32; 2], Box<dyn Error>> {
        let text = arg.to_str().ok_or("viewport must be UTF-8")?;
        let (w,h) = text.split_once('x').ok_or("viewport must be WIDTHxHEIGHT")?;
        let dimensions = [w.parse::<f32>()?, h.parse::<f32>()?];
        if !dimensions.iter().all(|v| v.is_finite() && *v > 0. && *v <= 16384.) {
            return Err("probe viewport dimensions must be finite and in (0,16384]".into());
        }
        Ok(dimensions)
    }).collect::<Result<Vec<_>, _>>()?;
    let output = PathBuf::from(&args[1]);
    if output.exists() { return Err("use a fresh output directory".into()); }
    let bytes = fs::read(&args[0])?;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory_handle = RuntimeFactoryHandle::from_factory(&mut factory)
        .ok_or("recording factory cannot provide a retained runtime context")?;
    let file = File::import(&bytes, factory_handle, None, None, None)
        .ok_or("ordinary baseline RIV import failed")?;
    let original = file.with_file(File::artboard_default).ok_or("RIV has no default artboard")?;
    original.update_pass(true);
    let clone = original.instance().ok_or("baseline artboard clone failed")?;
    fs::create_dir(&output)?;
    fs::write(output.join("scene.riv"), &bytes)?;
    let mut frames = String::from("{\"schema\":1,\"scope\":\"ordinary baseline import/clone/resize/draw observation\",\"frames\":[");
    let mut frame_index = 0;
    for (instance_index, instance) in [&original, &clone].into_iter().enumerate() {
        for (step, &[width,height]) in sizes.iter().enumerate() {
            instance.set_size(width,height);
            instance.update_pass(true);
            // The stream canvas has integer pixels; measured layout retains
            // fractional viewport dimensions exactly as passed to set_size.
            factory.borrow_mut().frame_size(width.ceil() as u32,height.ceil() as u32);
            factory.borrow_mut().clear_color(0xffffffff);
            factory.borrow_mut().add_sample(frame_index as f32);
            let mut renderer = factory.borrow().make_renderer();
            instance.draw(&mut renderer);
            drop(renderer);
            factory.borrow_mut().add_frame();
            let geometry = instance.with_artboard(|artboard| {
                artboard.objects().iter().enumerate().filter_map(|(object_id,object)| {
                    object.as_ref()?.with(|object| {
                        let layout = object.as_layout_component()?.layout();
                        let world = object.as_world_transform_component()?.world_transform();
                        Some((object_id,layout.width(),layout.height(),*world.values()))
                    }).flatten()
                }).collect::<Vec<_>>()
            });
            let mut objects = String::from("[");
            for (index,(id,w,h,matrix)) in geometry.iter().enumerate() {
                if !matrix.iter().chain([w,h]).all(|v| v.is_finite()) {
                    return Err(format!("nonfinite observed layout at frame {frame_index}, object {id}").into());
                }
                if index != 0 { objects.push(','); }
                write!(&mut objects,"{{\"objectId\":{id},\"width\":{w},\"height\":{h},\"worldMatrix\":[{},{},{},{},{},{}]}}",matrix[0],matrix[1],matrix[2],matrix[3],matrix[4],matrix[5])?;
            }
            objects.push(']');
            fs::write(output.join(format!("frame-{frame_index}.geometry.json")), &objects)?;
            fs::write(output.join(format!("frame-{frame_index}.stream")), factory.borrow().stream())?;
            if frame_index != 0 { frames.push(','); }
            write!(&mut frames,"{{\"frame\":{frame_index},\"instance\":{instance_index},\"step\":{step},\"width\":{width},\"height\":{height},\"geometry\":\"frame-{frame_index}.geometry.json\",\"stream\":\"frame-{frame_index}.stream\"}}")?;
            frame_index += 1;
        }
    }
    frames.push_str("]}\n");
    fs::write(output.join("frames.json"),frames)?;
    println!("Recorded {frame_index} ordinary baseline frames from two occurrences");
    Ok(())
}
