//! A layout host marks the instance it lays out: the mounted one, else a
//! vacated slot's placeholder, else nothing.
use crate::source::{
    artboard::Artboard, artboard_host::ArtboardHost, nested_artboard::NestedArtboard,
    nested_artboard_layout::NestedArtboardLayout,
};
use crate::{CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

fn mark(host: &CoreHandle, caller: &RuntimeArtboardInstanceHandle) {
    host.with_downcast_mut::<NestedArtboardLayout, _>(|host| {
        ArtboardHost::mark_hosting_layout_dirty(host, caller.downgrade())
    })
    .expect("layout host");
}

#[test]
fn a_vacated_layout_slot_marks_its_placeholder() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &crate::live_move_fixture::layout_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("LayoutPage")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let screen = crate::live_move::screens(&root).unwrap()["LayoutScreen"].clone();
    let source = screen
        .with_artboard(|b| b.find_handle::<NestedArtboardLayout>("source"))
        .unwrap();
    let destination = screen
        .with_artboard(|b| b.find_handle::<NestedArtboard>("destination"))
        .unwrap();
    let unit = super::child(&source).unwrap().unwrap();
    super::move_live(&source, &destination).unwrap();
    machine.advance_and_apply(0.0);
    let placeholder = super::read(&source, |s| {
        assert!(s.instance.is_none());
        assert!(s.artboard_referencer.referenced_artboard().is_none());
        s.live_move_placeholder_root()
    })
    .unwrap()
    .expect("vacated slot keeps its placeholder");
    assert!(!screen.with_artboard(|b| b.has_dirty_layout()));

    mark(&source, &unit);
    assert!(screen.with_artboard(|b| b.layout_is_dirty(&placeholder)));
    screen.with_artboard_mut(|b| b.clean_layout(&placeholder));
    assert!(
        !screen.with_artboard(|b| b.has_dirty_layout()),
        "the placeholder is the only layout marked"
    );
}

#[test]
fn an_unmounted_layout_host_marks_nothing() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &crate::live_move_fixture::layout_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    // Import nests the source definition without an instance.
    let definition = file
        .with_file(|f| f.artboard_named_source("LayoutScreen"))
        .unwrap();
    let host = definition
        .with_downcast::<Artboard, _>(|b| b.find_handle::<NestedArtboardLayout>("source"))
        .flatten()
        .unwrap();
    let referenced = super::read(&host, |s| {
        assert!(s.instance.is_none());
        s.artboard_referencer.referenced_artboard()
    })
    .unwrap()
    .expect("import records the source definition");
    let caller = file
        .with_file(|f| f.artboard_named("LayoutScreen"))
        .unwrap();
    definition.with_downcast_mut::<Artboard, _>(|b| b.clean_layout(&referenced));

    mark(&host, &caller);
    assert!(
        !definition
            .with_downcast::<Artboard, _>(|b| b.layout_is_dirty(&referenced))
            .unwrap()
    );
}
