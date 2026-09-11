use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn independent_factor_policy_resizes_clones_and_clears() {
    let output = compile(&CompileInput {
        html: "<div id=root><div id=a></div><div id=b></div></div>".into(),
        css: "#root{width:100%;height:100px;padding:10px;flex-direction:row}#a,#b{width:100px;height:20px}".into(),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let ids = ["a", "b"].map(|id| {
        output
            .source_map
            .iter()
            .find(|n| n.id == id)
            .unwrap()
            .object_id
    });
    let objects = ids.map(|id| {
        artboard
            .with_artboard(|a| a.objects()[id as usize].clone())
            .unwrap()
    });
    assert!(LayoutComponent::set_css_flex_factors_occurrence(
        &objects[0],
        CssFlexFactors::new(1., 0.)
    ));
    assert!(LayoutComponent::set_css_flex_factors_occurrence(
        &objects[1],
        CssFlexFactors::new(0., 1.)
    ));
    let cloned = artboard.instance().unwrap();
    for instance in [artboard.clone(), cloned.clone()] {
        for (width, expected) in [
            (390., [270., 100.]),
            (240., [120., 100.]),
            (180., [100., 60.]),
            (390., [270., 100.]),
        ] {
            instance.set_size(width, 320.);
            instance.update_pass(true);
            for (id, expected) in ids.into_iter().zip(expected) {
                let actual = instance.with_artboard(|a| {
                    a.objects()[id as usize]
                        .as_ref()
                        .unwrap()
                        .with(|o| o.as_layout_component().unwrap().layout().width())
                        .unwrap()
                });
                assert!(
                    (actual - expected).abs() < 0.1,
                    "width{width}: {actual} vs{expected}"
                );
            }
        }
    }
    // Changing installed factors must invalidate layout at the same viewport,
    // while the previously cloned occurrence keeps its own factors.
    assert!(LayoutComponent::set_css_flex_factors_occurrence(
        &objects[0], CssFlexFactors::new(0., 1.)
    ));
    assert!(LayoutComponent::set_css_flex_factors_occurrence(
        &objects[1], CssFlexFactors::new(1., 0.)
    ));
    assert!(LayoutComponent::set_css_flex_factors_occurrence(
        &objects[1], CssFlexFactors::new(1., 0.)
    ));
    for (instance, expected) in [(&artboard, [100., 270.]), (&cloned, [270., 100.])] {
        instance.update_pass(true);
        for (id, expected) in ids.into_iter().zip(expected) {
            let actual = instance.with_artboard(|a| a.objects()[id as usize]
                .as_ref().unwrap().with(|o| o.as_layout_component().unwrap().layout().width()).unwrap());
            assert!((actual - expected).abs() < 0.1, "changed policy: {actual} vs {expected}");
        }
    }
    for object in &objects {
        assert!(LayoutComponent::set_css_flex_factors_occurrence(
            object, None
        ));
    }
    artboard.set_size(390., 320.);
    artboard.update_pass(true);
    for object in &objects {
        assert_eq!(
            object
                .with(|o| o.as_layout_component().unwrap().layout().width())
                .unwrap(),
            100.
        );
    }
}

#[test]
fn factor_policy_rejects_invalid_numbers() {
    for invalid in [-1., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssFlexFactors::new(invalid, 1.).is_none());
        assert!(CssFlexFactors::new(1., invalid).is_none());
    }
    assert!(CssFlexFactors::new(0., 0.).is_some());
}
