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

/// Test-only probe for the actual retained list owner at the missing nullable
/// item boundary. `set_list_item_count` is the only current owner mutation
/// capable of requesting a slot without fabricating a typed child instance.
/// A same-index swap is identity-preserving, but it can succeed only when that
/// logical slot is backed by a concrete `ViewModelInstanceListItem` owner.
///
/// Today the first mutation raises `item_count` while retaining no wrapper, so
/// the real owner rejects the swap. Once nullable wrappers are retained, these
/// same owner operations leave the inserted item in place and return true.
fn add_nullable_list_item_at_actual_owner(
    owner: &nuxie_runtime::RuntimeOwnedViewModelHandle,
    property_path: &str,
    index: usize,
) -> (Option<usize>, bool) {
    let Some(item_count) = owner.list_item_count_by_property_name_path(property_path) else {
        return (None, false);
    };
    if index != item_count
        || !owner
            .borrow_mut()
            .set_list_item_count_by_property_name_path(property_path, item_count + 1)
    {
        return (
            owner.list_item_count_by_property_name_path(property_path),
            false,
        );
    }
    let addressable = owner.swap_list_items_by_property_name_path(property_path, index, index);
    (
        owner.list_item_count_by_property_name_path(property_path),
        addressable,
    )
}

#[test]
#[ignore = "expected-red: the actual list owner records the first nullable slot count but cannot retain an addressable ViewModelInstanceListItem wrapper"]
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

    assert_eq!(
        view_model
            .handle()
            .list_item_count_by_property_name_path("lis"),
        Some(0),
        "the pinned first list action starts from an empty retained list owner",
    );
    let (item_count, nullable_wrapper_is_addressable) =
        add_nullable_list_item_at_actual_owner(view_model.handle(), "lis", 0);
    assert_eq!(
        item_count,
        Some(1),
        "the actual list owner must record the first nullable add",
    );
    assert!(
        nullable_wrapper_is_addressable,
        "the actual retained list owner must add an addressable nullable wrapper at index 0",
    );
}
