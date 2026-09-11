//! Record real draw frames while changing clips on original and cloned scenes.
//! Each request is compiled once; every saved stream retains its resource history.
use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use serde_json::json;
use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(std::env::args().nth(1).ok_or("Expected new output directory")?);
    if out.exists() { return Err("Refusing to overwrite lifecycle evidence".into()); }
    fs::create_dir_all(&out)?;
    let mut cases = Vec::new();
    for direction in ["row", "row-reverse", "column", "column-reverse"] {
        for level in ["auto", "0", "-2"] {
            let name = format!("stacking-live-clip-{direction}-{level}");
            // The host starts without paint or clipping, exercising creation of
            // a live clipping boundary absent from the initial drawable list.
            // Relative offsets force both host and parent clip edges to matter.
            let request = CompileInput {
                html: "<div id=host><div id=a><div id=inner></div></div><div id=b></div></div>".into(),
                css: format!("#host{{width:100%;height:150px;padding:20px;flex-direction:{direction};gap:10px}}#a{{width:100px;height:100px;background:#e9a344;position:relative;top:70px;z-index:{level};overflow:clip}}#inner{{width:140px;height:120px;background:#318ea8;position:relative;left:40px;top:30px;z-index:10}}#b{{width:110px;height:100px;background:#86ae83;z-index:1}}"),
                width:390., height:320., ..Default::default()
            };
            let compiled = compile(&request).map_err(|errors| format!("{name}: {errors:?}"))?;
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(&compiled.riv, RuntimeFactoryHandle::from_factory(&mut factory).ok_or("factory")?, None, None, None).ok_or("import")?;
            let original = file.with_file(File::artboard_default).ok_or("artboard")?;
            assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&compiled.runtime_requirements.layout_positioned)));
            let contexts = compiled.runtime_requirements.layout_stacking.iter().map(|v| (v.object_id,v.level)).collect::<Vec<_>>();
            assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&contexts)));
            original.update_pass(true);
            let clone = original.instance().ok_or("clone")?;
            let id = |name: &str| compiled.source_map.iter().find(|v| v.id == name).unwrap().object_id;
            let mut views = Vec::new();
            for (instance_name, instance) in [("original", &original), ("clone", &clone)] {
                let host = instance.with_artboard(|a| a.objects()[id("host") as usize].clone()).unwrap();
                let parent = instance.with_artboard(|a| a.objects()[id("a") as usize].clone()).unwrap();
                for host_clip in [false, true, false] {
                    host.with_mut(|o| o.as_layout_component_mut().unwrap().set_clip(host_clip));
                    for parent_clip in [true, false, true] {
                        parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_clip(parent_clip));
                        for width in [240., 390., 768., 240.] {
                            instance.set_size(width,320.);
                            instance.update_pass(true);
                            let index = views.len();
                            factory.borrow_mut().frame_size(width as u32,320);
                            factory.borrow_mut().clear_color(0xffffffff);
                            factory.borrow_mut().add_sample(index as f32);
                            let mut renderer = factory.borrow().make_renderer();
                            instance.draw(&mut renderer);
                            factory.borrow_mut().add_frame();
                            let stream = out.join(format!("{name}-{index}.stream"));
                            fs::write(&stream, factory.borrow().stream())?;
                            let mut bounds = serde_json::Map::new();
                            for node in &compiled.source_map {
                                let object=instance.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                                let rect=object.with(|o| {
                                    let b=o.as_layout_component().unwrap().layout_bounds();
                                    let t=o.as_world_transform_component().unwrap().world_transform();
                                    json!({"x":t[4],"y":t[5],"width":b.width(),"height":b.height()})
                                }).unwrap();
                                bounds.insert(node.id.clone(),rect);
                            }
                            views.push(json!({"instance":instance_name,"width":width,"height":320,"hostClip":host_clip,"parentClip":parent_clip,"frame":index,"stream":stream,"bounds":bounds}));
                        }
                    }
                }
            }
            cases.push(json!({"name":name,"html":request.html,"css":request.css,"views":views}));
        }
    }
    fs::write(out.join("lifecycle.json"),serde_json::to_vec_pretty(&json!({"cases":cases}))?)?;
    Ok(())
}
