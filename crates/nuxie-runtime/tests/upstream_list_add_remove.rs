//! List additions/removal regressions from upstream f01989c6.
#![cfg(feature = "test-support")]
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    core::CoreArena,
    layout::layout_enums::LayoutDirection,
    layout_component::LayoutComponent,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};
use nuxie_runtime::RuntimeArtboardInstanceHandle;
use support::*;
fn load(name: &str) -> RuntimeFileHandle {
    let path = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    import(&std::fs::read(path).unwrap())
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

fn rows(board: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    board
        .with_artboard(|board| board.find_handle::<ArtboardComponentList>("Rows"))
        .unwrap()
}
fn check_positions(list: &CoreHandle, indices: &[usize], ax: f32, bx: f32) {
    for &index in indices {
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index as i32))
            .unwrap()
            .unwrap();
        for (name, x) in [("A", ax), ("B", bx)] {
            let child = row
                .with_artboard(|row| row.find_handle::<LayoutComponent>(name))
                .unwrap();
            assert_eq!(
                child
                    .with_downcast::<LayoutComponent, _>(|c| c.layout_x())
                    .unwrap(),
                x
            );
        }
    }
}
#[test]
fn component_list_adds_rtl_rows_before_first_solve() {
    let (file, board, vm, machine) = setup("component_list_rtl_row.riv", Some("Host"));
    machine.advance_and_apply(0.016);
    let list = rows(&board);
    let items = property(&vm, "rows");
    let model = model(&items);
    let arena = CoreArena::default();
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
            .unwrap(),
        1
    );
    for _ in 0..3 {
        append(&file, &items, &model, &arena);
    }
    let passes = Artboard::layout_pass_count();
    machine.advance_and_apply(0.016);
    assert_eq!(Artboard::layout_pass_count() - passes, 1);
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
            .unwrap(),
        4
    );
    check_positions(&list, &[0, 1, 2, 3], 50.0, 0.0);
}
#[test]
fn component_list_later_row_inherits_parent_direction() {
    let (file, board, vm, machine) = setup("component_list_rtl_host.riv", Some("Host"));
    machine.advance_and_apply(0.016);
    let list = rows(&board);
    let items = property(&vm, "rows");
    let model = model(&items);
    let arena = CoreArena::default();
    append(&file, &items, &model, &arena);
    let passes = Artboard::layout_pass_count();
    machine.advance_and_apply(0.016);
    assert_eq!(Artboard::layout_pass_count() - passes, 1);
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
            .unwrap(),
        2
    );
    // This fixture is translated Rive style, not LayoutCssStyle. Its physical-side
    // RTL mirroring is not an assertion about CSS records.
    for index in 0..2 {
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index))
            .unwrap()
            .unwrap();
        assert!(
            row.core_handle()
                .with(|r| r.as_layout_component().unwrap().actual_direction())
                .unwrap()
                == LayoutDirection::Rtl
        );
    }
    check_positions(&list, &[0, 1], 190.0, 140.0);
}
fn fill_virtualized(
    name: &str,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    CoreArena,
    u64,
) {
    let (file, board, vm, machine) = setup(name, Some("Host"));
    let items = property(&vm, "rows");
    let model = model(&items);
    clear(&items);
    let arena = CoreArena::default();
    for _ in 0..20 {
        append(&file, &items, &model, &arena);
    }
    let passes = Artboard::layout_pass_count();
    machine.advance_and_apply(0.016);
    (
        file,
        board,
        machine,
        arena,
        Artboard::layout_pass_count() - passes,
    )
}
#[test]
fn component_list_virtualized_rtl_rows_need_no_extra_passes() {
    let (_lf, _lb, _lm, _la, ltr) = fill_virtualized("component_list_virtualized_ltr_row.riv");
    let (_rf, rb, _rm, _ra, rtl) = fill_virtualized("component_list_virtualized_rtl_row.riv");
    assert_eq!(rtl, ltr);
    let list = rows(&rb);
    let mut realized = Vec::new();
    list.with_downcast::<ArtboardComponentList, _>(|l| l.realized_indices(&mut realized))
        .unwrap();
    assert!(realized.len() >= 5);
    check_positions(
        &list,
        &realized.iter().map(|&i| i as usize).collect::<Vec<_>>(),
        50.0,
        0.0,
    );
}
#[test]
fn runtime_removes_every_row_showing_an_instance() {
    let file = load("viewmodel_runtime_file.riv");
    let model = file.with_file(|f| f.view_model_by_name("vm")).unwrap();
    let instance = model.create_default_instance();
    let list = instance.property_list("lis").unwrap();
    let initial = list.size();
    let a = model.create_instance();
    let b = model.create_instance();
    let c = model.create_instance();
    for value in [&a, &b, &c, &b] {
        list.add_instance(value.clone());
    }
    assert_eq!(list.size(), initial + 4);
    list.remove_instance(&b);
    assert_eq!(list.size(), initial + 2);
    assert!(std::rc::Rc::ptr_eq(
        &list.instance_at(initial as i32).unwrap(),
        &a
    ));
    assert!(std::rc::Rc::ptr_eq(
        &list.instance_at(initial as i32 + 1).unwrap(),
        &c
    ));
    list.remove_instance(&b);
    assert_eq!(list.size(), initial + 2);
}
