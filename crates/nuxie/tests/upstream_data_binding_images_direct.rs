//! Exact direct ports for Wave B1 image cases that do not have a C++ silver.

use std::{path::PathBuf, rc::Rc};

use nuxie::File;
use nuxie::PersistentFactory;
use nuxie_render_api::{Factory, RecordingFactory, SerializingFactory};
use nuxie_runtime::RuntimeViewModelImage;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn property_key(owner: &str, property: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(owner).expect("schema owner");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|definition| definition.properties)
        .find(|candidate| candidate.name == property)
        .unwrap_or_else(|| panic!("{owner}.{property}"))
        .key
        .int
}

fn advance(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut nuxie::ViewModelInstance,
    seconds: f32,
) {
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(machine),
        seconds,
        view_model,
    );
}

fn encoded_asset_hex(file: &File, index: u64) -> String {
    let asset = file
        .asset(index as usize)
        .unwrap_or_else(|| panic!("file asset {index}"));
    let bytes = asset
        .contents()
        .unwrap_or_else(|| panic!("file asset {index} has no embedded contents"));
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn draw_recording(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    factory: &mut RecordingFactory,
) -> String {
    factory.clear();
    let mut renderer = factory.make_renderer();
    artboard
        .draw(factory, &mut renderer)
        .expect("image artboard draws");
    factory.canonical_recording().stream().to_owned()
}

fn advance_serialized(
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
        .expect("image artboard draws");
}

fn click(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    x: f32,
    y: f32,
) {
    machine.pointer_down(artboard.raw_mut(), x, y, 0);
    machine.pointer_up(artboard.raw_mut(), x, y, 0);
}

#[test]
fn data_binding_images_from_file_assets() {
    let file = Box::leak(Box::new(
        File::import(&pinned("data_binding_images_test.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("main")
        .expect("main artboard")
        .instantiate()
        .expect("artboard instantiates");
    let view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view model");
    assert!(artboard.bind_view_model(&view_model));
    artboard.advance(0.0);

    let main_initial = view_model
        .raw()
        .asset_value_by_property_name_path("main_im")
        .expect("main_im asset index");
    let sub = view_model
        .handle()
        .linked_view_model_by_property_name_path("sub_1")
        .expect("sub_1 linked view model");
    let sub_initial = sub
        .borrow()
        .asset_value_by_property_name_path("sub_1_im")
        .expect("sub_1_im asset index");

    let mut factory = RecordingFactory::new();
    let initial = draw_recording(&mut artboard, &mut factory);
    assert!(
        initial.contains(&format!("data={}", encoded_asset_hex(file, main_initial))),
        "root image did not draw the file asset selected by main_im"
    );
    assert!(
        initial.contains(&format!("data={}", encoded_asset_hex(file, sub_initial))),
        "nested image did not draw the file asset selected by sub_1/sub_1_im"
    );

    assert!(
        view_model
            .raw_mut()
            .set_asset_by_property_name_path("main_im", 2)
    );
    assert!(
        sub.borrow_mut()
            .set_asset_by_property_name_path("sub_1_im", 6)
    );
    artboard.advance(0.0);
    let updated = draw_recording(&mut artboard, &mut factory);
    assert!(
        updated.contains(&format!("data={}", encoded_asset_hex(file, 2))),
        "root image did not switch to file asset 2"
    );
    assert!(
        updated.contains(&format!("data={}", encoded_asset_hex(file, 6))),
        "nested image did not switch to file asset 6"
    );
    assert_ne!(main_initial, 2);
    assert_ne!(sub_initial, 6);
}

#[test]
#[ignore = "expected-red: exact live ViewModelInstanceAssetImage null action is rejected before the pinned two-frame draw flow"]
fn embedded_images_can_be_reset_by_passing_live_image_null() {
    let file = Box::leak(Box::new(
        File::import(&pinned("viewmodel_image_reset.riv")).expect("fixture imports"),
    ));
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

    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    assert!(
        view_model
            .raw_mut()
            .set_runtime_image_by_property_name_path("img", None),
        "the exact live ViewModelInstanceAssetImage null action must succeed"
    );
    assert!(
        view_model
            .raw()
            .runtime_image_by_property_name_path("img")
            .is_none()
    );
    assert_eq!(
        view_model.raw().asset_value_by_property_name_path("img"),
        Some(u64::MAX)
    );
    factory.borrow_mut().add_frame();
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.1,
        &mut view_model,
    );
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.1,
        &mut view_model,
    );
    let mut renderer = factory.borrow().make_renderer();
    artboard
        .draw(&mut factory, &mut renderer)
        .expect("image-reset artboard draws");

    let expected =
        parse_sriv(&pinned("../silvers/viewmodel_image_reset.sriv")).expect("pinned C++ silver");
    let actual = parse_sriv(&factory.borrow().bytes()).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("viewmodel_image_reset differs: {difference}"));
}

#[test]
#[ignore = "expected-red: after the exact image2 swap, the no-scale Image local transform translation is nonnegative instead of pinned C++ negative x/y"]
fn image_fit_alignment_preserves_generated_owners_and_all_three_twenty_frame_phases() {
    let file = Box::leak(Box::new(
        File::import(&pinned("image_fit_alignment.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("Main")
        .expect("Main artboard")
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
    advance(&mut artboard, &mut machine, &mut view_model, 0.1);

    let image_locals = artboard
        .artboard()
        .graph()
        .components
        .iter()
        .filter(|component| component.type_name == "Image")
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    let first_image = *image_locals.first().expect("first Image owner");
    let fit_key = property_key("Image", "fit");
    let alignment_x_key = property_key("Image", "alignmentX");
    let alignment_y_key = property_key("Image", "alignmentY");
    let original_fit = artboard
        .raw()
        .debug_uint_property(first_image, fit_key)
        .expect("Image.fit");
    let original_alignment_x = artboard
        .raw()
        .double_property(first_image, alignment_x_key)
        .expect("Image.alignmentX");
    let original_alignment_y = artboard
        .raw()
        .double_property(first_image, alignment_y_key)
        .expect("Image.alignmentY");
    let test_fit = if original_fit == 1 { 2 } else { 1 };
    let test_alignment_x = if original_alignment_x == -1.0 {
        1.0
    } else {
        -1.0
    };
    let test_alignment_y = if original_alignment_y == -1.0 {
        1.0
    } else {
        -1.0
    };
    assert!(
        artboard
            .raw_mut()
            .set_uint_property(first_image, fit_key, test_fit)
    );
    assert!(
        artboard
            .raw_mut()
            .set_double_property(first_image, alignment_x_key, test_alignment_x)
    );
    assert!(
        artboard
            .raw_mut()
            .set_double_property(first_image, alignment_y_key, test_alignment_y)
    );
    assert_eq!(
        artboard.raw().debug_uint_property(first_image, fit_key),
        Some(test_fit)
    );
    assert_eq!(
        artboard.raw().double_property(first_image, alignment_x_key),
        Some(test_alignment_x)
    );
    assert_eq!(
        artboard.raw().double_property(first_image, alignment_y_key),
        Some(test_alignment_y)
    );
    assert!(
        artboard
            .raw_mut()
            .set_uint_property(first_image, fit_key, original_fit)
    );
    assert!(artboard.raw_mut().set_double_property(
        first_image,
        alignment_x_key,
        original_alignment_x
    ));
    assert!(artboard.raw_mut().set_double_property(
        first_image,
        alignment_y_key,
        original_alignment_y
    ));
    assert_eq!(
        artboard.raw().debug_uint_property(first_image, fit_key),
        Some(original_fit)
    );
    assert_eq!(
        artboard.raw().double_property(first_image, alignment_x_key),
        Some(original_alignment_x)
    );
    assert_eq!(
        artboard.raw().double_property(first_image, alignment_y_key),
        Some(original_alignment_y)
    );

    let mut renderer = factory.borrow().make_renderer();
    artboard
        .draw(&mut factory, &mut renderer)
        .expect("initial fit/alignment frame draws");
    let asset_index = |name: &str| {
        file.assets()
            .find(|asset| asset.name() == Some(name))
            .map(|asset| asset.index() as u64)
            .unwrap_or_else(|| panic!("file asset {name}"))
    };
    let _image1_index = asset_index("image1");
    let image2_index = asset_index("image2");
    let image3_index = asset_index("image3");

    for _ in 0..20 {
        factory.borrow_mut().add_frame();
        advance_serialized(
            &mut artboard,
            &mut machine,
            &mut view_model,
            &mut factory,
            0.016,
        );
    }

    for next_asset in [image2_index, image3_index] {
        assert!(
            view_model
                .raw_mut()
                .set_asset_by_property_name_path("imageProperty", next_asset)
        );
        advance(&mut artboard, &mut machine, &mut view_model, 0.0);
        let no_scale = image_locals
            .iter()
            .copied()
            .find(|local| {
                artboard.raw().debug_uint_property(*local, fit_key) == Some(0)
            })
            .expect("Image with Fit::none");
        let parent_local = artboard
            .artboard()
            .graph()
            .components
            .iter()
            .find(|component| component.local_id == no_scale)
            .and_then(|component| component.parent_local);
        let world = artboard
            .raw_mut()
            .object_world_transform(no_scale)
            .expect("no-scale Image transform");
        let local_translation = match parent_local {
            Some(parent_local) => artboard
                .raw_mut()
                .object_world_transform(parent_local)
                .and_then(nuxie_render_api::Mat2D::invert)
                .map(|inverse_parent| {
                    inverse_parent.transform_point(nuxie_render_api::Vec2D::new(
                        world.0[4], world.0[5],
                    ))
                })
                .expect("no-scale Image parent transform is invertible"),
            None => nuxie_render_api::Vec2D::new(world.0[4], world.0[5]),
        };
        assert!(local_translation.x < 0.0);
        assert!(local_translation.y < 0.0);
        for _ in 0..20 {
            factory.borrow_mut().add_frame();
            advance_serialized(
                &mut artboard,
                &mut machine,
                &mut view_model,
                &mut factory,
                0.016,
            );
        }
    }

    let expected =
        parse_sriv(&pinned("../silvers/image_fit_alignment.sriv")).expect("pinned C++ silver");
    let actual = parse_sriv(&factory.borrow().bytes()).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("image_fit_alignment differs: {difference}"));
}

#[test]
#[ignore = "expected-red: after exact eager decode and assignment, dynamic listener image binding diverges at frame 3 op 101 (expected save, got restore)"]
fn dynamic_image_binding_with_listener_action() {
    let file = Box::leak(Box::new(
        File::import(&pinned("image_binding_with_listener.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("main")
        .expect("main artboard")
        .instantiate()
        .expect("artboard instantiates");
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut factory)
        .expect("renderer initializes");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("authored view model instance 0");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    let (width, height) = artboard.artboard_dimensions();
    factory.borrow_mut().frame_size(width as u32, height as u32);

    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );
    factory.borrow_mut().add_frame();
    click(&mut artboard, &mut machine, 650.0, 650.0);
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    let image = pinned("open_source.jpg");
    assert_eq!(image.len(), 8880);
    let decoded = factory
        .borrow_mut()
        .decode_image(&image)
        .expect("open_source.jpg decodes");
    assert!(
        view_model
            .raw_mut()
            .set_runtime_image_by_property_name_path(
                "image1",
                Some(RuntimeViewModelImage::from_render_image(Rc::from(decoded))),
            )
    );
    factory.borrow_mut().add_frame();
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );
    factory.borrow_mut().add_frame();
    click(&mut artboard, &mut machine, 650.0, 650.0);
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    assert!(
        view_model
            .raw_mut()
            .set_runtime_image_by_property_name_path("image1", None)
    );
    factory.borrow_mut().add_frame();
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );
    factory.borrow_mut().add_frame();
    click(&mut artboard, &mut machine, 650.0, 650.0);
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    let expected = parse_sriv(&pinned("../silvers/image_binding_with_listener.sriv"))
        .expect("pinned C++ silver");
    let actual = parse_sriv(&factory.borrow().bytes()).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("image_binding_with_listener differs: {difference}"));
}

fn layout_fixture(
    bytes: Vec<u8>,
) -> (
    nuxie::ArtboardInstance<'static>,
    nuxie::StateMachineInstance,
    nuxie::ViewModelInstance,
    SerializingFactory,
    Vec<usize>,
) {
    let file = Box::leak(Box::new(
        File::import(&bytes).expect("layout fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("Main")
        .expect("Main artboard")
        .instantiate()
        .expect("artboard instantiates");
    let mut factory = SerializingFactory::new();
    artboard
        .initialize_renderer(&mut factory)
        .expect("layout images decode");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("view model");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    let graph = artboard.artboard().graph();
    let images = graph
        .components
        .iter()
        .filter(|component| component.type_name == "Image")
        .filter(|component| {
            component.parent_local.is_some_and(|parent| {
                graph
                    .components
                    .iter()
                    .find(|candidate| candidate.local_id == parent)
                    .is_some_and(|candidate| {
                        nuxie_schema::definition_by_name(candidate.type_name)
                            .is_some_and(|definition| definition.is_a("LayoutComponent"))
                    })
            })
        })
        .map(|component| component.local_id)
        .collect::<Vec<_>>();
    assert!(!images.is_empty());
    (artboard, machine, view_model, factory, images)
}

fn x_axis_scale(transform: nuxie_render_api::Mat2D) -> f32 {
    transform.0[0].hypot(transform.0[1])
}

fn catch_approx_eq(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= f32::EPSILON * 100.0 * expected.abs()
}

#[test]
#[ignore = "expected-red: 7.2 layout image render scale is 3.65 instead of legacy 1.0105 multiplied by the authored user scale 3.65"]
fn layout_image_composes_user_scale_on_top_of_fit_for_7_2_files() {
    let legacy_bytes = pinned("image_fit_alignment.riv");
    assert!(legacy_bytes.len() > 6);
    assert_eq!(&legacy_bytes[..4], b"RIVE");
    assert_eq!(legacy_bytes[4], 7);
    assert!(legacy_bytes[5] < 2);
    let mut modern_bytes = legacy_bytes.clone();
    modern_bytes[5] = 2;

    let (mut legacy, mut legacy_machine, mut legacy_vm, _legacy_factory, legacy_images) =
        layout_fixture(legacy_bytes);
    let (mut modern, mut modern_machine, mut modern_vm, _modern_factory, modern_images) =
        layout_fixture(modern_bytes);
    assert_eq!(legacy_images.len(), modern_images.len());
    advance(&mut legacy, &mut legacy_machine, &mut legacy_vm, 0.1);
    advance(&mut modern, &mut modern_machine, &mut modern_vm, 0.1);

    let mut pick = None;
    for _ in 0..120 {
        pick = legacy_images.iter().copied().find(|local| {
            legacy
                .raw_mut()
                .object_world_transform(*local)
                .is_some_and(|transform| x_axis_scale(transform) > 1.0)
        });
        if pick.is_some() {
            break;
        }
        advance(&mut legacy, &mut legacy_machine, &mut legacy_vm, 0.016);
        advance(&mut modern, &mut modern_machine, &mut modern_vm, 0.016);
    }
    let legacy_local = pick.expect("an open legacy layout image");
    let image_index = legacy_images
        .iter()
        .position(|local| *local == legacy_local)
        .expect("picked image index");
    let modern_local = modern_images[image_index];
    let user_scale_x = modern
        .raw()
        .double_property(modern_local, property_key("Image", "scaleX"))
        .expect("modern Image.scaleX");
    let legacy_public_scale_x = legacy
        .raw()
        .double_property(legacy_local, property_key("Image", "scaleX"))
        .expect("legacy Image.scaleX");
    assert!(!catch_approx_eq(user_scale_x, 1.0));
    assert!(!catch_approx_eq(legacy_public_scale_x, user_scale_x));

    let legacy_scale = x_axis_scale(
        legacy
            .raw_mut()
            .object_world_transform(legacy_local)
            .expect("legacy image world transform"),
    );
    let modern_scale = x_axis_scale(
        modern
            .raw_mut()
            .object_world_transform(modern_local)
            .expect("modern image world transform"),
    );
    assert!(legacy_scale > 0.0);
    assert!(
        catch_approx_eq(modern_scale, legacy_scale * user_scale_x),
        "modern render scale {modern_scale} != legacy {legacy_scale} * user scale {user_scale_x}"
    );
}

#[test]
fn stateful_component_image_bind() {
    let file = Box::leak(Box::new(
        File::import(&pinned("stateful_component_image_test.riv")).expect("fixture imports"),
    ));
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
        .expect("authored view model instance 0");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    let (width, height) = artboard.artboard_dimensions();
    factory.borrow_mut().frame_size(width as u32, height as u32);

    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );
    factory.borrow_mut().add_frame();
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    let image = pinned("open_source.jpg");
    assert_eq!(image.len(), 8880);
    let decoded = factory
        .borrow_mut()
        .decode_image(&image)
        .expect("open_source.jpg decodes");
    assert!(
        view_model
            .raw_mut()
            .set_runtime_image_by_property_name_path(
                "img",
                Some(RuntimeViewModelImage::from_render_image(Rc::from(decoded)),)
            )
    );
    factory.borrow_mut().add_frame();
    advance_serialized(
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut factory,
        0.1,
    );

    let expected = parse_sriv(&pinned("../silvers/stateful_component_image_test.sriv"))
        .expect("pinned C++ silver");
    let actual = parse_sriv(&factory.borrow().bytes()).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("stateful_component_image_test differs: {difference}"));
}
