//! Exact owner-flow port of pinned `Transition self conditions`.

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime,
    runtime::viewmodel::{
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests").join(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn advance_draw(
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    factory: &PersistentFactory<SerializingFactory>,
    seconds: f32,
) {
    machine.advance_and_apply(seconds);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
}

fn nullable_list_item(list: &CoreHandle) -> CoreHandle {
    list.insert_sibling(ViewModelInstanceListItem::default())
        .expect("list arena creates nullable ViewModelInstanceListItem")
}

fn add_nullable_item(list: &CoreHandle) {
    let item = nullable_list_item(list);
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
        .expect("actual list owner");
}

fn add_nullable_item_at(list: &CoreHandle, index: i32) -> bool {
    let item = nullable_list_item(list);
    let inserted = list
        .with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item_at(item.clone(), index))
        .expect("actual list owner");
    if !inserted {
        item.remove_occurrence();
    }
    inserted
}

#[test]
fn transition_self_conditions() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned("assets/transition_self_comparator_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model: RuntimeViewModelInstanceHandle = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("view model instance");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));

    advance_draw(&artboard, &machine, &silver, 0.1);

    let number = view_model.property_number("num").expect("num number");
    let trigger = view_model.property_trigger("tri").expect("tri trigger");
    let color = view_model.property_color("col").expect("col color");
    let boolean = view_model.property_boolean("bol").expect("bol boolean");
    let string = view_model.property_string("str").expect("str string");
    let list = view_model.property_list("lis").expect("lis list");
    let list_owner = list.value_runtime().handle();

    silver.borrow_mut().add_frame();
    number.set_value(20.0);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    number.set_value(20.0);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    number.set_value(10.0);
    number.set_value(20.0);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    number.set_value(10.0);
    trigger.trigger();
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    color.set_value(0x6400_0a0fu32 as i32);
    color.set_value(0x6500_0a0fu32 as i32);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    color.set_value(0x6600_0a0fu32 as i32);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    boolean.set_value(true);
    boolean.set_value(false);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    boolean.set_value(true);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    string.set_value("a");
    string.set_value("b");
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    string.set_value("c");
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    add_nullable_item(&list_owner);
    add_nullable_item(&list_owner);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    add_nullable_item(&list_owner);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    assert!(add_nullable_item_at(&list_owner, 0));
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    assert!(!add_nullable_item_at(&list_owner, 10));
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    list.swap(0, 1);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    list.remove_instance_at(0);
    advance_draw(&artboard, &machine, &silver, 0.0);

    silver.borrow_mut().add_frame();
    list.remove_instance_at(10);
    advance_draw(&artboard, &machine, &silver, 0.0);

    let actual = parse_sriv(&silver.borrow().bytes()).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned("silvers/transition_self_comparator_test.sriv"))
        .expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("transition_self differs: {difference}"));
}
