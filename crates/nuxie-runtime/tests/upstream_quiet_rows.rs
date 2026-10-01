//! Direct translation of quiet_rows_test.cpp at 3330baec.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{RuntimeFactoryHandle, source::{
    artboard_component_list::ArtboardComponentList,
    core::CoreHandle,
    file::{File, RuntimeFileHandle},
}};

#[derive(Debug, PartialEq)]
struct RowLayout { width: f32, height: f32, min_x: f32, min_y: f32 }

fn row_layouts(list: &CoreHandle) -> Vec<RowLayout> {
    list.with_downcast::<ArtboardComponentList, _>(|list| {
        (0..list.artboard_count()).map(|i| {
            let row = list.artboard_instance(i as i32).unwrap();
            let bounds = list.layout_bounds_for_node(i);
            row.with_artboard(|row| RowLayout {
                width: row.layout_width(), height: row.layout_height(),
                min_x: bounds.left(), min_y: bounds.top(),
            })
        }).collect()
    }).unwrap()
}

fn resize_host(file: &RuntimeFileHandle, quiet_rows: bool) -> (Vec<RowLayout>, Vec<RowLayout>) {
    ArtboardComponentList::set_quiet_rows_enabled(quiet_rows);
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let view_model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let instance = file.with_file(|file| {
        if view_model_id == u32::MAX {
            file.create_view_model_instance_for_artboard(artboard.core_handle())
        } else {
            file.create_view_model_instance_at(view_model_id as usize, 0)
        }
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    machine.advance_and_apply(0.1);
    let lists = artboard.with_artboard(|artboard| artboard.find_all_handles::<ArtboardComponentList>());
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert!(list.with_downcast::<ArtboardComponentList, _>(|list| list.artboard_count() > 0).unwrap());
    let skips = ArtboardComponentList::quiet_row_skips();
    for _ in 0..90 { machine.advance_and_apply(1.0 / 60.0); }
    assert_eq!(ArtboardComponentList::quiet_row_skips() > skips, quiet_rows);
    list.with_downcast::<ArtboardComponentList, _>(|list| {
        for i in 0..list.artboard_count() {
            assert_eq!(list.artboard_instance(i as i32).unwrap().with_artboard(|row| row.quiet_host_row() == i as u32), quiet_rows);
        }
    });
    let before = row_layouts(list);
    let width = artboard.with_artboard(|artboard| artboard.width());
    assert!(nuxie_runtime::source::generated::core_registry::CoreRegistry::set_double_handle(
        &artboard.core_handle(),
        i32::from(nuxie_runtime::source::generated::layout_component_base::LayoutComponentBase::WIDTH_PROPERTY_KEY),
        width * 0.5,
    ));
    for _ in 0..60 { machine.advance_and_apply(1.0 / 60.0); }
    let after = row_layouts(list);
    ArtboardComponentList::set_quiet_rows_enabled(true);
    (before, after)
}

#[test]
fn quiet_list_rows_wake_for_layout_the_host_pushes_into_them() {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR points to pinned upstream");
    let bytes = std::fs::read(std::path::PathBuf::from(upstream).join("tests/unit_tests/assets/artboard_list_overrides.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&bytes, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
    let quiet = resize_host(&file, true);
    let awake = resize_host(&file, false);
    assert_ne!(quiet.0, quiet.1);
    assert_eq!(quiet.1.len(), awake.1.len());
    assert_eq!(quiet.1, awake.1);
}
