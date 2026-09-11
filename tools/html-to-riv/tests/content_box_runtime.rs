use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::LayoutComponent;
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn content_box_policy_resizes_clones_and_clears() {
    for (dimension, minimum, maximum) in [
        ("80px", None, None),
        ("50%", None, None),
        ("0", Some(90.0_f32), None),
        ("100%", None, Some(90.0_f32)),
        ("0", Some(90.0_f32), Some(40.0_f32)),
    ] {
        let limits = format!("{}{}",
            minimum.map(|v| format!("min-width:{v}px;")).unwrap_or_default(),
            maximum.map(|v| format!("max-width:{v}px;")).unwrap_or_default());
        let output = compile(&CompileInput {
            html: "<div id=root><div id=a></div></div>".into(),
            css: format!("#root{{width:100%;height:200px;align-items:flex-start;padding:10px}}#a{{width:{dimension};height:40px;padding:5px 11px 7px 13px;{limits}}}"),
            width:390., height:320., ..Default::default()
        }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
        let artboard = file.with_file(File::artboard_default).unwrap();
        let id = output.source_map.iter().find(|n| n.id == "a").unwrap().object_id;
        let object = artboard.with_artboard(|a| a.objects()[id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_content_box_occurrence(&object, true));
        let clone = artboard.instance().unwrap();
        for instance in [&artboard, &clone] {
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.);
                instance.update_pass(true);
                let content = match dimension { "80px" => 80., "50%" => (width-20.)/2., "100%" => width-20., _ => 0. };
                let content = content.min(maximum.unwrap_or(f32::INFINITY)).max(minimum.unwrap_or(0.));
                let (w,h) = instance.with_artboard(|a| a.objects()[id as usize].as_ref().unwrap()
                    .with(|o| { let l=o.as_layout_component().unwrap().layout(); (l.width(),l.height()) }).unwrap());
                assert!((w-content-24.).abs()<0.1, "{dimension} at {width}: {w} != {}", content+24.);
                assert!((h-52.).abs()<0.1, "height: {h}");
            }
        }
        assert!(LayoutComponent::set_css_content_box_occurrence(&object, false));
        artboard.update_pass(true);
        let height = object.with(|o| o.as_layout_component().unwrap().layout().height()).unwrap();
        assert_eq!(height,40.);
        clone.update_pass(true);
        let clone_height = clone.with_artboard(|a| a.objects()[id as usize].as_ref().unwrap()
            .with(|o| o.as_layout_component().unwrap().layout().height()).unwrap());
        assert_eq!(clone_height,52.);
    }
}
