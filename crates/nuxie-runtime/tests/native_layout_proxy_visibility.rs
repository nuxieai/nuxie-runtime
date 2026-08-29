//! LayoutComponent::isProxyHidden must retain its virtual isHidden call.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry, layout::layout_sizing_style_base::LayoutSizingStyleBase,
    },
    layout_component::LayoutComponent,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn layout_background_proxy_observes_style_display_without_base_collapsed_dirt() {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    );
    let bytes =
        std::fs::read(root.join("tests/unit_tests/assets/collapse_data_binds.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    artboard.update_pass(true);
    let layout = artboard
        .with_artboard(|artboard| {
            artboard
                .objects_typed::<LayoutComponent>()
                .iter()
                .find(|owner| {
                    owner
                        .with(|object| {
                            object.core_type() == 409
                                && object.as_layout_component().is_some_and(|layout| {
                                    layout.style_handle().is_some() && !layout.is_hidden()
                                })
                        })
                        .unwrap()
                })
        })
        .expect("visible styled LayoutComponent");
    let (style, proxy) = layout
        .with_downcast_mut::<LayoutComponent, _>(|layout| {
            (layout.style_handle().unwrap(), layout.proxy().unwrap())
        })
        .unwrap();

    for (display, hidden) in [(1, true), (0, false), (1, true)] {
        assert!(CoreRegistry::set_uint_handle(
            &style,
            i32::from(LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY),
            display
        ));
        assert_eq!(
            layout.with(|owner| owner.drawable_is_hidden()),
            Some(hidden)
        );
        assert!(
            !layout
                .with(|owner| owner.as_drawable().unwrap().is_hidden())
                .unwrap(),
            "the display override is not the base Drawable hidden/dirt flag"
        );
        assert_eq!(proxy.is_hidden(), hidden);
        assert_eq!(proxy.will_draw(), !hidden);
    }

    // Artboard inherits LayoutComponent's isHidden and Drawable's willDraw;
    // the latter must still dispatch to the former's display override.
    let root = artboard.core_handle();
    let root_style = root
        .with(|owner| owner.as_layout_component().unwrap().style_handle())
        .flatten()
        .expect("fixture Artboard style");
    for (display, hidden) in [(1, true), (0, false)] {
        assert!(CoreRegistry::set_uint_handle(
            &root_style,
            i32::from(LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY),
            display
        ));
        assert_eq!(root.with(|owner| owner.drawable_is_hidden()), Some(hidden));
        assert_eq!(root.with(|owner| owner.drawable_will_draw()), Some(!hidden));
    }
}
