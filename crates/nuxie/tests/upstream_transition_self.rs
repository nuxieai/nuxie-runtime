//! Executable owner-flow port of pinned `Transition self conditions`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
use nuxie_render_api::SerializingFactory;

fn pinned() -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path =
        PathBuf::from(root).join("tests/unit_tests/assets/transition_self_comparator_test.riv");
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn advance_draw(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut nuxie::ViewModelInstance,
    factory: &mut PersistentFactory<SerializingFactory>,
    seconds: f32,
) {
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(machine),
        seconds,
        view_model,
    );
    let mut renderer = factory.borrow().make_renderer();
    artboard
        .draw(factory, &mut renderer)
        .expect("transition comparator artboard draws");
}

#[test]
#[ignore = "expected-red: after faithfully replaying scalar and empty-list count transitions, Rust cannot swap the counted empty list items because the public list stores no concrete item occurrences"]
fn transition_self_conditions_reach_empty_list_swap_seam() {
    let file = Box::leak(Box::new(File::import(&pinned()).expect("fixture imports")));
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("artboard instantiates");
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut factory)
        .expect("renderer initializes");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("view model instance");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    let (width, height) = artboard.artboard_dimensions();
    factory.borrow_mut().frame_size(width as u32, height as u32);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_number("num", 20.0);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_number("num", 20.0);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_number("num", 10.0);
    let _ = view_model.set_number("num", 20.0);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_number("num", 10.0);
    let _ = view_model.fire_trigger("tri");
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_color("col", 0x6400_0a0f);
    let _ = view_model.set_color("col", 0x6500_0a0f);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_color("col", 0x6600_0a0f);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_bool("bol", true);
    let _ = view_model.set_bool("bol", false);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_bool("bol", true);
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_string("str", "a");
    let _ = view_model.set_string("str", "b");
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    let _ = view_model.set_string("str", "c");
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    // C++ adds two empty `ViewModelInstanceListItem`s. Rust's exact public
    // analogue for untyped empty items is the authored count seam.
    factory.borrow_mut().add_frame();
    assert!(
        view_model
            .raw_mut()
            .set_list_item_count_by_property_name_path("lis", 2)
    );
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    factory.borrow_mut().add_frame();
    assert!(
        view_model
            .raw_mut()
            .set_list_item_count_by_property_name_path("lis", 3)
    );
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    // Adding another indistinguishable empty item at index zero has the same
    // observable list value as increasing the empty-item count to four.
    factory.borrow_mut().add_frame();
    assert!(
        view_model
            .raw_mut()
            .set_list_item_count_by_property_name_path("lis", 4)
    );
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    // The pinned invalid add at index ten is a no-op.
    factory.borrow_mut().add_frame();
    assert_eq!(
        view_model
            .raw()
            .list_item_count_by_property_name_path("lis"),
        Some(4)
    );
    advance_draw(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.0,
    );

    // This is the first upstream action without an executable Rust analogue:
    // count-only empty items cannot be swapped, even though C++ notifies the
    // transition comparator for the swap.
    factory.borrow_mut().add_frame();
    assert!(
        view_model
            .handle()
            .swap_list_items_by_property_name_path("lis", 0, 1),
        "counted empty list items must be concrete occurrences for swap notification"
    );
}
