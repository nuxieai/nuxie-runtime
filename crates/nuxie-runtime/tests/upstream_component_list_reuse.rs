//! Six added and one changed component_list_test.cpp regression from upstream 6771fdad.
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    core::CoreArena,
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    shapes::rectangle::Rectangle,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};
use nuxie_runtime::{AdvanceFlags, RuntimeArtboardInstanceHandle};
use support::*;

fn load(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    import(
        &std::fs::read(
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets")
                .join(name),
        )
        .unwrap(),
    )
}
fn setup(
    name: &str,
    board: Option<&str>,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    CoreHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let file = load(name);
    let artboard = file
        .with_file(|file| match board {
            Some(name) => file.artboard_named(name),
            None => file.artboard_default(),
        })
        .unwrap();
    let vm = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(vm.clone()));
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vm.clone()));
    (file, artboard, vm, machine)
}
fn property(vm: &CoreHandle, name: &str) -> CoreHandle {
    vm.with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named(name))
        .unwrap()
        .unwrap()
}
fn items(list: &CoreHandle) -> Vec<CoreHandle> {
    list.with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
        .unwrap()
}
fn row_vm(item: &CoreHandle) -> CoreHandle {
    item.with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
        .unwrap()
        .unwrap()
}
fn model(list: &CoreHandle) -> CoreHandle {
    row_vm(&items(list)[0])
        .with_downcast::<ViewModelInstance, _>(|vm| vm.get_view_model())
        .unwrap()
        .unwrap()
}
fn clear(list: &CoreHandle) {
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.remove_all_items())
        .unwrap();
}
fn append(
    file: &RuntimeFileHandle,
    list: &CoreHandle,
    model: &CoreHandle,
    arena: &CoreArena,
) -> CoreHandle {
    let vm = file
        .with_file(|file| file.create_default_view_model_instance(model.clone()))
        .unwrap();
    let item = arena.insert(ViewModelInstanceListItem::default());
    item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
        item.set_view_model_instance(Some(vm.clone()))
    })
    .unwrap();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
        .unwrap();
    vm
}
fn list(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|board| board.find_handle::<ArtboardComponentList>("List"))
        .unwrap()
}
fn count(list: &CoreHandle) -> usize {
    list.with_downcast::<ArtboardComponentList, _>(|list| list.artboard_count())
        .unwrap()
}
fn boolean(vm: &CoreHandle, name: &str) -> bool {
    property(vm, name)
        .with_downcast::<ViewModelInstanceBoolean, _>(|value| value.base.property_value())
        .unwrap()
}

#[test]
fn number_to_list_artboard_count_leaves_no_style_changes() {
    let file = load("component_list_2.riv");
    let source = file
        .with_file(|file| file.artboard_named_source("Main"))
        .unwrap();
    let artboard = Artboard::instance_from_handle(&source).unwrap();
    let vm = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(vm.clone()));
    let list = list(&artboard);
    artboard.advance_default(0.0);
    list.with_downcast_mut::<ArtboardComponentList, _>(|list| {
        assert!(!list.sync_style_changes());
        assert_eq!(list.artboard_count(), 12);
        assert!(list.layout_node(13).is_none());
        assert!(list.artboard_instance(13).is_none());
        assert!(list.state_machine_instance(13).is_none());
        for index in 0..list.artboard_count() {
            assert!(list.list_item(index as i32).is_some());
        }
    })
    .unwrap();
    let count_property = property(&vm, "ItemCount");
    assert!(nuxie_runtime::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber::set_value_handle(&count_property, 6.0));
    artboard.advance_default(0.0);
    assert_eq!(count(&list), 6);
}

#[test]
fn component_list_drops_focus_rows_when_empty() {
    let (file, artboard, vm, machine) = setup("component_list_virtualized.riv", Some("Main"));
    machine.advance_and_apply(0.016);
    let list = list(&artboard);
    let scope = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.list_scope_focus_node())
        .unwrap()
        .unwrap();
    assert!(count(&list) > 0);
    assert_eq!(scope.borrow().children().len(), count(&list));
    let values = vm
        .with_downcast::<ViewModelInstance, _>(|vm| vm.property_values().to_vec())
        .unwrap();
    let item_list = values
        .into_iter()
        .find(|value| {
            value
                .with_downcast::<ViewModelInstanceList, _>(|_| ())
                .is_some()
        })
        .unwrap();
    let model = model(&item_list);
    clear(&item_list);
    machine.advance_and_apply(0.016);
    assert_eq!(count(&list), 0);
    assert!(scope.borrow().children().is_empty());
    let arena = CoreArena::default();
    for _ in 0..3 {
        append(&file, &item_list, &model, &arena);
    }
    machine.advance_and_apply(0.016);
    assert_eq!(count(&list), 3);
    assert_eq!(scope.borrow().children().len(), 3);
}

#[test]
fn reused_row_resets_its_later_default_state_machine() {
    let (file, artboard, vm, machine) =
        setup("component_list_default_state_machine.riv", Some("Main"));
    machine.advance_and_apply(0.016);
    let list = list(&artboard);
    assert_eq!(count(&list), 1);
    let row = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.state_machine_instance(0))
        .unwrap()
        .unwrap();
    row.with_instance(|row| {
        assert_eq!(row.name(), "Default");
        assert_eq!(row.get_number("level").unwrap().value(), 5.0);
    });
    row.set_number("level", 9.0);
    let item_list = property(&vm, "rows");
    let model = model(&item_list);
    clear(&item_list);
    let arena = CoreArena::default();
    append(&file, &item_list, &model, &arena);
    machine.advance_and_apply(0.016);
    assert_eq!(count(&list), 1);
    let reused = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.state_machine_instance(0))
        .unwrap()
        .unwrap();
    assert!(reused.downgrade().ptr_eq(&row.downgrade()));
    reused.with_instance(|row| {
        assert_eq!(row.name(), "Default");
        assert_eq!(row.get_number("level").unwrap().value(), 5.0);
    });
}

#[test]
fn reused_focused_row_does_not_replay_its_blur() {
    use nuxie_runtime::source::math::random::RandomProvider;
    struct TestingRandomMode;
    impl Drop for TestingRandomMode {
        fn drop(&mut self) {
            RandomProvider::clear_testing_mode();
        }
    }
    // The upstream TESTING fixture's random formula uses the empty FIFO.
    RandomProvider::clear_randoms();
    let _random_mode = TestingRandomMode;
    let (file, artboard, vm, machine) = setup("semantic/data_binding_lists.riv", None);
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    let manager = machine
        .with_instance(|machine| machine.focus_manager())
        .unwrap();
    let item_list = property(&vm, "menu");
    let old_items = items(&item_list);
    assert_eq!(old_items.len(), 4);
    let list = artboard
        .with_artboard(|board| board.artboard_component_lists().to_vec())
        .into_iter()
        .find(|candidate| {
            candidate
                .with_downcast::<ArtboardComponentList, _>(|list| {
                    list.artboard_count() > 0 && list.list_item(0).as_ref() == Some(&old_items[0])
                })
                .unwrap()
        })
        .unwrap();
    assert!(
        !list
            .with_downcast::<ArtboardComponentList, _>(|list| list.virtualization_enabled())
            .unwrap()
    );
    let scope = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.list_scope_focus_node())
        .unwrap()
        .unwrap();
    let row = scope.borrow().children()[0].clone();
    let focus = row.borrow().children()[0].clone();
    manager.with_focus_manager_mut(|manager| manager.set_focus(focus));
    machine.advance_and_apply(0.016);
    assert!(boolean(&row_vm(&old_items[0]), "isHovered"));
    let model = model(&item_list);
    clear(&item_list);
    let arena = CoreArena::default();
    let mut replacements = Vec::new();
    for _ in 0..4 {
        let replacement = file
            .with_file(|file| file.create_default_view_model_instance(model.clone()))
            .unwrap();
        property(&replacement, "isHovered")
            .with_downcast_mut::<ViewModelInstanceBoolean, _>(|value| value.set_value(true))
            .unwrap();
        let item = arena.insert(ViewModelInstanceListItem::default());
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
            item.set_view_model_instance(Some(replacement.clone()))
        })
        .unwrap();
        item_list
            .with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
            .unwrap();
        replacements.push(replacement);
    }
    for _ in 0..3 {
        machine.advance_and_apply(0.016);
    }
    assert_eq!(count(&list), 4);
    for replacement in replacements {
        assert!(boolean(&replacement, "isHovered"));
    }
}

#[test]
fn new_rows_stay_authored_without_nested_advancing() {
    let file = load("component_list_default_state_machine.riv");
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let vm = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(vm.clone()));
    let design_flags = AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME;
    let play_flags = design_flags | AdvanceFlags::ADVANCE_NESTED;
    artboard.advance(0.0, design_flags);
    let list = list(&artboard);
    assert_eq!(count(&list), 1);
    let width = |index| {
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index))
            .unwrap()
            .unwrap();
        let bar = row
            .with_artboard(|row| row.find_handle::<Rectangle>("Bar Path"))
            .unwrap();
        bar.with_downcast::<Rectangle, _>(|bar| bar.width())
            .unwrap()
    };
    assert_eq!(width(0), 100.0);
    let item_list = property(&vm, "rows");
    let model = model(&item_list);
    let arena = CoreArena::default();
    append(&file, &item_list, &model, &arena);
    artboard.advance(0.0, design_flags);
    assert_eq!(count(&list), 2);
    assert_eq!(width(1), 100.0);
    artboard.advance(0.0, play_flags);
    assert_eq!(width(0), 150.0);
    assert_eq!(width(1), 150.0);
    append(&file, &item_list, &model, &arena);
    artboard.advance(0.0, play_flags);
    assert_eq!(count(&list), 3);
    assert_eq!(width(2), 150.0);
}

#[test]
fn artboard_knows_whether_it_runs_scripts() {
    for (asset, board, expected) in [
        ("clear_viewmodel_list.riv", "Artboard", true),
        ("clear_viewmodel_list.riv", "Child", false),
        ("listener_action_inputs.riv", "Artboard", true),
        ("script_color_data_test.riv", "Artboard", true),
        ("component_list_default_state_machine.riv", "Row", false),
    ] {
        let file = load(asset);
        let source = file
            .with_file(|file| file.artboard_named_source(board))
            .unwrap();
        assert_eq!(
            source
                .with_downcast::<Artboard, _>(|board| board.uses_scripts())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn reused_pressed_row_forgets_its_press() {
    let (file, artboard, vm, machine) = setup("component_list_row_click.riv", Some("Main"));
    machine.advance_and_apply(0.016);
    let list = list(&artboard);
    assert_eq!(count(&list), 1);
    let item_list = property(&vm, "rows");
    let point = Vec2D::new(100.0, 20.0);
    let down = || {
        machine.with_instance_mut(|machine| machine.pointer_down(point, 0, PointerButton::Primary))
    };
    let up = || {
        machine.with_instance_mut(|machine| machine.pointer_up(point, 0, PointerButton::Primary))
    };
    down();
    up();
    machine.advance_and_apply(0.016);
    assert!(boolean(&row_vm(&items(&item_list)[0]), "clicked"));
    down();
    machine.advance_and_apply(0.016);
    let model = model(&item_list);
    clear(&item_list);
    let arena = CoreArena::default();
    let replacement = append(&file, &item_list, &model, &arena);
    machine.advance_and_apply(0.016);
    assert_eq!(count(&list), 1);
    up();
    machine.advance_and_apply(0.016);
    assert!(!boolean(&replacement, "clicked"));
    down();
    up();
    machine.advance_and_apply(0.016);
    assert!(boolean(&replacement, "clicked"));
}
