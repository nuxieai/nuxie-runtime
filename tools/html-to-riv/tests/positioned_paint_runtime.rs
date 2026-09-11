use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
fn input(clip: bool) -> CompileInput {
    CompileInput { html:"<div id=host><div id=a><div id=inner></div></div><div id=b></div></div>".into(),css:format!("#host{{width:100%;height:260px;padding:20px;flex-direction:row;gap:10px}}#a{{width:80px;height:100px;background:#e9a344;overflow:{}}}#inner{{width:70px;height:50px;background:#318ea8;position:relative;left:80px;top:10px}}#b{{width:90px;height:80px;background:#86ae83}}",if clip{"clip"}else{"visible"}),width:390.,height:320.,..Default::default() }
}
fn colors(stream: &str)->Vec<String> {
    stream.lines().filter(|l|l.starts_with("drawPath ")).map(|l|l.split("color=").nth(1).unwrap().split(',').next().unwrap().to_owned()).collect()
}
#[test]
fn positioned_policy_clones_clears_and_rejects_invalid_targets_atomically() {
    let output=compile(&input(false)).unwrap();
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
    let original=file.with_file(File::artboard_default).unwrap();
    original.with_artboard_mut(|a|a.set_css_paint_order(true));
    let inner=output.source_map.iter().find(|n|n.id=="inner").unwrap().object_id;
    let style_id=original.with_artboard(|a|a.objects()[inner as usize].clone()).unwrap().with(|o|o.as_layout_component().unwrap().style_id()).unwrap();
    let draw=|instance:&nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle|{
        instance.update_pass(true);let before=factory.borrow().stream().len();
        let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);
        colors(&factory.borrow().stream()[before..])
    };
    let ordinary=["0xffe9a344","0xff318ea8","0xff86ae83"];
    let positioned=["0xffe9a344","0xff86ae83","0xff318ea8"];
    assert_eq!(draw(&original),ordinary);
    assert!(original.with_artboard_mut(|a|a.set_css_positioned_paint_order(&[inner])));
    for invalid in [vec![0],vec![u32::MAX],vec![inner,inner],vec![style_id]] {
        assert!(!original.with_artboard_mut(|a|a.set_css_positioned_paint_order(&invalid)));
        assert_eq!(draw(&original),positioned);
    }
    let cloned=original.instance().unwrap();
    for width in [240.,390.,768.,240.] {
        cloned.set_size(width,320.);assert_eq!(draw(&cloned),positioned);
    }
    assert!(original.with_artboard_mut(|a|a.set_css_positioned_paint_order(&[])));
    assert_eq!(draw(&original),ordinary);
    assert_eq!(draw(&cloned),positioned);
    cloned.with_artboard_mut(|a|a.set_css_paint_order(true));
    assert!(cloned.with_artboard_mut(|a|a.set_css_positioned_paint_order(&[])));
    assert_eq!(draw(&cloned),ordinary);
}
#[test]
fn deferred_paint_reopens_only_active_ancestor_clips_and_balances_state() {
    let output=compile(&input(true)).unwrap();
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
    let instance=file.with_file(File::artboard_default).unwrap();
    let inner=output.source_map.iter().find(|n|n.id=="inner").unwrap().object_id;
    let parent=output.source_map.iter().find(|n|n.id=="a").unwrap().object_id;
    let parent=instance.with_artboard(|a|a.objects()[parent as usize].clone()).unwrap();
    assert!(instance.with_artboard_mut(|a|a.set_css_positioned_paint_order(&[inner])));
    for enabled in [true,false,true] {
        parent.with_mut(|o|o.as_layout_component_mut().unwrap().set_clip(enabled)).unwrap();
        instance.update_pass(true);let before=factory.borrow().stream().len();
        let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);
        let stream=factory.borrow().stream();let stream=&stream[before..];
        assert_eq!(colors(stream),["0xffe9a344","0xff86ae83","0xff318ea8"]);
        let mut stack=Vec::new();let mut active=0;let mut depth_at_draw=Vec::new();
        for line in stream.lines() {
            if line=="save" {stack.push(active);}
            else if line=="restore" {active=stack.pop().expect("restore without matching save");}
            else if line.starts_with("clipPath ") {active+=1;}
            else if line.starts_with("drawPath ") {depth_at_draw.push(active);}
        }
        assert!(stack.is_empty());assert_eq!(active,0);
        assert_eq!(depth_at_draw[2],depth_at_draw[1]+usize::from(enabled));
        assert_eq!(depth_at_draw[0],depth_at_draw[1]+usize::from(enabled));
    }
}

#[test]
fn integer_stacking_contexts_are_atomic_and_clone_independently() {
    let output = compile(&input(false)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let id = |name: &str| output.source_map.iter().find(|n| n.id == name).unwrap().object_id;
    let (parent, inner, sibling) = (id("a"), id("inner"), id("b"));
    let style_id = original.with_artboard(|a| a.objects()[inner as usize].clone()).unwrap()
        .with(|o| o.as_layout_component().unwrap().style_id()).unwrap();
    let draw = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        let before = factory.borrow().stream().len();
        let mut renderer = factory.borrow().make_renderer();
        instance.draw(&mut renderer);
        colors(&factory.borrow().stream()[before..])
    };
    assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&[inner])));
    // An auto parent allows the high descendant to escape above its sibling.
    assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&[(inner, 10), (sibling, 1)])));
    assert_eq!(draw(&original), ["0xffe9a344", "0xff86ae83", "0xff318ea8"]);
    // A static flex item's explicit zero contains that same high descendant.
    let policy = [(parent, 0), (inner, 10), (sibling, 1)];
    assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&policy)));
    let contained = ["0xffe9a344", "0xff318ea8", "0xff86ae83"];
    assert_eq!(draw(&original), contained);
    for invalid in [vec![(0, 1)], vec![(u32::MAX, 1)], vec![(style_id, 1)], vec![(parent, 1), (parent, 2)], vec![(inner, -1), (u32::MAX, 1)]] {
        assert!(!original.with_artboard_mut(|a| a.set_css_stacking_order(&invalid)));
        assert_eq!(draw(&original), contained);
    }
    let cloned = original.instance().unwrap();
    for width in [240., 390., 768., 240.] {
        cloned.set_size(width, 320.);
        assert_eq!(draw(&cloned), contained);
    }
    assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&[])));
    assert_eq!(draw(&original), ["0xffe9a344", "0xff86ae83", "0xff318ea8"]);
    assert_eq!(draw(&cloned), contained);
    assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&policy)));
    assert_eq!(draw(&original), contained);
    // Stacking works without any positioned markers (static flex items).
    assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&[])));
    assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&[(parent, 2), (sibling, -1)])));
    assert_eq!(draw(&original), ["0xff86ae83", "0xffe9a344", "0xff318ea8"]);
}

#[test]
fn stacking_ties_follow_css_order_independently_of_flex_direction() {
    for direction in ["row", "row-reverse", "column", "column-reverse"] {
        for reordered in [false, true] {
            let mut request = input(false);
            request.css.push_str(&format!("#host{{flex-direction:{direction}}}#a{{position:relative;z-index:1;order:{}}}#b{{position:relative;z-index:1;order:-1}}#inner{{z-index:3}}", if reordered { -2 } else { 2 }));
            let output = compile(&request).unwrap();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
            let original = file.with_file(File::artboard_default).unwrap();
            let requirements = &output.runtime_requirements;
            assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&requirements.layout_positioned)));
            let contexts = requirements.layout_stacking.iter().map(|entry| (entry.object_id, entry.level)).collect::<Vec<_>>();
            assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&contexts)));
            for instance in [original.clone(), original.instance().unwrap()] {
                for width in [240., 390., 768., 240.] {
                    instance.set_size(width, 320.); instance.update_pass(true);
                    let before = factory.borrow().stream().len();
                    let mut renderer = factory.borrow().make_renderer(); instance.draw(&mut renderer);
                    let actual = colors(&factory.borrow().stream()[before..]);
                    let expected = if reordered { ["0xffe9a344", "0xff318ea8", "0xff86ae83"] } else { ["0xff86ae83", "0xffe9a344", "0xff318ea8"] };
                    assert_eq!(actual, expected, "{direction}, reordered={reordered}, width={width}");
                }
            }
        }
    }
}

#[test]
fn stacking_contexts_reopen_live_clips_after_clone_and_resize() {
    for direction in ["row", "row-reverse", "column", "column-reverse"] {
        for parent_level in ["auto", "0", "-2"] {
            let mut request = input(true);
            request.css.push_str(&format!("#host{{flex-direction:{direction}}}#a{{z-index:{parent_level}}}#inner{{z-index:10}}#b{{z-index:1}}"));
            let output = compile(&request).unwrap();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
            let original = file.with_file(File::artboard_default).unwrap();
            let id = |name: &str| output.source_map.iter().find(|n| n.id == name).unwrap().object_id;
            let requirements = &output.runtime_requirements;
            assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&requirements.layout_positioned)));
            let contexts = requirements.layout_stacking.iter().map(|entry| (entry.object_id, entry.level)).collect::<Vec<_>>();
            assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&contexts)));
            let cloned = original.instance().unwrap();
            for instance in [&original, &cloned] {
                let host = instance.with_artboard(|a| a.objects()[id("host") as usize].clone()).unwrap();
                let parent = instance.with_artboard(|a| a.objects()[id("a") as usize].clone()).unwrap();
                for host_clip in [false, true, false] {
                    host.with_mut(|o| o.as_layout_component_mut().unwrap().set_clip(host_clip)).unwrap();
                    for parent_clip in [true, false, true] {
                        parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_clip(parent_clip)).unwrap();
                        for width in [240., 390., 768., 240.] {
                            instance.set_size(width, 320.);
                            instance.update_pass(true);
                            let before = factory.borrow().stream().len();
                            let mut renderer = factory.borrow().make_renderer();
                            instance.draw(&mut renderer);
                            let stream = factory.borrow().stream();
                            let stream = &stream[before..];
                            let expected = if parent_level == "auto" {
                                ["0xffe9a344", "0xff86ae83", "0xff318ea8"]
                            } else {
                                ["0xffe9a344", "0xff318ea8", "0xff86ae83"]
                            };
                            assert_eq!(colors(stream), expected, "{direction}/{parent_level}/{width}");
                            let mut stack = Vec::new();
                            let mut active = 0;
                            let mut draws = Vec::new();
                            for line in stream.lines() {
                                if line == "save" { stack.push(active); }
                                else if line == "restore" { active = stack.pop().expect("unbalanced restore"); }
                                else if line.starts_with("clipPath ") { active += 1; }
                                else if line.starts_with("drawPath ") { draws.push(active); }
                            }
                            assert!(stack.is_empty());
                            assert_eq!(active, 0);
                            for (color, depth) in expected.iter().zip(draws) {
                                let expected_depth = usize::from(instance.with_artboard(|a| a.clip()))
                                    + usize::from(host_clip)
                                    + usize::from(parent_clip && *color != "0xff86ae83");
                                assert_eq!(depth, expected_depth, "{direction}/{parent_level}/{width}/{color}/host_clip={host_clip}/parent_clip={parent_clip}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn axis_overflow_uses_live_sizes_and_clones_independently() {
    use nuxie_runtime::source::layout_component::CssOverflowAxis::{Horizontal, Vertical};
    let output = compile(&CompileInput {
        html: "<div id=host><div id=a><div id=inner></div></div><div id=b></div></div>".into(),
        css: "#host{width:100%;height:300px}#a{width:50%;height:100px;background:#e9a344}#inner{width:180px;height:140px;position:relative;left:20px;background:#318ea8}#b{width:60px;height:40px;background:#86ae83}".into(),
        width:390., height:320., ..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let parent_id = output.source_map.iter().find(|n| n.id == "a").unwrap().object_id;
    let inner_id = output.source_map.iter().find(|n| n.id == "inner").unwrap().object_id;
    let parent = original.with_artboard(|a| a.objects()[parent_id as usize].clone()).unwrap();
    parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_overflow_axis(Some(Horizontal))).unwrap();
    assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&[inner_id])));
    original.update_pass(true);
    let cloned = original.instance().unwrap();
    parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_overflow_axis(None)).unwrap();
    let clone_parent = cloned.with_artboard(|a| a.objects()[parent_id as usize].clone()).unwrap();
    assert_eq!(clone_parent.with(|o| o.as_layout_component().unwrap().css_overflow_axis()).unwrap(), Some(Horizontal));
    let mut frames = 0;
    for instance in [&original, &cloned] {
        let parent = instance.with_artboard(|a| a.objects()[parent_id as usize].clone()).unwrap();
        for axis in [Some(Horizontal), Some(Vertical), None, Some(Horizontal)] {
            parent.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_overflow_axis(axis)).unwrap();
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.); instance.update_pass(true);
                let before = factory.borrow().stream().len();
                let mut renderer = factory.borrow().make_renderer(); instance.draw(&mut renderer);
                let all = factory.borrow().stream(); let stream = &all[before..];
                let clips: Vec<_> = stream.lines().filter(|l| l.starts_with("clipAxis ")).collect();
                if let Some(axis) = axis {
                    assert_eq!(clips.len(), 2, "ordinary parent and deferred inner each need a clip");
                    let (name, extent) = if axis == Horizontal { ("x", width * 0.5) } else { ("y", 100.) };
                    let prefix = format!("clipAxis axis={name} range=[0,{extent}] matrix=");
                    assert!(clips.iter().all(|line| line.starts_with(&prefix)), "{clips:?}");
                } else { assert!(clips.is_empty()); }
                let mut stack = Vec::new(); let mut active = 0;
                for line in stream.lines() {
                    if line == "save" { stack.push(active); }
                    else if line == "restore" { active = stack.pop().unwrap(); }
                    else if line.starts_with("clipAxis ") { active += 1; }
                    else if line.starts_with("drawPath ") {
                        let expected = usize::from(axis.is_some() && !line.contains("color=0xff86ae83"));
                        assert_eq!(active, expected, "{line}");
                    }
                }
                assert!(stack.is_empty()); assert_eq!(active, 0); frames += 1;
            }
        }
    }
    assert_eq!(frames, 32);
}

#[test]
fn axis_overflow_installation_validates_before_mutating_and_clear_restores_imported_clip() {
    use nuxie_runtime::source::{artboard::Artboard, layout_component::CssOverflowAxis::{Horizontal,Vertical}};
    let output = compile(&input(true)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None,None,None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let root = original.core_handle();
    let parent_id = output.source_map.iter().find(|n| n.id == "a").unwrap().object_id;
    let parent = original.with_artboard(|a| a.objects()[parent_id as usize].clone()).unwrap();
    let style_id = parent.with(|o|o.as_layout_component().unwrap().style_id()).unwrap();
    assert!(!Artboard::set_css_overflow_axes_occurrence(&parent, &[]));
    assert!(Artboard::set_css_overflow_axes_occurrence(&root, &[(parent_id,Horizontal)]));
    for invalid in [vec![(parent_id,Vertical),(0,Horizontal)], vec![(parent_id,Vertical),(u32::MAX,Horizontal)], vec![(parent_id,Vertical),(style_id,Horizontal)], vec![(parent_id,Vertical),(parent_id,Horizontal)]] {
        assert!(!Artboard::set_css_overflow_axes_occurrence(&root,&invalid));
        assert_eq!(parent.with(|o|o.as_layout_component().unwrap().css_overflow_axis()).unwrap(),Some(Horizontal));
    }
    original.update_pass(true);
    let cloned = original.instance().unwrap();
    assert!(Artboard::set_css_overflow_axes_occurrence(&root,&[]));
    assert!(parent.with(|o|o.as_layout_component().unwrap().base.clip()).unwrap());
    assert_eq!(parent.with(|o|o.as_layout_component().unwrap().css_overflow_axis()).unwrap(),None);
    let draw = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle| {
        instance.update_pass(true); let start=factory.borrow().stream().len();
        let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);
        factory.borrow().stream()[start..].to_owned()
    };
    let cleared=draw(&original);let retained=draw(&cloned);
    assert!(!cleared.contains("clipAxis "));assert!(cleared.contains("clipPath "));
    assert_eq!(retained.lines().filter(|l|l.starts_with("clipAxis axis=x ")).count(),1);
    assert!(Artboard::set_css_overflow_axes_occurrence(&cloned.core_handle(),&[(parent_id,Vertical)]));
    assert!(draw(&cloned).contains("clipAxis axis=y range=[0,100]"));
    assert!(!draw(&original).contains("clipAxis "));
}
#[test]
fn plain_nested_axis_proxies_survive_reinstall_clear_clone_and_resize() {
    use nuxie_runtime::source::{artboard::Artboard, layout_component::CssOverflowAxis::{Horizontal, Vertical}};
    let output = compile(&CompileInput {
        html: "<div id=a><div id=b><div id=c></div></div></div><div id=s></div>".into(),
        css: "#a{width:75%;height:80px}#b{width:80%;height:60px}#c{width:200px;height:160px;background:red}#s{width:20px;height:20px;background:blue}".into(),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None,None,None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let id = |name| output.source_map.iter().find(|n|n.id==name).unwrap().object_id;
    original.with_artboard_mut(|a| a.set_css_paint_order(true));
    let targets = [(id("a"),Horizontal),(id("b"),Vertical)];
    assert!(Artboard::set_css_overflow_axes_occurrence(&original.core_handle(), &targets));
    original.update_pass(true);
    let cloned = original.instance().unwrap();
    for instance in [&original, &cloned] {
        for enabled in [true,true,false,true] {
            assert!(Artboard::set_css_overflow_axes_occurrence(&instance.core_handle(), if enabled { &targets } else { &[] }));
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.);instance.update_pass(true);
                let start=factory.borrow().stream().len();
                let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);
                let stream=factory.borrow().stream()[start..].to_owned();
                assert_eq!(stream.lines().filter(|l|l.starts_with("clipAxis ")).count(),if enabled {2}else{0});
                let mut stack=Vec::new();let mut active=0;
                for line in stream.lines() {
                    if line=="save" {stack.push(active);}
                    else if line=="restore" {active=stack.pop().unwrap();}
                    else if line.starts_with("clipAxis ") {active+=1;}
                    else if line.starts_with("drawPath ") {
                        assert_eq!(active,if enabled && line.contains("color=0xffff0000") {2}else{0},"enabled={enabled} width={width}\n{stream}");
                    }
                }
                assert!(stack.is_empty());assert_eq!(active,0);
            }
        }
    }
}

#[test]
fn ordinary_css_paint_order_survives_clone_without_reinstallation() {
    let output=compile(&CompileInput {
        html:"<div id=host><div id=a><div id=child></div></div><div id=b></div></div>".into(),
        css:"#host{flex-direction:row}#a{width:0;height:10px;background:#88ccff}#child{width:24px;height:18px;background:#996633}#b{width:35px;height:25px;background:#ffcc88}".into(),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
    let original=file.with_file(File::artboard_default).unwrap();
    original.with_artboard_mut(|a|a.set_css_paint_order(true));
    original.update_pass(true);
    let cloned=original.instance().unwrap();
    let draw=|instance:&nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle|{
        instance.update_pass(true);let before=factory.borrow().stream().len();
        let mut renderer=factory.borrow().make_renderer();instance.draw(&mut renderer);
        colors(&factory.borrow().stream()[before..])
    };
    for width in [240.,390.,768.,240.] {
        original.set_size(width,320.);cloned.set_size(width,320.);
        let expected=draw(&original);
        let brown=expected.iter().position(|c|c=="0xff996633").unwrap();
        let yellow=expected.iter().position(|c|c=="0xffffcc88").unwrap();
        assert!(brown<yellow,"later sibling must cover overflowing descendant");
        assert_eq!(draw(&cloned),expected,"clone must retain CSS subtree painting");
    }
}
