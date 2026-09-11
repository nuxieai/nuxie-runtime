use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::layout_component::{CssOverflowClipBox, CssOverflowClipMargin};

// Compile the public property, then exercise independent clone policy lifecycles.
#[test]
fn clip_margin_clone_resize_clear_keeps_background_geometry() {
    let recording = std::env::var_os("NUXIE_CLIP_MARGIN_RECORDING").map(std::path::PathBuf::from);
    if let Some(out) = &recording {
        assert!(!out.exists(), "Refusing to overwrite lifecycle evidence");
        std::fs::create_dir_all(out).unwrap();
    }
    let mut cases = Vec::new();
    for (radius, margin_pixels) in [0, 24].into_iter().flat_map(|r| [8., -8., -80.].map(|m| (r, m))) {
        let request = CompileInput {
            html: "<div id=a><div id=child></div></div><div id=sibling></div>".into(),
            css: format!("#a{{width:75%;height:100px;padding:12px 18px;overflow:clip;overflow-clip-margin:content-box {margin_pixels}px;border-radius:{radius}px;background:#e9a344}}#child{{width:280px;height:160px;position:relative;left:-40px;top:-35px;background:#318ea8}}#sibling{{width:35px;height:25px;background:#86ae83}}"),
            width: 390., height: 320., ..Default::default()
        };
        let output = compile(&request).unwrap();
        let name = format!("clip-margin-lifecycle-r{radius}-m{margin_pixels}");
        let mut views = Vec::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
        let original = file.with_file(File::artboard_default).unwrap();
        assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&output.runtime_requirements.layout_positioned)));
        let id = output.source_map.iter().find(|node| node.id == "a").unwrap().object_id;
        let parent = original.with_artboard(|a| a.objects()[id as usize].clone()).unwrap();
        let margin = CssOverflowClipMargin::new(CssOverflowClipBox::Content, margin_pixels).unwrap();
        let requirements = &output.runtime_requirements.layout_overflow_clip_margins;
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].object_id, id);
        assert_eq!(requirements[0].pixels, margin_pixels);
        assert!(nuxie_runtime::source::artboard::Artboard::set_css_overflow_clip_margins_occurrence(&original.core_handle(), &[(id, margin)]));
        original.update_pass(true);
        let cloned = original.instance().unwrap();
        parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_overflow_clip_margin(None)).unwrap();
        let clone_parent = cloned.with_artboard(|a| a.objects()[id as usize].clone()).unwrap();
        assert_eq!(clone_parent.with(|o| o.as_layout_component().unwrap().css_overflow_clip_margin()).unwrap(), Some(margin));
        for (instance_name, instance) in [("original", &original), ("clone", &cloned)] {
            let parent = instance.with_artboard(|a| a.objects()[id as usize].clone()).unwrap();
            for width in [240., 390., 768., 240.] {
                instance.set_size(width, 320.);
                let mut frames = Vec::new();
                for policy in [None, Some(margin), None, Some(margin)] {
                    parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_overflow_clip_margin(policy)).unwrap();
                    instance.update_pass(true);
                    let frame = views.len();
                    factory.borrow_mut().frame_size(width as u32, 320);
                    factory.borrow_mut().clear_color(0xffffffff);
                    factory.borrow_mut().add_sample(frame as f32);
                    let before = factory.borrow().stream().len();
                    let mut renderer = factory.borrow().make_renderer();
                    instance.draw(&mut renderer);
                    factory.borrow_mut().add_frame();
                    if let Some(out) = &recording {
                        let stream_path = out.join(format!("{name}-{frame}.stream"));
                        std::fs::write(&stream_path, factory.borrow().stream()).unwrap();
                        let mut bounds = serde_json::Map::new();
                        for node in &output.source_map {
                            let object = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                            let rect = object.with(|o| {
                                let b = o.as_layout_component().unwrap().layout_bounds();
                                let t = o.as_world_transform_component().unwrap().world_transform();
                                serde_json::json!({"x":t[4],"y":t[5],"width":b.width(),"height":b.height()})
                            }).unwrap();
                            bounds.insert(node.id.clone(), rect);
                        }
                        views.push(serde_json::json!({"instance":instance_name,"width":width,"height":320,"margin":if policy.is_some() { format!("content-box {margin_pixels}px") } else { "padding-box 0px".into() },"frame":frame,"stream":stream_path,"bounds":bounds}));
                    }
                    let all = factory.borrow().stream();
                    let stream = &all[before..];
                    let paths = |prefix: &str| stream.lines().filter(|l| l.starts_with(prefix))
                        .map(|line| line.split("path={verbs=").nth(1).unwrap().split("}} paint=").next().unwrap().to_owned())
                        .collect::<Vec<_>>();
                    frames.push((paths("clipPath "), paths("drawPath ")));
                    let mut stack = Vec::new();
                    let mut active_clips = 0;
                    let mut paint_depths = Vec::new();
                    for line in stream.lines() {
                        if line == "save" { stack.push(active_clips); }
                        if line == "restore" { active_clips = stack.pop().unwrap(); }
                        if line.starts_with("clipPath ") { active_clips += 1; }
                        if line.starts_with("drawPath ") {
                            paint_depths.push((line.contains("color=0xffe9a344"),
                                line.contains("color=0xff318ea8"), active_clips));
                        }
                    }
                    assert!(stack.is_empty());
                    assert_eq!(active_clips, 0);
                    if policy.is_some() {
                        let background = paint_depths.iter().find(|p| p.0).unwrap().2;
                        let child = paint_depths.iter().find(|p| p.1).unwrap().2;
                        let sibling = paint_depths.iter().find(|p| !p.0 && !p.1).unwrap().2;
                        assert_eq!(background, sibling, "own clip must not trim background");
                        assert_eq!(child, sibling + 1, "deferred descendant must retain parent clip");
                    }
                }
                assert_eq!(frames[0], frames[2], "clear must restore imported clipping");
                assert_eq!(frames[1], frames[3], "reinstallation must be stable");
                assert_ne!(frames[0].0, frames[1].0, "margin must change clipping");
                assert_eq!(frames[0].1, frames[1].1, "margin must not change background/child paths");
            }
        }
        cases.push(serde_json::json!({"name":name,"html":request.html,"css":request.css,"views":views}));
    }
    if let Some(out) = &recording {
        std::fs::write(out.join("lifecycle.json"), serde_json::to_vec_pretty(&serde_json::json!({"cases":cases})).unwrap()).unwrap();
    }
}

#[test]
fn checked_clip_margin_installation_is_atomic_and_clears_omitted_targets() {
    use nuxie_runtime::source::{artboard::Artboard, layout_component::CssOverflowAxis};
    let output=compile(&CompileInput {
        html:"<div id=a></div><div id=b></div>".into(),css:"#a{overflow:clip}#b{overflow:visible}".into(),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
    let instance=file.with_file(File::artboard_default).unwrap();
    let id=|name:&str|output.source_map.iter().find(|n|n.id==name).unwrap().object_id;
    let target=instance.with_artboard(|a|a.objects()[id("a") as usize].clone()).unwrap();
    let margin=CssOverflowClipMargin::new(CssOverflowClipBox::Content,8.).unwrap();
    let install=|targets:&[(u32,CssOverflowClipMargin)]|Artboard::set_css_overflow_clip_margins_occurrence(&instance.core_handle(),targets);
    assert!(install(&[(id("a"),margin)]));
    let style=target.with(|o|o.as_layout_component().unwrap().style_id()).unwrap();
    for bad in [0,u32::MAX,style,id("b"),id("a")] {
        assert!(!install(&[(id("a"),margin),(bad,margin)]));
        assert_eq!(target.with(|o|o.as_layout_component().unwrap().css_overflow_clip_margin()).unwrap(),Some(margin));
    }
    target.with_mut(|o|o.as_layout_component_mut().unwrap().set_css_overflow_axis(Some(CssOverflowAxis::Horizontal))).unwrap();
    assert!(!install(&[(id("a"),margin)]));
    assert!(install(&[]));
    assert_eq!(target.with(|o|o.as_layout_component().unwrap().css_overflow_clip_margin()).unwrap(),None);
    target.with_mut(|o|o.as_layout_component_mut().unwrap().set_css_overflow_axis(None)).unwrap();
    assert!(install(&[(id("a"),margin)]));
}
