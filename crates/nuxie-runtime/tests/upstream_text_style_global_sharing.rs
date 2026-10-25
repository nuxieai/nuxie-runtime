//! Literal public text_style_global_sharing_test.cpp, upstream 0238f299.
#[path = "support/text_style_global.rs"]
mod support;
use support::*;
#[test]
fn styles_shared_only_when_nothing_differs_per_text() {
    let f = load("text_style_global_sharing.riv");
    for (name, shareable) in [
        ("Static", true),
        ("Plain", true),
        ("Inert", true),
        ("Exposed", false),
        ("ReadsHost", false),
        ("Animated", false),
        ("WithBackground", false),
        ("WithDash", false),
    ] {
        let s = source(&f, name);
        assert_eq!(
            TextStyleShareCache::analyze(&s, Some(&f)).shareable,
            shareable,
            "{name}"
        );
        assert_eq!(
            f.with_file(|f| f.text_style_shares())
                .borrow_mut()
                .info(&s)
                .shareable,
            shareable,
            "{name}"
        );
    }
    assert!(
        TextStyleShareCache::analyze(&source(&f, "ReadsHost"), Some(&f))
            .reads_outside_own_view_model
    );
    assert!(
        !TextStyleShareCache::analyze(&source(&f, "Exposed"), Some(&f))
            .reads_outside_own_view_model
    );
}
#[test]
fn every_text_using_shareable_style_draws_one_instance() {
    let f = load("text_style_global_sharing.riv");
    {
        let (first, _) = bound_scene(&f);
        let (second, _) = bound_scene(&f);
        let a = named_handle(&first, "Text A");
        let b = named_handle(&first, "Text B");
        let other = named_handle(&second, "Text A");
        let card = named_handle(&card(&first), "Card Static");
        assert!(shared(&a));
        assert_eq!(resolved(&a), resolved(&b));
        assert_eq!(resolved(&a), resolved(&other));
        assert_eq!(resolved(&a), resolved(&card));
        assert!(
            global_instance(&a)
                .downgrade()
                .ptr_eq(&global_instance(&card).downgrade())
        );
        assert!(shared(&named_handle(&first, "Text E")));
        assert!(shared(&named_handle(&first, "Text G")));
        assert_eq!(live(&f), 3);
        for name in ["Text C", "Text D", "Text F", "Text H", "Text I"] {
            let own = named_handle(&first, name);
            assert!(!shared(&own));
            assert!(
                own.with_downcast::<TextStyleGlobal, _>(|g| g.artboard_instance())
                    .flatten()
                    .is_some()
            );
            assert_ne!(resolved(&own), resolved(&named_handle(&second, name)));
        }
        for (name, size) in [
            ("Text A", 30.0),
            ("Text G", 22.0),
            ("Text H", 24.0),
            ("Text F", 26.0),
            ("Text I", 28.0),
        ] {
            assert_eq!(named_font(&first, name), size);
        }
    }
    assert_eq!(live(&f), 0);
}
#[test]
fn shared_style_texts_keep_own_glyphs() {
    let f = load("text_style_global_sharing.riv");
    let (a, _) = bound_scene(&f);
    let t = named::<Text>(&a, "Text A");
    let p = resolved(&handle(&t));
    assert!(
        p.with_downcast::<TextStylePaint, _>(|p| p.is_shared())
            .unwrap()
    );
    assert!(
        p.with_downcast_mut::<TextStylePaint, _>(|p| p.local_path().empty())
            .unwrap()
    );
    let shape = p
        .with_downcast::<TextStylePaint, _>(|p| {
            assert!(!p.paints.shape_paints().is_empty());
            p.paints.shape_paints()[0].clone()
        })
        .unwrap();
    let color = shape
        .with(|s| {
            s.as_container_component()
                .unwrap()
                .children()
                .iter()
                .find(|c| c.is_type_of(SolidColor::TYPE_KEY))
                .cloned()
        })
        .flatten()
        .unwrap();
    let value = color
        .with_downcast::<SolidColor, _>(|c| c.base.color_value())
        .unwrap();
    assert!(CoreRegistry::set_double_handle(&t, 18, 0.5));
    frame(&a);
    assert_eq!(
        shape
            .with(|s| s.as_shape_paint().unwrap().render_opacity())
            .unwrap(),
        1.0
    );
    assert_eq!(
        color
            .with_downcast::<SolidColor, _>(|c| c.base.color_value())
            .unwrap(),
        value
    );
    assert!(
        p.with_downcast_mut::<TextStylePaint, _>(|p| p.local_path().empty())
            .unwrap()
    );
}
#[test]
fn style_binds_read_host_data() {
    let f = load("text_style_global_sharing.riv");
    let (a, v) = bound_scene(&f);
    assert_eq!(named_font(&a, "Text C"), 25.0);
    assert_eq!(named_font(&card(&a), "Card ReadsHost"), 25.0);
    assert_eq!(named_font(&a, "Text D"), 50.0);
    number(&v, "size", 35.0);
    number(&v, "exposedSize", 60.0);
    frame(&a);
    assert_eq!(named_font(&a, "Text C"), 35.0);
    assert_eq!(named_font(&card(&a), "Card ReadsHost"), 35.0);
    assert_eq!(named_font(&a, "Text D"), 60.0);
    let (other, _) = bound_scene(&f);
    assert_eq!(named_font(&other, "Text C"), 25.0);
    assert_eq!(named_font(&other, "Text D"), 50.0);
    let v = vm(&f, &a);
    number(&v, "size", 18.0);
    a.bind_view_model_instance(Some(v));
    frame(&a);
    assert_eq!(named_font(&a, "Text C"), 18.0);
    assert_eq!(named_font(&card(&a), "Card ReadsHost"), 18.0);
}
#[test]
fn shared_only_artboard_has_nothing_to_advance() {
    let f = load("text_style_global_sharing.riv");
    let a = instance(&f, "Quiet");
    frame(&a);
    let h = named_handle(&a, "Quiet Static");
    assert!(shared(&h));
    assert_eq!(
        h.with(|h| h.advancing_component_quiet_state()).unwrap(),
        QuietState::Quiet
    );
    assert!(
        !AdvancingComponentHandle::classified(&h)
            .advance_component(0.016, AdvanceFlags::ADVANCE_NESTED)
    );
    assert_eq!(named_font(&a, "Quiet Static"), 30.0);
}
#[test]
fn instances_teardown_any_order() {
    let f = load("text_style_global_sharing.riv");
    let first = bound_scene(&f);
    let second = bound_scene(&f);
    let quiet = instance(&f, "Quiet");
    frame(&quiet);
    assert_eq!(live(&f), 3);
    drop(first);
    frame(&second.0);
    assert_eq!(live(&f), 3);
    assert_eq!(named_font(&second.0, "Text A"), 30.0);
    drop(second);
    assert_eq!(live(&f), 1);
    frame(&quiet);
    drop(quiet);
    assert_eq!(live(&f), 0);
}
#[test]
fn style_metrics_reshape_texts() {
    let f = load("text_style_global_sharing.riv");
    let (a, _) = bound_scene(&f);
    for name in ["Text E", "Text H"] {
        let text = named::<Text>(&a, name);
        let p = paint(&run(&text, 0));
        let bounds = || text.with_downcast::<Text, _>(|t| t.local_bounds()).unwrap();
        let width = bounds().width();
        assert!(width > 0.0);
        let size = p
            .with_downcast::<TextStylePaint, _>(|p| p.base.font_size())
            .unwrap();
        assert!(CoreRegistry::set_double_handle(&p, 274, size * 2.0));
        frame(&a);
        assert!(bounds().width() > width);
        let width = bounds().width();
        let spacing = p
            .with_downcast::<TextStylePaint, _>(|p| p.base.letter_spacing())
            .unwrap();
        assert!(CoreRegistry::set_double_handle(&p, 390, spacing + 10.0));
        frame(&a);
        assert!(bounds().width() > width);
        let height = bounds().height();
        let size = p
            .with_downcast::<TextStylePaint, _>(|p| p.base.font_size())
            .unwrap();
        assert!(CoreRegistry::set_double_handle(&p, 370, size * 3.0));
        frame(&a);
        assert!(bounds().height() > height);
    }
}
#[test]
fn artboard_outlives_file_shared_styles() {
    let f = load("text_style_global_sharing.riv");
    let a = f.with_file(|f| f.artboard_named("Main")).unwrap();
    let v = vm(&f, &a);
    a.bind_view_model_instance(Some(v.clone()));
    frame(&a);
    assert!(live(&f) > 0);
    drop(f);
    drop(a);
    drop(v);
}
