//! Pinned160085: generated Node computed wrappers dispatch the live virtual getter.
//! Artboard inherits LayoutComponent dimensions; Text overrides with local bounds.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    component_dirt::ComponentDirt,
    core::{CoreArena, CoreHandle},
    data_bind::data_bind::DataBind,
    generated::{core_registry::CoreRegistry, node_base::{NodeBase, NodeBaseCallbacks}},
    text::text::Text,
};

fn artboard() -> Artboard {
    let mut artboard = Artboard::default();
    artboard.base.base.set_layout(0.0, 0.0, 123.0, 47.0);
    artboard
}

fn with_text(f: impl FnOnce(&CoreHandle)) {
    // A real imported/shaped Text gives public, source-valid nonzero bounds.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/text_style_background.riv");
    let bytes = std::fs::read(&fixture).expect("pinned text_style_background fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let artboard = file.with_file(|file| file.artboard()).expect("source artboard");
    Artboard::advance_handle(&artboard, 0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME);
    let texts = artboard.with_downcast::<Artboard, _>(|a| a.find_all_handles::<Text>()).unwrap();
    assert_eq!(texts.len(), 1);
    let bounds = texts[0].with_downcast::<Text, _>(Text::local_bounds).unwrap();
    assert!(bounds.width() > 0.0 && bounds.height() > 0.0, "fixture must exercise nonzero dimensions");
    f(&texts[0]);
}

fn assert_computed_equality(owner: &CoreHandle, occurrence: bool) {
    let observers = CoreArena::default();
    for key in [NodeBase::COMPUTED_WIDTH_PROPERTY_KEY, NodeBase::COMPUTED_HEIGHT_PROPERTY_KEY] {
        let expected = owner.with_mut(|o| CoreRegistry::get_double(o, key.into())).unwrap();
        assert!(expected > 0.0);
        let observed = observers.insert(DataBind::new(0, key.into(), 0));
        owner.with_mut(|o| observed.with_mut(|binding| {
            o.core_mut().add_property_observer(binding.as_data_bind_mut().unwrap());
        }).unwrap()).unwrap();
        let write = |value| {
            if occurrence { assert!(CoreRegistry::set_double_handle(owner, key.into(), value)); }
            else { owner.with_mut(|o| CoreRegistry::set_double(o, key.into(), value)).unwrap(); }
        };
        write(expected);
        assert_eq!(observed.with(|b| b.as_data_bind().unwrap().dirt()), Some(0),
            "equal native computed dimension must not notify: key={key}, occurrence={occurrence}");
        write(0.0);
        assert_eq!(observed.with(|b| b.as_data_bind().unwrap().dirt()), Some(ComponentDirt::BINDINGS_TARGET.0.into()),
            "zero differs from live dimension: key={key}, occurrence={occurrence}");
        assert_eq!(owner.with_mut(|o| CoreRegistry::get_double(o, key.into())), Some(expected));
    }
}

#[test]
fn artboard_computed_callbacks_use_inherited_layout_dimensions() {
    let mut value = artboard();
    assert_eq!(NodeBaseCallbacks::computed_width(&mut value), 123.0);
    assert_eq!(NodeBaseCallbacks::computed_height(&mut value), 47.0);
}

#[test]
fn text_computed_callbacks_use_local_bounds_dimensions() {
    with_text(|owner| {
        owner.with_downcast_mut::<Text, _>(|value| {
            let expected = value.local_bounds();
            assert_eq!(NodeBaseCallbacks::computed_width(value), expected.width());
            assert_eq!(NodeBaseCallbacks::computed_height(value), expected.height());
        }).unwrap();
    });
}

#[test]
fn artboard_computed_equality_uses_inherited_layout_dimensions() {
    for occurrence in [false, true] {
        let arena = CoreArena::default();
        let owner = arena.insert(artboard());
        assert_computed_equality(&owner, occurrence);
    }
}

#[test]
fn text_computed_equality_uses_local_bounds_dimensions() {
    for occurrence in [false, true] {
        with_text(|owner| assert_computed_equality(owner, occurrence));
    }
}
